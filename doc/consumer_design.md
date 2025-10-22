# Consumer Design Comparison and Analysis

## Overview

This document analyzes design approaches for implementing Consumer types in Rust, elucidating core semantics and design decisions.

## What is a Consumer?

### Core Semantics of Consumer

In functional programming, the core semantics of a **Consumer** is:

> **Accept a value and use it, potentially changing the consumer's own state (such as accumulation, counting), but should not modify the consumed value itself.**

This is similar to "consumption" behavior in real life:
- ✅ **Consuming food**: Food is eaten (used), consumer gains nutrition (state change)
- ✅ **Consuming information**: Information is read (used), consumer gains knowledge (state change)
- ❌ **Modifying food**: This is not "consumption", but "processing"

### Consumer vs Mutator

Based on this semantic understanding, we need to clearly distinguish between two types of operations:

| Type | Input Parameter | Modify Input? | Change Self? | Typical Use Cases | Java Equivalent |
|------|-----------------|----------------|--------------|------------------|-----------------|
| **Consumer** | `&T` | ❌ | ✅ | Observation, logging, statistics, notification | `Consumer<T>` |
| **Mutator** | `&mut T` | ✅ | ✅ | Modification, update, processing, transformation | `UnaryOperator<T>` |

**Key Insights**:
- If you need to **modify the input value**, that's not a Consumer, it should be called a **Mutator**
- Consumer can **modify its own state** (counting, accumulation), but **does not modify the input**

**Implementation Notes**:
- ✅ This project uses `Mutator` naming (`src/mutator.rs`)
- ✅ Consumer series maintains `&T` parameters (does not modify input)
- ✅ Mutator series uses `&mut T` parameters (can modify input)

### Main Uses of Consumer

The core value of Consumer types lies in:

1. **Storing function objects**: Save function bodies representing consumption operations in data structures (such as struct members)
2. **Delayed execution**: Call later when needed
3. **Simplifying interfaces**: As type constraints (like `C: Consumer<T>`) to improve readability

**If only using temporarily once, direct closure is more convenient**:
```rust
// ✅ Temporary use: direct closure
vec![1, 2, 3].iter().for_each(|x| println!("{}", x));

// ✅ Need to store: use Consumer
struct EventSystem {
    handlers: Vec<BoxConsumer<Event>>,  // Store multiple handlers
}
```

## Core Design Decisions

### 1. Mutability of Parameters

**Consensus**: All things called Consumer should have `&T` parameters, not `&mut T`.

```rust
// ✅ Consumer: consume but don't modify input
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}

// ✅ Mutator: modify input (not Consumer)
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}
```

### 2. Mutability of self

Does Consumer itself need to be mutable? This involves whether internal state can be modified:

```rust
// Option A: ReadonlyConsumer (immutable self)
pub trait ReadonlyConsumer<T> {
    fn accept(&self, value: &T);  // Don't modify self
}

// Option B: Consumer (mutable self)
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);  // Can modify own state
}
```

**Scenario Comparison**:

| Scenario | Need to Modify State? | Suitable Type |
|----------|----------------------|---------------|
| Pure observation (printing, logging) | ❌ | ReadonlyConsumer |
| Statistical counting | ✅ | Consumer |
| Data accumulation | ✅ | Consumer |
| Event notification (observer pattern) | ❌ | ReadonlyConsumer |

**Recommendation**: Provide both to meet different scenario needs.

### 3. Value of ConsumerOnce

**Key Understanding**: The value of ConsumerOnce is not in parameter ownership (`T` vs `&T`), but in:

1. **Can store FnOnce closures**: Allows moving captured variables
2. **Delayed execution of one-time operations**: Initialization callbacks, cleanup callbacks, etc.

```rust
pub trait ConsumerOnce<T> {
    fn accept(self, value: &T);  // Consume self, but parameter is still &T
}

// Use case: storing FnOnce closures
struct Initializer {
    on_complete: Option<BoxConsumerOnce<InitResult>>,
}

impl Initializer {
    fn new<F>(callback: F) -> Self
    where
        F: FnOnce(&InitResult) + 'static  // FnOnce closure
    {
        Self {
            on_complete: Some(BoxConsumerOnce::new(callback))
        }
    }

    fn run(mut self) {
        let result = self.do_init();
        if let Some(callback) = self.on_complete {
            callback.accept_once(&result);  // Only call once
        }
    }
}
```

**Conclusion**: ConsumerOnce is necessary, but the signature should be `accept(self, &T)` not `accept(self, T)`.

---

## Three Implementation Approaches Comparison

### Approach One: Type Aliases + Static Composition Methods

Use type aliases to define Consumer types and provide composition methods through static utility classes.

```rust
// Type alias definitions
pub type Consumer<T> = Box<dyn FnMut(&T)>;
pub type ReadonlyConsumer<T> = Arc<dyn Fn(&T) + Send>;

// Static composition utility class
pub struct Consumers;

impl Consumers {
    pub fn and_then<T, F1, F2>(first: F1, second: F2) -> Consumer<T>
    where
        T: 'static,
        F1: FnMut(&T) + 'static,
        F2: FnMut(&T) + 'static,
    {
        let mut first = first;
        let mut second = second;
        Box::new(move |t| {
            first(t);
            second(t);
        })
    }

    pub fn noop<T>() -> Consumer<T>
    where
        T: 'static,
    {
        Box::new(|_| {})
    }
}
```

**Usage Example**:
```rust
// Create consumer
let mut consumer: Consumer<i32> = Box::new(|x| println!("{}", x));

// Direct call
let value = 5;
consumer(&value);  // ✅ Can call directly

// Composition
let mut chained = Consumers::and_then(
    |x: &i32| println!("First: {}", x),
    |x: &i32| println!("Second: {}", x),
);
```

**Advantages**:
- ✅ Minimal API, direct call `consumer(&value)`
- ✅ Perfect integration with standard library (usable in `for_each` etc.)
- ✅ Zero-cost abstraction, single boxing
- ✅ Simple implementation, less code

**Disadvantages**:
- ❌ Cannot extend (cannot add fields, implement traits)
- ❌ Low type distinction (equivalent to `Box<dyn FnMut>`)
- ❌ Cannot implement method chaining (only nested calls)
- ❌ ReadonlyConsumer still needs explicit shared handling (Arc)

---

### Approach Two: Struct Wrapper + Instance Methods

Define Consumer as a struct, internally wrapping `Box<dyn FnMut>`, providing composition capabilities through instance methods.

```rust
pub struct Consumer<T> {
    func: Box<dyn FnMut(&T)>,
}

impl<T> Consumer<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        Consumer { func: Box::new(f) }
    }

    pub fn accept(&mut self, value: &T) {
        (self.func)(value)
    }

    pub fn and_then<C>(self, next: C) -> Self
    where
        C: FnMut(&T) + 'static,
    {
        let mut first = self.func;
        let mut second = next;
        Consumer::new(move |t| {
            first(t);
            second(t);
        })
    }

    pub fn noop() -> Self {
        Consumer::new(|_| {})
    }
}

pub struct ReadonlyConsumer<T> {
    func: Arc<dyn Fn(&T) + Send>,
}

impl<T> ReadonlyConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) + Send + 'static,
    {
        ReadonlyConsumer {
            func: Arc::new(f),
        }
    }

    pub fn accept(&self, value: &T) {
        (self.func)(value)
    }

    pub fn and_then(&self, next: &ReadonlyConsumer<T>) -> Self {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ReadonlyConsumer {
            func: Arc::new(move |t: &T| {
                first(t);
                second(t);
            }),
        }
    }
}

impl<T> Clone for ReadonlyConsumer<T> {
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
let mut consumer = Consumer::new(|x: &i32| println!("{}", x));
let value = 5;
consumer.accept_once(&value);  // Must use .accept_once()

// Method chaining
let mut chained = Consumer::new(|x: &i32| println!("First: {}", x))
    .and_then(|x| println!("Second: {}", x));

// ReadonlyConsumer can be cloned and shared
let shared = ReadonlyConsumer::new(|x: &i32| println!("{}", x));
let clone = shared.clone();
shared.accept_once(&5);
clone.accept_once(&10);
```

**Advantages**:
- ✅ Elegant method chaining (`.and_then()`)
- ✅ Strong extensibility (can add fields, implement traits)
- ✅ Type safety, independent types
- ✅ Rich factory methods

**Disadvantages**:
- ❌ Cannot call directly (must use `.accept_once()`)
- ❌ Need to maintain two separate implementations (Consumer and ReadonlyConsumer)
- ❌ Code duplication (composition methods need separate implementation)
- ❌ Ownership issues (`and_then` consumes self)

---

### Approach Three: Trait Abstraction + Multiple Implementations (Recommended, Currently Adopted)

Define unified `Consumer` trait, provide three specific implementations (Box/Arc/Rc), implement specialized composition methods on structs.

```rust
// ============================================================================
// 1. Unified Consumer trait
// ============================================================================

pub trait Consumer<T> {
    fn accept(&mut self, value: &T);

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
}

pub trait ReadonlyConsumer<T> {
    fn accept(&self, value: &T);

    // ... similar into_* methods
}

// ============================================================================
// 2. Implement Consumer trait for closures
// ============================================================================

impl<T, F> Consumer<T> for F
where
    F: FnMut(&T),
{
    fn accept(&mut self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(self)
    }

    // ... other into_* methods
}

// ============================================================================
// 3. BoxConsumer - Single ownership implementation
// ============================================================================

pub struct BoxConsumer<T> {
    func: Box<dyn FnMut(&T)>,
}

impl<T> BoxConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxConsumer { func: Box::new(f) }
    }

    /// Consume self, return BoxConsumer
    pub fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept_once(t);
        })
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.func)(value)
    }

    // ... into_* method implementations
}

// ============================================================================
// 4. ArcConsumer - Thread-safe shared ownership implementation
// ============================================================================

pub struct ArcConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&T) + Send>>,
}

impl<T> ArcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// Borrow &self, return ArcConsumer
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcConsumer {
            func: Arc::new(Mutex::new(move |t: &T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.func.lock().unwrap())(value)
    }

    // ... into_* method implementations
}

impl<T> Clone for ArcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 5. RcConsumer - Single-threaded shared ownership implementation
// ============================================================================

pub struct RcConsumer<T> {
    func: Rc<RefCell<dyn FnMut(&T)>>,
}

impl<T> RcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcConsumer {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// Borrow &self, return RcConsumer
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T>
    where
        T: 'static,
    {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcConsumer {
            func: Rc::new(RefCell::new(move |t: &T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.func.borrow_mut())(value)
    }

    // ... into_* method implementations
}

impl<T> Clone for RcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. ReadonlyConsumer implementations (similar structure)
// ============================================================================

pub struct BoxReadonlyConsumer<T> {
    func: Box<dyn Fn(&T)>,
}

pub struct ArcReadonlyConsumer<T> {
    func: Arc<dyn Fn(&T) + Send>,  // No need for Mutex
}

pub struct RcReadonlyConsumer<T> {
    func: Rc<dyn Fn(&T)>,  // No need for RefCell
}

// ... similar implementations, but using Fn instead of FnMut
```

**Usage Example**:
```rust
// 1. Closures automatically have .accept_once() method
let mut closure = |x: &i32| println!("{}", x);
closure.accept_once(&5);  // ✅ Direct use

// 2. Closures can be composed, return BoxConsumer
let mut chained = (|x: &i32| println!("First: {}", x))
    .and_then(|x| println!("Second: {}", x));
chained.accept_once(&5);

// 3. BoxConsumer - one-time use
let consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
let mut combined = consumer.and_then(|x| println!("Done: {}", x));

// 4. ArcConsumer - multi-threaded sharing, no explicit clone needed
let shared = ArcConsumer::new(|x: &i32| println!("{}", x));
let combined = shared.and_then(&ArcConsumer::new(|x| println!("Then: {}", x)));
// shared is still available
let clone = shared.clone();
std::thread::spawn(move || {
    let mut c = clone;
    c.accept_once(&5);
});

// 5. RcConsumer - single-threaded reuse
let rc = RcConsumer::new(|x: &i32| println!("{}", x));
let combined1 = rc.and_then(&RcConsumer::new(|x| println!("A: {}", x)));
let combined2 = rc.and_then(&RcConsumer::new(|x| println!("B: {}", x)));
// rc is still available

// 6. Unified interface
fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: i32) {
    let val = value;
    consumer.accept_once(&val);
}

let mut box_con = BoxConsumer::new(|x| println!("{}", x));
apply_consumer(&mut box_con, 5);

let mut arc_con = ArcConsumer::new(|x| println!("{}", x));
apply_consumer(&mut arc_con, 5);
```

**Advantages**:
- ✅ Unified trait interface (all types implement `Consumer<T>`)
- ✅ Clear semantics (`BoxConsumer`/`ArcConsumer`/`RcConsumer` names are documentation)
- ✅ Complete ownership model coverage (Box/Arc/Rc three types)
- ✅ Type preservation (`ArcConsumer.and_then()` returns `ArcConsumer`)
- ✅ Elegant API (Arc/Rc composition methods use `&self`, no explicit clone needed)
- ✅ Solves interior mutability (Arc uses Mutex, Rc uses RefCell, each optimized)
- ✅ Strongest extensibility (can add new implementations, fields, traits)
- ✅ Consistent with Rust standard library design philosophy

**Disadvantages**:
- ❌ Still cannot call directly (must use `.accept_once()`)
- ❌ Slightly higher learning cost (need to understand differences between three implementations)
- ❌ High implementation cost (need to implement separately for three structs)

---

## Three Approaches Comparison Summary

| Feature | Approach 1: Type Aliases | Approach 2: Struct Wrapper | Approach 3: Trait + Multi-impl ⭐ |
|:---|:---:|:---:|:---:|
| **Calling Method** | `consumer(&value)` ✅ | `consumer.accept_once(&value)` | `consumer.accept_once(&value)` |
| **Semantic Clarity** | 🟡 Medium | 🟢 Good | 🟢 **Excellent** ✨ |
| **Unified Interface** | ❌ None | ❌ Two separate | ✅ **Unified trait** ✨ |
| **Ownership Model** | Box + Arc (two) | Box + Arc (two) | Box + Arc + Rc (three) ✅ |
| **Method Chaining** | ❌ Only nesting | ✅ Supported | ✅ **Supported (with type preservation)** ✨ |
| **Extensibility** | ❌ Cannot extend | ✅ Extensible | ✅ **Highly extensible** |
| **Interior Mutability** | Manual handling | Manual handling | ✅ **Three optimized ways** |
| **Code Simplicity** | ✅ **Minimal** | 🟡 Medium | 🟡 Slightly complex |
| **Learning Cost** | ✅ **Lowest** | 🟡 Medium | 🟡 Slightly high |
| **Maintenance Cost** | 🟡 Medium | 🟡 Medium | ✅ **Low (clear architecture)** |
| **Standard Library Consistency** | 🟡 Medium | 🟡 Medium | ✅ **Perfect** ✨ |

### Use Case Comparison

| Scenario | Approach 1 | Approach 2 | Approach 3 ⭐ |
|:---|:---:|:---:|:---:|
| **Rapid Prototyping** | ✅ Best | 🟡 OK | 🟡 OK |
| **Complex Method Chaining** | ❌ Not suitable | ✅ Suitable | ✅ **Best** |
| **Multi-threaded Sharing** | 🟡 Manual Arc | 🟡 ReadonlyConsumer | ✅ **ArcConsumer (clear)** |
| **Single-threaded Reuse** | ❌ Not supported | ❌ Not supported | ✅ **RcConsumer (lock-free)** |
| **Library Development** | 🟡 OK | ✅ Suitable | ✅ **Best** |
| **Long-term Maintenance** | 🟡 Medium | 🟡 Medium | ✅ **Best** |

---

## Recommended Complete Design

### Core Trait Definitions

```rust
// === Consumer Series (don't modify input) ===

/// Read-only consumer: doesn't modify self, doesn't modify input
pub trait ReadonlyConsumer<T> {
    fn accept(&self, value: &T);
}

/// Consumer: can modify self, doesn't modify input
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}

/// One-time consumer: consumes self, doesn't modify input
pub trait ConsumerOnce<T> {
    fn accept(self, value: &T);
}

// === Mutator Series (modify input) ===

/// Mutator: can modify self, can modify input
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}

/// One-time mutator: consumes self, can modify input (not yet implemented)
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}
```

**Current Implementation Status**:
- ✅ `ReadonlyConsumer` - Implemented (`src/readonly_consumer.rs`)
- ✅ `Consumer` - Implemented (`src/consumer.rs`)
- ✅ `ConsumerOnce` - Implemented (`src/consumer_once.rs`)
- ✅ `Mutator` - Implemented (`src/mutator.rs`), originally named `ConsumerMut`
- ❌ `MutatorOnce` - Not yet implemented (low priority)

### Specific Implementations

#### Consumer Series (don't modify input)

```rust
// Box implementations (single ownership)
pub struct BoxReadonlyConsumer<T> { func: Box<dyn Fn(&T)> }
pub struct BoxConsumer<T> { func: Box<dyn FnMut(&T)> }
pub struct BoxConsumerOnce<T> { func: Box<dyn FnOnce(&T)> }

// Arc implementations (thread-safe sharing)
pub struct ArcReadonlyConsumer<T> { func: Arc<dyn Fn(&T) + Send> }
pub struct ArcConsumer<T> { func: Arc<Mutex<dyn FnMut(&T) + Send>> }

// Rc implementations (single-threaded sharing)
pub struct RcReadonlyConsumer<T> { func: Rc<dyn Fn(&T)> }
pub struct RcConsumer<T> { func: Rc<RefCell<dyn FnMut(&T)>> }
```

#### Mutator Series (modify input)

```rust
// Box implementations (single ownership)
pub struct BoxMutator<T> { func: Box<dyn FnMut(&mut T)> }

// Arc implementations (thread-safe sharing)
pub struct ArcMutator<T> { func: Arc<Mutex<dyn FnMut(&mut T) + Send>> }

// Rc implementations (single-threaded sharing)
pub struct RcMutator<T> { func: Rc<RefCell<dyn FnMut(&mut T)>> }
```

### Type Selection Guide

| Requirement | Recommended Type | Reason |
|-------------|------------------|--------|
| One-time use | `BoxConsumer` | Single ownership, no overhead |
| Don't modify state (pure observation) | `BoxReadonlyConsumer` | Uses `Fn`, can be called repeatedly |
| Multi-threaded sharing + modify state | `ArcConsumer` | Thread-safe, Mutex protection |
| Multi-threaded sharing + don't modify state | `ArcReadonlyConsumer` | Thread-safe, lock-free |
| Single-threaded reuse + modify state | `RcConsumer` | RefCell has no lock overhead |
| Single-threaded reuse + don't modify state | `RcReadonlyConsumer` | No overhead at all |
| One-time + FnOnce closure | `BoxConsumerOnce` | Store FnOnce |

---

## Summary

### Why Choose Approach Three?

**`prism3-rust-function` adopts approach three** for the following reasons:

1. **Unified trait abstraction**
   - Provides `Consumer<T>` and `ReadonlyConsumer<T>` traits
   - All types used through unified interface
   - Supports generic programming

2. **Complete ownership model coverage**
   - Box: Single ownership, zero overhead
   - Arc: Thread-safe sharing, Mutex protection
   - Rc: Single-threaded sharing, RefCell optimization

3. **Elegant API design**
   - Type preservation: `ArcConsumer.and_then()` returns `ArcConsumer`
   - No explicit clone needed: composition methods use `&self`
   - Method chaining: fluent API

4. **Consistent with Rust ecosystem**
   - Naming patterns consistent with standard library smart pointers (Box/Arc/Rc)
   - Design philosophy follows Rust conventions

5. **Long-term maintainability**
   - Clear architecture
   - Easy to extend (add new implementations, traits, metadata)
   - Type names are documentation

### Core Design Principles

1. **Consumer doesn't modify input**: Parameters must be `&T`
2. **Distinguish Consumer and Mutator**: Clear semantics
3. **Provide ReadonlyConsumer**: Pure observation scenarios (don't modify own state)
4. **Keep ConsumerOnce**: Store FnOnce closures
5. **Type names are semantically clear**: Box/Arc/Rc express ownership models

This design provides users with the most flexible, powerful, and clear API, making it the best choice for library projects.

---

## Refactoring History

### 2025-01-17: ConsumerMut → Mutator Refactoring

**Background**: The original `ConsumerMut` naming had semantic inconsistency issues:
- `ConsumerMut` used `FnMut(&mut T)` signature, can modify input values
- This violates the core semantics of Consumer (Consumer should only observe, not modify input)

**Refactoring Content**:
1. ✅ Renamed `src/consumer_mut.rs` to `src/mutator.rs`
2. ✅ Renamed all types:
   - `ConsumerMut<T>` → `Mutator<T>`
   - `BoxConsumerMut<T>` → `BoxMutator<T>`
   - `ArcConsumerMut<T>` → `ArcMutator<T>`
   - `RcConsumerMut<T>` → `RcMutator<T>`
   - `FnConsumerMutOps<T>` → `FnMutatorOps<T>`
3. ✅ Renamed methods: `accept()` → `mutate()`
4. ✅ Updated test files: `consumer_mut_tests.rs` → `mutator_tests.rs`
5. ✅ Updated module exports and documentation

**Refactoring Reasons**:
- **Clear semantics**: Mutator clearly indicates "modifier", distinguished from Consumer (observer)
- **Follows design principles**: Consumer series doesn't modify input, Mutator series modifies input
- **Avoid confusion**: Prevent users from mistakenly thinking Consumer can modify input values

**Impact**:
- 🔴 **Breaking change**: All code using `ConsumerMut` needs updating
- 🟢 **Forward compatibility**: If compatibility with old code is needed, can add type alias:
  ```rust
  #[deprecated(note = "Use Mutator instead")]
  pub type ConsumerMut<T> = Mutator<T>;
  ```

**Migration Guide**:
```rust
// Old code
use prism3_function::{ConsumerMut, BoxConsumerMut};
let mut consumer = BoxConsumerMut::new(|x: &mut i32| *x *= 2);
consumer.accept_once(&mut value);

// New code
use prism3_function::{Mutator, BoxMutator};
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
mutator.mutate(&mut value);
```