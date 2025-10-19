# Tester Design Document

## Overview

Tester is a functional abstraction for testing whether a state or condition
holds. It takes no parameters and returns a boolean value, representing the
test result of some state or condition.

## Core Semantics

### Definition of Tester

```rust
/// Tester - Tests whether a state or condition holds
///
/// Tester encapsulates the logic for "testing some state or condition", takes
/// no parameters, and returns a boolean value.
/// It captures the context information needed for testing through closures.
pub trait Tester {
    /// Execute the test and return the test result
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition holds, otherwise returns `false`
    fn test(&self) -> bool;
}
```

### Core Characteristics

- **No input parameters**: Captures context through closures
- **Returns boolean value**: Represents test result
- **Uses `&self`**: Does not modify its own state, only reads external state
- **Repeatable calls**: The same Tester can call `test()` multiple times

### Comparison of Tester with Other Functional Abstractions

| Abstraction | Input | Output | self signature | Core semantics | Closure signature |
|-------------|-------|--------|----------------|----------------|-------------------|
| **Tester** | None | `bool` | `&self` | Test state/condition | `Fn() -> bool` |
| **Predicate** | `&T` | `bool` | `&self` | Judge value | `Fn(&T) -> bool` |
| **Supplier** | None | `T` | `&mut self` | Generate value | `FnMut() -> T` |
| **Consumer** | `T` | None | `&mut self` | Consume value | `FnMut(T)` |

**Key Insights**:
- **Tester is similar to Predicate**: Both are "judgment/test" abstractions,
  use `&self`, and don't modify their own state
- **Tester differs from Supplier**: Although both take no input, Supplier may
  generate different values and needs `&mut self`; while Tester only reads
  state to judge conditions, using `&self`
- **State management is the caller's responsibility**: Tester is only
  responsible for "judging", not "managing state"

### Main Uses of Tester

The core value of Tester types lies in:

1. **Encapsulating condition judgment logic**
   ```rust
   let ready = Arc::new(AtomicBool::new(false));
   let ready_clone = Arc::clone(&ready);
   let tester = BoxTester::new(move || {
       ready_clone.load(Ordering::Acquire)
   });
   ```

2. **Supporting dependency injection**
   ```rust
   struct Executor {
       precondition: BoxTester,
   }

   impl Executor {
       fn execute(&self) {
           if self.precondition.test() {
               // Execute task
           }
       }
   }
   ```

3. **Abstracting retry logic**
   ```rust
   fn wait_until(tester: &dyn Tester, timeout: Duration) -> bool {
       let start = Instant::now();
       while !tester.test() {
           if start.elapsed() > timeout {
               return false;
           }
           thread::sleep(Duration::from_millis(100));
       }
       true
   }
   ```

4. **Combining condition judgments**
   ```rust
   let check1 = BoxTester::new(|| database_alive());
   let check2 = BoxTester::new(|| cache_ready());
   let combined = check1.and(check2);
   ```

## Core Design Decisions

### 1. Mutability of self: Using `&self`

**Core Question**: Should Tester use `&self` or `&mut self`?

**Answer**: **Should use `&self`**

#### Design Philosophy

Tester's responsibility is "testing and judging", not "state management":

```rust
// ✅ Correct understanding: Tester only responsible for judging
pub trait Tester {
    fn test(&self) -> bool;  // Read-only state, doesn't modify itself
}

// ✅ State management is the caller's responsibility
let count = Arc::new(AtomicUsize::new(0));
let count_clone = Arc::clone(&count);
let tester = BoxTester::new(move || {
    count_clone.load(Ordering::Relaxed) < 3  // Read-only state
});

// Caller manages state based on test results
loop {
    if !tester.test() {
        break;
    }
    if execute_task().is_ok() {
        count.fetch_add(1, Ordering::Relaxed);  // Caller updates state
    }
}
```

#### Consistency with Predicate

| Feature | Predicate | Tester |
|---------|-----------|--------|
| **Input** | Takes parameter `&T` | Takes no parameters |
| **Output** | `bool` | `bool` |
| **self signature** | `&self` | `&self` |
| **Core semantics** | Judge a value | Test a state |
| **Modify itself?** | ❌ No | ❌ No |
| **Typical scenarios** | Filtering, validation | State checking, condition waiting |

**Key Insight**: Predicate and Tester are both "judgment/test" abstractions,
semantically they should not modify their own state.

#### Real-world Scenario Analysis

Let's see who should manage state in typical scenarios:

**Scenario 1: Retry limits**
```rust
// ❌ Wrong: Tester manages state
let mut count = 0;
let mut tester = BoxTester::new(move || {
    count += 1;  // Problem: side effect of test()
    count <= 3
});

// ✅ Correct: Caller manages state
let max_attempts = 3;
let mut attempts = 0;
let tester = BoxTester::new(move || attempts < max_attempts);

while tester.test() {
    if execute_task().is_ok() {
        break;
    }
    attempts += 1;  // Caller controls when to count
}
```

**Scenario 2: Caching strategy**
```rust
// ❌ Wrong: Tester manages cache
let mut cached = None;
let mut tester = BoxTester::new(move || {
    if cached.is_none() {
        cached = Some(expensive_check());  // Problem: fixed caching strategy
    }
    cached.unwrap()
});

// ✅ Correct: Dedicated struct manages cache
struct CachedChecker {
    cached: Option<bool>,
    last_check: Instant,
    ttl: Duration,
}

impl CachedChecker {
    fn check(&mut self) -> bool {
        if self.should_refresh() {
            self.cached = Some(expensive_check());
            self.last_check = Instant::now();
        }
        self.cached.unwrap()
    }
}

// Tester is just an interface
let checker = Arc::new(Mutex::new(CachedChecker::new()));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});
```

**Scenario 3: Condition waiting (typical scenario)**
```rust
// ✅ Tester only responsible for judging conditions
let ready = Arc::new(AtomicBool::new(false));
let ready_clone = Arc::clone(&ready);
let tester = BoxTester::new(move || {
    ready_clone.load(Ordering::Acquire)
});

// Caller controls waiting strategy
let timeout = Duration::from_secs(30);
let start = Instant::now();
while !tester.test() {
    if start.elapsed() > timeout {
        return Err(TimeoutError);
    }
    thread::sleep(Duration::from_millis(100));
}
```

**Scenario 4: DoubleCheckedLockExecutor**
```rust
// ✅ Using &self semantics is correct
struct DoubleCheckedLockExecutor {
    tester: BoxTester,
}

impl DoubleCheckedLockExecutor {
    pub fn execute<T, F>(&self, mutex: &Mutex<T>, task: F)
    where F: FnOnce(&mut T) -> Result<(), Error>
    {
        // First check (lock-free)
        if !self.tester.test() {
            return Err(PreconditionFailed);
        }

        // Acquire lock
        let mut guard = mutex.lock().unwrap();

        // Second check (with lock)
        if !self.tester.test() {
            return Err(PreconditionFailed);
        }

        // Execute task
        task(&mut guard)
    }
}
```

#### Final Conclusion

**Reasons for using `&self`**:

1. **Clear responsibility**: Tester only responsible for "judging", state
   management is caller's responsibility
2. **Consistent with Predicate**: Both are judgment abstractions, both use
   `&self`
3. **Semantically correct**: Struct methods containing Tester can be `&self`
   (like `Executor::execute(&self)`)
4. **Repeatable calls**: Same Tester can be called multiple times without side
   effects
5. **Consistent with Java**: Corresponds to Java's `BooleanSupplier`
   (`Fn() -> bool`)
6. **Performance advantage**: Using `Fn()` doesn't need `Mutex`/`RefCell`,
   can be called concurrently

**Reasons for not using `&mut self`**:

1. **Violates single responsibility**: Makes Tester both judge and manage state
2. **Implicit side effects**: Calling `test()` changes state, caller can't
   control
3. **Hard to compose**: Can't use in immutable contexts
4. **Limits concurrency**: Can't share calls across threads

### 2. Necessity of TesterOnce

**Key Question**: Is `TesterOnce` trait needed?

```rust
pub trait TesterOnce {
    fn test(self) -> bool;  // Consumes self
}
```

#### Possible Use Cases

1. **One-time resource checking**: Check resource and consume it
2. **Delayed boolean computation**: Delay computing an expensive boolean value

```rust
// Scenario example
let resource = ExpensiveResource::new();
let once_tester = BoxTesterOnce::new(move || {
    resource.validate()  // Consumes resource
});

// Can only call once
let result = once_tester.test();
```

#### Analysis

**Problems**:
1. **Rare use cases**: Most tests need multiple calls
2. **Better alternatives**:
   ```rust
   // Instead of TesterOnce, use closure directly
   let check = || expensive_resource.validate();
   let result = check();

   // Or call directly
   let result = expensive_resource.validate();
   ```
3. **Doesn't match typical usage**: Tester's typical scenarios (condition
   waiting, retry) all need multiple calls

#### Conclusion

**Don't implement TesterOnce**

**Reasons**:
1. ❌ Very few use cases
2. ❌ Using closures or function calls directly is simpler
3. ❌ Doesn't match Tester's core purpose (repeatable testing)
4. ✅ Keep API simple

## Comparison of Three Implementation Approaches

### Approach 1: Type Alias + Static Composition Methods

Use type alias to define Tester type and provide helper methods through
static utility class.

```rust
// Type alias definition
pub type Tester = Box<dyn Fn() -> bool>;

// Static utility class provides helper methods
pub struct Testers;

impl Testers {
    pub fn from<F>(func: F) -> Tester
    where
        F: Fn() -> bool + 'static,
    {
        Box::new(func)
    }

    pub fn and(t1: Tester, t2: Tester) -> Tester {
        Box::new(move || t1() && t2())
    }

    pub fn or(t1: Tester, t2: Tester) -> Tester {
        Box::new(move || t1() || t2())
    }

    pub fn not(t: Tester) -> Tester {
        Box::new(move || !t())
    }
}

// Usage example
let test1 = Testers::from(|| check_condition());
let test2 = Testers::from(|| another_check());
let combined = Testers::and(test1, test2);
```

**Advantages**:
- ✅ Simple and direct, easy to understand
- ✅ Fully compatible with closures
- ✅ No runtime overhead

**Disadvantages**:
- ❌ Composition methods consume original Tester, can't chain calls
- ❌ API not fluent enough
- ❌ Hard to extend to Rc/Arc versions

**Evaluation**: Suitable for simple scenarios, but not flexible enough.

---

### Approach 2: Pure Trait (No Concrete Implementation)

Only provide trait definition, users implement themselves or use closures
directly.

```rust
pub trait Tester {
    fn test(&self) -> bool;
}

// Implement Tester for closures
impl<F> Tester for F
where
    F: Fn() -> bool,
{
    fn test(&self) -> bool {
        self()
    }
}

// Usage example
fn wait_until(tester: &dyn Tester) {
    while !tester.test() {
        thread::sleep(Duration::from_millis(100));
    }
}

// Pass closure directly
wait_until(&|| check_condition());

// Or custom implementation
struct MyTester;
impl Tester for MyTester {
    fn test(&self) -> bool {
        check_condition()
    }
}
```

**Advantages**:
- ✅ Maximum flexibility
- ✅ Fully compatible with closures
- ✅ No additional abstraction

**Disadvantages**:
- ❌ No convenient wrapper types (Box/Rc/Arc)
- ❌ No composition methods
- ❌ Users need to handle ownership themselves

**Evaluation**: Too simple, lacks common tools.

---

### Approach 3: Trait Abstraction + Multiple Implementations (Recommended) ⭐

Provide trait definition and multiple concrete implementations
(BoxTester, RcTester, ArcTester), each providing composition methods.

```rust
// Trait definition
pub trait Tester {
    fn test(&self) -> bool;
}

// Box implementation (single ownership)
pub struct BoxTester {
    func: Box<dyn Fn() -> bool>,
}

impl BoxTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }

    pub fn and(self, other: BoxTester) -> BoxTester {
        let func1 = self.func;
        let func2 = other.func;
        BoxTester::new(move || func1() && func2())
    }

    pub fn or(self, other: BoxTester) -> BoxTester {
        let func1 = self.func;
        let func2 = other.func;
        BoxTester::new(move || func1() || func2())
    }

    pub fn not(self) -> BoxTester {
        let func = self.func;
        BoxTester::new(move || !func())
    }

    // Conversion methods
    pub fn into_rc(self) -> RcTester { /* ... */ }
    pub fn into_arc(self) -> ArcTester { /* ... */ }
}

impl Tester for BoxTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

// Rc implementation (single-threaded shared ownership)
pub struct RcTester {
    func: Rc<dyn Fn() -> bool>,
}

impl RcTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        Self {
            func: Rc::new(func),
        }
    }

    // Similar and, or, not methods
    // Conversion methods
}

impl Tester for RcTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

impl Clone for RcTester {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// Arc implementation (multi-threaded shared ownership)
pub struct ArcTester {
    func: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl ArcTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
        }
    }

    // Similar and, or, not methods
    // Conversion methods
}

impl Tester for ArcTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

impl Clone for ArcTester {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// Implement Tester for closures
impl<F> Tester for F
where
    F: Fn() -> bool,
{
    fn test(&self) -> bool {
        self()
    }
}
```

**Usage Examples**:

```rust
// Single ownership
let test1 = BoxTester::new(|| check_db());
let test2 = BoxTester::new(|| check_cache());
let combined = test1.and(test2);

if combined.test() {
    execute_task();
}

// Single-threaded sharing
let test = RcTester::new(|| system_ready());
let test_clone = test.clone();

thread_1(test);
thread_2(test_clone);

// Multi-threaded sharing
let test = ArcTester::new(|| atomic_flag.load(Ordering::Acquire));
let test_clone = test.clone();

thread::spawn(move || {
    while !test_clone.test() {
        thread::sleep(Duration::from_millis(100));
    }
});

// Use closures directly (through Tester trait)
fn wait_until(tester: &dyn Tester) {
    while !tester.test() {
        thread::sleep(Duration::from_millis(100));
    }
}

wait_until(&|| check_condition());
```

**Advantages**:
- ✅ Provides multiple ownership models (Box/Rc/Arc)
- ✅ Each implementation has convenient composition methods
- ✅ Supports ownership conversion
- ✅ Compatible with closures (through trait)
- ✅ Type-safe concurrency control (ArcTester requires Send + Sync)
- ✅ Fluent API, chainable calls

**Disadvantages**:
- ⚠️ More implementation code
- ⚠️ Three types might confuse users

**Evaluation**: Most complete approach, suitable for production.

---

### Implementation Approach Comparison Summary

| Feature | Approach 1: Type Alias | Approach 2: Pure Trait | Approach 3: Trait + Implementation ⭐ |
|---------|------------------------|------------------------|--------------------------------------|
| **Usability** | Medium | Low | High |
| **Ownership Model** | Box only | None | Box/Rc/Arc |
| **Composition Methods** | Static methods | None | Instance methods |
| **Chainable Calls** | ❌ | ❌ | ✅ |
| **Closure Compatibility** | ✅ | ✅ | ✅ |
| **Type Safety** | ✅ | ✅ | ✅ |
| **Concurrency Control** | None | None | ✅ (ArcTester) |
| **Implementation Complexity** | Low | Low | High |
| **Recommendation** | ⭐⭐ | ⭐ | ⭐⭐⭐ |

**Final Recommendation**: **Approach 3 (Trait Abstraction + Multiple
Implementations)**

**Reasons**:
1. Provides complete ownership model support
2. Each implementation has convenient APIs
3. Type-safe concurrency control
4. Consistent with other functional abstractions (Predicate, Consumer,
   Supplier)

## Design Principles Summary

### Core Principles

1. **Single Responsibility**: Tester only responsible for "judging conditions",
   not "managing state"
2. **Immutability**: Uses `&self`, doesn't modify its own state
3. **Repeatability**: Same Tester can be called multiple times
4. **Consistent with Predicate**: Both are judgment abstractions, both use
   `&self`

### Design Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| **self mutability** | `&self` | Clear responsibility, consistent with Predicate |
| **TesterOnce** | Don't implement | Very few use cases, direct closure usage is better |
| **Implementation approach** | Trait + Multiple implementations | Complete ownership model, convenient APIs |
| **Ownership model** | Box/Rc/Arc | Covers all use cases |
| **Closure signature** | `Fn() -> bool` | No need to modify state |

### Relationship between Tester and Other Functional Abstractions

```
Judgment abstractions (using &self):
├── Predicate<T>: Fn(&T) -> bool   // Judge a value
└── Tester:       Fn() -> bool     // Test a state

Generation/Consumption abstractions (using &mut self):
├── Supplier<T>:     FnMut() -> T     // Generate value
├── Consumer<T>:      FnMut(T)         // Consume value
└── Transformer<T,R>: FnMut(T) -> R    // Transform value
```

### Current Implementation Status

| Component | Status | Description |
|-----------|--------|-------------|
| **Tester trait** | ✅ Implemented | Core trait, uses `&self` |
| **BoxTester** | ✅ Implemented | Single ownership implementation |
| **RcTester** | ✅ Implemented | Single-threaded shared ownership |
| **ArcTester** | ✅ Implemented | Multi-threaded shared ownership |
| **Closure implementation** | ✅ Implemented | `impl<F: Fn() -> bool> Tester for F` |
| **Composition methods** | ✅ Implemented | and, or, not |
| **Conversion methods** | ✅ Implemented | into_box, into_rc, into_arc |

## Usage Scenario Examples

### 1. Health Check

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

struct HealthChecker {
    database: Arc<Database>,
    cache: Arc<Cache>,
}

impl HealthChecker {
    fn create_health_tester(&self) -> BoxTester {
        let db = Arc::clone(&self.database);
        let cache = Arc::clone(&self.cache);

        BoxTester::new(move || {
            db.is_alive() && cache.is_connected()
        })
    }
}

// Usage
let checker = HealthChecker::new();
let health_test = checker.create_health_tester();

if health_test.test() {
    println!("System is healthy");
} else {
    println!("System is unhealthy");
}
```

### 2. Condition Waiting

```rust
use prism3_rust_function::tester::{ArcTester, Tester};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

fn wait_until(tester: &dyn Tester, timeout: Duration) -> bool {
    let start = Instant::now();
    while !tester.test() {
        if start.elapsed() > timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(100));
    }
    true
}

// Usage
let ready = Arc::new(AtomicBool::new(false));
let ready_clone = Arc::clone(&ready);
let tester = ArcTester::new(move || {
    ready_clone.load(Ordering::Acquire)
});

// Another thread sets the flag
let ready_clone2 = Arc::clone(&ready);
thread::spawn(move || {
    thread::sleep(Duration::from_secs(2));
    ready_clone2.store(true, Ordering::Release);
});

// Wait for condition to hold
if wait_until(&tester, Duration::from_secs(5)) {
    println!("Condition met!");
} else {
    println!("Timeout!");
}
```

### 3. Retry Limits

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

fn retry_with_limit<F>(task: F, max_attempts: usize) -> Result<(), Error>
where
    F: Fn() -> Result<(), Error>,
{
    let mut attempts = 0;
    let should_retry = BoxTester::new(move || attempts < max_attempts);

    loop {
        match task() {
            Ok(_) => return Ok(()),
            Err(e) if should_retry.test() => {
                attempts += 1;
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}

// Usage
retry_with_limit(|| {
    send_request()
}, 3)?;
```

### 4. Caching Test Results

```rust
use prism3_rust_function::tester::{BoxTester, Tester};
use std::time::{Duration, Instant};

struct CachedChecker {
    cached_result: Option<bool>,
    last_check_time: Instant,
    cache_duration: Duration,
}

impl CachedChecker {
    fn new(cache_duration: Duration) -> Self {
        Self {
            cached_result: None,
            last_check_time: Instant::now() - cache_duration,
            cache_duration,
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        if self.cached_result.is_none()
            || now.duration_since(self.last_check_time) > self.cache_duration {
            // Cache expired, recheck
            self.cached_result = Some(expensive_health_check());
            self.last_check_time = now;
        }
        self.cached_result.unwrap()
    }
}

// Use Tester wrapper
let checker = Arc::new(Mutex::new(CachedChecker::new(Duration::from_secs(60))));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});

// Call tester multiple times, caching strategy managed by CachedChecker
for _ in 0..10 {
    if tester.test() {
        println!("Health check passed");
    }
    thread::sleep(Duration::from_secs(5));
}
```

### 5. Logical Composition

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

let db_alive = BoxTester::new(|| check_database());
let cache_ready = BoxTester::new(|| check_cache());
let disk_ok = BoxTester::new(|| check_disk_space());

// AND composition
let all_healthy = db_alive
    .and(cache_ready)
    .and(disk_ok);

if all_healthy.test() {
    println!("All systems operational");
}

// OR composition
let db_check = BoxTester::new(|| check_primary_db());
let backup_check = BoxTester::new(|| check_backup_db());
let any_db_alive = db_check.or(backup_check);

// NOT composition
let maintenance_mode = BoxTester::new(|| is_maintenance());
let not_maintenance = maintenance_mode.not();

// Complex composition
let can_serve = any_db_alive.and(not_maintenance);
```

### 6. Multi-threaded Shared Checking

```rust
use prism3_rust_function::tester::{ArcTester, Tester};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

let shutdown = Arc::new(AtomicBool::new(false));
let shutdown_clone = Arc::clone(&shutdown);
let should_continue = ArcTester::new(move || {
    !shutdown_clone.load(Ordering::Acquire)
});

// Multiple worker threads share the same tester
for i in 0..4 {
    let tester = should_continue.clone();
    thread::spawn(move || {
        while tester.test() {
            // Execute work
            println!("Worker {} running", i);
            thread::sleep(Duration::from_millis(500));
        }
        println!("Worker {} shutdown", i);
    });
}

// Main thread controls shutdown
thread::sleep(Duration::from_secs(3));
shutdown.store(true, Ordering::Release);
```

### 7. Precondition Checking

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

struct Executor {
    precondition: BoxTester,
}

impl Executor {
    fn new(precondition: BoxTester) -> Self {
        Self { precondition }
    }

    fn execute<F>(&self, task: F) -> Result<(), Error>
    where
        F: FnOnce() -> Result<(), Error>,
    {
        if !self.precondition.test() {
            return Err(Error::PreconditionFailed);
        }
        task()
    }
}

// Usage
let authenticated = Arc::new(AtomicBool::new(false));
let auth_clone = Arc::clone(&authenticated);
let precondition = BoxTester::new(move || {
    auth_clone.load(Ordering::Acquire)
});

let executor = Executor::new(precondition);

// Try to execute (fails because not authenticated)
executor.execute(|| {
    println!("Executing sensitive operation");
    Ok(())
})?;  // Returns PreconditionFailed

// Execute after authentication
authenticated.store(true, Ordering::Release);
executor.execute(|| {
    println!("Executing sensitive operation");
    Ok(())
})?;  // Success
```

### 8. State Change Detection

```rust
use prism3_rust_function::tester::BoxTester;

struct ChangeDetector {
    last_value: i32,
    source: Arc<AtomicI32>,
}

impl ChangeDetector {
    fn new(source: Arc<AtomicI32>) -> Self {
        let initial = source.load(Ordering::Relaxed);
        Self {
            last_value: initial,
            source,
        }
    }

    fn has_changed(&mut self) -> bool {
        let current = self.source.load(Ordering::Relaxed);
        if current != self.last_value {
            self.last_value = current;
            true
        } else {
            false
        }
    }
}

// Usage
let value = Arc::new(AtomicI32::new(0));
let detector = Arc::new(Mutex::new(ChangeDetector::new(Arc::clone(&value))));

let detector_clone = Arc::clone(&detector);
let change_tester = BoxTester::new(move || {
    detector_clone.lock().unwrap().has_changed()
});

// Detect changes
loop {
    if change_tester.test() {
        println!("Value changed!");
        handle_change();
    }
    thread::sleep(Duration::from_millis(100));
}
```

## Implementation Notes

### 1. Thread Safety

```rust
// BoxTester: Cannot cross threads
let test = BoxTester::new(|| check());
// thread::spawn(move || test.test());  // Compile error: BoxTester is not Send

// ArcTester: Can cross threads
let test = ArcTester::new(|| check());
thread::spawn(move || test.test());  // ✅ Compiles
```

### 2. Ownership Conversion

```rust
// Box -> Rc
let box_test = BoxTester::new(|| check());
let rc_test = box_test.into_rc();

// Rc -> Arc (requires closure to satisfy Send + Sync)
let rc_test = RcTester::new(|| check());
// let arc_test = rc_test.into_arc();  // May panic, depends on closure

// Safe approach: Create Arc directly
let arc_test = ArcTester::new(|| check());
```

### 3. Error Handling

```rust
// Tester returns bool, not Result
// Error handling should be done inside the closure

let tester = BoxTester::new(|| {
    match risky_check() {
        Ok(result) => result,
        Err(e) => {
            log::error!("Check failed: {}", e);
            false  // Convert error to false
        }
    }
});
```

### 4. Performance Considerations

```rust
// ✅ Good: Lightweight check
let tester = BoxTester::new(|| {
    flag.load(Ordering::Relaxed)
});

// ⚠️ Note: Expensive checks should have caching strategy
let tester = BoxTester::new(|| {
    expensive_network_check()  // Executes on every call
});

// ✅ Better: Use caching
let checker = Arc::new(Mutex::new(CachedChecker::new(Duration::from_secs(60))));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});
```

## Comparison with Java Implementation

### Java Version

```java
// BooleanSupplier in Java
@FunctionalInterface
public interface BooleanSupplier {
    boolean getAsBoolean();
}

// Usage example
BooleanSupplier tester = () -> database.isAlive();
if (tester.getAsBoolean()) {
    // ...
}
```

### Rust Version

```rust
// Tester in Rust
pub trait Tester {
    fn test(&self) -> bool;
}

// Usage example
let tester = BoxTester::new(|| database.is_alive());
if tester.test() {
    // ...
}
```

### Main Differences

| Feature | Java BooleanSupplier | Rust Tester |
|---------|---------------------|-------------|
| **Method name** | `getAsBoolean()` | `test()` |
| **Ownership model** | GC managed | Box/Rc/Arc explicit management |
| **Thread safety** | Depends on object | ArcTester explicitly requires Send + Sync |
| **Composition methods** | None (needs utility class) | Yes (and, or, not) |
| **Type safety** | Runtime check | Compile-time check |

**Rust advantages**:
- Stronger type safety (compile-time thread safety guarantee)
- Explicit ownership management
- Zero-cost abstraction
- Built-in composition methods

## References

- [Rust Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Fn, FnMut, FnOnce](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- [Java BooleanSupplier](https://docs.oracle.com/javase/8/docs/api/java/util/function/BooleanSupplier.html)
- [Predicate Design Document](./predicate_design.zh_CN.md)
- [Supplier Design Document](./supplier_design.zh_CN.md)