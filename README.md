# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[English](#english) | [中文](README.zh_CN.md)

---

<a name="english"></a>

## English

Comprehensive functional programming abstractions for Rust, providing Java-style functional interfaces with Rust's ownership models.

### Overview

This crate provides a complete set of functional programming abstractions inspired by Java's functional interfaces, adapted to Rust's ownership system. It offers multiple implementations for each abstraction (Box/Arc/Rc) to cover various use cases from simple single-threaded scenarios to complex multi-threaded applications.

### Key Features

- **Complete Functional Interface Suite**: Predicate, Consumer, Supplier, Transformer, Mutator, BiConsumer, BiPredicate, and Comparator
- **Multiple Ownership Models**: Box-based single ownership, Arc-based thread-safe sharing, and Rc-based single-threaded sharing
- **Flexible API Design**: Trait-based unified interface with concrete implementations optimized for different scenarios
- **Method Chaining**: All types support fluent API and functional composition
- **Thread-Safety Options**: Choose between thread-safe (Arc) and efficient single-threaded (Rc) implementations
- **Zero-Cost Abstractions**: Efficient implementations with minimal runtime overhead

### Core Types

#### Predicate<T>

Tests whether a value satisfies a condition, returning `bool`. Similar to Java's `Predicate<T>` interface.

**Implementations:**
- `BoxPredicate<T>`: Single ownership, non-cloneable
- `ArcPredicate<T>`: Thread-safe shared ownership, cloneable
- `RcPredicate<T>`: Single-threaded shared ownership, cloneable

**Features:**
- Logical composition: `and`, `or`, `not`, `xor`
- Type-preserving method chaining
- Extension trait for closures (`FnPredicateOps`)

#### Consumer<T>

Accepts a single input parameter and performs operations without returning a result. Similar to Java's `Consumer<T>`.

**Implementations:**
- `BoxConsumer<T>`: Single ownership, uses `FnMut(&T)`
- `ArcConsumer<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcConsumer<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxConsumerOnce<T>`: One-time use with `FnOnce(&T)`

**Features:**
- Method chaining with `and_then`
- Interior mutability for stateful operations
- Both thread-safe and single-threaded options
- Readonly variant (`ReadonlyConsumer`) for pure observation

#### Mutator<T>

Modifies values in-place by accepting mutable references. Similar to Java's `UnaryOperator<T>` but with in-place modification.

**Implementations:**
- `BoxMutator<T>`: Single ownership, uses `FnMut(&mut T)`
- `ArcMutator<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcMutator<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxMutatorOnce<T>`: One-time use with `FnOnce(&mut T)`

**Features:**
- Method chaining with `and_then`
- Conditional execution with `when` and `or_else`
- Supports if-then-else logic
- Distinct from Consumer (reads vs modifies)

#### Supplier<T>

Generates values lazily without input parameters. Similar to Java's `Supplier<T>`.

**Implementations:**
- `BoxSupplier<T>`: Single ownership, uses `FnMut() -> T`
- `ArcSupplier<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcSupplier<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxSupplierOnce<T>`: One-time use with `FnOnce() -> T`

**Features:**
- Stateful value generation
- Method chaining: `map`, `filter`, `flat_map`
- Factory methods: `constant`, `counter`
- Support for sequences and generators

#### Transformer<T, R>

Transforms values from type `T` to type `R` by consuming input. Similar to Java's `Function<T, R>`.

**Implementations:**
- `BoxTransformer<T, R>`: Reusable, single ownership (Fn)
- `ArcTransformer<T, R>`: Thread-safe, cloneable (Arc<Fn>)
- `RcTransformer<T, R>`: Single-threaded, cloneable (Rc<Fn>)
- `BoxTransformerOnce<T, R>`: One-time use (FnOnce)

**Features:**
- Function composition: `and_then`, `compose`
- Factory methods: `identity`, `constant`
- Consumes input for maximum flexibility
- Conversion to standard closures with `into_fn`

#### BiConsumer<T, U>

Accepts two input parameters and performs operations without returning a result. Similar to Java's `BiConsumer<T, U>`.

**Implementations:**
- `BoxBiConsumer<T, U>`: Single ownership
- `ArcBiConsumer<T, U>`: Thread-safe, cloneable
- `RcBiConsumer<T, U>`: Single-threaded, cloneable
- `BoxBiConsumerOnce<T, U>`: One-time use

**Features:**
- Method chaining with `and_then`
- Readonly variant (`ReadonlyBiConsumer`)

#### BiPredicate<T, U>

Tests whether two values satisfy a condition, returning `bool`. Similar to Java's `BiPredicate<T, U>`.

**Implementations:**
- `BoxBiPredicate<T, U>`: Single ownership
- `ArcBiPredicate<T, U>`: Thread-safe, cloneable
- `RcBiPredicate<T, U>`: Single-threaded, cloneable

**Features:**
- Logical composition: `and`, `or`, `not`

#### Comparator<T>

Compares two values and returns an `Ordering`. Similar to Java's `Comparator<T>`.

**Implementations:**
- `BoxComparator<T>`: Single ownership
- `ArcComparator<T>`: Thread-safe, cloneable
- `RcComparator<T>`: Single-threaded, cloneable

**Features:**
- Method chaining: `reversed`, `then`

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
prism3-function = "0.1.0"
```

### Quick Start Examples

#### Using Predicate

```rust
use prism3_function::{ArcPredicate, Predicate, FnPredicateOps};

// Create predicates with logical composition
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// Compose predicates while preserving clonability
let is_even_and_positive = is_even.and(&is_positive);

assert!(is_even_and_positive.test(&4));
assert!(!is_even_and_positive.test(&3));

// Use with closures
let numbers = vec![1, 2, 3, 4, 5, 6];
let result: Vec<i32> = numbers
    .into_iter()
    .filter(|x| (|n: &i32| *n > 2).and(|n: &i32| n % 2 == 0).test(x))
    .collect();
```

#### Using Consumer

```rust
use prism3_function::{BoxConsumer, Consumer};

// Create a consumer for observation (not modification)
let mut consumer = BoxConsumer::new(|x: &i32| {
    println!("Observed value: {}", x);
});

let value = 10;
consumer.accept(&value);
// value is unchanged
```

#### Using Mutator

```rust
use prism3_function::{BoxMutator, Mutator};

// Create a mutator that modifies values
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
mutator.mutate(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1
```

#### Using Conditional Mutator

```rust
use prism3_function::{BoxMutator, Mutator};

// Create a conditional mutator with if-then-else logic
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)
    .or_else(|x: &mut i32| *x -= 1);

let mut positive = 5;
mutator.mutate(&mut positive);
assert_eq!(positive, 10); // 5 * 2

let mut negative = -5;
mutator.mutate(&mut negative);
assert_eq!(negative, -6); // -5 - 1
```

#### Using Supplier

```rust
use prism3_function::{BoxSupplier, Supplier};

// Create a counter supplier
let mut counter = {
    let mut count = 0;
    BoxSupplier::new(move || {
        count += 1;
        count
    })
};

assert_eq!(counter.get(), 1);
assert_eq!(counter.get(), 2);
assert_eq!(counter.get(), 3);
```

#### Using Transformer

```rust
use prism3_function::{BoxTransformer, Transformer};

// Chain transformers for data transformation
let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt: Option<i32>| opt.unwrap_or(0))
    .and_then(|x: i32| x * 2);

assert_eq!(parse_and_double.transform("21".to_string()), 42);
```

### Design Philosophy

This crate adopts the **Trait + Multiple Implementations** pattern, providing:

1. **Unified Interface**: Each functional type has a trait defining core behavior
2. **Specialized Implementations**: Multiple concrete types optimized for different scenarios
3. **Type Preservation**: Composition methods return the same concrete type
4. **Ownership Flexibility**: Choose between single ownership, thread-safe sharing, or single-threaded sharing
5. **Ergonomic API**: Natural method chaining without explicit cloning

### Comparison Table

| Type | Box (Single) | Arc (Thread-Safe) | Rc (Single-Thread) |
|------|--------------|-------------------|-------------------|
| Predicate | BoxPredicate | ArcPredicate | RcPredicate |
| Consumer | BoxConsumer | ArcConsumer | RcConsumer |
| Mutator | BoxMutator | ArcMutator | RcMutator |
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| Transformer | BoxTransformer | ArcTransformer | RcTransformer |
| BiConsumer | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
| BiPredicate | BoxBiPredicate | ArcBiPredicate | RcBiPredicate |
| Comparator | BoxComparator | ArcComparator | RcComparator |

**Legend:**
- **Box**: Single ownership, cannot be cloned, consumes self
- **Arc**: Shared ownership, thread-safe, cloneable
- **Rc**: Shared ownership, single-threaded, cloneable

### Documentation

- [Predicate Design](doc/predicate_design.md) | [中文](doc/predicate_design.zh_CN.md)
- [Consumer Design](doc/consumer_design.md) | [中文](doc/consumer_design.zh_CN.md)
- [Mutator Design](doc/mutator_design.md) | [中文](doc/mutator_design.zh_CN.md)
- [Supplier Design](doc/supplier_design.md) | [中文](doc/supplier_design.zh_CN.md)
- [Transformer Design](doc/transformer_design.md) | [中文](doc/transformer_design.zh_CN.md)

### Examples

The `examples/` directory contains comprehensive demonstrations:

- `predicate_demo.rs`: Predicate usage patterns
- `consumer_demo.rs`: Consumer usage patterns
- `mutator_demo.rs`: Mutator usage patterns
- `mutator_once_conditional_demo.rs`: Conditional mutator patterns
- `supplier_demo.rs`: Supplier usage patterns
- `transformer_demo.rs`: Transformer usage patterns
- And more...

Run examples with:
```bash
cargo run --example predicate_demo
cargo run --example consumer_demo
cargo run --example mutator_demo
```

### License

Licensed under Apache License, Version 2.0.

### Author

Haixing Hu <starfish.hu@gmail.com>
