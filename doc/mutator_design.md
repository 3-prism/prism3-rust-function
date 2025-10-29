# Mutator Design Document

## Overview

This document describes the design approach for implementing Mutator types in Rust, explaining core semantics and design decisions.

## What is a Mutator?

### Core Semantics of Mutator

In functional programming, the core semantics of a **Mutator** are:

> **Accepts a mutable reference and modifies it, can simultaneously change the mutator's own state (such as accumulation, counting), and can also modify the passed-in value itself.**

This is "in-place modification" behavior:
- ✅ **Modify input value**: Directly modify the passed mutable reference
- ✅ **Modify own state**: Mutator can accumulate state (such as counting, history records)
- ✅ **Composable usage**: Multiple mutators can be chained together

### Mutator vs Consumer

Based on semantic understanding, we need to clearly distinguish between two types of operations:

| Type | Input Parameter | Modify Input? | Change Self? | Typical Use | Java Equivalent |
|------|----------------|---------------|--------------|-------------|-----------------|
| **Consumer** | `&T` | ❌ | ✅ | Observe, log, statistics, notification | `Consumer<T>` |
| **Mutator** | `&mut T` | ✅ | ✅ | Modify, update, process, transform | `UnaryOperator<T>` |

**Key Insights**:
- Consumer can only **observe and accumulate**, not modify input values
- Mutator can **modify input values in-place** and also accumulate state
- Java's `UnaryOperator<T>` returns new values, while Rust's Mutator modifies in-place

### Main Uses of Mutator

The core value of Mutator types lies in:

1. **Save function objects**: Store function bodies representing modification operations in data structures (such as struct members)
2. **Delayed execution**: Call later when needed
3. **Simplify interfaces**: Use as type constraints (like `M: Mutator<T>`) to improve readability
4. **Conditional modification**: Combine with Predicate to implement conditional modification logic

**If only using temporarily once, using closures directly is more convenient**:
```rust
// ✅ Temporary use: use closure directly
vec![1, 2, 3].iter_mut().for_each(|x| *x *= 2);

// ✅ Need to save: use Mutator
struct DataProcessor {
    transformers: Vec<BoxMutator<Data>>,  // Save multiple transformers
}
```

## Core Design Decisions

### 1. Mutability of Parameters

**Consensus**: All types called Mutator should have parameters of `&mut T`.

```rust
// ✅ Mutator: modify input
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}
```

This forms a clear contrast with Consumer:
```rust
// Consumer: only observe
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}
```

### 2. Mutability of self

Does the Mutator itself need to be mutable? This involves whether it can modify internal state:

```rust
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);  // Can modify own state
}
```

**Scenario Comparison**:

| Scenario | Need to modify state? | Suitable type | Example |
|----------|----------------------|---------------|---------|
| Simple modification (double, add 10) | ❌ | Mutator | `\|x\| *x *= 2` |
| Modification with statistics | ✅ | Mutator | Modify and count |
| Accumulate history records | ✅ | Mutator | Modify and record each operation |

**Conclusion**: Using `&mut self` allows modifying internal state, providing maximum flexibility.

### 3. Value of MutatorOnce

**Key Understanding**: The value of MutatorOnce lies in:

1. **Can save FnOnce closures**: Allows moving captured variables
2. **Delayed execution of one-time operations**: Initialization callbacks, resource transfer, etc.

```rust
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);  // Consume self
}

// Usage scenario: save FnOnce closure
struct Initializer {
    on_complete: Option<BoxMutatorOnce<Data>>,
}

impl Initializer {
    fn new<F>(callback: F) -> Self
    where
        F: FnOnce(&mut Data) + 'static
    {
        Self {
            on_complete: Some(BoxMutatorOnce::new(callback))
        }
    }

    fn run(mut self, data: &mut Data) {
        self.do_init(data);
        if let Some(callback) = self.on_complete {
            callback.mutate_once(data);  // Call only once
        }
    }
}
```

**Conclusion**: MutatorOnce is valuable, but has lower priority than Mutator.

### 4. Rationality of ReadonlyMutator

**Analysis**: What is the semantics of ReadonlyMutator?

```rust
// ❌ Conceptual contradiction
pub trait ReadonlyMutator<T> {
    fn mutate(&self, value: &mut T);  // self immutable, but modify input
}
```

**Problems**:
- If self is immutable (`&self`), it means not modifying internal state
- But if need to modify input (`&mut T`), this is a modification operation
- **"Readonly"** conflicts with **"Mutator"** semantics

**Correct Type Selection**:

| Requirement | Correct Type | Reason |
|-------------|--------------|--------|
| Don't modify self, don't modify input | `ReadonlyConsumer<T>` | Pure observation |
| Modify self, don't modify input | `Consumer<T>` | Observe + accumulate |
| Don't modify self, modify input | ❌ Unreasonable | Modification operations need to be trackable |
| Modify self, modify input | `Mutator<T>` | ✅ Complete mutator |

**Conclusion**: ReadonlyMutator is conceptually contradictory and **should not exist**.

---

## Recommended Complete Design

### Core Trait Definitions

```rust
// === Mutator series (modify input) ===

/// Mutator: can modify self, can modify input
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}

/// One-time mutator: consume self, can modify input (lower priority)
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}
```

**Current Implementation Status**:
- ✅ `Mutator` - Fully implemented (`src/mutators/mutator.rs`)
  - ✅ `BoxMutator<T>` - Single ownership
  - ✅ `ArcMutator<T>` - Thread-safe sharing
  - ✅ `RcMutator<T>` - Single-thread sharing
  - ✅ Conditional mutators (`when` + `or_else`)
- ❌ `MutatorOnce` - Not implemented yet (low priority)
- ❌ `ReadonlyMutator` - **Should not be implemented** (conceptual contradiction)

### Specific Implementations

#### Mutator Series (modify input)

```rust
// Box implementation (single ownership)
pub struct BoxMutator<T> { func: Box<dyn FnMut(&mut T)> }

// Arc implementation (thread-safe sharing)
pub struct ArcMutator<T> { func: Arc<Mutex<dyn FnMut(&mut T) + Send>> }

// Rc implementation (single-thread sharing)
pub struct RcMutator<T> { func: Rc<RefCell<dyn FnMut(&mut T)>> }
```

#### MutatorOnce Series (optional future implementation)

```rust
// Box implementation (single ownership)
pub struct BoxMutatorOnce<T> { func: Box<dyn FnOnce(&mut T)> }

// Note: Arc/Rc variants are incompatible with FnOnce semantics, should not be implemented
```

### Conditional Mutator Design

An important feature of Mutator is support for conditional execution:

```rust
/// Conditional mutator (Box version)
pub struct BoxConditionalMutator<T> {
    mutator: BoxMutator<T>,
    predicate: BoxPredicate<T>,
}

impl<T> BoxConditionalMutator<T> {
    /// Add else branch
    pub fn or_else<C>(self, else_mutator: C) -> BoxMutator<T>
    where
        C: Mutator<T> + 'static
    {
        // Implement if-then-else logic
    }
}
```

**Usage Example**:
```rust
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)           // Condition: positive
    .or_else(|x: &mut i32| *x -= 1);  // Otherwise: subtract 1

let mut positive = 5;
mutator.mutate(&mut positive);
assert_eq!(positive, 10);  // 5 * 2

let mut negative = -5;
mutator.mutate(&mut negative);
assert_eq!(negative, -6);  // -5 - 1
```

### Type Selection Guide

| Requirement | Recommended Type | Reason |
|-------------|------------------|--------|
| One-time use | `BoxMutator` | Single ownership, no overhead |
| Multi-thread sharing | `ArcMutator` | Thread-safe, Mutex protection |
| Single-thread reuse | `RcMutator` | RefCell no-lock overhead |
| One-time + FnOnce | `BoxMutatorOnce` | Save FnOnce (not implemented) |
| Conditional modification | `BoxConditionalMutator` | Combine with Predicate |

---

## Design Pattern Comparison

### Complete Consumer vs Mutator Comparison

| Feature | Consumer | Mutator |
|---------|----------|---------|
| **Input Parameter** | `&T` | `&mut T` |
| **Modify Input?** | ❌ | ✅ |
| **Modify Self?** | ✅ | ✅ |
| **Java Equivalent** | `Consumer<T>` | `UnaryOperator<T>` |
| **Main Use** | Observe, log, statistics, notification | Modify, update, process, transform |
| **ReadOnly Variant** | ✅ `ReadonlyConsumer` | ❌ Conceptual contradiction |
| **Once Variant** | ✅ `ConsumerOnce` | 🟡 `MutatorOnce` (optional) |
| **Conditional Execution** | ❌ None yet | ✅ `when` + `or_else` |

### Three Ownership Model Comparison

| Feature | BoxMutator | ArcMutator | RcMutator |
|---------|------------|------------|-----------|
| **Ownership** | Single | Shared | Shared |
| **Cloneable** | ❌ | ✅ | ✅ |
| **Thread Safe** | ❌ | ✅ | ❌ |
| **Interior Mutability** | N/A | Mutex | RefCell |
| **`and_then` API** | Consume `self` | Borrow `&self` | Borrow `&self` |
| **Lock Overhead** | None | Yes | None |
| **Performance** | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |

---

## Implementation Details

### Conditional Mutator Implementation

Conditional mutators are one of the important features that distinguish Mutator from Consumer:

```rust
impl<T> BoxMutator<T> {
    /// Create conditional mutator
    pub fn when<P>(self, predicate: P) -> BoxConditionalMutator<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalMutator {
            mutator: self,
            predicate: predicate.into_box_once(),
        }
    }
}

impl<T> BoxConditionalMutator<T> {
    /// Add else branch
    pub fn or_else<C>(self, else_mutator: C) -> BoxMutator<T>
    where
        C: Mutator<T> + 'static,
    {
        let pred = self.predicate;
        let mut then_mut = self.mutator;
        let mut else_mut = else_mutator;
        BoxMutator::new(move |t| {
            if pred.test(t) {
                then_mut.mutate_once(t);
            } else {
                else_mut.mutate_once(t);
            }
        })
    }
}
```

### Unified Interface for Three Variants

All three variants implement the `Mutator` trait:

```rust
// BoxMutator
impl<T> Mutator<T> for BoxMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func)(value)
    }
}

// ArcMutator
impl<T> Mutator<T> for ArcMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }
}

// RcMutator
impl<T> Mutator<T> for RcMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func.borrow_mut())(value)
    }
}
```

### Automatic Closure Implementation

All `FnMut(&mut T)` closures automatically implement the `Mutator` trait:

```rust
impl<T, F> Mutator<T> for F
where
    F: FnMut(&mut T),
{
    fn mutate(&mut self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxMutator<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxMutator::new(self)
    }

    // ... other conversion methods
}
```

---

## Usage Examples

### Basic Usage

```rust
use prism3_function::{Mutator, BoxMutator};

// Simple modification
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 5;
mutator.mutate(&mut value);
assert_eq!(value, 10);

// Method chaining
let mut chained = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 10);
let mut value = 5;
chained.mutate(&mut value);
assert_eq!(value, 20);  // (5 * 2) + 10
```

### Conditional Modification

```rust
use prism3_function::{Mutator, BoxMutator};

// Simple condition
let mut conditional = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0);

let mut positive = 5;
conditional.mutate(&mut positive);
assert_eq!(positive, 10);  // Execute

let mut negative = -5;
conditional.mutate(&mut negative);
assert_eq!(negative, -5);  // Don't execute

// if-then-else
let mut branched = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)
    .or_else(|x: &mut i32| *x -= 1);

let mut positive = 5;
branched.mutate(&mut positive);
assert_eq!(positive, 10);  // then branch

let mut negative = -5;
branched.mutate(&mut negative);
assert_eq!(negative, -6);  // else branch
```

### Shared Usage

```rust
use prism3_function::{Mutator, ArcMutator, RcMutator};

// ArcMutator: thread-safe sharing
let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
let clone = mutator.clone();

let mut value = 5;
let mut m = mutator;
m.mutate(&mut value);
assert_eq!(value, 10);

// RcMutator: single-thread sharing (more efficient)
let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
let clone = mutator.clone();

let mut value = 5;
let mut m = mutator;
m.mutate(&mut value);
assert_eq!(value, 10);
```

### Generic Programming

```rust
use prism3_function::Mutator;

fn apply_mutator<M: Mutator<i32>>(
    mutator: &mut M,
    value: i32
) -> i32 {
    let mut val = value;
    mutator.mutate(&mut val);
    val
}

// Works with any Mutator type
let mut box_mut = BoxMutator::new(|x| *x *= 2);
assert_eq!(apply_mutator(&mut box_mut, 5), 10);

let mut closure = |x: &mut i32| *x *= 2;
assert_eq!(apply_mutator(&mut closure, 5), 10);
```

---

## Comparison with Java

### Java UnaryOperator vs Rust Mutator

```java
// Java: return new value
UnaryOperator<Integer> doubler = x -> x * 2;
Integer result = doubler.apply(5);  // result = 10, original unchanged
```

```rust
// Rust: modify in-place
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 5;
mutator.mutate(&mut value);  // value = 10, modified in-place
```

**Key Differences**:
- Java's `UnaryOperator` is `Function<T, T>`, returns new values
- Rust's `Mutator` uses mutable references, modifies in-place
- Rust approach is more efficient (no need to allocate new objects)

---

## Design Principles Summary

1. **Mutator modifies input**: Parameters must be `&mut T`
2. **Clear semantic distinction**: Mutator (modify) vs Consumer (observe)
3. **ReadonlyMutator doesn't exist**: Conceptual contradiction, should not be implemented
4. **MutatorOnce is optional**: Valuable but low priority
5. **Conditional execution support**: `when` + `or_else` provide if-then-else logic
6. **Three ownership models**: Box (single), Arc (thread-safe), Rc (single-thread)
7. **Unified trait interface**: All variants implement `Mutator<T>`
8. **Automatic closure implementation**: Zero-cost abstraction, natural integration

---

## Future Extensions

### MutatorOnce Implementation (Optional)

```rust
/// One-time mutator trait
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}

/// BoxMutatorOnce implementation
pub struct BoxMutatorOnce<T> {
    func: Box<dyn FnOnce(&mut T)>,
}

impl<T> BoxMutatorOnce<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut T) + 'static,
    {
        BoxMutatorOnce { func: Box::new(f) }
    }

    pub fn and_then<C>(self, next: C) -> Self
    where
        C: MutatorOnce<T> + 'static,
    {
        let first = self.func;
        BoxMutatorOnce::new(move |t| {
            first(t);
            next.mutate_once(t);
        })
    }
}
```

**Usage Scenarios**:
- Cleanup after resource transfer
- Callbacks after initialization completion
- One-time complex modification operations

**Note**: MutatorOnce should not have Arc/Rc variants, because FnOnce semantics conflict with shared ownership.

---

## Summary

### Why Design Mutator This Way?

**`prism3-rust-function` adopts the current approach** for the following reasons:

1. **Clear Semantics**
   - Mutator focuses on modifying input values
   - Forms clear contrast with Consumer (observe)
   - Avoids conceptual confusion (like ReadonlyMutator)

2. **Complete Ownership Model**
   - Box: Single ownership, zero overhead
   - Arc: Thread-safe sharing, Mutex protection
   - Rc: Single-thread sharing, RefCell optimization

3. **Conditional Execution Support**
   - `when` method creates conditional mutators
   - `or_else` adds else branches
   - Supports complex conditional modification logic

4. **Unified Trait Abstraction**
   - Provides `Mutator<T>` trait
   - All types use through unified interface
   - Supports generic programming

5. **Consistent with Rust Ecosystem**
   - Naming patterns consistent with standard library smart pointers (Box/Arc/Rc)
   - Design philosophy aligns with Rust conventions
   - In-place modification is more efficient than returning new values

6. **Long-term Maintainability**
   - Clear architecture
   - Easy to extend (can add MutatorOnce in the future)
   - Type names are self-documenting

### Core Design Principles

1. **Mutator modifies input**: Parameters must be `&mut T`
2. **Distinguish Consumer and Mutator**: Clear semantics
3. **ReadonlyMutator doesn't exist**: Conceptual contradiction
4. **Reserve MutatorOnce possibility**: Optional future implementation
5. **Type names are semantically clear**: Box/Arc/Rc express ownership models
6. **Conditional execution is a core feature**: Important functionality that distinguishes from Consumer

This design provides users with a flexible, powerful, and clear API, making it the best choice for library projects.
