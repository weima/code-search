use crate::error::{Result, SearchError};
use crate::parse::TranslationEntry;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

const CACHE_DIR_NAME: &str = "cs";
const PORT_FILE: &str = "cache.port";
const SERVER_FLAG: &str = "--cache-server";
const FRONT_CACHE_CAP: usize = 512;
const MAX_CACHE_SIZE: u64 = 1_000_000_000;
const MAX_CACHE_AGE_SECS: u64 = 30 * 24 * 60 * 60;
const CLEANUP_INTERVAL_SECS: u64 = 6 * 60 * 60;

/// Cache value stored for each (file, query) pair
#[derive(Serialize, Deserialize, Clone)]
struct CacheValue {
    mtime_secs: u64,
    file_size: u64,
    last_accessed: u64,
    results: Vec<TranslationEntry>,
}

/// Cross-process cache client: tries a background TCP server; falls back to local cache.
pub struct SearchResultCache {
    backend: CacheBackend,
}

enum CacheBackend {
    Local(LocalCache),
    Remote(RemoteCache),
}

#[derive(Serialize, Deserialize, Debug)]
enum CacheRequest {
    Get {
        file: PathBuf,
        query: String,
        case_sensitive: bool,
        mtime_secs: u64,
        file_size: u64,
    },
    Set {
        file: PathBuf,
        query: String,
        case_sensitive: bool,
        mtime_secs: u64,
        file_size: u64,
        results: Vec<TranslationEntry>,
    },
    Clear,
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
enum CacheResponse {
    Get(Option<Vec<TranslationEntry>>),
    Ack(bool),
}

impl SearchResultCache {
    /// Create a cache client. Prefers a background TCP server unless disabled.
    pub fn new() -> Result<Self> {
        if std::env::var("CS_DISABLE_CACHE_SERVER").is_ok() {
            return Ok(Self {
                backend: CacheBackend::Local(LocalCache::new()?),
            });
        }

        if let Some(remote) = RemoteCache::connect_or_spawn()? {
            return Ok(Self {
                backend: CacheBackend::Remote(remote),
            });
        }

        Ok(Self {
            backend: CacheBackend::Local(LocalCache::new()?),
        })
    }

    /// Test helper: force cache to use a specific directory (local only).
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            backend: CacheBackend::Local(LocalCache::with_cache_dir(cache_dir)?),
        })
    }

    pub fn get(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        current_mtime: SystemTime,
        current_size: u64,
    ) -> Option<Vec<TranslationEntry>> {
        match &self.backend {
            CacheBackend::Local(inner) => {
                inner.get(file, query, case_sensitive, current_mtime, current_size)
            }
            CacheBackend::Remote(remote) => remote
                .get(file, query, case_sensitive, current_mtime, current_size)
                .ok()
                .flatten(),
        }
    }

    pub fn set(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        mtime: SystemTime,
        file_size: u64,
        results: &[TranslationEntry],
    ) -> Result<()> {
        match &self.backend {
            CacheBackend::Local(inner) => {
                inner.set(file, query, case_sensitive, mtime, file_size, results)
            }
            CacheBackend::Remote(remote) => {
                remote.set(file, query, case_sensitive, mtime, file_size, results)
            }
        }
    }

    pub fn clear(&self) -> Result<()> {
        match &self.backend {
            CacheBackend::Local(inner) => inner.clear(),
            CacheBackend::Remote(remote) => remote.clear(),
        }
    }

    /// Hidden entrypoint: block and run cache server.
    pub fn start_server_blocking() -> Result<()> {
        run_cache_server()
    }
}

struct LocalCache {
    db: Db,
    last_cleanup: SystemTime,
    front_cache: Mutex<HashMap<Vec<u8>, CacheValue>>,
    cache_dir: PathBuf,
}

impl LocalCache {
    fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CACHE_DIR_NAME)
    }

    fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        let db = sled::open(cache_dir.join("db"))
            .map_err(|e| SearchError::Generic(format!("Failed to open cache: {}", e)))?;

        let last_cleanup = Self::read_last_cleanup_marker(&cache_dir)?;
        let cache = Self {
            db,
            last_cleanup,
            front_cache: Mutex::new(HashMap::new()),
            cache_dir,
        };
        cache.maybe_cleanup_on_open()?;
        Ok(cache)
    }

    fn new() -> Result<Self> {
        Self::with_cache_dir(Self::cache_dir())
    }

    fn get(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        current_mtime: SystemTime,
        current_size: u64,
    ) -> Option<Vec<TranslationEntry>> {
        let key = self.make_key(file, query, case_sensitive);

        if let Some(entries) = self.front_get(&key, current_mtime, current_size) {
            return Some(entries);
        }

        let cached_bytes = self.db.get(&key).ok()??;
        let mut cached: CacheValue = bincode::deserialize(&cached_bytes).ok()?;

        let current_secs = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        // Check if entry is expired (lazy cleanup)
        if now.saturating_sub(cached.last_accessed) > MAX_CACHE_AGE_SECS {
            // Entry expired - delete it and return None
            let _ = self.db.remove(&key);
            return None;
        }

        if cached.mtime_secs == current_secs && cached.file_size == current_size {
            cached.last_accessed = now;

            if let Ok(updated_bytes) = bincode::serialize(&cached) {
                let _ = self.db.insert(&key, updated_bytes);
            }

            self.front_set(key.clone(), cached.clone());
            Some(cached.results)
        } else {
            // File changed - delete stale entry
            let _ = self.db.remove(&key);
            None
        }
    }

    fn set(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        mtime: SystemTime,
        file_size: u64,
        results: &[TranslationEntry],
    ) -> Result<()> {
        let key = self.make_key(file, query, case_sensitive);

        let mtime_secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Invalid mtime: {}", e)))?
            .as_secs();

        let last_accessed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Failed to get current time: {}", e)))?
            .as_secs();

        let value = CacheValue {
            mtime_secs,
            file_size,
            last_accessed,
            results: results.to_vec(),
        };

        let value_bytes = bincode::serialize(&value)
            .map_err(|e| SearchError::Generic(format!("Failed to serialize cache: {}", e)))?;

        self.front_set(key.clone(), value.clone());

        self.db
            .insert(key, value_bytes)
            .map_err(|e| SearchError::Generic(format!("Failed to write cache: {}", e)))?;

        Ok(())
    }

    fn clear(&self) -> Result<()> {
        self.db
            .clear()
            .map_err(|e| SearchError::Generic(format!("Failed to clear cache: {}", e)))?;
        if let Ok(mut map) = self.front_cache.lock() {
            map.clear();
        }
        let _ = fs::remove_file(Self::meta_file_path(&self.cache_dir));
        Ok(())
    }

    fn front_get(
        &self,
        key: &[u8],
        current_mtime: SystemTime,
        current_size: u64,
    ) -> Option<Vec<TranslationEntry>> {
        let guard = self.front_cache.lock().ok()?;
        let entry = guard.get(key)?;
        let current_secs = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();
        if entry.mtime_secs == current_secs && entry.file_size == current_size {
            Some(entry.results.clone())
        } else {
            None
        }
    }

    fn front_set(&self, key: Vec<u8>, value: CacheValue) {
        if let Ok(mut map) = self.front_cache.lock() {
            if map.len() >= FRONT_CACHE_CAP {
                if let Some(oldest_key) = map
                    .iter()
                    .min_by_key(|(_, v)| v.last_accessed)
                    .map(|(k, _)| k.clone())
                {
                    map.remove(&oldest_key);
                }
            }
            map.insert(key, value);
        }
    }

    fn make_key(&self, file: &Path, query: &str, case_sensitive: bool) -> Vec<u8> {
        let normalized_query = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        format!("{}|{}", file.display(), normalized_query).into_bytes()
    }

    fn maybe_cleanup_on_open(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Failed to get current time: {}", e)))?
            .as_secs();

        let last = self
            .last_cleanup
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now.saturating_sub(last) >= CLEANUP_INTERVAL_SECS {
            self.cleanup_if_needed()?;
        }

        Ok(())
    }

    fn cleanup_if_needed(&self) -> Result<()> {
        // Check if cache size exceeded limit
        let size = self
            .db
            .size_on_disk()
            .map_err(|e| SearchError::Generic(format!("Failed to get cache size: {}", e)))?;

        // Only do cleanup if size limit exceeded (expiry handled lazily at read time)
        if size <= MAX_CACHE_SIZE {
            return Ok(());
        }

        // Collect all entries with their last access time
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Failed to get current time: {}", e)))?
            .as_secs();

        // ITERATOR IMPROVEMENT: Use filter_map instead of manual loop
        // Rust Book Chapter 13.2: Iterator Adapters
        // filter_map combines filtering and mapping in one pass
        let mut entries: Vec<(Vec<u8>, u64)> = self
            .db
            .iter()
            .flatten()
            .filter_map(|(key, value)| {
                // Try to deserialize, convert Result to Option
                bincode::deserialize::<CacheValue>(&value)
                    .ok()
                    // Filter out expired entries
                    .filter(|cache_value| {
                        now.saturating_sub(cache_value.last_accessed) <= MAX_CACHE_AGE_SECS
                    })
                    // Map to the tuple we need
                    .map(|cache_value| (key.to_vec(), cache_value.last_accessed))
            })
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        // Remove oldest entries until size is under limit
        for (key, _) in entries.iter() {
            if self
                .db
                .size_on_disk()
                .ok()
                .map(|s| s <= MAX_CACHE_SIZE)
                .unwrap_or(true)
            {
                break;
            }
            let _ = self.db.remove(key);
        }

        let _ = self.db.flush();
        self.write_last_cleanup_marker(&self.cache_dir);
        Ok(())
    }

    fn meta_file_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("meta.last")
    }

    fn write_last_cleanup_marker(&self, cache_dir: &Path) {
        let _ = fs::write(
            Self::meta_file_path(cache_dir),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string()),
        );
    }

    fn read_last_cleanup_marker(cache_dir: &Path) -> Result<SystemTime> {
        let path = Self::meta_file_path(cache_dir);

        let contents = fs::read_to_string(path).ok();
        if let Some(s) = contents {
            if let Ok(secs) = s.trim().parse::<u64>() {
                return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(secs));
            }
        }

        Ok(SystemTime::UNIX_EPOCH)
    }
}

struct RemoteCache {
    addr: String,
}

impl RemoteCache {
    fn connect_or_spawn() -> Result<Option<Self>> {
        if let Some(addr) = read_port_file() {
            if Self::ping_addr(&addr).is_ok() {
                return Ok(Some(Self { addr }));
            }
        }

        spawn_server()?;

        if let Some(addr) = read_port_file() {
            if Self::ping_addr(&addr).is_ok() {
                return Ok(Some(Self { addr }));
            }
        }

        Ok(None)
    }

    fn ping_addr(addr: &str) -> Result<()> {
        let client = Self {
            addr: addr.to_string(),
        };
        match client.send_request(CacheRequest::Ping)? {
            CacheResponse::Ack(true) => Ok(()),
            _ => Err(SearchError::Generic(
                "Cache server did not acknowledge ping".to_string(),
            )),
        }
    }

    fn get(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        current_mtime: SystemTime,
        current_size: u64,
    ) -> Result<Option<Vec<TranslationEntry>>> {
        let mtime_secs = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Invalid mtime: {}", e)))?
            .as_secs();

        let req = CacheRequest::Get {
            file: file.to_path_buf(),
            query: query.to_string(),
            case_sensitive,
            mtime_secs,
            file_size: current_size,
        };

        match self.send_request(req)? {
            CacheResponse::Get(res) => Ok(res),
            _ => Err(SearchError::Generic("Invalid cache response".to_string())),
        }
    }

    fn set(
        &self,
        file: &Path,
        query: &str,
        case_sensitive: bool,
        mtime: SystemTime,
        file_size: u64,
        results: &[TranslationEntry],
    ) -> Result<()> {
        let mtime_secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| SearchError::Generic(format!("Invalid mtime: {}", e)))?
            .as_secs();

        let req = CacheRequest::Set {
            file: file.to_path_buf(),
            query: query.to_string(),
            case_sensitive,
            mtime_secs,
            file_size,
            results: results.to_vec(),
        };

        match self.send_request(req)? {
            CacheResponse::Ack(true) => Ok(()),
            _ => Err(SearchError::Generic("Cache write failed".to_string())),
        }
    }

    fn clear(&self) -> Result<()> {
        match self.send_request(CacheRequest::Clear)? {
            CacheResponse::Ack(true) => Ok(()),
            _ => Err(SearchError::Generic("Failed to clear cache".to_string())),
        }
    }

    fn send_request(&self, req: CacheRequest) -> Result<CacheResponse> {
        let mut stream = TcpStream::connect(&self.addr)
            .map_err(|e| SearchError::Generic(format!("Failed to connect cache server: {}", e)))?;

        let bytes = bincode::serialize(&req)
            .map_err(|e| SearchError::Generic(format!("Failed to encode cache request: {}", e)))?;

        stream
            .write_all(&bytes)
            .map_err(|e| SearchError::Generic(format!("Failed to write cache request: {}", e)))?;
        let _ = stream.shutdown(Shutdown::Write);

        let mut buf = Vec::new();
        stream
            .read_to_end(&mut buf)
            .map_err(|e| SearchError::Generic(format!("Failed to read cache response: {}", e)))?;

        let resp: CacheResponse = bincode::deserialize(&buf)
            .map_err(|e| SearchError::Generic(format!("Failed to decode cache response: {}", e)))?;
        Ok(resp)
    }
}

/// ---------- Server ----------
fn run_cache_server() -> Result<()> {
    let cache_dir = LocalCache::cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| SearchError::Generic(format!("Failed to bind cache server: {}", e)))?;
    let addr = listener
        .local_addr()
        .map_err(|e| SearchError::Generic(format!("Failed to get cache server address: {}", e)))?;
    write_port_file(&cache_dir, &addr.to_string())?;

    let local = LocalCache::with_cache_dir(cache_dir)?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = handle_connection(&local, &mut stream);
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn handle_connection(local: &LocalCache, stream: &mut TcpStream) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let req: CacheRequest = bincode::deserialize(&buf)
        .map_err(|e| SearchError::Generic(format!("Failed to decode cache request: {}", e)))?;

    let resp = match req {
        CacheRequest::Get {
            file,
            query,
            case_sensitive,
            mtime_secs,
            file_size,
        } => {
            let ts = SystemTime::UNIX_EPOCH + Duration::from_secs(mtime_secs);
            let hit = local.get(&file, &query, case_sensitive, ts, file_size);
            CacheResponse::Get(hit)
        }
        CacheRequest::Set {
            file,
            query,
            case_sensitive,
            mtime_secs,
            file_size,
            results,
        } => {
            let ts = SystemTime::UNIX_EPOCH + Duration::from_secs(mtime_secs);
            let res = local.set(&file, &query, case_sensitive, ts, file_size, &results);
            CacheResponse::Ack(res.is_ok())
        }
        CacheRequest::Clear => {
            let res = local.clear();
            CacheResponse::Ack(res.is_ok())
        }
        CacheRequest::Ping => CacheResponse::Ack(true),
    };

    let resp_bytes = bincode::serialize(&resp)
        .map_err(|e| SearchError::Generic(format!("Failed to encode cache response: {}", e)))?;
    stream.write_all(&resp_bytes)?;
    let _ = stream.shutdown(Shutdown::Write);
    Ok(())
}

/// ---------- Helpers ----------
fn cache_port_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(PORT_FILE)
}

fn write_port_file(cache_dir: &Path, addr: &str) -> Result<()> {
    fs::write(cache_port_path(cache_dir), addr)
        .map_err(|e| SearchError::Generic(format!("Failed to write cache port: {}", e)))
}

fn read_port_file() -> Option<String> {
    let path = cache_port_path(&LocalCache::cache_dir());
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn spawn_server() -> Result<()> {
    let exe = resolve_server_binary()?;

    Command::new(exe)
        .arg(SERVER_FLAG)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| SearchError::Generic(format!("Failed to spawn cache server: {}", e)))?;
    std::thread::sleep(Duration::from_millis(150));
    Ok(())
}

fn resolve_server_binary() -> Result<PathBuf> {
    let exe = std::env::current_exe()
        .map_err(|e| SearchError::Generic(format!("Failed to get current exe: {}", e)))?;

    let bin_name = if cfg!(target_os = "windows") {
        "cs.exe"
    } else {
        "cs"
    };

    // Prefer the real CLI binary (in target/debug or target/release) before falling back to the
    // current executable (which is a test harness when running integration tests).
    let mut candidates = Vec::new();
    if let Some(dir) = exe.parent() {
        candidates.push(dir.join(bin_name));
        if let Some(parent) = dir.parent() {
            candidates.push(parent.join(bin_name));
        }
    }
    candidates.push(exe.clone());

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(SearchError::Generic(
        "Could not locate cache server binary".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_cache_hit_local() {
        let cache_dir = TempDir::new().unwrap();
        let cache = SearchResultCache::with_cache_dir(cache_dir.path().to_path_buf()).unwrap();
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, "test content").unwrap();

        let metadata = fs::metadata(file.path()).unwrap();
        let mtime = metadata.modified().unwrap();
        let size = metadata.len();

        let results = vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            file: file.path().to_path_buf(),
            line: 1,
        }];

        cache
            .set(file.path(), "query", false, mtime, size, &results)
            .unwrap();
        let cached = cache.get(file.path(), "query", false, mtime, size);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_cache_invalidation_on_file_change_local() {
        let cache_dir = TempDir::new().unwrap();
        let cache = SearchResultCache::with_cache_dir(cache_dir.path().to_path_buf()).unwrap();
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, "original content").unwrap();

        let metadata = fs::metadata(file.path()).unwrap();
        let mtime = metadata.modified().unwrap();
        let size = metadata.len();

        let results = vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            file: file.path().to_path_buf(),
            line: 1,
        }];

        cache
            .set(file.path(), "query", false, mtime, size, &results)
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
        fs::write(&file, "modified content with different size").unwrap();

        let new_metadata = fs::metadata(file.path()).unwrap();
        let new_mtime = new_metadata.modified().unwrap();
        let new_size = new_metadata.len();

        assert!(new_size != size || new_mtime != mtime);

        let cached = cache.get(file.path(), "query", false, new_mtime, new_size);
        assert!(cached.is_none());
    }

    #[test]
    fn test_case_insensitive_normalization_local() {
        let cache_dir = TempDir::new().unwrap();
        let cache = SearchResultCache::with_cache_dir(cache_dir.path().to_path_buf()).unwrap();
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, "test content").unwrap();

        let metadata = fs::metadata(file.path()).unwrap();
        let mtime = metadata.modified().unwrap();
        let size = metadata.len();

        let results = vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            file: file.path().to_path_buf(),
            line: 1,
        }];

        cache
            .set(file.path(), "query", false, mtime, size, &results)
            .unwrap();

        let cached = cache.get(file.path(), "QUERY", false, mtime, size);
        assert!(cached.is_some());
    }
}
