# Phase 2: Code Improvements Based on The Rust Book

This document tracks code improvements based on Rust Book best practices for issue #140 Phase 2.

## 1. Iterator and Closure Improvements (Chapter 13)

### Opportunity 1: Use `filter_map` instead of manual loop
**File**: `src/cache/mod.rs` (lines ~363-370)

**Current code:**
```rust
let mut entries: Vec<(Vec<u8>, u64)> = Vec::new();

for (key, value) in self.db.iter().flatten() {
    if let Ok(cache_value) = bincode::deserialize::<CacheValue>(&value) {
        if now.saturating_sub(cache_value.last_accessed) <= MAX_CACHE_AGE_SECS {
            entries.push((key.to_vec(), cache_value.last_accessed));
        }
    }
}
```

**Improved with `filter_map`:**
```rust
let entries: Vec<(Vec<u8>, u64)> = self
    .db
    .iter()
    .flatten()
    .filter_map(|(key, value)| {
        bincode::deserialize::<CacheValue>(&value)
            .ok()
            .filter(|cache_value| {
                now.saturating_sub(cache_value.last_accessed) <= MAX_CACHE_AGE_SECS
            })
            .map(|cache_value| (key.to_vec(), cache_value.last_accessed))
    })
    .collect();
```

**Benefits:**
- More functional, declarative style
- No mutable variable needed
- Clearer intent: transform and filter in one pass
- Demonstrates Chapter 13.2 iterator adapters

**Educational value:**
- Shows `filter_map` for combined filter + map operations
- Demonstrates chaining Option methods
- Illustrates zero-cost abstractions

---

### Opportunity 2: Use `take_while` for early termination
**File**: `src/cache/mod.rs` (lines ~376-385)

**Current code:**
```rust
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
```

**Improved with `take_while` or early return:**
```rust
for (key, _) in entries.iter() {
    let size_ok = self
        .db
        .size_on_disk()
        .ok()
        .map(|s| s <= MAX_CACHE_SIZE)
        .unwrap_or(true);
    
    if size_ok {
        break;
    }
    
    let _ = self.db.remove(key);
}
```

**Note:** This one is tricky because we need to check size after each removal. The current code is actually reasonable. We could document why the imperative approach is better here.

---

## 2. Error Handling Improvements (Chapter 9)

### Opportunity 1: Add `#[must_use]` to Result-returning functions
**Files**: Various

**Current:** Functions return `Result` but don't have `#[must_use]`

**Improved:**
```rust
#[must_use = "this returns the result of the operation, without modifying the original"]
pub fn search(&self, text: &str) -> Result<Vec<Match>> {
    // ...
}
```

**Benefits:**
- Compiler warns if Result is ignored
- Prevents silent error swallowing
- Documents that the function doesn't have side effects

---

### Opportunity 2: Use `anyhow::Context` for error context
**Status:** Already using custom error types with context - this is good!

The current approach with `SearchError` enum variants that carry context is actually better than `anyhow` for a library. `anyhow` is more for applications.

---

## 3. Smart Pointers (Chapter 15)

### Assessment: Not needed in this codebase

**Why:**
- Most data structures use owned values (`String`, `PathBuf`, `Vec`)
- Lifetimes are used where appropriate (`CallGraphBuilder<'a>`)
- No circular references that would need `Rc<RefCell<T>>`
- No shared ownership across threads that would need `Arc<Mutex<T>>`

**Current concurrency approach is good:**
- Uses channels for message passing (no shared state)
- Thread-local accumulators
- `Mutex` only for the front cache (appropriate use)

**Educational note:** Document why we DON'T need smart pointers here - that's also valuable!

---

## 4. Concurrency Examples (Chapter 16)

### Already excellent examples!

**Current code demonstrates:**
- ✅ Message passing with channels (`src/search/text_search.rs`)
- ✅ The critical `drop(tx)` pattern
- ✅ Shared state with `Mutex` (`src/cache/mod.rs`)
- ✅ Thread-local accumulators to avoid contention

**Potential addition:**
- Could add `Arc` example if we want to share read-only data across threads
- But current design doesn't need it (good design!)

---

## Implementation Priority

1. **High Priority:**
   - Add `filter_map` improvement to cache eviction (clear win)
   - Add `#[must_use]` to public Result-returning functions
   - Document why smart pointers aren't needed (educational)

2. **Medium Priority:**
   - Review other iterator chains for improvements
   - Add more inline comments about iterator choices

3. **Low Priority:**
   - Consider adding an `Arc` example if there's a natural fit
   - Add performance benchmarks for iterator improvements

---

## Next Steps

1. Implement `filter_map` improvement in `src/cache/mod.rs`
2. Add `#[must_use]` attributes to public API
3. Add educational comments about when NOT to use certain patterns
4. Run benchmarks to verify no performance regression
5. Update documentation

