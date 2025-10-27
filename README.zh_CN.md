# Prism3 Function

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-function.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-function)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-function/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-function?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-function.svg?color=blue)](https://crates.io/crates/prism3-function)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Rust 提供全面的函数式编程抽象,实现类似 Java 的函数式接口,并适配 Rust 的所有权模型。

## 概述

本 crate 为 Rust 提供一套完整的函数式编程抽象,灵感来自 Java 的函数式接口,并精心适配 Rust 的所有权系统。它为每种抽象提供多种实现(Box/Arc/Rc),涵盖从简单的单线程场景到复杂的多线程应用的各种使用场景。

## 核心特性

- **完整的函数式接口套件**: 11 种核心函数式抽象及其多种变体
- **多种所有权模型**: 基于 Box 的单一所有权、基于 Arc 的线程安全共享、基于 Rc 的单线程共享
- **灵活的 API 设计**: 基于 trait 的统一接口,针对不同场景优化的具体实现
- **方法链式调用**: 所有类型都支持流式 API 和函数组合
- **线程安全选项**: 在线程安全(Arc)和高效单线程(Rc)实现之间选择
- **零成本抽象**: 高效的实现,最小的运行时开销

## 安装

在 `Cargo.toml` 中添加:

```toml
[dependencies]
prism3-function = "0.1.0"
```

## 核心抽象

本 crate 提供 11 种核心函数式抽象,每种都有多个实现:

### 1. Predicate - 条件测试

测试值是否满足条件,返回 `bool`。

**Trait**: `Predicate<T>`
**核心方法**: `test(&self, value: &T) -> bool`
**等价闭包**: `Fn(&T) -> bool`

**实现类型**:
- `BoxPredicate<T>` - 单一所有权,不可克隆
- `ArcPredicate<T>` - 线程安全,可克隆
- `RcPredicate<T>` - 单线程,可克隆

**示例**:
```rust
use prism3_function::{Predicate, ArcPredicate};

let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

let combined = is_even.and(is_positive.clone());
assert!(combined.test(&4));
assert!(!combined.test(&-2));
```

### 2. BiPredicate - 双值条件测试

测试两个值是否满足条件,返回 `bool`。

**Trait**: `BiPredicate<T, U>`
**核心方法**: `test(&self, first: &T, second: &U) -> bool`
**等价闭包**: `Fn(&T, &U) -> bool`

**实现类型**:
- `BoxBiPredicate<T, U>` - 单一所有权
- `ArcBiPredicate<T, U>` - 线程安全
- `RcBiPredicate<T, U>` - 单线程

**示例**:
```rust
use prism3_function::{BiPredicate, BoxBiPredicate};

let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
assert!(sum_positive.test(&3, &4));
assert!(!sum_positive.test(&-5, &2));
```

### 3. Consumer - 值观察

接受值引用并执行操作,不返回结果。

**Trait**: `Consumer<T>`
**核心方法**: `accept(&self, value: &T)`
**等价闭包**: `Fn(&T)`

**实现类型**:
- `BoxConsumer<T>` - 单一所有权
- `ArcConsumer<T>` - 线程安全
- `RcConsumer<T>` - 单线程

**示例**:
```rust
use prism3_function::{Consumer, BoxConsumer};

let logger = BoxConsumer::new(|x: &i32| {
    println!("值: {}", x);
});
logger.accept(&42);
```

### 4. ConsumerOnce - 一次性值观察

接受值引用并执行一次操作。

**Trait**: `ConsumerOnce<T>`
**核心方法**: `accept_once(self, value: &T)`
**等价闭包**: `FnOnce(&T)`

**实现类型**:
- `BoxConsumerOnce<T>` - 单一所有权,一次性使用

### 5. BiConsumer - 双值观察

接受两个值引用并执行操作,不返回结果。

**Trait**: `BiConsumer<T, U>`
**核心方法**: `accept(&self, first: &T, second: &U)`
**等价闭包**: `Fn(&T, &U)`

**实现类型**:
- `BoxBiConsumer<T, U>` - 单一所有权
- `ArcBiConsumer<T, U>` - 线程安全
- `RcBiConsumer<T, U>` - 单线程

**示例**:
```rust
use prism3_function::{BiConsumer, BoxBiConsumer};

let sum_logger = BoxBiConsumer::new(|x: &i32, y: &i32| {
    println!("和: {}", x + y);
});
sum_logger.accept(&10, &20);
```

### 6. BiConsumerOnce - 一次性双值观察

接受两个值引用并执行一次操作。

**Trait**: `BiConsumerOnce<T, U>`
**核心方法**: `accept_once(self, first: &T, second: &U)`
**等价闭包**: `FnOnce(&T, &U)`

**实现类型**:
- `BoxBiConsumerOnce<T, U>` - 单一所有权,一次性使用

### 7. Mutator - 就地值修改

通过接受可变引用就地修改值。

**Trait**: `Mutator<T>`
**核心方法**: `mutate(&mut self, value: &mut T)`
**等价闭包**: `FnMut(&mut T)`

**实现类型**:
- `BoxMutator<T>` - 单一所有权
- `ArcMutator<T>` - 线程安全
- `RcMutator<T>` - 单线程

**示例**:
```rust
use prism3_function::{Mutator, BoxMutator};

let mut doubler = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 10;
doubler.mutate(&mut value);
assert_eq!(value, 20);
```

### 8. MutatorOnce - 一次性就地修改

就地修改值一次。

**Trait**: `MutatorOnce<T>`
**核心方法**: `mutate_once(self, value: &mut T)`
**等价闭包**: `FnOnce(&mut T)`

**实现类型**:
- `BoxMutatorOnce<T>` - 单一所有权,一次性使用

### 9. Supplier - 值生成

无需输入参数即可生成值。

**Trait**: `Supplier<T>`
**核心方法**: `get(&self) -> T`
**等价闭包**: `Fn() -> T`

**实现类型**:
- `BoxSupplier<T>` - 单一所有权,无锁
- `ArcSupplier<T>` - 线程安全,无锁
- `RcSupplier<T>` - 单线程

**示例**:
```rust
use prism3_function::{Supplier, BoxSupplier};

let factory = BoxSupplier::new(|| String::from("你好"));
assert_eq!(factory.get(), "你好");
```

### 10. SupplierOnce - 一次性值生成

无需输入参数生成一次值。

**Trait**: `SupplierOnce<T>`
**核心方法**: `get_once(self) -> T`
**等价闭包**: `FnOnce() -> T`

**实现类型**:
- `BoxSupplierOnce<T>` - 单一所有权,一次性使用

### 11. StatefulSupplier - 有状态值生成

使用可变状态生成值。

**Trait**: `StatefulSupplier<T>`
**核心方法**: `get(&mut self) -> T`
**等价闭包**: `FnMut() -> T`

**实现类型**:
- `BoxStatefulSupplier<T>` - 单一所有权
- `ArcStatefulSupplier<T>` - 线程安全(使用 Mutex)
- `RcStatefulSupplier<T>` - 单线程(使用 RefCell)

**示例**:
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

### 12. Function - 引用转换

转换值引用以产生结果,不消耗输入。

**Trait**: `Function<T, R>`
**核心方法**: `apply(&self, input: &T) -> R`
**等价闭包**: `Fn(&T) -> R`

**实现类型**:
- `BoxFunction<T, R>` - 单一所有权
- `ArcFunction<T, R>` - 线程安全
- `RcFunction<T, R>` - 单线程

**示例**:
```rust
use prism3_function::{Function, BoxFunction};

let to_string = BoxFunction::new(|x: &i32| format!("值: {}", x));
assert_eq!(to_string.apply(&42), "值: 42");
```

### 13. FunctionOnce - 一次性引用转换

转换值引用一次以产生结果。

**Trait**: `FunctionOnce<T, R>`
**核心方法**: `apply_once(self, input: &T) -> R`
**等价闭包**: `FnOnce(&T) -> R`

**实现类型**:
- `BoxFunctionOnce<T, R>` - 单一所有权,一次性使用

### 14. StatefulFunction - 有状态引用转换

使用可变状态转换值引用。

**Trait**: `StatefulFunction<T, R>`
**核心方法**: `apply(&mut self, input: &T) -> R`
**等价闭包**: `FnMut(&T) -> R`

**实现类型**:
- `BoxStatefulFunction<T, R>` - 单一所有权
- `ArcStatefulFunction<T, R>` - 线程安全(使用 Mutex)
- `RcStatefulFunction<T, R>` - 单线程(使用 RefCell)

### 15. Transformer - 消耗式值转换

通过消耗输入将类型 `T` 的值转换为类型 `R`。

**Trait**: `Transformer<T, R>`
**核心方法**: `transform(&self, input: T) -> R`
**等价闭包**: `Fn(T) -> R`

**实现类型**:
- `BoxTransformer<T, R>` - 单一所有权
- `ArcTransformer<T, R>` - 线程安全
- `RcTransformer<T, R>` - 单线程

**类型别名**: `UnaryOperator<T>` = `Transformer<T, T>`

**示例**:
```rust
use prism3_function::{Transformer, BoxTransformer};

let parse = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));
assert_eq!(parse.transform("42".to_string()), 42);
```

### 16. TransformerOnce - 一次性值转换

通过消耗转换器和输入转换值一次。

**Trait**: `TransformerOnce<T, R>`
**核心方法**: `transform_once(self, input: T) -> R`
**等价闭包**: `FnOnce(T) -> R`

**实现类型**:
- `BoxTransformerOnce<T, R>` - 单一所有权,一次性使用

**类型别名**: `UnaryOperatorOnce<T>` = `TransformerOnce<T, T>`

### 17. StatefulTransformer - 有状态值转换

使用可变状态通过消耗输入转换值。

**Trait**: `StatefulTransformer<T, R>`
**核心方法**: `transform(&mut self, input: T) -> R`
**等价闭包**: `FnMut(T) -> R`

**实现类型**:
- `BoxStatefulTransformer<T, R>` - 单一所有权
- `ArcStatefulTransformer<T, R>` - 线程安全(使用 Mutex)
- `RcStatefulTransformer<T, R>` - 单线程(使用 RefCell)

### 18. BiTransformer - 双值转换

通过消耗输入转换两个输入值以产生结果。

**Trait**: `BiTransformer<T, U, R>`
**核心方法**: `transform(&self, first: T, second: U) -> R`
**等价闭包**: `Fn(T, U) -> R`

**实现类型**:
- `BoxBiTransformer<T, U, R>` - 单一所有权
- `ArcBiTransformer<T, U, R>` - 线程安全
- `RcBiTransformer<T, U, R>` - 单线程

**类型别名**: `BinaryOperator<T>` = `BiTransformer<T, T, T>`

**示例**:
```rust
use prism3_function::{BiTransformer, BoxBiTransformer};

let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
assert_eq!(add.transform(10, 20), 30);
```

### 20. BiTransformerOnce - 一次性双值转换

通过消耗所有内容转换两个值一次。

**Trait**: `BiTransformerOnce<T, U, R>`
**核心方法**: `transform_once(self, first: T, second: U) -> R`
**等价闭包**: `FnOnce(T, U) -> R`

**实现类型**:
- `BoxBiTransformerOnce<T, U, R>` - 单一所有权,一次性使用

**类型别名**: `BinaryOperatorOnce<T>` = `BiTransformerOnce<T, T, T>`

### 21. StatefulConsumer - 有状态值观察

使用可变状态接受值引用。

**Trait**: `StatefulConsumer<T>`
**核心方法**: `accept(&mut self, value: &T)`
**等价闭包**: `FnMut(&T)`

**实现类型**:
- `BoxStatefulConsumer<T>` - 单一所有权
- `ArcStatefulConsumer<T>` - 线程安全(使用 Mutex)
- `RcStatefulConsumer<T>` - 单线程(使用 RefCell)

### 22. StatefulBiConsumer - 有状态双值观察

使用可变状态接受两个值引用。

**Trait**: `StatefulBiConsumer<T, U>`
**核心方法**: `accept(&mut self, first: &T, second: &U)`
**等价闭包**: `FnMut(&T, &U)`

**实现类型**:
- `BoxStatefulBiConsumer<T, U>` - 单一所有权
- `ArcStatefulBiConsumer<T, U>` - 线程安全(使用 Mutex)
- `RcStatefulBiConsumer<T, U>` - 单线程(使用 RefCell)

### 23. Comparator - 值比较

比较两个值并返回 `Ordering`。

**Trait**: `Comparator<T>`
**核心方法**: `compare(&self, a: &T, b: &T) -> Ordering`
**等价闭包**: `Fn(&T, &T) -> Ordering`

**实现类型**:
- `BoxComparator<T>` - 单一所有权
- `ArcComparator<T>` - 线程安全
- `RcComparator<T>` - 单线程

**示例**:
```rust
use prism3_function::{Comparator, BoxComparator};
use std::cmp::Ordering;

let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
```

### 24. Tester - 无输入条件测试

测试状态或条件是否成立,不接受输入。

**Trait**: `Tester`
**核心方法**: `test(&self) -> bool`
**等价闭包**: `Fn() -> bool`

**实现类型**:
- `BoxTester` - 单一所有权
- `ArcTester` - 线程安全
- `RcTester` - 单线程

**示例**:
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

## Trait 与闭包对应表

| Trait | 核心方法签名 | 等价闭包类型 |
|-------|------------|-------------|
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

## 实现类型对比

每个 trait 基于所有权模型都有多种实现:

| Trait | Box(单一所有权) | Arc(线程安全) | Rc(单线程) |
|-------|----------------|--------------|-----------|
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

**图例**:
- **Box**: 单一所有权,不可克隆,消耗 self
- **Arc**: 共享所有权,线程安全,可克隆
- **Rc**: 共享所有权,单线程,可克隆
- **-**: 不适用(Once 类型不需要共享)

## 设计理念

本 crate 采用 **Trait + 多实现** 模式:

1. **统一接口**: 每个函数式类型都有一个定义核心行为的 trait
2. **专门实现**: 针对不同场景优化的多个具体类型
3. **类型保持**: 组合方法返回相同的具体类型
4. **所有权灵活性**: 在单一所有权、线程安全共享或单线程共享之间选择
5. **人体工学 API**: 自然的方法链式调用和函数组合

## 示例

`examples/` 目录包含每种类型的全面演示。运行示例:

```bash
cargo run --example predicate_demo
cargo run --example consumer_demo
cargo run --example transformer_demo
```

## 文档

`doc/` 目录中提供了每个主要抽象的详细设计文档。

## 许可证

采用 Apache License, Version 2.0 许可证。

## 作者

胡海星 <starfish.hu@gmail.com>
