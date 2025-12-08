# Learning Rust with code-search

This guide maps concepts from [The Rust Programming Language](https://doc.rust-lang.org/book/) (The Rust Book) to real-world examples in this codebase.

## 🚀 The Book in a Nutshell (For Impatient Programmers)

**TL;DR:** Rust is about memory safety without garbage collection. Here's what you need to know:

### Core Concepts (5 minutes)

1. **Ownership** - Every value has one owner. When owner goes out of scope, value is dropped.
   ```rust
   let s = String::from("hello");  // s owns the string
   // s is dropped here, memory freed
   ```

2. **Borrowing** - You can borrow references without taking ownership.
   ```rust
   fn len(s: &String) -> usize { s.len() }  // Borrows, doesn't own
   let s = String::from("hello");
   len(&s);  // s still valid here!
   ```

3. **Mutability** - Immutable by default. Use `mut` for mutable.
   ```rust
   let x = 5;        // Immutable
   let mut y = 5;    // Mutable
   y += 1;           // OK
   ```

4. **Error Handling** - No exceptions. Use `Result<T, E>` and `?` operator.
   ```rust
   fn read_file(path: &str) -> Result<String, io::Error> {
       let contents = fs::read_to_string(path)?;  // ? propagates errors
       Ok(contents)
   }
   ```

5. **Traits** - Like interfaces. Define shared behavior.
   ```rust
   trait Summary {
       fn summarize(&self) -> String;
   }
   ```

6. **Lifetimes** - Tell compiler how long references are valid.
   ```rust
   fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
       if x.len() > y.len() { x } else { y }
   }
   ```

### Where to See These in Action

| Concept | File | What to Look For |
|---------|------|------------------|
| **Ownership** | `src/tree/builder.rs` | `.clone()` when we need to own data |
| **Borrowing** | `src/tree/builder.rs` | `&SearchResult` parameter |
| **Error Handling** | `src/error.rs` | Custom error enum with `#[derive(Error)]` |
| **Traits** | `src/tree/node.rs` | `#[derive(Debug, Clone)]` |
| **Lifetimes** | `src/trace/graph_builder.rs` | `CallGraphBuilder<'a>` |
| **Iterators** | `src/cache/mod.rs` | `filter_map()` instead of loops |
| **Concurrency** | `src/search/text_search.rs` | Channels with `mpsc::channel()` |

### The Rust Mindset

**Coming from garbage collected languages (Java, Python, JavaScript)?**
- ✅ No null pointer exceptions (use `Option<T>`)
- ✅ No data races (compiler prevents them)
- ✅ No garbage collector pauses
- ⚠️ Must think about ownership
- ⚠️ Compiler is strict (but helpful!)

**Coming from C/C++?**
- ✅ Same performance, zero-cost abstractions
- ✅ No manual memory management
- ✅ No use-after-free, no double-free
- ⚠️ Borrow checker takes getting used to
- ⚠️ Can't just cast pointers everywhere

### Quick Wins in This Codebase

**Want to see Rust's benefits immediately?**

1. **Memory safety** - Run `cargo build`. If it compiles, no memory bugs!
2. **Fearless concurrency** - See `src/search/text_search.rs` - parallel search with no data races
3. **Zero-cost abstractions** - See `src/cache/mod.rs` - `filter_map` compiles to same code as manual loop
4. **Helpful errors** - Try breaking something and see the compiler help you fix it

### 30-Second Rule of Thumb

- **Own it** (`String`, `Vec<T>`) - When you need to keep the data
- **Borrow it** (`&T`, `&mut T`) - When you just need to read/modify temporarily  
- **Clone it** (`.clone()`) - When ownership is complex and performance isn't critical
- **Use `Result`** - For errors that should be handled
- **Use `Option`** - For values that might not exist

Now dive into the detailed guide below! 👇

---

## How to Use This Guide

1. **Read a chapter** in The Rust Book
2. **Find the examples** below for that chapter
3. **Read the code** with the inline educational comments
4. **Try the exercises** to test your understanding
5. **Experiment** by modifying the code and running tests

## Quick Reference: Chapters → Files

| Chapter | Topic | Files |
|---------|-------|-------|
| 4 | Ownership & Borrowing | `src/tree/builder.rs` |
| 5 | Structs & Methods | `src/search/text_search.rs`, `src/tree/node.rs` |
| 6 | Enums & Pattern Matching | `src/tree/node.rs`, `src/error.rs` |
| 9 | Error Handling | `src/error.rs`, `src/lib.rs` |
| 10 | Generics, Traits, Lifetimes | `src/tree/node.rs`, `src/trace/graph_builder.rs`, `src/error.rs` |
| 13 | Iterators & Closures | `src/cache/mod.rs`, `src/search/text_search.rs` |
| 15 | Smart Pointers | `src/smart_pointers_analysis.md` (why NOT to use them) |
| 16 | Concurrency | `src/search/text_search.rs`, `src/cache/mod.rs` |

---

## Chapter 4: Understanding Ownership

**Rust Book:** https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html

### Concepts Demonstrated

**File:** `src/tree/builder.rs`

1. **Borrowing with `&` references**
   - The `build()` method takes `&SearchResult` instead of `SearchResult`
   - Allows reading data without taking ownership
   - Zero-cost abstraction

2. **Ownership transfer with `.clone()`**
   - When `TreeNode` needs to own its data
   - Trade-off: memory cost vs. ownership flexibility

3. **Iterators and borrowing**
   - `.iter()` creates references, not owned values
   - Original collection remains usable

### Exercise 4.1: Borrowing vs. Ownership

**Question:** In `src/tree/builder.rs`, why does `build()` take `&SearchResult` instead of `SearchResult`?

<details>
<summary>Click to see answer</summary>

**Answer:** Taking `&SearchResult` (a reference) allows:
1. The caller to keep ownership and reuse the data
2. No expensive cloning of the entire structure
3. Multiple functions to read the same data

If it took `SearchResult` (ownership), the caller would lose access to the data after calling `build()`.
</details>

### Exercise 4.2: Clone vs. Move

**Question:** In `build_translation_node()`, why do we clone `entry.key` and `entry.value`?

<details>
<summary>Click to see answer</summary>

**Answer:** 
- We borrow `entry` (don't take ownership)
- But `TreeNode` needs to own its data (must outlive the function)
- So we clone the strings we need to store
- This is more efficient than cloning the entire `TranslationEntry`
</details>

### Try It Yourself

1. Open `src/tree/builder.rs`
2. Try changing `build(result: &SearchResult)` to `build(result: SearchResult)`
3. Run `cargo check` - what errors do you see?
4. Why does the compiler complain?

---

## Chapter 5: Using Structs

**Rust Book:** https://doc.rust-lang.org/book/ch05-00-structs.html

### Concepts Demonstrated

**File:** `src/search/text_search.rs`

1. **Builder pattern with method chaining**
   - Methods take `mut self` and return `Self`
   - Enables fluent API: `TextSearcher::new(dir).case_sensitive(true).search("text")`

2. **Associated functions vs. methods**
   - `new()` is an associated function (no `self`)
   - `search()` is a method (takes `&self`)

### Exercise 5.1: Builder Pattern

**Question:** Why do builder methods take `mut self` instead of `&mut self`?

<details>
<summary>Click to see answer</summary>

**Answer:** Taking `mut self` (ownership) enables method chaining:
```rust
TextSearcher::new(dir)
    .case_sensitive(true)   // Consumes and returns Self
    .respect_gitignore(false) // Consumes and returns Self
    .search("text")          // Final method takes &self
```

With `&mut self`, you'd need to write:
```rust
let mut searcher = TextSearcher::new(dir);
searcher.case_sensitive(true);
searcher.respect_gitignore(false);
searcher.search("text")
```
</details>

### Try It Yourself

1. Open `src/search/text_search.rs`
2. Create a new builder method that sets multiple options at once
3. Test it with `cargo test --lib search::text_search`

---

## Chapter 9: Error Handling

**Rust Book:** https://doc.rust-lang.org/book/ch09-00-error-handling.html

### Concepts Demonstrated

**File:** `src/error.rs`

1. **Custom error types with `thiserror`**
   - Enum variants with associated data
   - `#[error("...")]` for Display implementation
   - `#[from]` for automatic error conversion

2. **The `?` operator**
   - Propagates errors up the call stack
   - Automatically converts error types with `From`

3. **Type aliases for Result**
   - `type Result<T> = std::result::Result<T, SearchError>`
   - Makes function signatures cleaner

### Exercise 9.1: Error Propagation

**Question:** How does `#[from]` enable the `?` operator to work with different error types?

<details>
<summary>Click to see answer</summary>

**Answer:** The `#[from]` attribute generates a `From` implementation:
```rust
impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        SearchError::Io(err)
    }
}
```

When you use `?` on an `io::Error`, Rust automatically calls `From::from()` to convert it to `SearchError`.
</details>

### Exercise 9.2: Must Use

**Question:** In `src/lib.rs`, what does `#[must_use]` do on `run_search()`?

<details>
<summary>Click to see answer</summary>

**Answer:** It causes a compiler warning if the Result is ignored:
```rust
run_search(query);  // WARNING: unused Result that must be used
```

This prevents accidentally ignoring errors. You must either:
- Handle it: `match run_search(query) { ... }`
- Propagate it: `let result = run_search(query)?;`
- Explicitly ignore: `let _ = run_search(query);`
</details>

### Try It Yourself

1. Open `src/error.rs`
2. Add a new error variant for a specific case
3. Use it somewhere in the codebase
4. Run tests to ensure it works

---

## Chapter 10: Generic Types, Traits, and Lifetimes

**Rust Book:** https://doc.rust-lang.org/book/ch10-00-generics.html

### Concepts Demonstrated

**Traits:** `src/tree/node.rs`
**Lifetimes:** `src/trace/graph_builder.rs`
**Generics:** `src/error.rs`

1. **Derive macros for traits**
   - `#[derive(Debug, Clone, PartialEq, Eq)]`
   - Compiler-generated implementations

2. **Lifetime parameters**
   - `CallGraphBuilder<'a>` holds references with lifetime `'a`
   - Prevents dangling references at compile time

3. **`impl Into<T>` for flexible APIs**
   - Accepts multiple types that can convert to `T`
   - Makes APIs more ergonomic

### Exercise 10.1: Derive Traits

**Question:** In `src/tree/node.rs`, why doesn't `TreeNode` derive `PartialEq`?

<details>
<summary>Click to see answer</summary>

**Answer:** 
- `TreeNode` contains `Vec<TreeNode>` (recursive structure)
- Comparing entire trees could be expensive
- We don't need tree equality in this application
- Omitting `PartialEq` prevents accidental expensive comparisons
</details>

### Exercise 10.2: Lifetimes

**Question:** What does `CallGraphBuilder<'a>` mean?

<details>
<summary>Click to see answer</summary>

**Answer:** The `'a` is a lifetime parameter that says:
- This struct holds references that live for lifetime `'a`
- The struct cannot outlive the data it references
- Rust enforces this at compile time (no runtime cost)

Without lifetimes, you'd need `Box` or `Rc` (heap allocation + runtime overhead).
</details>

### Try It Yourself

1. Open `src/trace/graph_builder.rs`
2. Try removing the `<'a>` lifetime parameter
3. Run `cargo check` - what errors do you see?
4. Why does Rust need the lifetime annotation?

---

## Chapter 13: Functional Language Features

**Rust Book:** https://doc.rust-lang.org/book/ch13-00-functional-features.html

### Concepts Demonstrated

**File:** `src/cache/mod.rs`, `src/search/text_search.rs`

1. **Iterator adapters**
   - `filter_map()` combines filtering and mapping
   - More efficient than separate `filter()` and `map()`

2. **Closures capturing environment**
   - `move` closures transfer ownership to threads
   - Cloning for shared access

### Exercise 13.1: filter_map

**Question:** In `src/cache/mod.rs`, why is `filter_map` better than a manual loop?

<details>
<summary>Click to see answer</summary>

**Answer:** 
- More functional, declarative style
- No mutable variable needed
- Clearer intent: transform and filter in one pass
- Demonstrates iterator adapters
- Often optimized better by the compiler
</details>

### Exercise 13.2: Move Closures

**Question:** In `src/search/text_search.rs`, why do we need `move` in the closure?

<details>
<summary>Click to see answer</summary>

**Answer:** The `move` keyword forces the closure to take ownership of captured variables:
```rust
Box::new(move |entry| {
    // tx and matcher are moved into this closure
    // This closure will run in a different thread
})
```

Without `move`, the closure would try to borrow, which doesn't work across threads (the thread might outlive the borrowed data).
</details>

### Try It Yourself

1. Open `src/cache/mod.rs`
2. Find the `filter_map` usage
3. Try rewriting it as a manual loop
4. Compare the two approaches - which is clearer?

---

## Chapter 15: Smart Pointers

**Rust Book:** https://doc.rust-lang.org/book/ch15-00-smart-pointers.html

### Concepts Demonstrated

**File:** `src/smart_pointers_analysis.md`

**Important:** This codebase demonstrates when NOT to use smart pointers!

1. **Why no `Box<T>`?**
   - Types like `PathBuf` are already heap-allocated
   - Ownership is clear without `Box`

2. **Why no `Rc<T>`?**
   - Lifetimes handle borrowing with zero cost
   - No need for reference counting

3. **Why no `Arc<Mutex<T>>` everywhere?**
   - Message passing (channels) is better for producer-consumer
   - Minimal shared state reduces complexity

### Exercise 15.1: When to Use Smart Pointers

**Question:** When SHOULD you use `Box<T>`?

<details>
<summary>Click to see answer</summary>

**Answer:** Use `Box<T>` when you need:
1. Recursive types (like linked lists): `Box<List>` breaks infinite size
2. Trait objects: `Box<dyn Trait>` for dynamic dispatch
3. Moving large data without copying: `Box::new([0u8; 1_000_000])`
</details>

### Try It Yourself

1. Read `src/smart_pointers_analysis.md`
2. Think about your own projects - do you overuse smart pointers?
3. Could you use simpler alternatives?

---

## Chapter 16: Fearless Concurrency

**Rust Book:** https://doc.rust-lang.org/book/ch16-00-concurrency.html

### Concepts Demonstrated

**Message Passing:** `src/search/text_search.rs`
**Shared State:** `src/cache/mod.rs`

1. **Channels for message passing**
   - `mpsc::channel()` for thread communication
   - The critical `drop(tx)` pattern
   - Thread-local accumulators

2. **Mutex for shared state**
   - `Mutex<HashMap>` for thread-safe cache
   - Lock held for minimal time
   - Appropriate use case

### Exercise 16.1: The drop(tx) Pattern

**Question:** In `src/search/text_search.rs`, why is `drop(tx)` critical?

<details>
<summary>Click to see answer</summary>

**Answer:** 
- We clone `tx` for each worker thread
- But we still have the original `tx` in the main thread
- The receiver's iterator only ends when ALL senders are dropped
- Without `drop(tx)`, the receiver would wait forever!

```rust
let (tx, rx) = mpsc::channel();
// Clone tx for workers...
drop(tx);  // Drop original so rx.iter() can terminate
for result in rx { ... }  // This loop ends when all tx are dropped
```
</details>

### Exercise 16.2: Mutex vs. Channels

**Question:** When should you use `Mutex` vs. channels?

<details>
<summary>Click to see answer</summary>

**Answer:** 

**Use channels when:**
- Producer-consumer pattern
- Can accumulate results locally
- Want to avoid lock contention

**Use `Mutex` when:**
- Need shared mutable state
- Small, infrequent updates
- Lock duration is minimal
- Simpler than alternatives

This codebase uses both appropriately!
</details>

### Try It Yourself

1. Open `src/search/text_search.rs`
2. Try commenting out the `drop(tx)` line
3. Run a search - what happens?
4. Why does it hang?

---

## Learning Paths

### Beginner (Chapters 1-8)

1. **Start here:** `src/main.rs` - See how a Rust program is structured
2. **Then:** `src/error.rs` - Understand Result and error types
3. **Next:** `src/tree/node.rs` - Simple struct definitions

### Intermediate (Chapters 9-13)

1. **Error handling:** `src/error.rs` - Custom errors, `?` operator
2. **Builder pattern:** `src/search/text_search.rs` - Method chaining
3. **Ownership:** `src/tree/builder.rs` - Borrowing in complex structures

### Advanced (Chapters 14-16)

1. **Concurrency:** `src/search/text_search.rs` - Channels and parallel search
2. **Lifetimes:** `src/trace/graph_builder.rs` - Lifetime parameters
3. **Smart pointers:** `src/smart_pointers_analysis.md` - When NOT to use them

---

## Additional Exercises

### Exercise: Add a Feature

Try adding a new feature to practice what you've learned:

1. **Add a new search option** to `TextSearcher`
   - Practice: Builder pattern (Chapter 5)
   - Hint: Follow the pattern of `case_sensitive()`

2. **Add a new error variant** to `SearchError`
   - Practice: Error handling (Chapter 9)
   - Hint: Use `#[error("...")]` for the message

3. **Add a new node type** to the tree
   - Practice: Enums and pattern matching (Chapter 6)
   - Hint: Update all match statements

### Exercise: Refactor

Try refactoring existing code:

1. **Convert a manual loop to iterator methods**
   - Practice: Iterators (Chapter 13)
   - Look for `for` loops that build collections

2. **Add documentation to an undocumented function**
   - Practice: Documentation (Chapter 14)
   - Include examples and Rust Book references

---

## Getting Help

- **Questions?** Open an issue with the `learning-question` label
- **Found a bug?** Open an issue with the `bug` label
- **Want to contribute?** See `CONTRIBUTING.md`

## Additional Resources

- [The Rust Book](https://doc.rust-lang.org/book/) - Official guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings) - Small exercises
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Best practices

Happy learning! 🦀
