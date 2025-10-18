# Supplier Design Comparison and Analysis

## Overview

This document analyzes design approaches for implementing Supplier types in Rust, elucidating core semantics and design decisions.

## What is a Supplier?

### Core Semantics of Supplier

In functional programming, the **Supplier (supplier)** has the core semantics of:

> **Generate and provide a value, accepting no input parameters. May generate new values each time (like a factory) or return fixed values (like constants).**

This is similar to real-world "supply" behavior:
- ✅ **Factory produces products**: Each call generates new instances
- ✅ **Warehouse provides inventory**: Returns existing values (or their references)
- ✅ **Counter generates sequence numbers**: Each call increments state, returning different values
- ✅ **Configuration provides default values**: Returns fixed default configurations

### Supplier vs Other Functional Abstractions

Based on this semantic understanding, we need to clarify the differences between Supplier and other types:

| Type | Input | Output | Modify Self? | Typical Use Cases | Java Equivalent |
|------|-------|--------|--------------|------------------|-----------------|
| **Supplier** | None | `T` | ✅ | Factory, generator, lazy initialization | `Supplier<T>` |
| **Function** | `&T` | `R` | ❌ | Transformation, mapping, computation | `Function<T, R>` |
| **Consumer** | `&T` | `()` | ✅ | Observation, logging, statistics | `Consumer<T>` |
| **Predicate** | `&T` | `bool` | ❌ | Filtering, validation, judgment | `Predicate<T>` |

**Key Insights**:
- Supplier is the **only functional abstraction that requires no input**
- Supplier **can modify its own state** (generating different values)
- Supplier must return **owned `T`** (not references, avoiding lifetime issues)

### Main Use Cases of Supplier

The core value of Supplier types lies in:

1. **Lazy initialization**: Defer expensive computations until actually needed
2. **Factory pattern**: Encapsulate object creation logic
3. **Dependency injection**: Provide configurable value sources
4. **Generator pattern**: Generate sequence values on demand
5. **Default value provision**: Provide default values for optional parameters

**If you just need a fixed value, using a variable directly is simpler**:
```rust
// ❌ No need for Supplier: use variable directly
let default_config = Config::default();

// ✅ Need Supplier: lazy initialization, avoid unnecessary computation
struct Service {
    config_supplier: BoxSupplier<Config>,  // Only create when needed
}

// ✅ Need Supplier: generate new values each time
let id_generator = BoxSupplier::new(|| generate_uuid());
```

## Core Design Decisions

### 1. Ownership of Return Values

Should Supplier return `T` or `&T`? This is the most fundamental design question.

#### Option A: Return Ownership `T`

```rust
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // Return ownership
}

// Use case: Factory pattern
let mut factory = BoxSupplier::new(|| User::new("Alice"));
let user1 = factory.get();  // Generate new instance each time
let user2 = factory.get();  // Independent new instance
```

**Advantages**:
- ✅ Clear semantics: "produce" new values each time
- ✅ High flexibility: can generate different instances
- ✅ No lifetime issues: return values exist independently
- ✅ Consistent with Java `Supplier<T>` semantics

**Disadvantages**:
- ❌ Cannot return reference types
- ❌ Must clone or recreate each time (potentially costly)

#### Option B: Return Reference `&T`

```rust
pub trait RefSupplier<T> {
    fn get(&self) -> &T;  // Return reference
}

// Use case: provide references to existing values
let config = Config::default();
let supplier = BoxRefSupplier::new(move || &config);  // ❌ Lifetime issues!
```

**Problem**: Lifetime constraints are extremely complex, making generic `RefSupplier` nearly impossible to implement!

```rust
// Lifetime issue example
pub trait RefSupplier<'a, T> {
    fn get(&'a self) -> &'a T;  // 'a must be fixed
}

// User code
let supplier = create_supplier();
let ref1 = supplier.get();
let ref2 = supplier.get();  // ref1 and ref2 interfere with each other!
```

**Conclusion**: Returning references is nearly infeasible in Rust (unless with explicit lifetime guarantees).

#### Recommended Approach: Only Support Returning Ownership `T`

```rust
/// Supplier - generate and return values
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // Return ownership
}

// If you need to provide references, wrap as returning Arc<T> or Rc<T>
let arc_config = Arc::new(Config::default());
let supplier = BoxSupplier::new(move || Arc::clone(&arc_config));
let config = supplier.get();  // Returns Arc<Config>
```

**Reasons**:
1. **Avoid lifetime traps**: Returning `T` has no lifetime issues
2. **Clear semantics**: Supplier is a "producer", returning new values each time
3. **Flexibility**: Users can choose to return `Arc<T>`, `Rc<T>`, or cloned values
4. **Consistent with Java**: Java's `Supplier<T>` also returns values, not references

### 2. Mutability of self

Does Supplier itself need to be mutable? This relates to whether it can generate different values:

```rust
// Option A: ReadonlySupplier (immutable self)
pub trait ReadonlySupplier<T> {
    fn get(&self) -> T;  // Don't modify self
}

// Option B: Supplier (mutable self)
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // Can modify its own state
}
```

**Scenario Comparison**:

| Scenario | Need to Modify State? | Suitable Type |
|----------|----------------------|---------------|
| Fixed default values | ❌ | ReadonlySupplier |
| Counter generator | ✅ | Supplier |
| Random number generation | ✅ | Supplier |
| Factory (new instance each time) | 🟡 May need | Supplier |
| Iterator pattern | ✅ | Supplier |

**Key Question**: Does ReadonlySupplier really have value?

#### ReadonlySupplier Scenario Analysis

```rust
// Scenario 1: Return fixed values
let supplier = BoxReadonlySupplier::new(|| 42);
let value1 = supplier.get();  // 42
let value2 = supplier.get();  // 42

// ❌ No meaning: wouldn't using constants directly be better?
const DEFAULT_VALUE: i32 = 42;
let value1 = DEFAULT_VALUE;
let value2 = DEFAULT_VALUE;

// Scenario 2: Factory pattern (create new objects each time)
let factory = BoxReadonlySupplier::new(|| User::new("Alice"));
let user1 = factory.get();  // New object
let user2 = factory.get();  // Another new object

// 🟡 Feasible: closure itself doesn't modify state, but returns new objects each time
// But the problem is: factory scenarios are rare, most Supplier scenarios need state

// Scenario 3: Lazy computation (compute only once)
let cached = {
    let mut cache = None;
    BoxSupplier::new(move || {
        if cache.is_none() {
            cache = Some(expensive_computation());
        }
        cache.clone().unwrap()
    })
};
let v1 = cached.get();  // First time: compute
let v2 = cached.get();  // Second time: return cached

// ✅ Use Supplier (`&mut self`) to implement directly, no need for interior mutability!
```

#### Comparison with Consumer/Predicate

| Type | Value of `&self` Variant | Reason |
|------|-------------------------|--------|
| **Consumer** | ✅ High (ReadonlyConsumer) | Main scenarios (logging, notifications) indeed don't need to modify state |
| **Predicate** | N/A (only `&self`) | Judgment operations naturally shouldn't modify state |
| **Supplier** | ❌ Low (ReadonlySupplier) | Main scenarios (counters, generators, stateful factories) all need state modification |

#### Why Supplier Doesn't Need ReadonlySupplier?

**Key Difference**: Supplier itself uses `&mut self`, already allowing state modification, **no need** for interior mutability:

```rust
// Supplier: directly modify state, no need for interior mutability
let mut counter = {
    let mut count = 0;
    BoxSupplier::new(move || {
        count += 1;  // Direct modification, because get(&mut self)
        count
    })
};

// Predicate: needs interior mutability to modify state
let counter_pred = {
    let count = Cell::new(0);  // ❗ Must use Cell
    BoxPredicate::new(move |x: &i32| {
        count.set(count.get() + 1);  // Modify through Cell
        *x > 0
    })
};
```

**Conclusion**:
- ✅ **Only provide `Supplier<T>` (using `&mut self`)**: covers all scenarios
- ❌ **No need for ReadonlySupplier**: extremely low value, adds complexity

### 3. Value of SupplierOnce

**Key Understanding**: The difference between SupplierOnce and Supplier is not just about `self` ownership, but more about **one-time resource consumption**.

```rust
pub trait SupplierOnce<T> {
    fn get(self) -> T;  // Consume self, return value
}

// Use case 1: Lazy initialization (initialize only once)
let initializer = BoxSupplierOnce::new(|| {
    expensive_initialization()
});
let value = initializer.get();  // Consume supplier

// Use case 2: Consume resources to generate values
let resource = acquire_resource();
let supplier = BoxSupplierOnce::new(move || {
    consume_resource(resource)  // resource is moved
});

// Use case 3: Implement lazy computation with Option
struct LazyValue<T> {
    supplier: Option<BoxSupplierOnce<T>>,
    value: Option<T>,
}

impl<T> LazyValue<T> {
    fn get_or_init(&mut self) -> &T {
        if self.value.is_none() {
            let supplier = self.supplier.take().unwrap();
            self.value = Some(supplier.get());
        }
        self.value.as_ref().unwrap()
    }
}
```

**Comparison with Supplier**:

```rust
// Supplier: can be called multiple times (but needs &mut self)
let mut counter = BoxSupplier::new(|| next_id());
let id1 = counter.get();
let id2 = counter.get();

// SupplierOnce: can only be called once, consumes self
let once = BoxSupplierOnce::new(|| initialize_db());
let db = once.get();  // once is consumed
```

**Real Value of SupplierOnce**:

1. **Type system guarantees one-time use**: Compile-time prevention of multiple calls
2. **Preserve FnOnce closures**: Closures can move captured variables
3. **Lazy initialization pattern**: Implement lazy loading with Option
4. **Resource consumption scenarios**: Consume non-cloneable resources when generating values

**Conclusion**: SupplierOnce is **necessary**, complementing Supplier.

---

## Three Implementation Approaches Comparison

### Approach One: Type Aliases + Static Utility Methods

Define Supplier types using type aliases and provide helper methods through static utility classes.

```rust
// Type alias definitions
pub type Supplier<T> = Box<dyn FnMut() -> T>;
pub type SupplierOnce<T> = Box<dyn FnOnce() -> T>;
pub type ArcSupplier<T> = Arc<Mutex<dyn FnMut() -> T + Send>>;

// Static utility class
pub struct Suppliers;

impl Suppliers {
    pub fn constant<T: Clone + 'static>(value: T) -> Supplier<T> {
        Box::new(move || value.clone())
    }

    pub fn lazy<T, F>(f: F) -> SupplierOnce<T>
    where
        F: FnOnce() -> T + 'static,
    {
        Box::new(f)
    }
}
```

**Usage Example**:
```rust
// Create supplier
let mut supplier: Supplier<i32> = Box::new(|| 42);
let value = supplier();  // ✅ Can call directly

// Use utility methods
let constant = Suppliers::constant(100);
let lazy = Suppliers::lazy(|| expensive_init());
```

**Advantages**:
- ✅ Minimal API, direct call `supplier()`
- ✅ Perfect integration with standard library
- ✅ Zero-cost abstraction, single boxing
- ✅ Simple implementation, minimal code

**Disadvantages**:
- ❌ Cannot extend (cannot add fields, implement traits)
- ❌ Low type distinction (equivalent to `Box<dyn FnMut>`)
- ❌ Cannot implement method chaining
- ❌ Need to maintain multiple APIs (Supplier, ArcSupplier, etc.)

---

### Approach Two: Struct Wrapper + Instance Methods

Define Supplier as a struct, internally wrapping `Box<dyn FnMut>`, providing functionality through instance methods.

```rust
pub struct Supplier<T> {
    func: Box<dyn FnMut() -> T>,
}

impl<T> Supplier<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        Supplier { func: Box::new(f) }
    }

    pub fn get(&mut self) -> T {
        (self.func)()
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone,
    {
        Supplier::new(move || value.clone())
    }

    pub fn map<R, F>(self, mapper: F) -> Supplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
    {
        let mut func = self.func;
        let mut mapper = mapper;
        Supplier::new(move || mapper(func()))
    }
}

pub struct SupplierOnce<T> {
    func: Option<Box<dyn FnOnce() -> T>>,
}

impl<T> SupplierOnce<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        SupplierOnce {
            func: Some(Box::new(f)),
        }
    }

    pub fn get(mut self) -> T {
        (self.func.take().unwrap())()
    }
}

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcSupplier {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn get(&self) -> T {
        (self.func.lock().unwrap())()
    }
}

impl<T> Clone for ArcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}
```

**Usage Example**:
```rust
// Create and call
let mut supplier = Supplier::new(|| 42);
let value = supplier.get();  // Must use .get()

// Factory methods
let constant = Supplier::constant(100);
let mut counter = {
    let mut count = 0;
    Supplier::new(move || {
        count += 1;
        count
    })
};

// Method chaining
let mut mapped = Supplier::new(|| 5)
    .map(|x| x * 2)
    .map(|x| x + 1);
assert_eq!(mapped.get(), 11);

// ArcSupplier can be shared across threads
let arc_supplier = ArcSupplier::new(|| generate_id());
let clone = arc_supplier.clone();
std::thread::spawn(move || {
    let id = clone.get();
    println!("Generated: {}", id);
});
```

**Advantages**:
- ✅ Elegant method chaining (`.map()` etc.)
- ✅ Strong extensibility (can add fields, implement traits)
- ✅ Type safety, independent types
- ✅ Rich factory methods

**Disadvantages**:
- ❌ Cannot call directly (must use `.get()`)
- ❌ Need to maintain multiple independent implementations (Supplier, ArcSupplier, etc.)
- ❌ Code duplication (factory methods need separate implementation)

---

### Approach Three: Trait Abstraction + Multiple Implementations (Recommended, Currently Adopted)

Define unified `Supplier` trait, provide three specific implementations (Box/Arc/Rc), implement specialized methods on structs.

```rust
// ============================================================================
// 1. Unified Supplier trait
// ============================================================================

pub trait Supplier<T> {
    fn get(&mut self) -> T;

    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
}

pub trait SupplierOnce<T> {
    fn get(self) -> T;

    fn into_box(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. Implement Supplier trait for closures
// ============================================================================

impl<T, F> Supplier<T> for F
where
    F: FnMut() -> T,
{
    fn get(&mut self) -> T {
        self()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplier::new(self)
    }

    // ... other into_* methods
}

// ============================================================================
// 3. BoxSupplier - Single ownership implementation
// ============================================================================

pub struct BoxSupplier<T> {
    func: Box<dyn FnMut() -> T>,
}

impl<T> BoxSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        BoxSupplier { func: Box::new(f) }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        BoxSupplier::new(move || value.clone())
    }

    /// Map: transform Supplier's output
    pub fn map<R, F>(self, mapper: F) -> BoxSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
    {
        let mut func = self.func;
        let mut mapper = mapper;
        BoxSupplier::new(move || mapper(func()))
    }
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&mut self) -> T {
        (self.func)()
    }

    // ... into_* method implementations
}

// ============================================================================
// 4. BoxSupplierOnce - One-time supplier
// ============================================================================

pub struct BoxSupplierOnce<T> {
    func: Option<Box<dyn FnOnce() -> T>>,
}

impl<T> BoxSupplierOnce<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        BoxSupplierOnce {
            func: Some(Box::new(f)),
        }
    }
}

impl<T> SupplierOnce<T> for BoxSupplierOnce<T> {
    fn get(mut self) -> T {
        (self.func.take().unwrap())()
    }
}

// ============================================================================
// 5. ArcSupplier - Thread-safe shared ownership implementation
// ============================================================================

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcSupplier {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + Send + 'static,
    {
        ArcSupplier::new(move || value.clone())
    }

    /// ArcSupplier's map: borrow &self, return new ArcSupplier
    pub fn map<R, F>(&self, mapper: F) -> ArcSupplier<R>
    where
        F: FnMut(T) -> R + Send + 'static,
        R: Send + 'static,
        T: 'static,
    {
        let func = Arc::clone(&self.func);
        let mut mapper = mapper;
        ArcSupplier::new(move || mapper((func.lock().unwrap())()))
    }
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()
    }

    // ... into_* method implementations
}

impl<T> Clone for ArcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. RcSupplier - Single-threaded shared ownership implementation
// ============================================================================

pub struct RcSupplier<T> {
    func: Rc<RefCell<dyn FnMut() -> T>>,
}

impl<T> RcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        RcSupplier {
            func: Rc::new(RefCell::new(f)),
        }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        RcSupplier::new(move || value.clone())
    }

    /// RcSupplier's map: borrow &self, return new RcSupplier
    pub fn map<R, F>(&self, mapper: F) -> RcSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
        T: 'static,
    {
        let func = Rc::clone(&self.func);
        let mut mapper = mapper;
        RcSupplier::new(move || mapper((func.borrow_mut())()))
    }
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.borrow_mut())()
    }

    // ... into_* method implementations
}

impl<T> Clone for RcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
```

**Usage Example**:
```rust
// 1. Closures automatically have .get() method
let mut closure = || 42;
let value = closure.get();  // ✅ Direct use

// 2. BoxSupplier - one-time use
let mut counter = {
    let mut count = 0;
    BoxSupplier::new(move || {
        count += 1;
        count
    })
};
assert_eq!(counter.get(), 1);
assert_eq!(counter.get(), 2);

// 3. BoxSupplier method chaining
let mut mapped = BoxSupplier::new(|| 5)
    .map(|x| x * 2)
    .map(|x| x + 1);
assert_eq!(mapped.get(), 11);

// 4. BoxSupplierOnce - lazy initialization
let once = BoxSupplierOnce::new(|| {
    println!("Expensive initialization");
    expensive_init()
});
let value = once.get();  // Initialize only once

// 5. ArcSupplier - multi-threaded sharing, no need for explicit clone
let shared = ArcSupplier::new(|| generate_uuid());
let mapped = shared.map(|id| format!("ID: {}", id));
// shared is still available
let clone = shared.clone();
std::thread::spawn(move || {
    let mut c = clone;
    let id = c.get();
    println!("{}", id);
});

// 6. RcSupplier - single-threaded reuse
let rc = RcSupplier::constant(100);
let mapped1 = rc.map(|x| x * 2);
let mapped2 = rc.map(|x| x + 10);
// rc is still available

// 7. Unified interface
fn use_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
    supplier.get()
}

let mut box_sup = BoxSupplier::new(|| 42);
use_supplier(&mut box_sup);

let mut arc_sup = ArcSupplier::new(|| 100);
use_supplier(&mut arc_sup);
```

**Advantages**:
- ✅ Unified trait interface (all types implement `Supplier<T>`)
- ✅ Clear semantics (`BoxSupplier`/`ArcSupplier`/`RcSupplier` names are self-documenting)
- ✅ Complete ownership model coverage (Box/Arc/Rc three types)
- ✅ Type preservation (`ArcSupplier.map()` returns `ArcSupplier`)
- ✅ Elegant API (Arc/Rc methods use `&self`, no need for explicit clone)
- ✅ Solves interior mutability (Arc uses Mutex, Rc uses RefCell)
- ✅ Strongest extensibility (can add new implementations, fields, traits)
- ✅ Consistent with Rust standard library design philosophy

**Disadvantages**:
- ❌ Still cannot call directly (must use `.get()`)
- ❌ Slightly higher learning cost (need to understand differences between three implementations)
- ❌ High implementation cost (need to implement separately for three structs)

---

## Three Approaches Comparison Summary

| Feature | Approach 1: Type Aliases | Approach 2: Struct Wrapper | Approach 3: Trait + Multi-impl ⭐ |
|:---|:---:|:---:|:---:|
| **Calling Method** | `supplier()` ✅ | `supplier.get()` | `supplier.get()` |
| **Semantic Clarity** | 🟡 Medium | 🟢 Good | 🟢 **Excellent** ✨ |
| **Unified Interface** | ❌ None | ❌ Two independent sets | ✅ **Unified trait** ✨ |
| **Ownership Model** | Box + Arc (two) | Box + Arc (two) | Box + Arc + Rc (three) ✅ |
| **Method Chaining** | ❌ Can only nest | ✅ Supported | ✅ **Supported (with type preservation)** ✨ |
| **Extensibility** | ❌ Cannot extend | ✅ Extensible | ✅ **Highly extensible** |
| **Code Simplicity** | ✅ **Minimal** | 🟡 Medium | 🟡 Slightly complex |
| **Learning Cost** | ✅ **Lowest** | 🟡 Medium | 🟡 Slightly high |
| **Maintenance Cost** | 🟡 Medium | 🟡 Medium | ✅ **Low (clear architecture)** |
| **Standard Library Consistency** | 🟡 Medium | 🟡 Medium | ✅ **Perfect** ✨ |

### Use Case Comparison

| Scenario | Approach 1 | Approach 2 | Approach 3 ⭐ |
|:---|:---:|:---:|:---:|
| **Rapid prototyping** | ✅ Best | 🟡 OK | 🟡 OK |
| **Complex method chaining** | ❌ Not suitable | ✅ Suitable | ✅ **Best** |
| **Multi-threaded sharing** | 🟡 Manual Arc | 🟡 ArcSupplier | ✅ **ArcSupplier (clear)** |
| **Single-threaded reuse** | ❌ Not supported | ❌ Not supported | ✅ **RcSupplier (lock-free)** |
| **Library development** | 🟡 OK | ✅ Suitable | ✅ **Best** |
| **Long-term maintenance** | 🟡 Medium | 🟡 Medium | ✅ **Best** |

---

## Recommended Complete Design

### Core Trait Definitions

```rust
// === Supplier Series (Generate Values) ===

/// Supplier: generate and return values
pub trait Supplier<T> {
    /// Get value (can be called multiple times)
    fn get(&mut self) -> T;
}

/// One-time supplier: generate and return values, can only be called once
pub trait SupplierOnce<T> {
    /// Get value (consumes self, can only be called once)
    fn get(self) -> T;
}
```

**Current Implementation Status**:
- ✅ `Supplier` - needs implementation
- ✅ `SupplierOnce` - needs implementation
- ❌ `ReadonlySupplier` - not needed (main scenarios all need state modification, extremely low value)

### Specific Implementations

```rust
// Box implementation (single ownership)
pub struct BoxSupplier<T> { func: Box<dyn FnMut() -> T> }
pub struct BoxSupplierOnce<T> { func: Option<Box<dyn FnOnce() -> T>> }

// Arc implementation (thread-safe sharing)
pub struct ArcSupplier<T> { func: Arc<Mutex<dyn FnMut() -> T + Send>> }

// Rc implementation (single-threaded sharing)
pub struct RcSupplier<T> { func: Rc<RefCell<dyn FnMut() -> T>> }
```

### Type Selection Guide

| Requirement | Recommended Type | Reason |
|-------------|------------------|--------|
| One-time use | `BoxSupplier` | Single ownership, no overhead |
| Lazy initialization (compute only once) | `BoxSupplierOnce` | Consume self, preserve FnOnce |
| Multi-threaded sharing | `ArcSupplier` | Thread-safe, Mutex protection |
| Single-threaded reuse | `RcSupplier` | RefCell lock-free overhead |
| Fixed constants | `BoxSupplier::constant()` | Factory method |
| Counters/generators | `BoxSupplier` | Can modify state |

### Common Factory Methods

```rust
impl<T> BoxSupplier<T> {
    /// Create constant supplier (returns clone of same value each time)
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static;

    /// Create incrementing counter
    pub fn counter(start: i32) -> BoxSupplier<i32> {
        let mut count = start;
        BoxSupplier::new(move || {
            let result = count;
            count += 1;
            result
        })
    }

    /// Map supplier's output
    pub fn map<R, F>(self, mapper: F) -> BoxSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static;
}

impl<T> BoxSupplierOnce<T> {
    /// Create lazy initialization supplier
    pub fn lazy<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static;
}
```

---

## Summary

### Why Choose Approach Three?

**`prism3-rust-function` adopts Approach Three** for the following reasons:

1. **Unified trait abstraction**
   - Provide `Supplier<T>` and `SupplierOnce<T>` traits
   - All types use through unified interface
   - Support generic programming

2. **Complete ownership model coverage**
   - Box: single ownership, zero overhead
   - Arc: thread-safe sharing, Mutex protection
   - Rc: single-threaded sharing, RefCell optimization

3. **Elegant API design**
   - Type preservation: `ArcSupplier.map()` returns `ArcSupplier`
   - No need for explicit clone: Arc/Rc methods use `&self`
   - Method chaining: fluent API

4. **Consistent with Rust ecosystem**
   - Naming patterns consistent with standard library smart pointers (Box/Arc/Rc)
   - Design philosophy follows Rust conventions

5. **Long-term maintainability**
   - Clear architecture
   - Easy to extend (add new implementations, traits, metadata)
   - Type names are self-documenting

### Core Design Principles

1. **Supplier returns ownership `T`**: Avoid lifetime issues, clear semantics
2. **Supplier uses `&mut self`**: Typical scenarios (counters, generators) all need state modification
3. **Keep SupplierOnce**: Lazy initialization, one-time resource consumption
4. **No need for ReadonlySupplier**: Main scenarios all need state modification, extremely low value
5. **Type names are semantically clear**: Box/Arc/Rc express ownership models

### Supplier vs Other Functional Abstractions

| | Supplier | Consumer | Predicate | Function |
|---|---|---|---|---|
| **Input** | None | `&T` | `&T` | `&T` |
| **Output** | `T` | `()` | `bool` | `R` |
| **self signature** | `&mut self` | `&mut self` | `&self` | `&self` |
| **Modify self** | ✅ Typical scenario | ✅ Typical scenario | ❌ Should not | ❌ Should not |
| **Once variant** | ✅ Valuable | ✅ Valuable | ❌ Meaningless | 🟡 Edge case |
| **Readonly variant** | ❌ Not needed | ✅ Valuable | N/A (only `&self`) | N/A (only `&self`) |
| **Core use** | Factory, generator | Observation, accumulation | Filtering, validation | Transformation, mapping |

### Design Consistency

All functional abstractions follow unified design patterns:

1. **Unified trait interfaces**: Each abstraction has core traits
2. **Three implementations**: Box (single), Arc (shared + thread-safe), Rc (shared + single-threaded)
3. **Type-preserving method chaining**: Composition methods return same type
4. **Closures automatically implement traits**: Seamless integration
5. **Extension traits provide composition capabilities**: Such as `FnSupplierOps`

This design provides users with the most flexible, powerful, and clear API, making it the best choice for library projects.

---

## Appendix: Common Usage Patterns

### 1. Lazy Initialization

```rust
struct Database {
    connection: OnceCell<Connection>,
    supplier: BoxSupplierOnce<Connection>,
}

impl Database {
    fn new<F>(init: F) -> Self
    where
        F: FnOnce() -> Connection + 'static,
    {
        Database {
            connection: OnceCell::new(),
            supplier: BoxSupplierOnce::new(init),
        }
    }

    fn get_connection(&mut self) -> &Connection {
        self.connection.get_or_init(|| self.supplier.get())
    }
}
```

### 2. Factory Pattern

```rust
struct UserFactory {
    id_generator: BoxSupplier<u64>,
}

impl UserFactory {
    fn new() -> Self {
        let mut id = 0;
        UserFactory {
            id_generator: BoxSupplier::new(move || {
                id += 1;
                id
            }),
        }
    }

    fn create_user(&mut self, name: &str) -> User {
        User {
            id: self.id_generator.get(),
            name: name.to_string(),
        }
    }
}
```

### 3. Configuration Default Values

```rust
struct Config {
    timeout: Duration,
    max_retries: u32,
}

impl Config {
    fn default_timeout() -> BoxSupplier<Duration> {
        BoxSupplier::constant(Duration::from_secs(30))
    }

    fn default_max_retries() -> BoxSupplier<u32> {
        BoxSupplier::constant(3)
    }
}
```

### 4. Random Number Generator

```rust
use rand::Rng;

fn random_supplier() -> BoxSupplier<u32> {
    BoxSupplier::new(|| rand::thread_rng().gen())
}

fn random_range_supplier(min: i32, max: i32) -> BoxSupplier<i32> {
    BoxSupplier::new(move || rand::thread_rng().gen_range(min..max))
}
```

### 5. Multi-threaded Shared Supplier

```rust
let id_gen = ArcSupplier::new({
    let mut id = AtomicU64::new(0);
    move || id.fetch_add(1, Ordering::SeqCst)
});

let handles: Vec<_> = (0..10)
    .map(|_| {
        let gen = id_gen.clone();
        std::thread::spawn(move || {
            let mut g = gen;
            g.get()
        })
    })
    .collect();
```
