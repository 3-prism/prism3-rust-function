# Consumer 设计方案对比分析

## 概述

本文档分析 Rust 中实现 Consumer（消费者）类型的设计方案，阐明核心语义和设计决策。

## 什么是 Consumer？

### Consumer 的本质语义

在函数式编程中，**Consumer（消费者）**的核心语义是：

> **接受一个值并使用它，可能改变消费者自己的状态（如累积、计数），但不应该修改被消费的值本身。**

这类似于现实生活中的"消费"行为：
- ✅ **消费食物**：食物被吃掉（使用），消费者获得营养（状态改变）
- ✅ **消费信息**：信息被读取（使用），消费者获得知识（状态改变）
- ❌ **修改食物**：这不是"消费"，而是"加工"

### Consumer vs Mutator

基于这个语义理解，我们需要明确区分两类操作：

| 类型 | 输入参数 | 修改输入？| 改变自己？| 典型用途 | Java 对应 |
|------|---------|----------|----------|---------|-----------|
| **Consumer** | `&T` | ❌ | ✅ | 观察、日志、统计、通知 | `Consumer<T>` |
| **Mutator** | `&mut T` | ✅ | ✅ | 修改、更新、处理、转换 | `UnaryOperator<T>` |

**关键洞察**：
- 如果你需要**修改输入值**，那不是 Consumer，应该叫 **Mutator**（变异器）
- Consumer 可以**修改自己的状态**（计数、累积），但**不修改输入**

**实现说明**：
- ✅ 本项目采用了 `Mutator` 命名（`src/mutator.rs`）
- ✅ Consumer 系列保持 `&T` 参数（不修改输入）
- ✅ Mutator 系列使用 `&mut T` 参数（可修改输入）

### Consumer 的主要用途

Consumer 类型的核心价值在于：

1. **保存函数对象**：将表示消费操作的函数体保存在数据结构中（如 struct 的成员）
2. **延迟执行**：稍后在需要的地方调用
3. **简化接口**：作为类型约束（如 `C: Consumer<T>`）提高可读性

**如果只是临时使用一次，直接用闭包更方便**：
```rust
// ✅ 临时使用：直接用闭包
vec![1, 2, 3].iter().for_each(|x| println!("{}", x));

// ✅ 需要保存：用 Consumer
struct EventSystem {
    handlers: Vec<BoxConsumer<Event>>,  // 保存多个处理器
}
```

## 核心设计决策

### 1. 参数的可变性

**共识**：所有叫做 Consumer 的，参数都应该是 `&T` 而不是 `&mut T`。

```rust
// ✅ Consumer：消费但不修改输入
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}

// ✅ Mutator：修改输入（不是 Consumer）
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}
```

### 2. self 的可变性

Consumer 自己是否需要可变？这涉及到是否可以修改内部状态：

```rust
// 方案 A：ReadonlyConsumer（不可变 self）
pub trait ReadonlyConsumer<T> {
    fn accept(&self, value: &T);  // 不修改自己
}

// 方案 B：Consumer（可变 self）
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);  // 可修改自己的状态
}
```

**场景对比**：

| 场景 | 需要修改状态？| 适合的类型 |
|------|------------|-----------|
| 纯观察（打印、日志）| ❌ | ReadonlyConsumer |
| 统计计数 | ✅ | Consumer |
| 累积数据 | ✅ | Consumer |
| 事件通知（观察者模式）| ❌ | ReadonlyConsumer |

**建议**：同时提供两者，满足不同场景的需求。

### 3. ConsumerOnce 的价值

**关键理解**：ConsumerOnce 的价值不在于参数的所有权（`T` vs `&T`），而在于：

1. **可以保存 FnOnce 闭包**：允许移动捕获的变量
2. **延迟执行的一次性操作**：初始化回调、清理回调等

```rust
pub trait ConsumerOnce<T> {
    fn accept(self, value: &T);  // 消费 self，但参数仍是 &T
}

// 使用场景：保存 FnOnce 闭包
struct Initializer {
    on_complete: Option<BoxConsumerOnce<InitResult>>,
}

impl Initializer {
    fn new<F>(callback: F) -> Self
    where
        F: FnOnce(&InitResult) + 'static  // FnOnce 闭包
    {
        Self {
            on_complete: Some(BoxConsumerOnce::new(callback))
        }
    }

    fn run(mut self) {
        let result = self.do_init();
        if let Some(callback) = self.on_complete {
            callback.accept_once(&result);  // 只调用一次
        }
    }
}
```

**结论**：ConsumerOnce 是必要的，但签名应该是 `accept(self, &T)` 而不是 `accept(self, T)`。

---

## 三种实现方案对比

### 方案一：类型别名 + 静态组合方法

使用类型别名定义 Consumer 类型，并通过静态工具类提供组合方法。

```rust
// 类型别名定义
pub type Consumer<T> = Box<dyn FnMut(&T)>;
pub type ReadonlyConsumer<T> = Arc<dyn Fn(&T) + Send>;

// 静态组合工具类
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

**使用示例**：
```rust
// 创建 consumer
let mut consumer: Consumer<i32> = Box::new(|x| println!("{}", x));

// 直接调用
let value = 5;
consumer(&value);  // ✅ 可以直接调用

// 组合
let mut chained = Consumers::and_then(
    |x: &i32| println!("First: {}", x),
    |x: &i32| println!("Second: {}", x),
);
```

**优点**：
- ✅ 极简的 API，直接调用 `consumer(&value)`
- ✅ 与标准库完美集成（可用于 `for_each` 等）
- ✅ 零成本抽象，单次装箱
- ✅ 实现简单，代码量少

**缺点**：
- ❌ 无法扩展（不能添加字段、实现 trait）
- ❌ 类型区分度低（与 `Box<dyn FnMut>` 等价）
- ❌ 无法实现方法链（只能嵌套调用）
- ❌ ReadonlyConsumer 仍需要显式处理共享（Arc）

---

### 方案二：Struct 封装 + 实例方法

将 Consumer 定义为 struct，内部包装 `Box<dyn FnMut>`，通过实例方法提供组合能力。

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

**使用示例**：
```rust
// 创建和调用
let mut consumer = Consumer::new(|x: &i32| println!("{}", x));
let value = 5;
consumer.accept_once(&value);  // 必须使用 .accept_once()

// 方法链
let mut chained = Consumer::new(|x: &i32| println!("First: {}", x))
    .and_then(|x| println!("Second: {}", x));

// ReadonlyConsumer 可以克隆和共享
let shared = ReadonlyConsumer::new(|x: &i32| println!("{}", x));
let clone = shared.clone();
shared.accept_once(&5);
clone.accept_once(&10);
```

**优点**：
- ✅ 优雅的方法链（`.and_then()`）
- ✅ 强大的扩展性（可添加字段、实现 trait）
- ✅ 类型安全，独立的类型
- ✅ 丰富的工厂方法

**缺点**：
- ❌ 无法直接调用（必须用 `.accept_once()`）
- ❌ 需要维护两套独立实现（Consumer 和 ReadonlyConsumer）
- ❌ 代码重复（组合方法需要分别实现）
- ❌ 所有权问题（`and_then` 消耗 self）

---

### 方案三：Trait 抽象 + 多种实现（推荐，当前采用）

定义统一的 `Consumer` trait，提供三种具体实现（Box/Arc/Rc），在 struct 上实现特例化的组合方法。

```rust
// ============================================================================
// 1. 统一的 Consumer trait
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

    // ... 类似的 into_* 方法
}

// ============================================================================
// 2. 为闭包实现 Consumer trait
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

    // ... 其他 into_* 方法
}

// ============================================================================
// 3. BoxConsumer - 单一所有权实现
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

    /// 消耗 self，返回 BoxConsumer
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

    // ... into_* 方法实现
}

// ============================================================================
// 4. ArcConsumer - 线程安全的共享所有权实现
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

    /// 借用 &self，返回 ArcConsumer
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

    // ... into_* 方法实现
}

impl<T> Clone for ArcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 5. RcConsumer - 单线程的共享所有权实现
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

    /// 借用 &self，返回 RcConsumer
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

    // ... into_* 方法实现
}

impl<T> Clone for RcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. ReadonlyConsumer 实现（类似结构）
// ============================================================================

pub struct BoxReadonlyConsumer<T> {
    func: Box<dyn Fn(&T)>,
}

pub struct ArcReadonlyConsumer<T> {
    func: Arc<dyn Fn(&T) + Send>,  // 不需要 Mutex
}

pub struct RcReadonlyConsumer<T> {
    func: Rc<dyn Fn(&T)>,  // 不需要 RefCell
}

// ... 实现类似，但使用 Fn 而不是 FnMut
```

**使用示例**：
```rust
// 1. 闭包自动拥有 .accept_once() 方法
let mut closure = |x: &i32| println!("{}", x);
closure.accept_once(&5);  // ✅ 直接使用

// 2. 闭包可以组合，返回 BoxConsumer
let mut chained = (|x: &i32| println!("First: {}", x))
    .and_then(|x| println!("Second: {}", x));
chained.accept_once(&5);

// 3. BoxConsumer - 一次性使用
let consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
let mut combined = consumer.and_then(|x| println!("Done: {}", x));

// 4. ArcConsumer - 多线程共享，不需要显式 clone
let shared = ArcConsumer::new(|x: &i32| println!("{}", x));
let combined = shared.and_then(&ArcConsumer::new(|x| println!("Then: {}", x)));
// shared 仍然可用
let clone = shared.clone();
std::thread::spawn(move || {
    let mut c = clone;
    c.accept_once(&5);
});

// 5. RcConsumer - 单线程复用
let rc = RcConsumer::new(|x: &i32| println!("{}", x));
let combined1 = rc.and_then(&RcConsumer::new(|x| println!("A: {}", x)));
let combined2 = rc.and_then(&RcConsumer::new(|x| println!("B: {}", x)));
// rc 仍然可用

// 6. 统一的接口
fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: i32) {
    let val = value;
    consumer.accept_once(&val);
}

let mut box_con = BoxConsumer::new(|x| println!("{}", x));
apply_consumer(&mut box_con, 5);

let mut arc_con = ArcConsumer::new(|x| println!("{}", x));
apply_consumer(&mut arc_con, 5);
```

**优点**：
- ✅ 统一的 trait 接口（所有类型实现 `Consumer<T>`）
- ✅ 语义清晰（`BoxConsumer`/`ArcConsumer`/`RcConsumer` 名称即文档）
- ✅ 完整的所有权模型覆盖（Box/Arc/Rc 三种）
- ✅ 类型保持（`ArcConsumer.and_then()` 返回 `ArcConsumer`）
- ✅ 优雅的 API（Arc/Rc 的组合方法使用 `&self`，无需显式 clone）
- ✅ 解决内部可变性（Arc 用 Mutex，Rc 用 RefCell，各有优化）
- ✅ 最强的扩展性（可添加新实现、字段、trait）
- ✅ 与 Rust 标准库设计哲学一致

**缺点**：
- ❌ 仍然无法直接调用（必须用 `.accept_once()`）
- ❌ 学习成本略高（需要理解三种实现的区别）
- ❌ 实现成本高（需要为三个 struct 分别实现）

---

## 三种方案对比总结

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 ⭐ |
|:---|:---:|:---:|:---:|
| **调用方式** | `consumer(&value)` ✅ | `consumer.accept_once(&value)` | `consumer.accept_once(&value)` |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 **极好** ✨ |
| **统一接口** | ❌ 无 | ❌ 两套独立 | ✅ **统一 trait** ✨ |
| **所有权模型** | Box + Arc（两种）| Box + Arc（两种）| Box + Arc + Rc（三种）✅ |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ **支持（且类型保持）** ✨ |
| **扩展性** | ❌ 无法扩展 | ✅ 可扩展 | ✅ **极易扩展** |
| **内部可变性** | 手动处理 | 手动处理 | ✅ **三种方式优化** |
| **代码简洁度** | ✅ **极简** | 🟡 中等 | 🟡 略复杂 |
| **学习成本** | ✅ **最低** | 🟡 中等 | 🟡 略高 |
| **维护成本** | 🟡 中等 | 🟡 中等 | ✅ **低（架构清晰）** |
| **与标准库一致** | 🟡 中等 | 🟡 中等 | ✅ **完美** ✨ |

### 适用场景对比

| 场景 | 方案一 | 方案二 | 方案三 ⭐ |
|:---|:---:|:---:|:---:|
| **快速原型开发** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **复杂方法链** | ❌ 不适合 | ✅ 适合 | ✅ **最佳** |
| **多线程共享** | 🟡 手动 Arc | 🟡 ReadonlyConsumer | ✅ **ArcConsumer（清晰）** |
| **单线程复用** | ❌ 不支持 | ❌ 不支持 | ✅ **RcConsumer（无锁）** |
| **库开发** | 🟡 可以 | ✅ 适合 | ✅ **最佳** |
| **长期维护** | 🟡 中等 | 🟡 中等 | ✅ **最佳** |

---

## 推荐的完整设计

### 核心 Trait 定义

```rust
// === Consumer 系列（不修改输入）===

/// 只读消费者：不修改自己，不修改输入
pub trait ReadonlyConsumer<T> {
    fn accept(&self, value: &T);
}

/// 消费者：可修改自己，不修改输入
pub trait Consumer<T> {
    fn accept(&mut self, value: &T);
}

/// 一次性消费者：消费自己，不修改输入
pub trait ConsumerOnce<T> {
    fn accept(self, value: &T);
}

// === Mutator 系列（修改输入）===

/// 修改器：可修改自己，可修改输入
pub trait Mutator<T> {
    fn mutate(&mut self, value: &mut T);
}

/// 一次性修改器：消费自己，可修改输入（暂未实现）
pub trait MutatorOnce<T> {
    fn mutate_once(self, value: &mut T);
}
```

**当前实现状态**：
- ✅ `ReadonlyConsumer` - 已实现（`src/readonly_consumer.rs`）
- ✅ `Consumer` - 已实现（`src/consumer.rs`）
- ✅ `ConsumerOnce` - 已实现（`src/consumer_once.rs`）
- ✅ `Mutator` - 已实现（`src/mutator.rs`），原名为 `ConsumerMut`
- ❌ `MutatorOnce` - 暂未实现（低优先级）

### 具体实现

#### Consumer 系列（不修改输入）

```rust
// Box 实现（单一所有权）
pub struct BoxReadonlyConsumer<T> { func: Box<dyn Fn(&T)> }
pub struct BoxConsumer<T> { func: Box<dyn FnMut(&T)> }
pub struct BoxConsumerOnce<T> { func: Box<dyn FnOnce(&T)> }

// Arc 实现（线程安全共享）
pub struct ArcReadonlyConsumer<T> { func: Arc<dyn Fn(&T) + Send> }
pub struct ArcConsumer<T> { func: Arc<Mutex<dyn FnMut(&T) + Send>> }

// Rc 实现（单线程共享）
pub struct RcReadonlyConsumer<T> { func: Rc<dyn Fn(&T)> }
pub struct RcConsumer<T> { func: Rc<RefCell<dyn FnMut(&T)>> }
```

#### Mutator 系列（修改输入）

```rust
// Box 实现（单一所有权）
pub struct BoxMutator<T> { func: Box<dyn FnMut(&mut T)> }

// Arc 实现（线程安全共享）
pub struct ArcMutator<T> { func: Arc<Mutex<dyn FnMut(&mut T) + Send>> }

// Rc 实现（单线程共享）
pub struct RcMutator<T> { func: Rc<RefCell<dyn FnMut(&mut T)>> }
```

### 类型选择指南

| 需求 | 推荐类型 | 理由 |
|------|---------|------|
| 一次性使用 | `BoxConsumer` | 单一所有权，无开销 |
| 不修改状态（纯观察）| `BoxReadonlyConsumer` | 用 `Fn`，可重复调用 |
| 多线程共享 + 修改状态 | `ArcConsumer` | 线程安全，Mutex 保护 |
| 多线程共享 + 不修改状态 | `ArcReadonlyConsumer` | 线程安全，无锁 |
| 单线程复用 + 修改状态 | `RcConsumer` | RefCell 无锁开销 |
| 单线程复用 + 不修改状态 | `RcReadonlyConsumer` | 无任何开销 |
| 一次性 + FnOnce 闭包 | `BoxConsumerOnce` | 保存 FnOnce |

---

## 总结

### 为什么选择方案三？

**`prism3-rust-function` 采用方案三**，原因如下：

1. **统一的 trait 抽象**
   - 提供 `Consumer<T>` 和 `ReadonlyConsumer<T>` trait
   - 所有类型通过统一接口使用
   - 支持泛型编程

2. **完整的所有权模型覆盖**
   - Box：单一所有权，零开销
   - Arc：线程安全共享，Mutex 保护
   - Rc：单线程共享，RefCell 优化

3. **优雅的 API 设计**
   - 类型保持：`ArcConsumer.and_then()` 返回 `ArcConsumer`
   - 无需显式 clone：组合方法使用 `&self`
   - 方法链：流式 API

4. **与 Rust 生态一致**
   - 命名模式与标准库智能指针一致（Box/Arc/Rc）
   - 设计哲学符合 Rust 惯例

5. **长期可维护性**
   - 清晰的架构
   - 易于扩展（添加新实现、trait、元数据）
   - 类型名称即文档

### 核心设计原则

1. **Consumer 不修改输入**：参数必须是 `&T`
2. **区分 Consumer 和 Mutator**：语义清晰
3. **提供 ReadonlyConsumer**：纯观察场景（不修改自身状态）
4. **保留 ConsumerOnce**：保存 FnOnce 闭包
5. **类型名称语义明确**：Box/Arc/Rc 表达所有权模型

这个设计为用户提供了最灵活、最强大、最清晰的 API，是库项目的最佳选择。

---

## 重构历史

### 2025-01-17: ConsumerMut → Mutator 重构

**背景**：原先的 `ConsumerMut` 命名存在语义不一致问题：
- `ConsumerMut` 使用 `FnMut(&mut T)` 签名，可以修改输入值
- 这违反了 Consumer 的核心语义（Consumer 应该只观察，不修改输入）

**重构内容**：
1. ✅ 将 `src/consumer_mut.rs` 重命名为 `src/mutator.rs`
2. ✅ 所有类型重命名：
   - `ConsumerMut<T>` → `Mutator<T>`
   - `BoxConsumerMut<T>` → `BoxMutator<T>`
   - `ArcConsumerMut<T>` → `ArcMutator<T>`
   - `RcConsumerMut<T>` → `RcMutator<T>`
   - `FnConsumerMutOps<T>` → `FnMutatorOps<T>`
3. ✅ 方法重命名：`accept()` → `mutate()`
4. ✅ 更新测试文件：`consumer_mut_tests.rs` → `mutator_tests.rs`
5. ✅ 更新模块导出和文档

**重构理由**：
- **语义清晰**：Mutator 明确表示"修改器"，与 Consumer（观察者）区分开
- **符合设计原则**：Consumer 系列不修改输入，Mutator 系列修改输入
- **避免混淆**：防止用户误以为 Consumer 可以修改输入值

**影响**：
- 🔴 **破坏性变更**：所有使用 `ConsumerMut` 的代码需要更新
- 🟢 **向前兼容**：如需兼容旧代码，可添加 type alias：
  ```rust
  #[deprecated(note = "Use Mutator instead")]
  pub type ConsumerMut<T> = Mutator<T>;
  ```

**迁移指南**：
```rust
// 旧代码
use prism3_function::{ConsumerMut, BoxConsumerMut};
let mut consumer = BoxConsumerMut::new(|x: &mut i32| *x *= 2);
consumer.accept_once(&mut value);

// 新代码
use prism3_function::{Mutator, BoxMutator};
let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
mutator.mutate(&mut value);
```

