# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

[English](#english) | [中文](#中文)

---

<a name="english"></a>

## English

Common functional programming type aliases for Rust, providing Java-style functional interfaces.

### Overview

This crate provides comprehensive functional programming abstractions for Rust, inspired by Java's functional interfaces. It offers a complete set of functional types with multiple ownership models to cover various use cases, from simple single-threaded scenarios to complex multi-threaded applications.

### Key Features

- **Complete Functional Interface Suite**: Predicate, Consumer, Supplier, Function, and Transformer
- **Multiple Ownership Models**: Box-based single ownership, Arc-based thread-safe sharing, and Rc-based single-threaded sharing
- **Flexible API Design**: Trait-based unified interface with concrete implementations optimized for different scenarios
- **Method Chaining**: All types support fluent API and functional composition
- **Thread-Safety Options**: Choose between thread-safe (Arc) and efficient single-threaded (Rc) implementations
- **Zero-Cost Abstractions**: Efficient implementations with minimal runtime overhead

### Core Types

#### Predicate<T>

Represents a condition test that returns `bool`. Similar to Java's `Predicate<T>` interface.

**Implementations:**
- `BoxPredicate<T>`: Single ownership, one-time use
- `ArcPredicate<T>`: Thread-safe shared ownership, cloneable
- `RcPredicate<T>`: Single-threaded shared ownership, cloneable

**Features:**
- Logical composition: `and`, `or`, `not`, `xor`
- Method chaining with type preservation
- Extension trait for closures

#### Consumer<T>

Accepts a single input parameter and performs operations without returning a result. Similar to Java's `Consumer<T>`.

**Implementations:**
- `BoxConsumer<T>`: Single ownership, consumes self
- `ArcConsumer<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcConsumer<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable

**Features:**
- Method chaining with `and_then`
- Interior mutability for stateful operations
- Both thread-safe and single-threaded options

#### Supplier<T>

Generates values lazily without input parameters. Similar to Java's `Supplier<T>`.

**Implementations:**
- `BoxSupplier<T>`: Single ownership, one-time use
- `ArcSupplier<T>`: Thread-safe with `Arc<Mutex<>>`, cloneable
- `RcSupplier<T>`: Single-threaded with `Rc<RefCell<>>`, cloneable

**Features:**
- Stateful value generation
- Method chaining: `map`, `filter`, `flat_map`
- Support for sequences and counters

#### Function<T, R>

Transforms values from type `T` to type `R`. Similar to Java's `Function<T, R>`.

**Implementations:**
- `BoxOnceFunction<T, R>`: Single ownership, one-time use (FnOnce)
- `BoxFnFunction<T, R>`: Reusable, single ownership (Fn)
- `ArcFnFunction<T, R>`: Thread-safe, cloneable (Arc<Fn>)
- `RcFnFunction<T, R>`: Single-threaded, cloneable (Rc<Fn>)

**Features:**
- Function composition: `and_then`, `compose`
- Both one-time and reusable variants
- Support for consuming transformations

#### Transformer<T>

Specialization of `Function<T, T>` for same-type transformations.

**Implementations:**
- `BoxTransformer<T>`: Single ownership, one-time use
- `BoxFnTransformer<T>`: Reusable, single ownership
- `ArcFnTransformer<T>`: Thread-safe, cloneable
- `RcFnTransformer<T>`: Single-threaded, cloneable

**Features:**
- Optimized for same-type transformations
- Convenient `into_fn()` for iterator integration
- Method chaining: `and_then`, `compose`

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

// Create a predicate with logical composition
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

// Create a consumer that modifies values
let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
consumer.accept(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1
```

#### Using Supplier

```rust
use prism3_function::{BoxSupplier, Supplier};

// Create a counter supplier
let mut counter = 0;
let mut supplier = BoxSupplier::new(move || {
    counter += 1;
    counter
});

assert_eq!(supplier.get(), 1);
assert_eq!(supplier.get(), 2);
assert_eq!(supplier.get(), 3);
```

#### Using Function

```rust
use prism3_function::{BoxOnceFunction, Function};

// Chain functions for data transformation
let parse_and_double = BoxOnceFunction::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt| opt.unwrap_or(0))
    .and_then(|x| x * 2);

assert_eq!(parse_and_double.apply("21".to_string()), 42);
```

#### Using Transformer

```rust
use prism3_function::{BoxFnTransformer, Transformer};

// Create a reusable transformer
let double = BoxFnTransformer::new(|x: i32| x * 2);

// Use with iterators
let values = vec![1, 2, 3, 4, 5];
let result: Vec<i32> = values.into_iter()
    .map(double.into_fn())
    .collect();

assert_eq!(result, vec![2, 4, 6, 8, 10]);
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
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| Function | BoxOnceFunction<br>BoxFnFunction | ArcFnFunction | RcFnFunction |
| Transformer | BoxTransformer<br>BoxFnTransformer | ArcFnTransformer | RcFnTransformer |

**Legend:**
- **Box**: Single ownership, cannot be cloned, consumes self
- **Arc**: Shared ownership, thread-safe, cloneable
- **Rc**: Shared ownership, single-threaded, cloneable

### License

Licensed under Apache License, Version 2.0.

### Author

Hu Haixing <starfish.hu@gmail.com>

---

<a name="中文"></a>

## 中文

为 Rust 提供常用函数式编程类型别名，实现类似 Java 的函数式接口。

### 概述

本 crate 为 Rust 提供全面的函数式编程抽象，灵感来自 Java 的函数式接口。它提供了一套完整的函数式类型，支持多种所有权模型，涵盖从简单的单线程场景到复杂的多线程应用的各种使用场景。

### 核心特性

- **完整的函数式接口套件**：Predicate（谓词）、Consumer（消费者）、Supplier（供应者）、Function（函数）和 Transformer（转换器）
- **多种所有权模型**：基于 Box 的单一所有权、基于 Arc 的线程安全共享、基于 Rc 的单线程共享
- **灵活的 API 设计**：基于 trait 的统一接口，针对不同场景优化的具体实现
- **方法链式调用**：所有类型都支持流式 API 和函数组合
- **线程安全选项**：在线程安全（Arc）和高效单线程（Rc）实现之间选择
- **零成本抽象**：高效的实现，最小的运行时开销

### 核心类型

#### Predicate<T>（谓词）

表示返回 `bool` 的条件测试。类似于 Java 的 `Predicate<T>` 接口。

**实现类型：**
- `BoxPredicate<T>`：单一所有权，一次性使用
- `ArcPredicate<T>`：线程安全的共享所有权，可克隆
- `RcPredicate<T>`：单线程共享所有权，可克隆

**特性：**
- 逻辑组合：`and`、`or`、`not`、`xor`
- 类型保持的方法链式调用
- 为闭包提供的扩展 trait

#### Consumer<T>（消费者）

接受单个输入参数并执行操作，不返回结果。类似于 Java 的 `Consumer<T>`。

**实现类型：**
- `BoxConsumer<T>`：单一所有权，消耗 self
- `ArcConsumer<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcConsumer<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆

**特性：**
- 使用 `and_then` 的方法链式调用
- 支持有状态操作的内部可变性
- 提供线程安全和单线程两种选项

#### Supplier<T>（供应者）

无需输入参数即可惰性生成值。类似于 Java 的 `Supplier<T>`。

**实现类型：**
- `BoxSupplier<T>`：单一所有权，一次性使用
- `ArcSupplier<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcSupplier<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆

**特性：**
- 有状态的值生成
- 方法链式调用：`map`、`filter`、`flat_map`
- 支持序列和计数器

#### Function<T, R>（函数）

将类型 `T` 的值转换为类型 `R`。类似于 Java 的 `Function<T, R>`。

**实现类型：**
- `BoxOnceFunction<T, R>`：单一所有权，一次性使用（FnOnce）
- `BoxFnFunction<T, R>`：可重复使用，单一所有权（Fn）
- `ArcFnFunction<T, R>`：线程安全，可克隆（Arc<Fn>）
- `RcFnFunction<T, R>`：单线程，可克隆（Rc<Fn>）

**特性：**
- 函数组合：`and_then`、`compose`
- 同时提供一次性和可重复使用的变体
- 支持消耗型转换

#### Transformer<T>（转换器）

`Function<T, T>` 的特化版本，用于相同类型的转换。

**实现类型：**
- `BoxTransformer<T>`：单一所有权，一次性使用
- `BoxFnTransformer<T>`：可重复使用，单一所有权
- `ArcFnTransformer<T>`：线程安全，可克隆
- `RcFnTransformer<T>`：单线程，可克隆

**特性：**
- 针对相同类型转换进行优化
- 提供 `into_fn()` 方便集成迭代器
- 方法链式调用：`and_then`、`compose`

### 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-function = "0.1.0"
```

### 快速入门示例

#### 使用 Predicate（谓词）

```rust
use prism3_function::{ArcPredicate, Predicate, FnPredicateOps};

// 创建带逻辑组合的谓词
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// 组合谓词，同时保持可克隆性
let is_even_and_positive = is_even.and(&is_positive);

assert!(is_even_and_positive.test(&4));
assert!(!is_even_and_positive.test(&3));

// 与闭包一起使用
let numbers = vec![1, 2, 3, 4, 5, 6];
let result: Vec<i32> = numbers
    .into_iter()
    .filter(|x| (|n: &i32| *n > 2).and(|n: &i32| n % 2 == 0).test(x))
    .collect();
```

#### 使用 Consumer（消费者）

```rust
use prism3_function::{BoxConsumer, Consumer};

// 创建修改值的消费者
let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
consumer.accept(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1
```

#### 使用 Supplier（供应者）

```rust
use prism3_function::{BoxSupplier, Supplier};

// 创建计数器供应者
let mut counter = 0;
let mut supplier = BoxSupplier::new(move || {
    counter += 1;
    counter
});

assert_eq!(supplier.get(), 1);
assert_eq!(supplier.get(), 2);
assert_eq!(supplier.get(), 3);
```

#### 使用 Function（函数）

```rust
use prism3_function::{BoxOnceFunction, Function};

// 链式调用函数进行数据转换
let parse_and_double = BoxOnceFunction::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt| opt.unwrap_or(0))
    .and_then(|x| x * 2);

assert_eq!(parse_and_double.apply("21".to_string()), 42);
```

#### 使用 Transformer（转换器）

```rust
use prism3_function::{BoxFnTransformer, Transformer};

// 创建可重复使用的转换器
let double = BoxFnTransformer::new(|x: i32| x * 2);

// 与迭代器一起使用
let values = vec![1, 2, 3, 4, 5];
let result: Vec<i32> = values.into_iter()
    .map(double.into_fn())
    .collect();

assert_eq!(result, vec![2, 4, 6, 8, 10]);
```

### 设计理念

本 crate 采用 **Trait + 多实现** 模式，提供：

1. **统一接口**：每个函数式类型都有一个定义核心行为的 trait
2. **专门实现**：针对不同场景优化的多个具体类型
3. **类型保持**：组合方法返回相同的具体类型
4. **所有权灵活性**：在单一所有权、线程安全共享或单线程共享之间选择
5. **人体工学 API**：无需显式克隆的自然方法链式调用

### 类型对比表

| 类型 | Box（单一所有权） | Arc（线程安全） | Rc（单线程） |
|------|--------------|-------------------|-------------------|
| Predicate | BoxPredicate | ArcPredicate | RcPredicate |
| Consumer | BoxConsumer | ArcConsumer | RcConsumer |
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| Function | BoxOnceFunction<br>BoxFnFunction | ArcFnFunction | RcFnFunction |
| Transformer | BoxTransformer<br>BoxFnTransformer | ArcFnTransformer | RcFnTransformer |

**图例：**
- **Box**：单一所有权，不可克隆，消耗 self
- **Arc**：共享所有权，线程安全，可克隆
- **Rc**：共享所有权，单线程，可克隆

### 许可证

采用 Apache License, Version 2.0 许可证。

### 作者

胡海星 <starfish.hu@gmail.com>

