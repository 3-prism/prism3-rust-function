# Mutator 设计方案

## 概述

本文档阐述 Rust 中实现 Mutator（变异器）类型的设计方案，说明核心语义和设计决策。

## 什么是 Mutator？

### Mutator 的本质语义

在函数式编程中，**Mutator（变异器）**的核心语义是：

> **接受一个可变引用并修改它，可以同时改变变异器自己的状态（如累积、计数），也可以修改被传入的值本身。**

这是对值的"就地修改"行为：
- ✅ **修改输入值**：直接修改传入的可变引用
- ✅ **修改自身状态**：变异器可以累积状态（如计数、历史记录）
- ✅ **组合使用**：多个变异器可以串联执行

### Mutator vs Consumer

基于语义理解，我们需要明确区分两类操作：

| 类型 | 输入参数 | 修改输入？| 改变自己？| 典型用途 | Java 对应 |
|------|---------|----------|----------|---------|-----------|
| **Consumer** | `&T` | ❌ | ✅ | 观察、日志、统计、通知 | `Consumer<T>` |
| **Mutator** | `&mut T` | ✅ | ✅ | 修改、更新、处理、转换 | `UnaryOperator<T>` |

**关键洞察**：
- Consumer 只能**观察和累积**，不修改输入值
- Mutator 可以**就地修改**输入值，也可以累积状态
- Java 的 `UnaryOperator<T>` 返回新值，而 Rust 的 Mutator 就地修改

### Mutator 的主要用途

Mutator 类型的核心价值在于：

1. **保存函数对象**：将表示修改操作的函数体保存在数据结构中（如 struct 的成员）
2. **延迟执行**：稍后在需要的地方调用
3. **简化接口**：作为类型约束（如 `M: Mutator<T>`）提高可读性
4. **条件修改**：结合 Predicate 实现条件修改逻辑

**如果只是临时使用一次，直接用闭包更方便**：
```rust
// ✅ 临时使用：直接用闭包
vec![1, 2, 3].iter_mut().for_each(|x| *x *= 2);

// ✅ 需要保存：用 Mutator
struct DataProcessor {
    transformers: Vec<BoxMutator<Data>>,  // 保存多个转换器
}
```

## 核心设计决策

### 1. 参数的可变性

**共识**：所有叫做 Mutator 的，参数都应该是 `&mut T`。

```rust
// ✅ Mutator：修改输入
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}
```

这与 Consumer 形成清晰对比：
```rust
// Consumer：只观察
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}
```

### 2. self 的可变性

Mutator 自己是否需要可变？这涉及到是否可以修改内部状态：

```rust
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);  // 可修改自己的状态
}
```

**场景对比**：

| 场景 | 需要修改状态？| 适合的类型 | 示例 |
|------|------------|-----------|------|
| 简单修改（翻倍、加10）| ❌ | Mutator | `\|x\| *x *= 2` |
| 带统计的修改 | ✅ | Mutator | 修改并计数 |
| 累积历史记录 | ✅ | Mutator | 修改并记录每次操作 |

**结论**：使用 `&mut self` 允许修改内部状态，提供最大灵活性。

### 3. MutatorOnce 的价值

**关键理解**：MutatorOnce 的价值在于：

1. **可以保存 FnOnce 闭包**：允许移动捕获的变量
2. **延迟执行的一次性操作**：初始化回调、资源转移等

```rust
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);  // 消费 self
}

// 使用场景：保存 FnOnce 闭包
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
            callback.mutate_once(data);  // 只调用一次
        }
    }
}
```

**结论**：MutatorOnce 是有价值的，但优先级低于 Mutator。

### 4. ReadonlyMutator 的合理性

**分析**：ReadonlyMutator 的语义是什么？

```rust
// ❌ 概念矛盾
pub trait ReadonlyMutator<T> {
    fn mutate(&self, value: &mut T);  // self 不可变，但修改输入
}
```

**问题**：
- 如果 self 不可变（`&self`），意味着不修改内部状态
- 但如果需要修改输入（`&mut T`），这是修改操作
- **"Readonly"** 与 **"Mutator"** 语义冲突

**正确的类型选择**：

| 需求 | 正确的类型 | 理由 |
|------|----------|------|
| 不修改自己，不修改输入 | `ReadonlyConsumer<T>` | 纯观察 |
| 修改自己，不修改输入 | `Consumer<T>` | 观察+累积 |
| 不修改自己，修改输入 | ❌ 不合理 | 修改操作需要可追踪 |
| 修改自己，修改输入 | `Mutator<T>` | ✅ 完整的变异器 |

**结论**：ReadonlyMutator 概念矛盾，**不应该存在**。

---

## 推荐的完整设计

### 核心 Trait 定义

```rust
// === Mutator 系列（修改输入）===

/// 变异器：可修改自己，可修改输入
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}

/// 一次性变异器：消费自己，可修改输入（优先级较低）
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}
```

**当前实现状态**：
- ✅ `Mutator` - 已完整实现（`src/mutators/mutator.rs`）
  - ✅ `BoxMutator<T>` - 单一所有权
  - ✅ `ArcMutator<T>` - 线程安全共享
  - ✅ `RcMutator<T>` - 单线程共享
  - ✅ 条件变异器（`when` + `or_else`）
- ❌ `MutatorOnce` - 暂未实现（低优先级）
- ❌ `ReadonlyMutator` - **不应该实现**（概念矛盾）

### 具体实现

#### Mutator 系列（修改输入）

```rust
// Box 实现（单一所有权）
pub struct BoxMutator<T> { func: Box<dyn FnMut(&mut T)> }

// Arc 实现（线程安全共享）
pub struct ArcMutator<T> { func: Arc<Mutex<dyn FnMut(&mut T) + Send>> }

// Rc 实现（单线程共享）
pub struct RcMutator<T> { func: Rc<RefCell<dyn FnMut(&mut T)>> }
```

#### MutatorOnce 系列（未来可选实现）

```rust
// Box 实现（单一所有权）
pub struct BoxMutatorOnce<T> { func: Box<dyn FnOnce(&mut T)> }

// 注意：Arc/Rc 变体与 FnOnce 语义不兼容，不应该实现
```

### 条件变异器设计

Mutator 的一个重要特性是支持条件执行：

```rust
/// 条件变异器（Box 版本）
pub struct BoxConditionalMutator<T> {
    mutator: BoxMutator<T>,
    predicate: BoxPredicate<T>,
}

impl<T> BoxConditionalMutator<T> {
    /// 添加 else 分支
    pub fn or_else<C>(self, else_mutator: C) -> BoxMutator<T>
    where
        C: Mutator<T> + 'static
    {
        // 实现 if-then-else 逻辑
    }
}
```

**使用示例**：
```rust
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)           // 条件：正数
    .or_else(|x: &mut i32| *x -= 1);  // 否则：减1

let mut positive = 5;
mutator.mutate(&mut positive);
assert_eq!(positive, 10);  // 5 * 2

let mut negative = -5;
mutator.mutate(&mut negative);
assert_eq!(negative, -6);  // -5 - 1
```

### 类型选择指南

| 需求 | 推荐类型 | 理由 |
|------|---------|------|
| 一次性使用 | `BoxMutator` | 单一所有权，无开销 |
| 多线程共享 | `ArcMutator` | 线程安全，Mutex 保护 |
| 单线程复用 | `RcMutator` | RefCell 无锁开销 |
| 一次性 + FnOnce | `BoxMutatorOnce` | 保存 FnOnce（未实现）|
| 条件修改 | `BoxConditionalMutator` | 结合 Predicate |

---

## 设计模式对比

### Consumer vs Mutator 完整对比

| 特性 | Consumer | Mutator |
|------|----------|---------|
| **输入参数** | `&T` | `&mut T` |
| **修改输入？** | ❌ | ✅ |
| **修改自己？** | ✅ | ✅ |
| **Java 类似** | `Consumer<T>` | `UnaryOperator<T>` |
| **主要用途** | 观察、日志、统计、通知 | 修改、更新、处理、转换 |
| **ReadOnly 变体** | ✅ `ReadonlyConsumer` | ❌ 概念矛盾 |
| **Once 变体** | ✅ `ConsumerOnce` | 🟡 `MutatorOnce`（可选）|
| **条件执行** | ❌ 暂无 | ✅ `when` + `or_else` |

### 三种所有权模型对比

| 特性 | BoxMutator | ArcMutator | RcMutator |
|------|-----------|-----------|----------|
| **所有权** | 单一 | 共享 | 共享 |
| **克隆性** | ❌ | ✅ | ✅ |
| **线程安全** | ❌ | ✅ | ❌ |
| **内部可变性** | N/A | Mutex | RefCell |
| **`and_then` API** | 消费 `self` | 借用 `&self` | 借用 `&self` |
| **锁开销** | 无 | 有 | 无 |
| **性能** | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |

---

## 实现细节

### 条件变异器的实现

条件变异器是 Mutator 区别于 Consumer 的重要特性之一：

```rust
impl<T> BoxMutator<T> {
    /// 创建条件 mutator
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
    /// 添加 else 分支
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

### 三种变体的统一接口

所有三种变体都实现 `Mutator` trait：

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

### 闭包自动实现

所有 `FnMut(&mut T)` 闭包自动实现 `Mutator` trait：

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

    // ... 其他转换方法
}
```

---

## 使用示例

### 基本使用

```rust
use prism3_function::{Mutator, BoxMutator};

// 简单修改
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 5;
mutator.mutate(&mut value);
assert_eq!(value, 10);

// 方法链
let mut chained = BoxMutator::new(|x: &mut i32| *x *= 2)
    .and_then(|x: &mut i32| *x += 10);
let mut value = 5;
chained.mutate(&mut value);
assert_eq!(value, 20);  // (5 * 2) + 10
```

### 条件修改

```rust
use prism3_function::{Mutator, BoxMutator};

// 简单条件
let mut conditional = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0);

let mut positive = 5;
conditional.mutate(&mut positive);
assert_eq!(positive, 10);  // 执行

let mut negative = -5;
conditional.mutate(&mut negative);
assert_eq!(negative, -5);  // 不执行

// if-then-else
let mut branched = BoxMutator::new(|x: &mut i32| *x *= 2)
    .when(|x: &i32| *x > 0)
    .or_else(|x: &mut i32| *x -= 1);

let mut positive = 5;
branched.mutate(&mut positive);
assert_eq!(positive, 10);  // then 分支

let mut negative = -5;
branched.mutate(&mut negative);
assert_eq!(negative, -6);  // else 分支
```

### 共享使用

```rust
use prism3_function::{Mutator, ArcMutator, RcMutator};

// ArcMutator：线程安全共享
let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
let clone = mutator.clone();

let mut value = 5;
let mut m = mutator;
m.mutate(&mut value);
assert_eq!(value, 10);

// RcMutator：单线程共享（更高效）
let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
let clone = mutator.clone();

let mut value = 5;
let mut m = mutator;
m.mutate(&mut value);
assert_eq!(value, 10);
```

### 泛型编程

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

// 适用于任何 Mutator 类型
let mut box_mut = BoxMutator::new(|x| *x *= 2);
assert_eq!(apply_mutator(&mut box_mut, 5), 10);

let mut closure = |x: &mut i32| *x *= 2;
assert_eq!(apply_mutator(&mut closure, 5), 10);
```

---

## 与 Java 的对比

### Java UnaryOperator vs Rust Mutator

```java
// Java：返回新值
UnaryOperator<Integer> doubler = x -> x * 2;
Integer result = doubler.apply(5);  // result = 10, 原值不变
```

```rust
// Rust：就地修改
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
let mut value = 5;
mutator.mutate(&mut value);  // value = 10, 就地修改
```

**关键差异**：
- Java 的 `UnaryOperator` 是 `Function<T, T>`，返回新值
- Rust 的 `Mutator` 使用可变引用，就地修改
- Rust 方案更高效（无需分配新对象）

---

## 设计原则总结

1. **Mutator 修改输入**：参数必须是 `&mut T`
2. **清晰的语义区分**：Mutator（修改）vs Consumer（观察）
3. **ReadonlyMutator 不存在**：概念矛盾，不应该实现
4. **MutatorOnce 可选**：有价值但优先级低
5. **条件执行支持**：`when` + `or_else` 提供 if-then-else 逻辑
6. **三种所有权模型**：Box（单一）、Arc（线程安全）、Rc（单线程）
7. **统一的 trait 接口**：所有变体实现 `Mutator<T>`
8. **闭包自动实现**：零成本抽象，自然集成

---

## 未来扩展

### MutatorOnce 实现（可选）

```rust
/// 一次性变异器 trait
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}

/// BoxMutatorOnce 实现
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

**使用场景**：
- 资源转移后的清理
- 初始化完成后的回调
- 一次性的复杂修改操作

**注意**：MutatorOnce 不应该有 Arc/Rc 变体，因为 FnOnce 与共享所有权语义冲突。

---

## 总结

### 为什么这样设计 Mutator？

**`prism3-rust-function` 采用当前方案**，原因如下：

1. **清晰的语义**
   - Mutator 专注于修改输入值
   - 与 Consumer（观察）形成清晰对比
   - 避免概念混淆（如 ReadonlyMutator）

2. **完整的所有权模型**
   - Box：单一所有权，零开销
   - Arc：线程安全共享，Mutex 保护
   - Rc：单线程共享，RefCell 优化

3. **条件执行支持**
   - `when` 方法创建条件变异器
   - `or_else` 添加 else 分支
   - 支持复杂的条件修改逻辑

4. **统一的 trait 抽象**
   - 提供 `Mutator<T>` trait
   - 所有类型通过统一接口使用
   - 支持泛型编程

5. **与 Rust 生态一致**
   - 命名模式与标准库智能指针一致（Box/Arc/Rc）
   - 设计哲学符合 Rust 惯例
   - 就地修改比返回新值更高效

6. **长期可维护性**
   - 清晰的架构
   - 易于扩展（未来可添加 MutatorOnce）
   - 类型名称即文档

### 核心设计原则

1. **Mutator 修改输入**：参数必须是 `&mut T`
2. **区分 Consumer 和 Mutator**：语义清晰
3. **不存在 ReadonlyMutator**：概念矛盾
4. **保留 MutatorOnce 可能性**：未来可选实现
5. **类型名称语义明确**：Box/Arc/Rc 表达所有权模型
6. **条件执行是核心特性**：区别于 Consumer 的重要功能

这个设计为用户提供了灵活、强大、清晰的 API，是库项目的最佳选择。

