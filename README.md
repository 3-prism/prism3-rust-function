# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Comprehensive functional programming abstractions for Rust, providing Java-style functional interfaces with Rust's ownership models.

## Overview

This crate provides a complete set of functional programming abstractions inspired by Java's functional interfaces, adapted to Rust's ownership system. It offers multiple implementations for each abstraction (Box/Arc/Rc) to cover various use cases from simple single-threaded scenarios to complex multi-threaded applications.

## Key Features

- **Complete Functional Interface Suite**: Predicate, Consumer, Supplier, ReadonlySupplier, Transformer, Mutator, BiConsumer, BiPredicate, BiTransformer, Comparator, and Tester
- **Multiple Ownership Models**: Box-based single ownership, Arc-based thread-safe sharing, and Rc-based single-threaded sharing
- **Flexible API Design**: Trait-based unified interface with concrete implementations optimized for different scenarios
- **Method Chaining**: All types support fluent API and functional composition
- **Thread-Safety Options**: Choose between thread-safe (Arc) and efficient single-threaded (Rc) implementations
- **Lock-Free Concurrency**: ReadonlySupplier provides lock-free concurrent access for stateless scenarios
- **Zero-Cost Abstractions**: Efficient implementations with minimal runtime overhead

## Core Types

### Predicate<T>

Tests whether a value satisfies a condition, returning `bool`. Similar to Java's `Predicate<T>` interface.

#### Core Function
- `test(&self, value: &T) -> bool` - Tests if the value satisfies the predicate condition
- Corresponds to `Fn(&T) -> bool` closure

#### Implementations
- `BoxPredicate<T>`: Single ownership, non-cloneable
- `ArcPredicate<T>`: Thread-safe shared ownership, cloneable
- `RcPredicate<T>`: Single-threaded shared ownership, cloneable

#### Convenience Methods
- Logical composition: `and`, `or`, `not`, `xor`, `nand`, `nor`
- Type-preserving method chaining (each returns the same concrete type)
- Extension trait `FnPredicateOps` for closures - provides composition methods that return `BoxPredicate`

**⚠️ Important: Ownership Transfer in Logical Operations**

All logical composition methods (`and`, `or`, `xor`, `nand`, `nor`) accept the `other` parameter **by value**, which means:

- **Ownership is transferred**: The `other` predicate is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcPredicate` and `RcPredicate`)
- **`BoxPredicate` cannot be cloned**: Once used in a composition, it's consumed

```rust
use prism3_function::{ArcPredicate, RcPredicate, BoxPredicate, Predicate};

// ArcPredicate and RcPredicate can be cloned
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// Option 1: Clone to preserve the original
let combined = is_even.and(is_positive.clone());
// is_positive is still usable because we cloned it
assert!(is_positive.test(&2));

// Option 2: Use &self methods (for Rc/Arc only)
let is_even_rc = RcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive_rc = RcPredicate::new(|x: &i32| *x > 0);
let combined_rc = is_even_rc.and(is_positive_rc.clone());
// Both predicates remain usable
assert!(is_even_rc.test(&2));
assert!(is_positive_rc.test(&2));

// BoxPredicate: Cannot be cloned, will be consumed
let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
let combined_box = box_pred.and(|x: &i32| x % 2 == 0);
// box_pred is no longer available here
```

#### Example

```rust
use prism3_function::{ArcPredicate, Predicate, FnPredicateOps};

// Create predicates with logical composition
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// Clone to preserve the original predicate
let is_even_and_positive = is_even.and(is_positive.clone());

assert!(is_even_and_positive.test(&4));
assert!(!is_even_and_positive.test(&3));
// is_positive is still usable
assert!(is_positive.test(&5));

// Use with closures - extension trait automatically provides composition
let numbers = vec![1, 2, 3, 4, 5, 6];
let result: Vec<i32> = numbers
    .into_iter()
    .filter(|x| (|n: &i32| *n > 2).and(|n: &i32| n % 2 == 0).test(x))
    .collect();
```

### Consumer<T>

Accepts a single input parameter and performs operations without returning a result. Similar to Java's `Consumer<T>`.

#### Core Function
- `accept(&mut self, value: &T)` - Performs an operation on the value reference
- Corresponds to `FnMut(&T)` closure

#### Implementations
- `BoxConsumer<T>`: Single ownership, uses `FnMut(&T)`
- `ArcConsumer<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcConsumer<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxConsumerOnce<T>`: One-time use with `FnOnce(&T)`

#### Convenience Methods
- `and_then` - Chains consumers sequentially
- `when` - Conditional execution with predicate
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnConsumerOps` for closures

#### Related Types
- `ReadonlyConsumer` - For pure observation without modifying consumer state

**⚠️ Important: Ownership Transfer in Composition Methods**

All composition methods (`and_then`, `when`, `or_else`) accept their parameters **by value**, which means:

- **Ownership is transferred**: The parameter (consumer or predicate) is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcConsumer` and `RcConsumer`)
- **BoxConsumer cannot be cloned**: Once used in a composition, it's consumed and no longer available

#### Examples

##### Basic Usage

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

##### Chaining with `and_then`

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// Chain multiple consumers
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log1.lock().unwrap().push(format!("First: {}", x));
})
.and_then(move |x: &i32| {
    log2.lock().unwrap().push(format!("Second: {}", x));
});

consumer.accept(&42);
assert_eq!(log.lock().unwrap().len(), 2);
```

##### Conditional Execution with `when`

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log_clone = log.clone();

// Only execute consumer when predicate is true
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log_clone.lock().unwrap().push(*x);
})
.when(|x: &i32| *x > 0);  // Only log positive numbers

consumer.accept(&10);   // Logged
consumer.accept(&-5);   // Not logged
assert_eq!(log.lock().unwrap().len(), 1);
```

##### If-Then-Else with `or_else`

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// Execute different consumers based on condition
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log1.lock().unwrap().push(format!("Positive: {}", x));
})
.when(|x: &i32| *x > 0)
.or_else(move |x: &i32| {
    log2.lock().unwrap().push(format!("Non-positive: {}", x));
});

consumer.accept(&10);   // "Positive: 10"
consumer.accept(&-5);   // "Non-positive: -5"
assert_eq!(log.lock().unwrap().len(), 2);
```

### Mutator<T>

Modifies values in-place by accepting mutable references. Similar to Java's `UnaryOperator<T>` but with in-place modification.

#### Core Function
- `mutate(&mut self, value: &mut T)` - Modifies the value in-place
- Corresponds to `FnMut(&mut T)` closure

#### Implementations
- `BoxMutator<T>`: Single ownership, uses `FnMut(&mut T)`
- `ArcMutator<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcMutator<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxMutatorOnce<T>`: One-time use with `FnOnce(&mut T)`

#### Convenience Methods
- `and_then` - Chains mutators sequentially
- `when` - Creates conditional mutator (if-then pattern)
- `or_else` - Adds else branch to conditional mutator (if-then-else pattern)
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnMutatorOps` for closures

#### Key Difference from Consumer
- **Consumer**: Accepts `&T` (reads values, doesn't modify input)
- **Mutator**: Accepts `&mut T` (modifies values in-place)

#### Example

```rust
use prism3_function::{BoxMutator, Mutator};

// Create a mutator that modifies values
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
mutator.mutate(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1

// Conditional mutator with if-then-else logic
let mut conditional = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)
    .or_else(|x: &mut i32| *x -= 1);

let mut positive = 5;
conditional.mutate(&mut positive);
assert_eq!(positive, 10); // 5 * 2

let mut negative = -5;
conditional.mutate(&mut negative);
assert_eq!(negative, -6); // -5 - 1
```

### Supplier<T>

Generates values lazily without input parameters. Similar to Java's `Supplier<T>`.

#### Core Function
- `get(&mut self) -> T` - Generates and returns a value
- Corresponds to `FnMut() -> T` closure

#### Implementations
- `BoxSupplier<T>`: Single ownership, uses `FnMut() -> T`
- `ArcSupplier<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcSupplier<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable
- `BoxSupplierOnce<T>`: One-time use with `FnOnce() -> T`

#### Convenience Methods
- `map` - Transforms supplier output
- `filter` - Filters supplier output with predicate
- `flat_map` - Chains suppliers
- Factory methods: `constant`, `counter`
- Type conversions: `into_box`, `into_arc`, `into_rc`

#### Example

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

// Method chaining with map
let mut pipeline = BoxSupplier::new(|| 10)
    .map(|x| x * 2)
    .map(|x| x + 5);

assert_eq!(pipeline.get(), 25);
```

### ReadonlySupplier<T>

Generates values lazily without input parameters and **without modifying its own state**. Unlike `Supplier<T>`, it uses `&self` instead of `&mut self`, enabling usage in read-only contexts and lock-free concurrent access.

#### Core Function
- `get(&self) -> T` - Generates and returns a value (note: `&self`, not `&mut self`)
- Corresponds to `Fn() -> T` closure

#### Key Differences from Supplier

| Aspect | Supplier | ReadonlySupplier |
|--------|----------|------------------|
| self signature | `&mut self` | `&self` |
| Closure type | `FnMut() -> T` | `Fn() -> T` |
| Can modify state | Yes | No |
| Arc implementation | `Arc<Mutex<FnMut>>` | `Arc<Fn>` (lock-free!) |
| Use cases | Counter, generator | Factory, constant, high concurrency |

#### Implementations
- `BoxReadonlySupplier<T>`: Single ownership, zero overhead
- `ArcReadonlySupplier<T>`: **Lock-free** thread-safe sharing (no `Mutex` needed!)
- `RcReadonlySupplier<T>`: Single-threaded sharing, lightweight

#### Convenience Methods
- `map` - Transforms supplier output
- `filter` - Filters supplier output with predicate
- `zip` - Combines two suppliers
- Factory methods: `constant`
- Type conversions: `into_box`, `into_arc`, `into_rc`

#### Use Cases

##### 1. Calling in `&self` Methods

```rust
use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};

struct Executor<E> {
    error_supplier: ArcReadonlySupplier<E>,
}

impl<E> Executor<E> {
    fn execute(&self) -> Result<(), E> {
        // Can call directly in &self method!
        Err(self.error_supplier.get())
    }
}
```

##### 2. High-Concurrency Lock-Free Access

```rust
use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
use std::thread;

let factory = ArcReadonlySupplier::new(|| String::from("Hello, World!"));

let handles: Vec<_> = (0..10)
    .map(|_| {
        let f = factory.clone();
        thread::spawn(move || f.get()) // Lock-free!
    })
    .collect();

for h in handles {
    assert_eq!(h.join().unwrap(), "Hello, World!");
}
```

##### 3. Fixed Factories

```rust
use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};

#[derive(Clone)]
struct Config {
    timeout: u64,
}

let config_factory = BoxReadonlySupplier::new(|| Config { timeout: 30 });

assert_eq!(config_factory.get().timeout, 30);
assert_eq!(config_factory.get().timeout, 30);
```

#### Performance Comparison

For stateless scenarios in multi-threaded environments:

- `ArcSupplier<T>`: Requires `Mutex`, lock contention on every `get()` call
- `ArcReadonlySupplier<T>`: Lock-free, can call `get()` concurrently without contention

Benchmark results show `ArcReadonlySupplier` can be **10x faster** than `ArcSupplier` in high-concurrency scenarios.

### Transformer<T, R>

Transforms values from type `T` to type `R` by consuming input. Similar to Java's `Function<T, R>`.

#### Core Function
- `transform(&self, input: T) -> R` - Transforms input value to output value (consumes input)
- Corresponds to `Fn(T) -> R` closure

#### Implementations
- `BoxTransformer<T, R>`: Reusable, single ownership (Fn)
- `ArcTransformer<T, R>`: Thread-safe, cloneable (Arc<Fn>)
- `RcTransformer<T, R>`: Single-threaded, cloneable (Rc<Fn>)
- `BoxTransformerOnce<T, R>`: One-time use (FnOnce)

#### Convenience Methods
- `and_then` - Composes transformers sequentially (f.and_then(g) = g(f(x)))
- `compose` - Composes transformers in reverse order (f.compose(g) = f(g(x)))
- `when` - Creates conditional transformer with predicate
- Factory methods: `identity`, `constant`
- Type conversions: `into_box`, `into_arc`, `into_rc`, `into_fn`
- Extension trait `FnTransformerOps` for closures

#### Related Types
- `UnaryOperator<T>` - Type alias for `Transformer<T, T>`

**⚠️ Important: Ownership Transfer in Composition Methods**

All composition methods (`and_then`, `compose`, `when`, `or_else`) accept their parameters **by value**, which means:

- **Ownership is transferred**: The parameter (transformer or predicate) is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcTransformer` and `RcTransformer`)
- **BoxTransformer cannot be cloned**: Once used in a composition, it's consumed and no longer available

#### Examples

##### Basic Usage and `and_then` Chaining

```rust
use prism3_function::{BoxTransformer, Transformer};

// Chain transformers for data transformation
let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt: Option<i32>| opt.unwrap_or(0))
    .and_then(|x: i32| x * 2);

assert_eq!(parse_and_double.apply("21".to_string()), 42);
assert_eq!(parse_and_double.apply("invalid".to_string()), 0);
```

##### Conditional Transformation with `when`

```rust
use prism3_function::{BoxTransformer, Transformer};

// Apply transformation only when predicate is true
let double_if_positive = BoxTransformer::new(|x: i32| x * 2)
    .when(|x: &i32| *x > 0);

assert_eq!(double_if_positive.apply(5), Some(10));
assert_eq!(double_if_positive.apply(-5), None);
```

##### If-Then-Else with `or_else`

```rust
use prism3_function::{BoxTransformer, Transformer};

// Different transformations based on condition
let transform = BoxTransformer::new(|x: i32| format!("Positive: {}", x * 2))
    .when(|x: &i32| *x > 0)
    .or_else(|x: i32| format!("Non-positive: {}", x - 1));

assert_eq!(transform.apply(5), "Positive: 10");
assert_eq!(transform.apply(-5), "Non-positive: -6");
```

### BiConsumer<T, U>

Accepts two input parameters and performs operations without returning a result. Similar to Java's `BiConsumer<T, U>`.

#### Core Function
- `accept(&mut self, first: &T, second: &U)` - Performs an operation on two value references
- Corresponds to `FnMut(&T, &U)` closure

#### Implementations
- `BoxBiConsumer<T, U>`: Single ownership
- `ArcBiConsumer<T, U>`: Thread-safe, cloneable
- `RcBiConsumer<T, U>`: Single-threaded, cloneable
- `BoxBiConsumerOnce<T, U>`: One-time use

#### Convenience Methods
- `and_then` - Chains bi-consumers sequentially
- `when` - Conditional execution with bi-predicate
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnBiConsumerOps` for closures

#### Related Types
- `ReadonlyBiConsumer` - For pure observation without modifying consumer state

**⚠️ Important: Ownership Transfer in Composition Methods**

All composition methods (`and_then`, `when`, `or_else`) accept their parameters **by value**, which means:

- **Ownership is transferred**: The parameter (bi-consumer or bi-predicate) is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcBiConsumer` and `RcBiConsumer`)
- **BoxBiConsumer cannot be cloned**: Once used in a composition, it's consumed and no longer available

#### Examples

##### Basic Usage

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};

// Create a bi-consumer for pair operations
let mut bi_consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
    println!("Sum: {}", x + y);
});

bi_consumer.accept(&10, &20);
```

##### Chaining with `and_then`

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// Chain multiple bi-consumers
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log1.lock().unwrap().push(format!("Sum: {}", x + y));
})
.and_then(move |x: &i32, y: &i32| {
    log2.lock().unwrap().push(format!("Product: {}", x * y));
});

bi_consumer.accept(&3, &4);
assert_eq!(log.lock().unwrap().len(), 2);
// log contains: ["Sum: 7", "Product: 12"]
```

##### Conditional Execution with `when`

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log_clone = log.clone();

// Only execute when both values are positive
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log_clone.lock().unwrap().push(format!("{} + {} = {}", x, y, x + y));
})
.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

bi_consumer.accept(&3, &4);   // Logged
bi_consumer.accept(&-1, &4);  // Not logged
assert_eq!(log.lock().unwrap().len(), 1);
```

##### If-Then-Else with `or_else`

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// Different operations based on condition
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log1.lock().unwrap().push(format!("Both positive: {} + {} = {}", x, y, x + y));
})
.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
.or_else(move |x: &i32, y: &i32| {
    log2.lock().unwrap().push(format!("Has negative: {} * {} = {}", x, y, x * y));
});

bi_consumer.accept(&3, &4);   // "Both positive: 3 + 4 = 7"
bi_consumer.accept(&-1, &4);  // "Has negative: -1 * 4 = -4"
assert_eq!(log.lock().unwrap().len(), 2);
```

### BiPredicate<T, U>

Tests whether two values satisfy a condition, returning `bool`. Similar to Java's `BiPredicate<T, U>`.

#### Core Function
- `test(&self, first: &T, second: &U) -> bool` - Tests if two values satisfy the predicate condition
- Corresponds to `Fn(&T, &U) -> bool` closure

#### Implementations
- `BoxBiPredicate<T, U>`: Single ownership, non-cloneable
- `ArcBiPredicate<T, U>`: Thread-safe, cloneable
- `RcBiPredicate<T, U>`: Single-threaded, cloneable

#### Convenience Methods
- Logical composition: `and`, `or`, `not`, `xor`, `nand`, `nor`
- Type-preserving method chaining
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnBiPredicateOps` for closures

**⚠️ Important: Ownership Transfer in Logical Operations**

All logical composition methods (`and`, `or`, `xor`, `nand`, `nor`) accept the `other` parameter **by value**, which means:

- **Ownership is transferred**: The `other` bi-predicate is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcBiPredicate` and `RcBiPredicate`)
- **`BoxBiPredicate` cannot be cloned**: Once used in a composition, it's consumed

```rust
use prism3_function::{ArcBiPredicate, RcBiPredicate, BoxBiPredicate, BiPredicate};

// ArcBiPredicate and RcBiPredicate can be cloned
let is_sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let first_larger = ArcBiPredicate::new(|x: &i32, y: &i32| x > y);

// Clone to preserve the original
let combined = is_sum_positive.and(first_larger.clone());
// first_larger is still usable because we cloned it
assert!(first_larger.test(&10, &5));

// BoxBiPredicate: Cannot be cloned, will be consumed
let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let combined_box = box_pred.and(|x: &i32, y: &i32| x > y);
// box_pred is no longer available here
```

#### Example

```rust
use prism3_function::{ArcBiPredicate, BiPredicate};

// Create bi-predicates with logical composition
let is_sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let first_larger = ArcBiPredicate::new(|x: &i32, y: &i32| x > y);

// Clone to preserve the original predicate
let combined = is_sum_positive.and(first_larger.clone());

assert!(combined.test(&10, &5));
assert!(!combined.test(&3, &8));
// first_larger is still usable
assert!(first_larger.test(&10, &5));
```

### BiTransformer<T, U, R>

Transforms two input values to produce a result value. Similar to Java's `BiFunction<T, U, R>`.

#### Core Function
- `transform(&self, first: T, second: U) -> R` - Transforms two input values to output value (consumes inputs)
- Corresponds to `Fn(T, U) -> R` closure

#### Implementations
- `BoxBiTransformer<T, U, R>`: Reusable, single ownership (Fn)
- `ArcBiTransformer<T, U, R>`: Thread-safe, cloneable (Arc<Fn>)
- `RcBiTransformer<T, U, R>`: Single-threaded, cloneable (Rc<Fn>)
- `BoxBiTransformerOnce<T, U, R>`: One-time use (FnOnce)

#### Convenience Methods
- `and_then` - Composes bi-transformer with transformer
- `when` - Creates conditional bi-transformer with bi-predicate
- Type conversions: `into_box`, `into_arc`, `into_rc`, `into_fn`
- Extension trait `FnBiTransformerOps` for closures

#### Related Types
- `BinaryOperator<T>` - Type alias for `BiTransformer<T, T, T>`

**⚠️ Important: Ownership Transfer in Composition Methods**

All composition methods (`and_then`, `when`, `or_else`) accept their parameters **by value**, which means:

- **Ownership is transferred**: The parameter (transformer or bi-predicate) is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcBiTransformer` and `RcBiTransformer`)
- **BoxBiTransformer cannot be cloned**: Once used in a composition, it's consumed and no longer available

#### Examples

##### Basic Usage and `and_then` Chaining

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// Create a bi-transformer for combining two values
let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);

assert_eq!(add.apply(10, 20), 30);

// Chain with transformer for further processing
let add_and_double = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .and_then(|sum: i32| sum * 2);
assert_eq!(add_and_double.apply(10, 20), 60);

// Multiple chaining
let complex = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .and_then(|sum: i32| sum * 2)
    .and_then(|doubled: i32| format!("Result: {}", doubled));
assert_eq!(complex.apply(10, 20), "Result: 60");
```

##### Conditional Transformation with `when`

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// Only transform when both values are positive
let add_if_positive = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0);

assert_eq!(add_if_positive.apply(3, 4), Some(7));
assert_eq!(add_if_positive.apply(-1, 4), None);
assert_eq!(add_if_positive.apply(3, -4), None);
```

##### If-Then-Else with `or_else`

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// Different transformations based on condition
let transform = BoxBiTransformer::new(|x: i32, y: i32| format!("Sum: {}", x + y))
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(|x: i32, y: i32| format!("Product: {}", x * y));

assert_eq!(transform.apply(3, 4), "Sum: 7");
assert_eq!(transform.apply(-1, 4), "Product: -4");
assert_eq!(transform.apply(3, -4), "Product: -12");
```

### Comparator<T>

Compares two values and returns an `Ordering`. Similar to Java's `Comparator<T>`.

#### Core Function
- `compare(&self, a: &T, b: &T) -> Ordering` - Compares two values and returns ordering
- Corresponds to `Fn(&T, &T) -> Ordering` closure

#### Implementations
- `BoxComparator<T>`: Single ownership
- `ArcComparator<T>`: Thread-safe, cloneable
- `RcComparator<T>`: Single-threaded, cloneable

#### Convenience Methods
- `reversed` - Reverses the comparison order
- `then_comparing` - Chains comparators (secondary sort key)
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnComparatorOps` for closures

#### Example

```rust
use prism3_function::{ArcComparator, Comparator};
use std::cmp::Ordering;

// Create a comparator
let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));

assert_eq!(cmp.compare(&5, &3), Ordering::Greater);

// Reverse the order
let reversed = cmp.reversed();
assert_eq!(reversed.compare(&5, &3), Ordering::Less);
```

### Tester

Tests whether a state or condition holds without accepting input parameters. Similar to Java's `BooleanSupplier` but with Rust's ownership semantics.

#### Core Function
- `test(&self) -> bool` - Tests if a state or condition holds
- Corresponds to `Fn() -> bool` closure

#### Implementations
- `BoxTester`: Single ownership, non-cloneable
- `ArcTester`: Thread-safe shared ownership, cloneable
- `RcTester`: Single-threaded shared ownership, cloneable

#### Convenience Methods
- Logical composition: `and`, `or`, `not`
- Type conversions: `into_box`, `into_arc`, `into_rc`
- Extension trait `FnTesterOps` for closures

#### Key Design Philosophy
- **Uses `&self`**: Tester is only responsible for "judgment", not "state management"
- **State management is caller's responsibility**: Tester only reads state, does not modify state
- **Repeatable calls**: The same Tester can call `test()` multiple times
- **No TesterOnce**: Very limited use cases, directly using closures is better

**⚠️ Important: Ownership Transfer in Logical Operations**

All logical composition methods (`and`, `or`, `not`) accept the `other` parameter **by value**, which means:

- **Ownership is transferred**: The `other` tester is consumed and becomes unavailable after the operation
- **To preserve the original**: You must explicitly `clone()` it first (only works for `ArcTester` and `RcTester`)
- **`BoxTester` cannot be cloned**: Once used in a composition, it's consumed

```rust
use prism3_function::{ArcTester, RcTester, BoxTester, Tester};

// ArcTester and RcTester can be cloned
let is_ready = ArcTester::new(|| system_ready());
let is_healthy = ArcTester::new(|| health_check());

// Clone to preserve the original
let combined = is_ready.and(is_healthy.clone());
// is_healthy is still usable because we cloned it
assert!(is_healthy.test());

// BoxTester: Cannot be cloned, will be consumed
let box_tester = BoxTester::new(|| check_condition());
let combined_box = box_tester.and(|| another_check());
// box_tester is no longer available here
```

#### Examples

##### Basic State Checking

```rust
use prism3_function::{BoxTester, Tester};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

// State managed externally
let count = Arc::new(AtomicUsize::new(0));
let count_clone = Arc::clone(&count);

let tester = BoxTester::new(move || {
    count_clone.load(Ordering::Relaxed) <= 3
});

assert!(tester.test());  // true (0)
count.fetch_add(1, Ordering::Relaxed);
assert!(tester.test());  // true (1)
count.fetch_add(1, Ordering::Relaxed);
assert!(tester.test());  // true (2)
count.fetch_add(1, Ordering::Relaxed);
assert!(tester.test());  // true (3)
count.fetch_add(1, Ordering::Relaxed);
assert!(!tester.test()); // false (4)
```

##### Logical Combination

```rust
use prism3_function::{BoxTester, Tester};
use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};

// 模拟系统状态 - 使用原子类型
let cpu_usage = Arc::new(AtomicUsize::new(45)); // 45%
let memory_usage = Arc::new(AtomicUsize::new(60)); // 60%
let disk_space = Arc::new(AtomicUsize::new(25)); // 25% free
let network_connected = Arc::new(AtomicBool::new(true));
let service_running = Arc::new(AtomicBool::new(true));

// 创建各种条件检查器
let cpu_ok = BoxTester::new({
    let cpu = Arc::clone(&cpu_usage);
    move || cpu.load(Ordering::Relaxed) < 80
});

let memory_ok = BoxTester::new({
    let memory = Arc::clone(&memory_usage);
    move || memory.load(Ordering::Relaxed) < 90
});

let disk_ok = BoxTester::new({
    let disk = Arc::clone(&disk_space);
    move || disk.load(Ordering::Relaxed) > 10
});

let network_ok = BoxTester::new({
    let net = Arc::clone(&network_connected);
    move || net.load(Ordering::Relaxed)
});

let service_ok = BoxTester::new({
    let svc = Arc::clone(&service_running);
    move || svc.load(Ordering::Relaxed)
});

// 组合条件：系统健康检查
let system_healthy = cpu_ok
    .and(memory_ok)
    .and(disk_ok)
    .and(network_ok)
    .and(service_ok);

// 组合条件：紧急状态检查（CPU或内存过高）
let emergency_state = BoxTester::new({
    let cpu = Arc::clone(&cpu_usage);
    move || cpu.load(Ordering::Relaxed) > 95
}).or(BoxTester::new({
    let memory = Arc::clone(&memory_usage);
    move || memory.load(Ordering::Relaxed) > 95
}));

// 组合条件：服务可用性检查（网络和服务都正常）
let service_available = network_ok.and(service_ok);

// 组合条件：资源充足检查（CPU、内存、磁盘都正常）
let resources_adequate = cpu_ok.and(memory_ok).and(disk_ok);

// 使用组合条件
assert!(system_healthy.test());
assert!(!emergency_state.test());
assert!(service_available.test());
assert!(resources_adequate.test());

// 复杂的逻辑组合：系统可以处理新请求的条件
let can_handle_requests = service_available
    .and(resources_adequate)
    .and(BoxTester::new({
        let cpu = Arc::clone(&cpu_usage);
        let memory = Arc::clone(&memory_usage);
        move || cpu.load(Ordering::Relaxed) <= 95 && memory.load(Ordering::Relaxed) <= 95
    }));

assert!(can_handle_requests.test());

// 测试状态变化
cpu_usage.store(50, Ordering::Relaxed);
assert!(can_handle_requests.test());

// 模拟CPU使用率过高
cpu_usage.store(98, Ordering::Relaxed);
assert!(!can_handle_requests.test());
```

##### Thread-Safe Sharing

```rust
use prism3_function::{ArcTester, Tester};
use std::thread;

let shared = ArcTester::new(|| true);
let clone = shared.clone();

let handle = thread::spawn(move || {
    clone.test()
});

assert!(handle.join().unwrap());
```

##### Condition Waiting

```rust
use prism3_function::{ArcTester, Tester};
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

// Wait for condition
if wait_until(&tester, Duration::from_secs(5)) {
    println!("Condition met!");
} else {
    println!("Timeout!");
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
prism3-function = "0.1.0"
```

## Design Philosophy

This crate adopts the **Trait + Multiple Implementations** pattern, providing:

1. **Unified Interface**: Each functional type has a trait defining core behavior
2. **Specialized Implementations**: Multiple concrete types optimized for different scenarios
3. **Type Preservation**: Composition methods return the same concrete type
4. **Ownership Flexibility**: Choose between single ownership, thread-safe sharing, or single-threaded sharing
5. **Ergonomic API**: Natural method chaining without explicit cloning

## Comparison Table

| Type | Box (Single) | Arc (Thread-Safe) | Rc (Single-Thread) |
|------|--------------|-------------------|-------------------|
| Predicate | BoxPredicate | ArcPredicate | RcPredicate |
| Consumer | BoxConsumer | ArcConsumer | RcConsumer |
| Mutator | BoxMutator | ArcMutator | RcMutator |
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| Transformer | BoxTransformer | ArcTransformer | RcTransformer |
| BiConsumer | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
| BiPredicate | BoxBiPredicate | ArcBiPredicate | RcBiPredicate |
| BiTransformer | BoxBiTransformer | ArcBiTransformer | RcBiTransformer |
| Comparator | BoxComparator | ArcComparator | RcComparator |
| Tester | BoxTester | ArcTester | RcTester |

#### Legend
- **Box**: Single ownership, cannot be cloned, consumes self
- **Arc**: Shared ownership, thread-safe, cloneable
- **Rc**: Shared ownership, single-threaded, cloneable

## Documentation

- [Predicate Design](doc/predicate_design.md) | [中文](doc/predicate_design.zh_CN.md)
- [Consumer Design](doc/consumer_design.md) | [中文](doc/consumer_design.zh_CN.md)
- [Mutator Design](doc/mutator_design.md) | [中文](doc/mutator_design.zh_CN.md)
- [Supplier Design](doc/supplier_design.md) | [中文](doc/supplier_design.zh_CN.md)
- [Transformer Design](doc/transformer_design.md) | [中文](doc/transformer_design.zh_CN.md)
- [Tester Design](doc/tester_design.md) | - [中文](doc/tester_design.zh_CN.md)

## Examples

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

## License

Licensed under Apache License, Version 2.0.

## Author

Haixing Hu <starfish.hu@gmail.com>
