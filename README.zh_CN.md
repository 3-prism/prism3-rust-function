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

- **完整的函数式接口套件**：Predicate（谓词）、Consumer（消费者）、Supplier（供应者）、ReadonlySupplier（只读供应者）、Transformer（转换器）、Mutator（变异器）、BiConsumer（双参数消费者）、BiPredicate（双参数谓词）、BiTransformer（双参数转换器）、Comparator（比较器）、Tester（测试器）
- **多种所有权模型**：基于 Box 的单一所有权、基于 Arc 的线程安全共享、基于 Rc 的单线程共享
- **灵活的 API 设计**：基于 trait 的统一接口，针对不同场景优化的具体实现
- **方法链式调用**：所有类型都支持流式 API 和函数组合
- **线程安全选项**：在线程安全（Arc）和高效单线程（Rc）实现之间选择
- **无锁并发**：ReadonlySupplier 为无状态场景提供无锁并发访问
- **零成本抽象**：高效的实现，最小的运行时开销

## 核心类型

### Predicate<T>（谓词）

测试值是否满足条件，返回 `bool`。类似于 Java 的 `Predicate<T>` 接口。

#### 核心函数
- `test(&self, value: &T) -> bool` - 测试值是否满足谓词条件
- 对应于 `Fn(&T) -> bool` 闭包

#### 实现类型
- `BoxPredicate<T>`：单一所有权，不可克隆
- `ArcPredicate<T>`：线程安全的共享所有权，可克隆
- `RcPredicate<T>`：单线程共享所有权，可克隆

#### 便利方法
- 逻辑组合：`and`、`or`、`not`、`xor`、`nand`、`nor`
- 类型保持的方法链式调用（每个方法返回相同的具体类型）
- 扩展 trait `FnPredicateOps` 用于闭包 - 提供返回 `BoxPredicate` 的组合方法

**⚠️ 重要：逻辑操作中的所有权转移**

所有逻辑组合方法（`and`、`or`、`xor`、`nand`、`nor`）都是**按值传递** `other` 参数，这意味着：

- **所有权被转移**：`other` 谓词被消耗，操作后将不可再使用
- **保留原始谓词**：必须显式调用 `clone()` 克隆它（仅适用于 `ArcPredicate` 和 `RcPredicate`）
- **`BoxPredicate` 不可克隆**：一旦用于组合操作，就会被消耗

```rust
use prism3_function::{ArcPredicate, RcPredicate, BoxPredicate, Predicate};

// ArcPredicate 和 RcPredicate 可以被克隆
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// 选项 1：克隆以保留原始谓词
let combined = is_even.and(is_positive.clone());
// is_positive 仍然可用，因为我们克隆了它
assert!(is_positive.test(&2));

// 选项 2：使用 &self 方法（仅适用于 Rc/Arc）
let is_even_rc = RcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive_rc = RcPredicate::new(|x: &i32| *x > 0);
let combined_rc = is_even_rc.and(is_positive_rc.clone());
// 两个谓词都仍然可用
assert!(is_even_rc.test(&2));
assert!(is_positive_rc.test(&2));

// BoxPredicate：不可克隆，会被消耗
let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
let combined_box = box_pred.and(|x: &i32| x % 2 == 0);
// box_pred 在这里已不可用
```

#### 示例

```rust
use prism3_function::{ArcPredicate, Predicate, FnPredicateOps};

// 创建带逻辑组合的谓词
let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
let is_positive = ArcPredicate::new(|x: &i32| *x > 0);

// 克隆以保留原始谓词
let is_even_and_positive = is_even.and(is_positive.clone());

assert!(is_even_and_positive.test(&4));
assert!(!is_even_and_positive.test(&3));
// is_positive 仍然可用
assert!(is_positive.test(&5));

// 与闭包一起使用 - 扩展 trait 自动提供组合功能
let numbers = vec![1, 2, 3, 4, 5, 6];
let result: Vec<i32> = numbers
    .into_iter()
    .filter(|x| (|n: &i32| *n > 2).and(|n: &i32| n % 2 == 0).test(x))
    .collect();
```

### Consumer<T>（消费者）

接受单个输入参数并执行操作，不返回结果。类似于 Java 的 `Consumer<T>`。

#### 核心函数
- `accept(&mut self, value: &T)` - 对值引用执行操作
- 对应于 `FnMut(&T)` 闭包

#### 实现类型
- `BoxConsumer<T>`：单一所有权，使用 `FnMut(&T)`
- `ArcConsumer<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcConsumer<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxConsumerOnce<T>`：一次性使用，使用 `FnOnce(&T)`

#### 便利方法
- `and_then` - 顺序链接消费者
- `when` - 使用谓词进行条件执行
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnConsumerOps` 用于闭包

#### 相关类型
- `ReadonlyConsumer` - 用于纯观察，不修改消费者状态

**⚠️ 重要：组合方法中的所有权转移**

所有组合方法（`and_then`、`when`、`or_else`）都是**按值传递**参数，这意味着：

- **所有权会被转移**：参数（消费者或谓词）会被消耗，操作后将不可再用
- **如需保留原始对象**：必须先显式 `clone()`（仅适用于 `ArcConsumer` 和 `RcConsumer`）
- **BoxConsumer 不可克隆**：一旦用于组合，就会被消耗且不再可用

#### 示例

##### 基本用法

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

##### 使用 `and_then` 链式调用

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// 链式组合多个消费者
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log1.lock().unwrap().push(format!("第一步: {}", x));
})
.and_then(move |x: &i32| {
    log2.lock().unwrap().push(format!("第二步: {}", x));
});

consumer.accept(&42);
assert_eq!(log.lock().unwrap().len(), 2);
```

##### 使用 `when` 条件执行

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log_clone = log.clone();

// 仅当谓词为真时执行消费者
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log_clone.lock().unwrap().push(*x);
})
.when(|x: &i32| *x > 0);  // 只记录正数

consumer.accept(&10);   // 被记录
consumer.accept(&-5);   // 不被记录
assert_eq!(log.lock().unwrap().len(), 1);
```

##### 使用 `or_else` 实现条件分支

```rust
use prism3_function::{BoxConsumer, Consumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// 根据条件执行不同的消费者
let mut consumer = BoxConsumer::new(move |x: &i32| {
    log1.lock().unwrap().push(format!("正数: {}", x));
})
.when(|x: &i32| *x > 0)
.or_else(move |x: &i32| {
    log2.lock().unwrap().push(format!("非正数: {}", x));
});

consumer.accept(&10);   // "正数: 10"
consumer.accept(&-5);   // "非正数: -5"
assert_eq!(log.lock().unwrap().len(), 2);
```

### Mutator<T>（变异器）

通过接受可变引用就地修改值。类似于 Java 的 `UnaryOperator<T>`，但采用就地修改。

#### 核心函数
- `mutate(&mut self, value: &mut T)` - 就地修改值
- 对应于 `FnMut(&mut T)` 闭包

#### 实现类型
- `BoxMutator<T>`：单一所有权，使用 `FnMut(&mut T)`
- `ArcMutator<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcMutator<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxMutatorOnce<T>`：一次性使用，使用 `FnOnce(&mut T)`

#### 便利方法
- `and_then` - 顺序链接变异器
- `when` - 创建条件变异器（if-then 模式）
- `or_else` - 为条件变异器添加 else 分支（if-then-else 模式）
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnMutatorOps` 用于闭包

#### 与 Consumer 的主要区别
- **Consumer**：接受 `&T`（读取值，不修改输入）
- **Mutator**：接受 `&mut T`（就地修改值）

#### 示例

```rust
use prism3_function::{BoxMutator, Mutator};

// 创建修改值的变异器
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 1);

let mut value = 10;
mutator.mutate(&mut value);
assert_eq!(value, 21); // (10 * 2) + 1

// 带 if-then-else 逻辑的条件变异器
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

### Supplier<T>（供应者）

无需输入参数即可惰性生成值。类似于 Java 的 `Supplier<T>`。

#### 核心函数
- `get(&mut self) -> T` - 生成并返回一个值
- 对应于 `FnMut() -> T` 闭包

#### 实现类型
- `BoxSupplier<T>`：单一所有权，使用 `FnMut() -> T`
- `ArcSupplier<T>`：使用 `Arc<Mutex<>>` 的线程安全实现，可克隆
- `RcSupplier<T>`：使用 `Rc<RefCell<>>` 的单线程实现，可克隆
- `BoxSupplierOnce<T>`：一次性使用，使用 `FnOnce() -> T`

#### 便利方法
- `map` - 转换供应者输出
- `filter` - 使用谓词过滤供应者输出
- `flat_map` - 链接供应者
- 工厂方法：`constant`、`counter`
- 类型转换：`into_box`、`into_arc`、`into_rc`

#### 示例

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

// 使用 map 进行方法链式调用
let mut pipeline = BoxSupplier::new(|| 10)
    .map(|x| x * 2)
    .map(|x| x + 5);

assert_eq!(pipeline.get(), 25);
```

### ReadonlySupplier<T>（只读供应者）

无需输入参数即可惰性生成值，且**不修改自身状态**。与 `Supplier<T>` 不同，它使用 `&self` 而不是 `&mut self`，可在只读上下文中使用，并支持无锁并发访问。

#### 核心函数
- `get(&self) -> T` - 生成并返回一个值（注意：是 `&self`，不是 `&mut self`）
- 对应于 `Fn() -> T` 闭包

#### 与 Supplier 的关键区别

| 方面 | Supplier | ReadonlySupplier |
|------|----------|------------------|
| self 签名 | `&mut self` | `&self` |
| 闭包类型 | `FnMut() -> T` | `Fn() -> T` |
| 可修改状态 | 是 | 否 |
| Arc 实现 | `Arc<Mutex<FnMut>>` | `Arc<Fn>`（无锁！） |
| 使用场景 | 计数器、生成器 | 工厂、常量、高并发 |

#### 实现类型
- `BoxReadonlySupplier<T>`：单一所有权，零开销
- `ArcReadonlySupplier<T>`：**无锁**线程安全共享（不需要 `Mutex`！）
- `RcReadonlySupplier<T>`：单线程共享，轻量级

#### 便利方法
- `map` - 转换供应者输出
- `filter` - 使用谓词过滤供应者输出
- `zip` - 组合两个供应者
- 工厂方法：`constant`
- 类型转换：`into_box`、`into_arc`、`into_rc`

#### 使用场景

##### 1. 在 `&self` 方法中调用

```rust
use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};

struct Executor<E> {
    error_supplier: ArcReadonlySupplier<E>,
}

impl<E> Executor<E> {
    fn execute(&self) -> Result<(), E> {
        // 可以在 &self 方法中直接调用！
        Err(self.error_supplier.get())
    }
}
```

##### 2. 高并发无锁访问

```rust
use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
use std::thread;

let factory = ArcReadonlySupplier::new(|| String::from("Hello, World!"));

let handles: Vec<_> = (0..10)
    .map(|_| {
        let f = factory.clone();
        thread::spawn(move || f.get()) // 无锁！
    })
    .collect();

for h in handles {
    assert_eq!(h.join().unwrap(), "Hello, World!");
}
```

##### 3. 固定工厂

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

#### 性能对比

对于多线程环境中的无状态场景：

- `ArcSupplier<T>`：需要 `Mutex`，每次 `get()` 调用都有锁竞争
- `ArcReadonlySupplier<T>`：无锁，可以并发调用 `get()` 而无竞争

基准测试结果显示，在高并发场景下 `ArcReadonlySupplier` 比 `ArcSupplier` **快 10 倍**。

### Transformer<T, R>（转换器）

通过消耗输入将类型 `T` 的值转换为类型 `R`。类似于 Java 的 `Function<T, R>`。

#### 核心函数
- `transform(&self, input: T) -> R` - 将输入值转换为输出值（消耗输入）
- 对应于 `Fn(T) -> R` 闭包

#### 实现类型
- `BoxTransformer<T, R>`：可重复使用，单一所有权（Fn）
- `ArcTransformer<T, R>`：线程安全，可克隆（Arc<Fn>）
- `RcTransformer<T, R>`：单线程，可克隆（Rc<Fn>）
- `BoxTransformerOnce<T, R>`：一次性使用（FnOnce）

#### 便利方法
- `and_then` - 顺序组合转换器（f.and_then(g) = g(f(x))）
- `compose` - 逆序组合转换器（f.compose(g) = f(g(x))）
- `when` - 使用谓词创建条件转换器
- 工厂方法：`identity`、`constant`
- 类型转换：`into_box`、`into_arc`、`into_rc`、`into_fn`
- 扩展 trait `FnTransformerOps` 用于闭包

#### 相关类型
- `UnaryOperator<T>` - `Transformer<T, T>` 的类型别名

**⚠️ 重要：组合方法中的所有权转移**

所有组合方法（`and_then`、`compose`、`when`、`or_else`）都是**按值传递**参数，这意味着：

- **所有权会被转移**：参数（转换器或谓词）会被消耗，操作后将不可再用
- **如需保留原始对象**：必须先显式 `clone()`（仅适用于 `ArcTransformer` 和 `RcTransformer`）
- **BoxTransformer 不可克隆**：一旦用于组合，就会被消耗且不再可用

#### 示例

##### 基本用法和 `and_then` 链式调用

```rust
use prism3_function::{BoxTransformer, Transformer};

// 链式调用转换器进行数据转换
let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt: Option<i32>| opt.unwrap_or(0))
    .and_then(|x: i32| x * 2);

assert_eq!(parse_and_double.apply("21".to_string()), 42);
assert_eq!(parse_and_double.apply("invalid".to_string()), 0);
```

##### 使用 `when` 条件转换

```rust
use prism3_function::{BoxTransformer, Transformer};

// 仅当谓词为真时应用转换
let double_if_positive = BoxTransformer::new(|x: i32| x * 2)
    .when(|x: &i32| *x > 0);

assert_eq!(double_if_positive.apply(5), Some(10));
assert_eq!(double_if_positive.apply(-5), None);
```

##### 使用 `or_else` 实现条件分支

```rust
use prism3_function::{BoxTransformer, Transformer};

// 根据条件执行不同的转换
let transform = BoxTransformer::new(|x: i32| format!("正数: {}", x * 2))
    .when(|x: &i32| *x > 0)
    .or_else(|x: i32| format!("非正数: {}", x - 1));

assert_eq!(transform.apply(5), "正数: 10");
assert_eq!(transform.apply(-5), "非正数: -6");
```

### BiConsumer<T, U>（双参数消费者）

接受两个输入参数并执行操作，不返回结果。类似于 Java 的 `BiConsumer<T, U>`。

#### 核心函数
- `accept(&mut self, first: &T, second: &U)` - 对两个值引用执行操作
- 对应于 `FnMut(&T, &U)` 闭包

#### 实现类型
- `BoxBiConsumer<T, U>`：单一所有权
- `ArcBiConsumer<T, U>`：线程安全，可克隆
- `RcBiConsumer<T, U>`：单线程，可克隆
- `BoxBiConsumerOnce<T, U>`：一次性使用

#### 便利方法
- `and_then` - 顺序链接双参数消费者
- `when` - 使用双参数谓词进行条件执行
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnBiConsumerOps` 用于闭包

#### 相关类型
- `ReadonlyBiConsumer` - 用于纯观察，不修改消费者状态

**⚠️ 重要：组合方法中的所有权转移**

所有组合方法（`and_then`、`when`、`or_else`）都是**按值传递**参数，这意味着：

- **所有权会被转移**：参数（双参数消费者或双参数谓词）会被消耗，操作后将不可再用
- **如需保留原始对象**：必须先显式 `clone()`（仅适用于 `ArcBiConsumer` 和 `RcBiConsumer`）
- **BoxBiConsumer 不可克隆**：一旦用于组合，就会被消耗且不再可用

#### 示例

##### 基本用法

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};

// 创建用于配对操作的双参数消费者
let mut bi_consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
    println!("和: {}", x + y);
});

bi_consumer.accept(&10, &20);
```

##### 使用 `and_then` 链式调用

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// 链式组合多个双参数消费者
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log1.lock().unwrap().push(format!("和: {}", x + y));
})
.and_then(move |x: &i32, y: &i32| {
    log2.lock().unwrap().push(format!("积: {}", x * y));
});

bi_consumer.accept(&3, &4);
assert_eq!(log.lock().unwrap().len(), 2);
// log 包含: ["和: 7", "积: 12"]
```

##### 使用 `when` 条件执行

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log_clone = log.clone();

// 仅当两个值都为正时执行
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log_clone.lock().unwrap().push(format!("{} + {} = {}", x, y, x + y));
})
.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

bi_consumer.accept(&3, &4);   // 被记录
bi_consumer.accept(&-1, &4);  // 不被记录
assert_eq!(log.lock().unwrap().len(), 1);
```

##### 使用 `or_else` 实现条件分支

```rust
use prism3_function::{BoxBiConsumer, BiConsumer};
use std::sync::{Arc, Mutex};

let log = Arc::new(Mutex::new(Vec::new()));
let log1 = log.clone();
let log2 = log.clone();

// 根据条件执行不同的操作
let mut bi_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    log1.lock().unwrap().push(format!("都为正: {} + {} = {}", x, y, x + y));
})
.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
.or_else(move |x: &i32, y: &i32| {
    log2.lock().unwrap().push(format!("有负数: {} * {} = {}", x, y, x * y));
});

bi_consumer.accept(&3, &4);   // "都为正: 3 + 4 = 7"
bi_consumer.accept(&-1, &4);  // "有负数: -1 * 4 = -4"
assert_eq!(log.lock().unwrap().len(), 2);
```

### BiPredicate<T, U>（双参数谓词）

测试两个值是否满足条件，返回 `bool`。类似于 Java 的 `BiPredicate<T, U>`。

#### 核心函数
- `test(&self, first: &T, second: &U) -> bool` - 测试两个值是否满足谓词条件
- 对应于 `Fn(&T, &U) -> bool` 闭包

#### 实现类型
- `BoxBiPredicate<T, U>`：单一所有权，不可克隆
- `ArcBiPredicate<T, U>`：线程安全，可克隆
- `RcBiPredicate<T, U>`：单线程，可克隆

#### 便利方法
- 逻辑组合：`and`、`or`、`not`、`xor`、`nand`、`nor`
- 类型保持的方法链式调用
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnBiPredicateOps` 用于闭包

**⚠️ 重要：逻辑操作中的所有权转移**

所有逻辑组合方法（`and`、`or`、`xor`、`nand`、`nor`）都是**按值传递** `other` 参数，这意味着：

- **所有权被转移**：`other` 双参数谓词被消耗，操作后将不可再使用
- **保留原始谓词**：必须显式调用 `clone()` 克隆它（仅适用于 `ArcBiPredicate` 和 `RcBiPredicate`）
- **`BoxBiPredicate` 不可克隆**：一旦用于组合操作，就会被消耗

```rust
use prism3_function::{ArcBiPredicate, RcBiPredicate, BoxBiPredicate, BiPredicate};

// ArcBiPredicate 和 RcBiPredicate 可以被克隆
let is_sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let first_larger = ArcBiPredicate::new(|x: &i32, y: &i32| x > y);

// 克隆以保留原始谓词
let combined = is_sum_positive.and(first_larger.clone());
// first_larger 仍然可用，因为我们克隆了它
assert!(first_larger.test(&10, &5));

// BoxBiPredicate：不可克隆，会被消耗
let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let combined_box = box_pred.and(|x: &i32, y: &i32| x > y);
// box_pred 在这里已不可用
```

#### 示例

```rust
use prism3_function::{ArcBiPredicate, BiPredicate};

// 创建带逻辑组合的双参数谓词
let is_sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
let first_larger = ArcBiPredicate::new(|x: &i32, y: &i32| x > y);

// 克隆以保留原始谓词
let combined = is_sum_positive.and(first_larger.clone());

assert!(combined.test(&10, &5));
assert!(!combined.test(&3, &8));
// first_larger 仍然可用
assert!(first_larger.test(&10, &5));
```

### BiTransformer<T, U, R>（双参数转换器）

转换两个输入值以产生结果值。类似于 Java 的 `BiFunction<T, U, R>`。

#### 核心函数
- `transform(&self, first: T, second: U) -> R` - 将两个输入值转换为输出值（消耗输入）
- 对应于 `Fn(T, U) -> R` 闭包

#### 实现类型
- `BoxBiTransformer<T, U, R>`：可重复使用，单一所有权（Fn）
- `ArcBiTransformer<T, U, R>`：线程安全，可克隆（Arc<Fn>）
- `RcBiTransformer<T, U, R>`：单线程，可克隆（Rc<Fn>）
- `BoxBiTransformerOnce<T, U, R>`：一次性使用（FnOnce）

#### 便利方法
- `and_then` - 将双参数转换器与转换器组合
- `when` - 使用双参数谓词创建条件双参数转换器
- 类型转换：`into_box`、`into_arc`、`into_rc`、`into_fn`
- 扩展 trait `FnBiTransformerOps` 用于闭包

#### 相关类型
- `BinaryOperator<T>` - `BiTransformer<T, T, T>` 的类型别名

**⚠️ 重要：组合方法中的所有权转移**

所有组合方法（`and_then`、`when`、`or_else`）都是**按值传递**参数，这意味着：

- **所有权会被转移**：参数（转换器或双参数谓词）会被消耗，操作后将不可再用
- **如需保留原始对象**：必须先显式 `clone()`（仅适用于 `ArcBiTransformer` 和 `RcBiTransformer`）
- **BoxBiTransformer 不可克隆**：一旦用于组合，就会被消耗且不再可用

#### 示例

##### 基本用法和 `and_then` 链式调用

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// 创建用于组合两个值的双参数转换器
let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);

assert_eq!(add.apply(10, 20), 30);

// 与转换器链接进行进一步处理
let add_and_double = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .and_then(|sum: i32| sum * 2);
assert_eq!(add_and_double.apply(10, 20), 60);

// 多重链式调用
let complex = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .and_then(|sum: i32| sum * 2)
    .and_then(|doubled: i32| format!("结果: {}", doubled));
assert_eq!(complex.apply(10, 20), "结果: 60");
```

##### 使用 `when` 条件转换

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// 仅当两个值都为正时转换
let add_if_positive = BoxBiTransformer::new(|x: i32, y: i32| x + y)
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0);

assert_eq!(add_if_positive.apply(3, 4), Some(7));
assert_eq!(add_if_positive.apply(-1, 4), None);
assert_eq!(add_if_positive.apply(3, -4), None);
```

##### 使用 `or_else` 实现条件分支

```rust
use prism3_function::{BoxBiTransformer, BiTransformer};

// 根据条件执行不同的转换
let transform = BoxBiTransformer::new(|x: i32, y: i32| format!("和: {}", x + y))
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(|x: i32, y: i32| format!("积: {}", x * y));

assert_eq!(transform.apply(3, 4), "和: 7");
assert_eq!(transform.apply(-1, 4), "积: -4");
assert_eq!(transform.apply(3, -4), "积: -12");
```

### Comparator<T>（比较器）

比较两个值并返回 `Ordering`。类似于 Java 的 `Comparator<T>`。

#### 核心函数
- `compare(&self, a: &T, b: &T) -> Ordering` - 比较两个值并返回排序
- 对应于 `Fn(&T, &T) -> Ordering` 闭包

#### 实现类型
- `BoxComparator<T>`：单一所有权
- `ArcComparator<T>`：线程安全，可克隆
- `RcComparator<T>`：单线程，可克隆

#### 便利方法
- `reversed` - 反转比较顺序
- `then_comparing` - 链接比较器（次要排序键）
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnComparatorOps` 用于闭包

#### 示例

```rust
use prism3_function::{ArcComparator, Comparator};
use std::cmp::Ordering;

// 创建比较器
let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));

assert_eq!(cmp.compare(&5, &3), Ordering::Greater);

// 反转顺序
let reversed = cmp.reversed();
assert_eq!(reversed.compare(&5, &3), Ordering::Less);
```

### Tester（测试器）

测试状态或条件是否成立，不接受输入参数。类似于 Java 的 `BooleanSupplier`，但具有 Rust 的所有权语义。

#### 核心函数
- `test(&self) -> bool` - 测试状态或条件是否成立
- 对应于 `Fn() -> bool` 闭包

#### 实现类型
- `BoxTester`：单一所有权，不可克隆
- `ArcTester`：线程安全的共享所有权，可克隆
- `RcTester`：单线程共享所有权，可克隆

#### 便利方法
- 逻辑组合：`and`、`or`、`not`
- 类型转换：`into_box`、`into_arc`、`into_rc`
- 扩展 trait `FnTesterOps` 用于闭包

#### 核心设计理念
- **使用 `&self`**：Tester 只负责"判断"，不负责"状态管理"
- **状态管理是调用方的职责**：Tester 只读取状态，不修改状态
- **可重复调用**：同一个 Tester 可以多次调用 `test()`
- **无 TesterOnce**：使用场景极少，直接使用闭包更好

**⚠️ 重要：逻辑操作中的所有权转移**

所有逻辑组合方法（`and`、`or`、`not`）都是**按值传递** `other` 参数，这意味着：

- **所有权被转移**：`other` 测试器被消耗，操作后将不可再使用
- **保留原始测试器**：必须显式调用 `clone()` 克隆它（仅适用于 `ArcTester` 和 `RcTester`）
- **`BoxTester` 不可克隆**：一旦用于组合操作，就会被消耗

```rust
use prism3_function::{ArcTester, RcTester, BoxTester, Tester};

// ArcTester 和 RcTester 可以被克隆
let is_ready = ArcTester::new(|| system_ready());
let is_healthy = ArcTester::new(|| health_check());

// 克隆以保留原始测试器
let combined = is_ready.and(is_healthy.clone());
// is_healthy 仍然可用，因为我们克隆了它
assert!(is_healthy.test());

// BoxTester：不可克隆，会被消耗
let box_tester = BoxTester::new(|| check_condition());
let combined_box = box_tester.and(|| another_check());
// box_tester 在这里已不可用
```

#### 示例

##### 基本状态检查

```rust
use prism3_function::{BoxTester, Tester};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

// 状态由外部管理
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

##### 逻辑组合

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

##### 线程安全共享

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

##### 条件等待

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

// 使用
let ready = Arc::new(AtomicBool::new(false));
let ready_clone = Arc::clone(&ready);
let tester = ArcTester::new(move || {
    ready_clone.load(Ordering::Acquire)
});

// 另一个线程设置标志
let ready_clone2 = Arc::clone(&ready);
thread::spawn(move || {
    thread::sleep(Duration::from_secs(2));
    ready_clone2.store(true, Ordering::Release);
});

// 等待条件
if wait_until(&tester, Duration::from_secs(5)) {
    println!("条件满足！");
} else {
    println!("超时！");
}
```

##### 健康检查

```rust
use prism3_function::{BoxTester, Tester};

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

// 使用
let checker = HealthChecker::new();
let health_test = checker.create_health_tester();

if health_test.test() {
    println!("系统健康");
} else {
    println!("系统不健康");
}
```

##### 重试限制

```rust
use prism3_function::{BoxTester, Tester};

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

// 使用
retry_with_limit(|| {
    send_request()
}, 3)?;
```

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-function = "0.1.0"
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
| BiTransformer | BoxBiTransformer | ArcBiTransformer | RcBiTransformer |
| Comparator | BoxComparator | ArcComparator | RcComparator |
| Tester | BoxTester | ArcTester | RcTester |

#### 图例
- **Box**：单一所有权，不可克隆，消耗 self
- **Arc**：共享所有权，线程安全，可克隆
- **Rc**：共享所有权，单线程，可克隆

## 文档

- [Predicate 设计](doc/predicate_design.zh_CN.md) | [English](doc/predicate_design.md)
- [Consumer 设计](doc/consumer_design.zh_CN.md) | [English](doc/consumer_design.md)
- [Mutator 设计](doc/mutator_design.zh_CN.md) | [English](doc/mutator_design.md)
- [Supplier 设计](doc/supplier_design.zh_CN.md) | [English](doc/supplier_design.md)
- [Transformer 设计](doc/transformer_design.zh_CN.md) | [English](doc/transformer_design.md)
- [Tester 设计](doc/tester_design.zh_CN.md) | [English](doc/tester_design.md)

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
