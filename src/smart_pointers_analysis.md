# Smart Pointers Analysis - Rust Book Chapter 15

This document explains why this codebase does NOT extensively use smart pointers from Chapter 15, which is also an important lesson.

## Rust Book Chapter 15: Smart Pointers

Smart pointers are data structures that act like pointers but have additional metadata and capabilities:
- `Box<T>` - Heap allocation with single ownership
- `Rc<T>` - Reference counting for shared ownership
- `Arc<T>` - Atomic reference counting for thread-safe shared ownership
- `RefCell<T>` - Interior mutability with runtime borrow checking

## Why This Codebase Doesn't Need Them

### 1. Ownership is Clear and Simple

**Current approach:**
```rust
pub struct TextSearcher {
    base_dir: PathBuf,        // Owned
    globs: Vec<String>,       // Owned
    exclusions: Vec<String>,  // Owned
}
```

**Why no `Box<PathBuf>`?**
- `PathBuf` is already heap-allocated internally
- Adding `Box` would add an extra indirection with no benefit
- Ownership is clear: the struct owns its data

**Lesson:** Don't use `Box` just because data is "large" - use it when you need:
- Recursive types (like linked lists)
- Trait objects (`Box<dyn Trait>`)
- Moving large data without copying

### 2. Lifetimes Handle Borrowing

**Current approach:**
```rust
pub struct CallGraphBuilder<'a> {
    finder: &'a mut FunctionFinder,
    extractor: &'a CallExtractor,
}
```

**Why no `Rc<FunctionFinder>`?**
- Lifetimes ensure references are valid
- Zero runtime cost
- Ownership is clear: caller owns the data

**Alternative with `Rc`:**
```rust
pub struct CallGraphBuilder {
    finder: Rc<RefCell<FunctionFinder>>,  // Runtime overhead!
    extractor: Rc<CallExtractor>,
}
```

**Downsides:**
- Reference counting has runtime cost
- `RefCell` adds runtime borrow checking
- Less clear ownership semantics
- Can create reference cycles (memory leaks)

**Lesson:** Prefer lifetimes over `Rc` when possible. Use `Rc` only when:
- Multiple owners need to outlive each other
- Ownership graph is dynamic (like a GUI tree)
- Lifetimes become too complex

### 3. Message Passing Instead of Shared State

**Current approach (Chapter 16.2):**
```rust
pub fn search(&self, text: &str) -> Result<Vec<Match>> {
    let (tx, rx) = mpsc::channel();
    
    walk_builder.build_parallel().run(|| {
        let tx = tx.clone();  // Clone sender
        Box::new(move |entry| {
            // Thread-local accumulator
            let mut file_matches = Vec::new();
            // ... search ...
            let _ = tx.send(file_matches);  // Send to main thread
        })
    });
    
    // Collect results
    for file_matches in rx {
        all_matches.extend(file_matches);
    }
}
```

**Why no `Arc<Mutex<Vec<Match>>>`?**
- Message passing is safer and clearer
- No lock contention
- Each thread has its own accumulator
- Main thread collects results

**Alternative with shared state:**
```rust
let results = Arc::new(Mutex::new(Vec::new()));

walk_builder.build_parallel().run(|| {
    let results = Arc::clone(&results);
    Box::new(move |entry| {
        // ... search ...
        results.lock().unwrap().extend(file_matches);  // Lock contention!
    })
});
```

**Downsides:**
- Lock contention slows down parallel code
- More complex error handling (poisoned locks)
- Harder to reason about

**Lesson:** Prefer message passing (channels) over shared state (`Arc<Mutex<T>>`) when:
- You have a producer-consumer pattern
- You can accumulate results locally
- You want to avoid lock contention

### 4. When We DO Use `Mutex`

**Current approach:**
```rust
pub struct LocalCache {
    front_cache: Mutex<HashMap<Vec<u8>, CacheValue>>,
    // ...
}
```

**Why `Mutex` here?**
- Small, infrequent updates (cache hits/misses)
- Need to share mutable state across method calls
- Lock is held for very short time
- No alternative that's simpler

**This is appropriate use of `Mutex`:**
- Not in a hot loop
- Lock scope is minimal
- Clear ownership (struct owns the Mutex)

## When You SHOULD Use Smart Pointers

### Use `Box<T>` when:
1. **Recursive types:**
   ```rust
   enum List {
       Cons(i32, Box<List>),  // Box breaks infinite size
       Nil,
   }
   ```

2. **Trait objects:**
   ```rust
   fn get_formatter() -> Box<dyn Formatter> {
       Box::new(JsonFormatter::new())
   }
   ```

3. **Large stack allocations:**
   ```rust
   let huge_array = Box::new([0u8; 1_000_000]);  // Heap, not stack
   ```

### Use `Rc<T>` when:
1. **Multiple owners with unknown lifetimes:**
   ```rust
   let shared_config = Rc::new(Config::load());
   let worker1 = Worker::new(Rc::clone(&shared_config));
   let worker2 = Worker::new(Rc::clone(&shared_config));
   ```

2. **Graph structures:**
   ```rust
   struct Node {
       value: i32,
       children: Vec<Rc<Node>>,  // Multiple parents possible
   }
   ```

### Use `Arc<T>` when:
1. **Sharing read-only data across threads:**
   ```rust
   let config = Arc::new(Config::load());
   for _ in 0..4 {
       let config = Arc::clone(&config);
       thread::spawn(move || {
           // Use config (read-only)
       });
   }
   ```

### Use `RefCell<T>` when:
1. **Interior mutability in single-threaded code:**
   ```rust
   struct Cache {
       data: RefCell<HashMap<String, String>>,
   }
   
   impl Cache {
       fn get(&self, key: &str) -> Option<String> {
           self.data.borrow().get(key).cloned()
       }
   }
   ```

## Summary

**This codebase demonstrates:**
- ✅ Clear ownership with owned types
- ✅ Borrowing with lifetimes (zero cost)
- ✅ Message passing for concurrency
- ✅ Minimal use of `Mutex` where appropriate

**Key lesson:** Smart pointers are powerful, but not always necessary. Prefer:
1. Owned types (`String`, `Vec`, `PathBuf`)
2. Lifetimes (`&T`, `&mut T`)
3. Message passing (channels)
4. Smart pointers only when the above don't work

**This is idiomatic Rust!** Using the simplest solution that works.

## References

- [Rust Book Chapter 15: Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Rust Book Chapter 16: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust API Guidelines: Ownership](https://rust-lang.github.io/api-guidelines/flexibility.html)
