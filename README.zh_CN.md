# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Rust 提供全面的函数式编程抽象，实现类似 Java 的函数式接口，并结合 Rust 的所有权模型。

## 概述

本 crate 为 Rust 提供一套完整的函数式编程抽象，灵感来自 Java 的函数式接口，并适配 Rust 的所有权系统。它为每种抽象提供多种实现（Box/Arc/Rc），涵盖从简单的单线程场景到复杂的多线程应用的各种使用场景。

## 核心特性

- **完整的函数式接口套件**：Predicate（谓词）、Consumer（消费者）、Supplier（供应者）、Transformer（转换器）、Mutator（变异器）、BiConsumer（双参数消费者）、BiPredicate（双参数谓词）、Comparator（比较器）
- **多种所有权模型**：基于 Box 的单一所有权、基于 Arc 的线程安全共享、基于 Rc 的单线程共享
- **灵活的 API 设计**：基于 trait 的统一接口，针对不同场景优化的具体实现
- **方法链式调用**：所有类型都支持流式 API 和函数组合
- **线程安全选项**：在线程安全（Arc）和高效单线程（Rc）实现之间选择
- **零成本抽象**：高效的实现，最小的运行时开销

## 核心类型

### Predicate<T>（谓词）

测试值是否满足条件，返回 `bool`。类似于 Java 的 `Predicate<T>` 接口。

**实现类型：**
- `BoxPredicate<T>`：单一所有权，不可克隆
- `ArcPredicate<T>`：线程安全的共享所有权，可克隆
- `RcPredicate<T>`：单线程共享所有权，可克隆

**特性：**
- 逻辑组合：`and`、`or`、`not`、`xor`
- 类型保持的方法链式调用
- 为闭包提供的扩展 trait（`FnPredicateOps`）

### Consumer<T>（消费者）

接受单个输入参数并执行操作，不返回结果。类似于 Java 的 `Consumer<T>`。

**实现类型：**
- `BoxConsumer<T>`：单一所有权，使用 `FnMut(&T)`
- `ArcConsumer<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcConsumer<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxConsumerOnce<T>`：一次性使用，使用 `FnOnce(&T)`

**特性：**
- 使用 `and_then` 的方法链式调用
- 支持有状态操作的内部可变性
- 提供线程安全和单线程两种选项
- 只读变体（`ReadonlyConsumer`）用于纯观察

### Mutator<T>（变异器）

通过接受可变引用就地修改值。类似于 Java 的 `UnaryOperator<T>`，但采用就地修改。

**实现类型：**
- `BoxMutator<T>`：单一所有权，使用 `FnMut(&mut T)`
- `ArcMutator<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcMutator<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxMutatorOnce<T>`：一次性使用，使用 `FnOnce(&mut T)`

**特性：**
- 使用 `and_then` 的方法链式调用
- 使用 `when` 和 `or_else` 的条件执行
- 支持 if-then-else 逻辑
- 与 Consumer 区分（读取 vs 修改）

### Supplier<T>（供应者）

无需输入参数即可惰性生成值。类似于 Java 的 `Supplier<T>`。

**实现类型：**
- `BoxSupplier<T>`：单一所有权，使用 `FnMut() -> T`
- `ArcSupplier<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcSupplier<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxSupplierOnce<T>`：一次性使用，使用 `FnOnce() -> T`

**特性：**
- 有状态的值生成
- 方法链式调用：`map`、`filter`、`flat_map`
- 工厂方法：`constant`、`counter`
- 支持序列和生成器

### Transformer<T, R>（转换器）

通过消耗输入将类型 `T` 的值转换为类型 `R`。类似于 Java 的 `Function<T, R>`。

**实现类型：**
- `BoxTransformer<T, R>`：可重复使用，单一所有权（Fn）
- `ArcTransformer<T, R>`：线程安全，可克隆（Arc<Fn>）
- `RcTransformer<T, R>`：单线程，可克隆（Rc<Fn>）
- `BoxTransformerOnce<T, R>`：一次性使用（FnOnce）

**特性：**
- 函数组合：`and_then`、`compose`
- 工厂方法：`identity`、`constant`
- 消耗输入以获得最大灵活性
- 使用 `into_fn` 转换为标准闭包

### BiConsumer<T, U>（双参数消费者）

接受两个输入参数并执行操作，不返回结果。类似于 Java 的 `BiConsumer<T, U>`。

**实现类型：**
- `BoxBiConsumer<T, U>`：单一所有权
- `ArcBiConsumer<T, U>`：线程安全，可克隆
- `RcBiConsumer<T, U>`：单线程，可克隆
- `BoxBiConsumerOnce<T, U>`：一次性使用

**特性：**
- 使用 `and_then` 的方法链式调用
- 只读变体（`ReadonlyBiConsumer`）

### BiPredicate<T, U>（双参数谓词）

测试两个值是否满足条件，返回 `bool`。类似于 Java 的 `BiPredicate<T, U>`。

**实现类型：**
- `BoxBiPredicate<T, U>`：单一所有权
- `ArcBiPredicate<T, U>`：线程安全，可克隆
- `RcBiPredicate<T, U>`：单线程，可克隆

**特性：**
- 逻辑组合：`and`、`or`、`not`

### Comparator<T>（比较器）

比较两个值并返回 `Ordering`。类似于 Java 的 `Comparator<T>`。

**实现类型：**
- `BoxComparator<T>`：单一所有权
- `ArcComparator<T>`：线程安全，可克隆
- `RcComparator<T>`：单线程，可克隆

**特性：**
- 方法链式调用：`reversed`、`then`

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-function = "0.1.0"
```

## 快速入门示例

### 使用 Predicate（谓词）

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

### 使用 Consumer（消费者）

```rust
use prism3_function::{BoxConsumer, Consumer};

// 创建用于观察的消费者（不修改值）
let mut consumer = BoxConsumer::new(|x: &i32| {
    println!("观察到的值: {}", x);
});

let value = 10;
consumer.accept(&value);
// value 保持不变
```

### 使用 Mutator（变异器）

```rust
use prism3_function::{BoxMutator, Mutator};

// 创建修改值的变异器
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
mutator.mutate(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1
```

### 使用条件变异器

```rust
use prism3_function::{BoxMutator, Mutator};

// 创建带 if-then-else 逻辑的条件变异器
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

### 使用 Supplier（供应者）

```rust
use prism3_function::{BoxSupplier, Supplier};

// 创建计数器供应者
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

### 使用 Transformer（转换器）

```rust
use prism3_function::{BoxTransformer, Transformer};

// 链式调用转换器进行数据转换
let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt: Option<i32>| opt.unwrap_or(0))
    .and_then(|x: i32| x * 2);

assert_eq!(parse_and_double.transform("21".to_string()), 42);
```

## 设计理念

本 crate 采用 **Trait + 多实现** 模式，提供：

1. **统一接口**：每个函数式类型都有一个定义核心行为的 trait
2. **专门实现**：针对不同场景优化的多个具体类型
3. **类型保持**：组合方法返回相同的具体类型
4. **所有权灵活性**：在单一所有权、线程安全共享或单线程共享之间选择
5. **人体工学 API**：无需显式克隆的自然方法链式调用

## 类型对比表

| 类型 | Box（单一所有权） | Arc（线程安全） | Rc（单线程） |
|------|--------------|-------------------|-------------------|
| Predicate | BoxPredicate | ArcPredicate | RcPredicate |
| Consumer | BoxConsumer | ArcConsumer | RcConsumer |
| Mutator | BoxMutator | ArcMutator | RcMutator |
| Supplier | BoxSupplier | ArcSupplier | RcSupplier |
| Transformer | BoxTransformer | ArcTransformer | RcTransformer |
| BiConsumer | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
| BiPredicate | BoxBiPredicate | ArcBiPredicate | RcBiPredicate |
| Comparator | BoxComparator | ArcComparator | RcComparator |

**图例：**
- **Box**：单一所有权，不可克隆，消耗 self
- **Arc**：共享所有权，线程安全，可克隆
- **Rc**：共享所有权，单线程，可克隆

## 文档

- [Predicate 设计](doc/predicate_design.zh_CN.md) | [English](doc/predicate_design.md)
- [Consumer 设计](doc/consumer_design.zh_CN.md) | [English](doc/consumer_design.md)
- [Mutator 设计](doc/mutator_design.zh_CN.md) | [English](doc/mutator_design.md)
- [Supplier 设计](doc/supplier_design.zh_CN.md) | [English](doc/supplier_design.md)
- [Transformer 设计](doc/transformer_design.zh_CN.md) | [English](doc/transformer_design.md)

## 示例

`examples/` 目录包含全面的演示：

- `predicate_demo.rs`：Predicate 使用模式
- `consumer_demo.rs`：Consumer 使用模式
- `mutator_demo.rs`：Mutator 使用模式
- `mutator_once_conditional_demo.rs`：条件变异器模式
- `supplier_demo.rs`：Supplier 使用模式
- `transformer_demo.rs`：Transformer 使用模式
- 更多...

运行示例：
```bash
cargo run --example predicate_demo
cargo run --example consumer_demo
cargo run --example mutator_demo
```

## 许可证

采用 Apache License, Version 2.0 许可证。

## 作者

胡海星 <starfish.hu@gmail.com>
