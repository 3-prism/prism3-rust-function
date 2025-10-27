# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Comprehensive functional programming abstractions for Rust, providing Java-style functional interfaces adapted to Rust's ownership model.

## Overview

This crate provides a complete set of functional programming abstractions inspired by Java's functional interfaces, carefully adapted to Rust's ownership system. It offers multiple implementations for each abstraction (Box/Arc/Rc) to cover various use cases from simple single-threaded scenarios to complex multi-threaded applications.

## Key Features

- **Complete Functional Interface Suite**: 11 core functional abstractions with multiple variants
- **Multiple Ownership Models**: Box-based single ownership, Arc-based thread-safe sharing, and Rc-based single-threaded sharing
- **Flexible API Design**: Trait-based unified interface with concrete implementations optimized for different scenarios
- **Method Chaining**: All types support fluent API and functional composition
- **Thread-Safety Options**: Choose between thread-safe (Arc) and efficient single-threaded (Rc) implementations
- **Zero-Cost Abstractions**: Efficient implementations with minimal runtime overhead

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
prism3-function = "0.1.0"
```

## Core Abstractions

This crate provides 11 core functional abstractions, each with multiple implementations:

### 1. Predicate - Condition Testing

Tests whether a value satisfies a condition, returning `bool`.

**Trait**: `Predicate<T>`
**Core Method**: `test(&self, value: &T) -> bool`
**Closure Equivalent**: `Fn(&T) -> bool`

**Implementations**:
- `BoxPredicate<T>` - Single ownership, non-cloneable
- `ArcPredicate<T>` - Thread-safe, cloneable
- `RcPredicate<T>` - Single-threaded, cloneable

**Example**:
```rust
use prism3_function::{Predicate, ArcPredicate};

let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

let combined = is_even.and(is_positive.clone());
assert!(combined.test(&4));
assert!(!combined.test(&-2));
```

### 2. BiPredicate - Two-Value Condition Testing

Tests whether two values satisfy a condition, returning `bool`.

**Trait**: `BiPredicate<T, U>`
**Core Method**: `test(&self, first: &T, second: &U) -> bool`
**Closure Equivalent**: `Fn(&T, &U) -> bool`

**Implementations**:
- `BoxBiPredicate<T, U>` - Single ownership
- `ArcBiPredicate<T, U>` - Thread-safe
- `RcBiPredicate<T, U>` - Single-threaded

**Example**:
```rust
use prism3_function::{BiPredicate, BoxBiPredicate};

let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
assert!(sum_positive.test(&3, &4));
assert!(!sum_positive.test(&-5, &2));
```

### 3. Consumer - Value Observation

Accepts a value reference and performs operations without returning a result.

**Trait**: `Consumer<T>`
**Core Method**: `accept(&self, value: &T)`
**Closure Equivalent**: `Fn(&T)`

**Implementations**:
- `BoxConsumer<T>` - Single ownership
- `ArcConsumer<T>` - Thread-safe
- `RcConsumer<T>` - Single-threaded

**Example**:
```rust
use prism3_function::{Consumer, BoxConsumer};

let logger = BoxConsumer::new(|x: &i32| {
    println!("Value: {}", x);
});
logger.accept(&42);
```

### 4. ConsumerOnce - One-Time Value Observation

Accepts a value reference and performs operations once.

**Trait**: `ConsumerOnce<T>`
**Core Method**: `accept_once(self, value: &T)`
**Closure Equivalent**: `FnOnce(&T)`

**Implementations**:
- `BoxConsumerOnce<T>` - Single ownership, one-time use

### 5. BiConsumer - Two-Value Observation

Accepts two value references and performs operations without returning a result.

**Trait**: `BiConsumer<T, U>`
**Core Method**: `accept(&self, first: &T, second: &U)`
**Closure Equivalent**: `Fn(&T, &U)`

**Implementations**:
- `BoxBiConsumer<T, U>` - Single ownership
- `ArcBiConsumer<T, U>` - Thread-safe
- `RcBiConsumer<T, U>` - Single-threaded

**Example**:
```rust
use prism3_function::{BiConsumer, BoxBiConsumer};

let sum_logger = BoxBiConsumer::new(|x: &i32, y: &i32| {
    println!("Sum: {}", x + y);
});
sum_logger.accept(&10, &20);
```

### 6. BiConsumerOnce - One-Time Two-Value Observation

Accepts two value references and performs operations once.

**Trait**: `BiConsumerOnce<T, U>`
**Core Method**: `accept_once(self, first: &T, second: &U)`
**Closure Equivalent**: `FnOnce(&T, &U)`

**Implementations**:
- `BoxBiConsumerOnce<T, U>` - Single ownership, one-time use

### 7. Mutator - In-Place Value Modification

Modifies values in-place by accepting mutable references.

**Trait**: `Mutator<T>`
**Core Method**: `mutate(&mut self, value: &mut T)`
**Closure Equivalent**: `FnMut(&mut T)`

**Implementations**:
- `BoxMutator<T>` - Single ownership
- `ArcMutator<T>` - Thread-safe
- `RcMutator<T>` - Single-threaded

**Example**:
```rust
use prism3_function::{Mutator, BoxMutator};

let mut doubler = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 10;
doubler.mutate(&mut value);
assert_eq!(value, 20);
```

### 8. MutatorOnce - One-Time In-Place Modification

Modifies a value in-place once.

**Trait**: `MutatorOnce<T>`
**Core Method**: `mutate_once(self, value: &mut T)`
**Closure Equivalent**: `FnOnce(&mut T)`

**Implementations**:
- `BoxMutatorOnce<T>` - Single ownership, one-time use

### 9. Supplier - Value Generation

Generates values without input parameters.

**Trait**: `Supplier<T>`
**Core Method**: `get(&self) -> T`
**Closure Equivalent**: `Fn() -> T`

**Implementations**:
- `BoxSupplier<T>` - Single ownership, lock-free
- `ArcSupplier<T>` - Thread-safe, lock-free
- `RcSupplier<T>` - Single-threaded

**Example**:
```rust
use prism3_function::{Supplier, BoxSupplier};

let factory = BoxSupplier::new(|| String::from("Hello"));
assert_eq!(factory.get(), "Hello");
```

### 10. SupplierOnce - One-Time Value Generation

Generates a value once without input parameters.

**Trait**: `SupplierOnce<T>`
**Core Method**: `get_once(self) -> T`
**Closure Equivalent**: `FnOnce() -> T`

**Implementations**:
- `BoxSupplierOnce<T>` - Single ownership, one-time use

### 11. StatefulSupplier - Stateful Value Generation

Generates values with mutable state.

**Trait**: `StatefulSupplier<T>`
**Core Method**: `get(&mut self) -> T`
**Closure Equivalent**: `FnMut() -> T`

**Implementations**:
- `BoxStatefulSupplier<T>` - Single ownership
- `ArcStatefulSupplier<T>` - Thread-safe with Mutex
- `RcStatefulSupplier<T>` - Single-threaded with RefCell

**Example**:
```rust
use prism3_function::{StatefulSupplier, BoxStatefulSupplier};

let mut counter = {
    let mut count = 0;
    BoxStatefulSupplier::new(move || {
        count += 1;
        count
    })
};

assert_eq!(counter.get(), 1);
assert_eq!(counter.get(), 2);
```

### 12. Function - Reference Transformation

Transforms a value reference to produce a result without consuming the input.

**Trait**: `Function<T, R>`
**Core Method**: `apply(&self, input: &T) -> R`
**Closure Equivalent**: `Fn(&T) -> R`

**Implementations**:
- `BoxFunction<T, R>` - Single ownership
- `ArcFunction<T, R>` - Thread-safe
- `RcFunction<T, R>` - Single-threaded

**Example**:
```rust
use prism3_function::{Function, BoxFunction};

let to_string = BoxFunction::new(|x: &i32| format!("Value: {}", x));
assert_eq!(to_string.apply(&42), "Value: 42");
```

### 13. FunctionOnce - One-Time Reference Transformation

Transforms a value reference once to produce a result.

**Trait**: `FunctionOnce<T, R>`
**Core Method**: `apply_once(self, input: &T) -> R`
**Closure Equivalent**: `FnOnce(&T) -> R`

**Implementations**:
- `BoxFunctionOnce<T, R>` - Single ownership, one-time use

### 14. StatefulFunction - Stateful Reference Transformation

Transforms a value reference with mutable state.

**Trait**: `StatefulFunction<T, R>`
**Core Method**: `apply(&mut self, input: &T) -> R`
**Closure Equivalent**: `FnMut(&T) -> R`

**Implementations**:
- `BoxStatefulFunction<T, R>` - Single ownership
- `ArcStatefulFunction<T, R>` - Thread-safe with Mutex
- `RcStatefulFunction<T, R>` - Single-threaded with RefCell

### 15. Transformer - Value Transformation by Consumption

Transforms values from type `T` to type `R` by consuming input.

**Trait**: `Transformer<T, R>`
**Core Method**: `transform(&self, input: T) -> R`
**Closure Equivalent**: `Fn(T) -> R`

**Implementations**:
- `BoxTransformer<T, R>` - Single ownership
- `ArcTransformer<T, R>` - Thread-safe
- `RcTransformer<T, R>` - Single-threaded

**Type Alias**: `UnaryOperator<T>` = `Transformer<T, T>`

**Example**:
```rust
use prism3_function::{Transformer, BoxTransformer};

let parse = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));
assert_eq!(parse.transform("42".to_string()), 42);
```

### 16. TransformerOnce - One-Time Value Transformation

Transforms a value once by consuming both the transformer and input.

**Trait**: `TransformerOnce<T, R>`
**Core Method**: `transform_once(self, input: T) -> R`
**Closure Equivalent**: `FnOnce(T) -> R`

**Implementations**:
- `BoxTransformerOnce<T, R>` - Single ownership, one-time use

**Type Alias**: `UnaryOperatorOnce<T>` = `TransformerOnce<T, T>`

### 17. StatefulTransformer - Stateful Value Transformation

Transforms values with mutable state by consuming input.

**Trait**: `StatefulTransformer<T, R>`
**Core Method**: `transform(&mut self, input: T) -> R`
**Closure Equivalent**: `FnMut(T) -> R`

**Implementations**:
- `BoxStatefulTransformer<T, R>` - Single ownership
- `ArcStatefulTransformer<T, R>` - Thread-safe with Mutex
- `RcStatefulTransformer<T, R>` - Single-threaded with RefCell

### 18. BiTransformer - Two-Value Transformation

Transforms two input values to produce a result by consuming inputs.

**Trait**: `BiTransformer<T, U, R>`
**Core Method**: `transform(&self, first: T, second: U) -> R`
**Closure Equivalent**: `Fn(T, U) -> R`

**Implementations**:
- `BoxBiTransformer<T, U, R>` - Single ownership
- `ArcBiTransformer<T, U, R>` - Thread-safe
- `RcBiTransformer<T, U, R>` - Single-threaded

**Type Alias**: `BinaryOperator<T>` = `BiTransformer<T, T, T>`

**Example**:
```rust
use prism3_function::{BiTransformer, BoxBiTransformer};

let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
assert_eq!(add.transform(10, 20), 30);
```

### 20. BiTransformerOnce - One-Time Two-Value Transformation

Transforms two values once by consuming everything.

**Trait**: `BiTransformerOnce<T, U, R>`
**Core Method**: `transform_once(self, first: T, second: U) -> R`
**Closure Equivalent**: `FnOnce(T, U) -> R`

**Implementations**:
- `BoxBiTransformerOnce<T, U, R>` - Single ownership, one-time use

**Type Alias**: `BinaryOperatorOnce<T>` = `BiTransformerOnce<T, T, T>`

### 21. StatefulConsumer - Stateful Value Observation

Accepts a value reference with mutable state.

**Trait**: `StatefulConsumer<T>`
**Core Method**: `accept(&mut self, value: &T)`
**Closure Equivalent**: `FnMut(&T)`

**Implementations**:
- `BoxStatefulConsumer<T>` - Single ownership
- `ArcStatefulConsumer<T>` - Thread-safe with Mutex
- `RcStatefulConsumer<T>` - Single-threaded with RefCell

### 22. StatefulBiConsumer - Stateful Two-Value Observation

Accepts two value references with mutable state.

**Trait**: `StatefulBiConsumer<T, U>`
**Core Method**: `accept(&mut self, first: &T, second: &U)`
**Closure Equivalent**: `FnMut(&T, &U)`

**Implementations**:
- `BoxStatefulBiConsumer<T, U>` - Single ownership
- `ArcStatefulBiConsumer<T, U>` - Thread-safe with Mutex
- `RcStatefulBiConsumer<T, U>` - Single-threaded with RefCell

### 23. Comparator - Value Comparison

Compares two values and returns an `Ordering`.

**Trait**: `Comparator<T>`
**Core Method**: `compare(&self, a: &T, b: &T) -> Ordering`
**Closure Equivalent**: `Fn(&T, &T) -> Ordering`

**Implementations**:
- `BoxComparator<T>` - Single ownership
- `ArcComparator<T>` - Thread-safe
- `RcComparator<T>` - Single-threaded

**Example**:
```rust
use prism3_function::{Comparator, BoxComparator};
use std::cmp::Ordering;

let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
```

### 24. Tester - Condition Testing Without Input

Tests whether a state or condition holds without accepting input.

**Trait**: `Tester`
**Core Method**: `test(&self) -> bool`
**Closure Equivalent**: `Fn() -> bool`

**Implementations**:
- `BoxTester` - Single ownership
- `ArcTester` - Thread-safe
- `RcTester` - Single-threaded

**Example**:
```rust
use prism3_function::{Tester, BoxTester};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

let flag = Arc::new(AtomicBool::new(true));
let flag_clone = flag.clone();
let tester = BoxTester::new(move || flag_clone.load(Ordering::Relaxed));

assert!(tester.test());
flag.store(false, Ordering::Relaxed);
assert!(!tester.test());
```

## Trait and Closure Correspondence Table

| Trait | Core Method Signature | Equivalent Closure Type |
|-------|----------------------|------------------------|
| `Predicate<T>` | `test(&self, value: &T) -> bool` | `Fn(&T) -> bool` |
| `BiPredicate<T, U>` | `test(&self, first: &T, second: &U) -> bool` | `Fn(&T, &U) -> bool` |
| `Consumer<T>` | `accept(&self, value: &T)` | `Fn(&T)` |
| `ConsumerOnce<T>` | `accept_once(self, value: &T)` | `FnOnce(&T)` |
| `StatefulConsumer<T>` | `accept(&mut self, value: &T)` | `FnMut(&T)` |
| `BiConsumer<T, U>` | `accept(&self, first: &T, second: &U)` | `Fn(&T, &U)` |
| `BiConsumerOnce<T, U>` | `accept_once(self, first: &T, second: &U)` | `FnOnce(&T, &U)` |
| `StatefulBiConsumer<T, U>` | `accept(&mut self, first: &T, second: &U)` | `FnMut(&T, &U)` |
| `Mutator<T>` | `mutate(&mut self, value: &mut T)` | `FnMut(&mut T)` |
| `MutatorOnce<T>` | `mutate_once(self, value: &mut T)` | `FnOnce(&mut T)` |
| `Supplier<T>` | `get(&self) -> T` | `Fn() -> T` |
| `SupplierOnce<T>` | `get_once(self) -> T` | `FnOnce() -> T` |
| `StatefulSupplier<T>` | `get(&mut self) -> T` | `FnMut() -> T` |
| `Function<T, R>` | `apply(&self, input: &T) -> R` | `Fn(&T) -> R` |
| `FunctionOnce<T, R>` | `apply_once(self, input: &T) -> R` | `FnOnce(&T) -> R` |
| `StatefulFunction<T, R>` | `apply(&mut self, input: &T) -> R` | `FnMut(&T) -> R` |
| `Transformer<T, R>` | `transform(&self, input: T) -> R` | `Fn(T) -> R` |
| `TransformerOnce<T, R>` | `transform_once(self, input: T) -> R` | `FnOnce(T) -> R` |
| `StatefulTransformer<T, R>` | `transform(&mut self, input: T) -> R` | `FnMut(T) -> R` |
| `BiTransformer<T, U, R>` | `transform(&self, first: T, second: U) -> R` | `Fn(T, U) -> R` |
| `BiTransformerOnce<T, U, R>` | `transform_once(self, first: T, second: U) -> R` | `FnOnce(T, U) -> R` |
| `Comparator<T>` | `compare(&self, a: &T, b: &T) -> Ordering` | `Fn(&T, &T) -> Ordering` |
| `Tester` | `test(&self) -> bool` | `Fn() -> bool` |

## Implementation Types Comparison

Each trait has multiple implementations based on ownership model:

| Trait | Box (Single) | Arc (Thread-Safe) | Rc (Single-Thread) |
|-------|--------------|-------------------|-------------------|
| Predicate | BoxPredicate | ArcPredicate | RcPredicate |
| BiPredicate | BoxBiPredicate | ArcBiPredicate | RcBiPredicate |
| Consumer | BoxConsumer | ArcConsumer | RcConsumer |
| ConsumerOnce | BoxConsumerOnce | - | - |
| StatefulConsumer | BoxStatefulConsumer | ArcStatefulConsumer | RcStatefulConsumer |
| BiConsumer | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
| BiConsumerOnce | BoxBiConsumerOnce | - | - |
| StatefulBiConsumer | BoxStatefulBiConsumer | ArcStatefulBiConsumer | RcStatefulBiConsumer |
| Mutator | BoxMutator | ArcMutator | RcMutator |
| MutatorOnce | BoxMutatorOnce | - | - |
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| SupplierOnce | BoxSupplierOnce | - | - |
| StatefulSupplier | BoxStatefulSupplier | ArcStatefulSupplier | RcStatefulSupplier |
| Function | BoxFunction | ArcFunction | RcFunction |
| FunctionOnce | BoxFunctionOnce | - | - |
| StatefulFunction | BoxStatefulFunction | ArcStatefulFunction | RcStatefulFunction |
| Transformer | BoxTransformer | ArcTransformer | RcTransformer |
| TransformerOnce | BoxTransformerOnce | - | - |
| StatefulTransformer | BoxStatefulTransformer | ArcStatefulTransformer | RcStatefulTransformer |
| BiTransformer | BoxBiTransformer | ArcBiTransformer | RcBiTransformer |
| BiTransformerOnce | BoxBiTransformerOnce | - | - |
| Comparator | BoxComparator | ArcComparator | RcComparator |
| Tester | BoxTester | ArcTester | RcTester |

**Legend**:
- **Box**: Single ownership, cannot be cloned, consumes self
- **Arc**: Shared ownership, thread-safe, cloneable
- **Rc**: Shared ownership, single-threaded, cloneable
- **-**: Not applicable (Once types don't need sharing)

## Design Philosophy

This crate adopts the **Trait + Multiple Implementations** pattern:

1. **Unified Interface**: Each functional type has a trait defining core behavior
2. **Specialized Implementations**: Multiple concrete types optimized for different scenarios
3. **Type Preservation**: Composition methods return the same concrete type
4. **Ownership Flexibility**: Choose between single ownership, thread-safe sharing, or single-threaded sharing
5. **Ergonomic API**: Natural method chaining and functional composition

## Examples

The `examples/` directory contains comprehensive demonstrations for each type. Run examples with:

```bash
cargo run --example predicate_demo
cargo run --example consumer_demo
cargo run --example transformer_demo
```

## Documentation

Detailed design documents are available in the `doc/` directory for each major abstraction.

## License

Licensed under Apache License, Version 2.0.

## Author

Haixing Hu <starfish.hu@gmail.com>
