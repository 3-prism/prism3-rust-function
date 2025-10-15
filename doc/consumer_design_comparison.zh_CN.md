# Consumer 设计方案对比分析

## 概述

本文档详细分析了 Rust 中实现 Consumer（消费者）类型的三种不同设计方案，对比了它们的优缺点、适用场景和实现细节。

Consumer 的核心功能是接受一个值并对其执行操作（通常带有副作用），但不返回结果，类似于 Java 中的 `Consumer<T>` 接口。在 Rust 中，我们需要在以下几个方面做出权衡：

- **类型表达**：类型别名 vs Struct vs Trait
- **可变性**：`FnMut` 允许修改捕获的环境和输入值
- **所有权模型**：Box（单一所有权）vs Arc（共享所有权）vs Rc（单线程共享）
- **调用方式**：直接调用 vs 方法调用
- **组合能力**：`and_then` 链式调用
- **扩展性**：是否可以添加元数据、实现其他 trait

---

## 方案一：类型别名 + 静态组合方法

### 设计概述

使用类型别名定义 Consumer 类型，并通过静态工具类提供组合方法。这是最简单直接的实现方式。

### 核心设计

```rust
// 类型别名定义
pub type Consumer<T> = Box<dyn FnMut(&mut T)>;
pub type SharedConsumer<T> = Arc<Mutex<dyn FnMut(&mut T) + Send>>;

// 静态组合工具类
pub struct Consumers;

impl Consumers {
    /// 创建 AND_THEN 组合
    pub fn and_then<T, F1, F2>(first: F1, second: F2) -> Consumer<T>
    where
        T: 'static,
        F1: FnMut(&mut T) + 'static,
        F2: FnMut(&mut T) + 'static,
    {
        let mut first = first;
        let mut second = second;
        Box::new(move |t| {
            first(t);
            second(t);
        })
    }

    /// 创建 no-op consumer
    pub fn noop<T>() -> Consumer<T>
    where
        T: 'static,
    {
        Box::new(|_| {})
    }

    /// 创建条件 consumer
    pub fn if_then<T, P, C>(predicate: P, consumer: C) -> Consumer<T>
    where
        T: 'static,
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        Box::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }
}

// SharedConsumer 的组合工具类
pub struct SharedConsumers;

impl SharedConsumers {
    pub fn and_then<T>(
        first: SharedConsumer<T>,
        second: SharedConsumer<T>,
    ) -> SharedConsumer<T>
    where
        T: 'static,
    {
        Arc::new(Mutex::new(move |t: &mut T| {
            first.lock().unwrap()(t);
            second.lock().unwrap()(t);
        }))
    }

    // ... 其他方法类似
}
```

### 使用示例

```rust
// 创建 consumer
let mut consumer: Consumer<i32> = Box::new(|x| *x *= 2);

// 直接调用
let mut value = 5;
consumer(&mut value);
assert_eq!(value, 10);

// 组合 consumer（传入闭包）
let mut chained = Consumers::and_then(
    |x: &mut i32| *x *= 2,
    |x: &mut i32| *x += 10,
);

let mut value = 5;
chained(&mut value);
assert_eq!(value, 20); // (5 * 2) + 10

// 复杂组合
let mut complex = Consumers::and_then(
    Consumers::if_then(|x: &i32| *x > 0, |x| *x *= 2),
    |x| *x += 1,
);

let mut value = 5;
complex(&mut value);
assert_eq!(value, 11); // (5 * 2) + 1

// 使用 SharedConsumer（需要 Mutex 保护可变性）
let shared: SharedConsumer<i32> = Arc::new(Mutex::new(|x| *x *= 2));
let cloned = Arc::clone(&shared);

// 在多个地方使用
let mut value1 = 5;
shared.lock().unwrap()(&mut value1);
assert_eq!(value1, 10);

let mut value2 = 7;
cloned.lock().unwrap()(&mut value2);
assert_eq!(value2, 14);
```

### 作为函数参数使用

```rust
// 定义接受 consumer 参数的函数
fn for_each<T, F>(values: &mut [T], mut consumer: F)
where
    F: FnMut(&mut T),
{
    for value in values.iter_mut() {
        consumer(value);
    }
}

// 使用示例
let mut values = vec![1, 2, 3, 4, 5];

// 1. 传入闭包
for_each(&mut values, |x: &mut i32| *x *= 2);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. 传入 Consumer 对象（注意：会转移所有权）
let mut consumer: Consumer<i32> = Box::new(|x| *x += 1);
for_each(&mut values, consumer); // consumer 被移动
assert_eq!(values, vec![3, 5, 7, 9, 11]);
// consumer 在此处不再可用

// 3. 传入组合后的 consumer
let mut combined = Consumers::and_then(|x: &mut i32| *x *= 2, |x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### 优点

#### 1. **极简的 API 和使用体验**
- ✅ **直接调用**：`consumer(&mut value)` 而不是 `consumer.accept(&mut value)`
- ✅ **零心智负担**：类型别名完全透明，用户可以直接使用 `Box<dyn FnMut>`
- ✅ **与标准库完美集成**：可以直接用在 `for_each`、`iter_mut` 等方法中

```rust
// 在标准库中使用非常自然
vec![1, 2, 3]
    .iter_mut()
    .for_each(|x| consumer(x)); // ✅ 直接作为闭包使用
```

#### 2. **完美的泛型支持**
- ✅ **统一的 FnMut trait**：闭包、Consumer 都通过 `FnMut(&mut T)` 统一
- ✅ **无需转换**：所有可调用类型都可以直接传入组合方法
- ✅ **类型推断友好**：编译器可以自动推断闭包类型

```rust
// 支持所有可调用类型
let c1 = Consumers::and_then(|x| *x *= 2, |x| *x += 1);     // 闭包
let c2 = Consumers::and_then(multiply_fn, add_fn);          // 函数指针
let c3 = Consumers::and_then(c1, |x| println!("{}", x));    // Consumer + 闭包
```

#### 3. **零成本抽象**
- ✅ **单次装箱**：每个闭包只装箱一次
- ✅ **内联优化**：编译器可以优化闭包调用
- ✅ **无额外间接调用**：直接通过 `Box::call_mut()` 调用

#### 4. **实现简单**
- ✅ **代码量少**：无需定义复杂的 struct 或 trait
- ✅ **维护成本低**：类型别名易于理解和维护
- ✅ **文档简洁**：用户只需理解函数签名

### 缺点

#### 1. **无法扩展**
- ❌ **不能添加字段**：无法为 Consumer 添加名称、统计信息等元数据
- ❌ **不能实现 trait**：类型别名无法实现 `Display`、`Debug` 等 trait
- ❌ **不能添加方法**：无法为 Consumer 添加实例方法

```rust
// ❌ 无法实现
impl<T> Display for Consumer<T> {  // 编译错误：类型别名无法有 impl
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Consumer")
    }
}
```

#### 2. **类型区分度低**
- ❌ **无法在类型系统层面区分**：`Consumer<T>` 和 `Box<dyn FnMut(&mut T)>` 完全等价
- ❌ **容易混淆**：用户可能直接使用 `Box::new()` 而不是通过 `Consumers`
- ❌ **语义不够明确**：类型名称不能反映更多信息

#### 3. **两套平行的 API**
- ⚠️ **Consumer vs SharedConsumer**：需要维护两套类型和组合方法
- ⚠️ **SharedConsumer 需要 Mutex**：由于 `FnMut` 需要可变性，Arc 必须配合 Mutex 使用
- ⚠️ **性能开销**：SharedConsumer 每次调用都需要加锁
- ⚠️ **缺少 Rc 支持**：没有为单线程场景提供 Rc 实现

```rust
// 两套平行的 API
struct Consumers;           // 为 Consumer 提供组合方法
struct SharedConsumers;     // 为 SharedConsumer 提供组合方法

// SharedConsumer 的性能问题
let shared: SharedConsumer<i32> = Arc::new(Mutex::new(|x| *x *= 2));
// 每次调用都需要加锁
shared.lock().unwrap()(&mut value); // ⚠️ 锁开销
```

#### 4. **无法实现方法链**
- ❌ **只能嵌套调用**：复杂组合时嵌套较深
- ❌ **可读性较差**：多层嵌套不如链式调用清晰

```rust
// 复杂组合需要嵌套
let complex = Consumers::and_then(
    Consumers::if_then(
        is_positive,
        multiply_by_two
    ),
    add_one
);

// 无法使用方法链（理想形式）：
// let complex = multiply_by_two.if_then(is_positive).and_then(add_one);
```

### 适用场景

✅ **最适合以下场景：**

1. **简单的操作组合**：不需要复杂的元数据或方法链
2. **追求极简 API**：希望代码尽可能简洁
3. **与标准库深度集成**：需要在 `for_each` 等方法中直接使用
4. **一次性使用**：consumer 创建后不需要多次复用
5. **快速原型开发**：快速实现功能，不考虑长期扩展

❌ **不适合以下场景：**

1. 需要为 consumer 添加名称、调试信息等元数据
2. 需要实现 `Display`、`Debug` 等 trait
3. 需要复杂的方法链式调用
4. 需要在多线程环境中频繁使用（SharedConsumer 的锁开销）

---

## 方案二：Struct 封装 + 实例方法

### 设计概述

将 Consumer 定义为 struct，内部包装 `Box<dyn FnMut>`，通过实例方法提供组合能力，支持方法链式调用。这是当前 `prism3-rust-function` 采用的方案。

### 核心设计

```rust
// Struct 定义
pub struct Consumer<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> Consumer<T>
where
    T: 'static,
{
    /// 创建新的 Consumer
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        Consumer { func: Box::new(f) }
    }

    /// 执行 consumer
    pub fn accept(&mut self, value: &mut T) {
        (self.func)(value)
    }

    /// 链式组合（消耗 self）
    pub fn and_then<F>(self, next: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        let mut first = self.func;
        let mut second = next;

        Consumer {
            func: Box::new(move |t| {
                first(t);
                second(t);
            }),
        }
    }

    /// 与另一个 Consumer 实例组合
    pub fn and_then_consumer(self, next: Consumer<T>) -> Self {
        let mut first = self.func;
        let mut second = next.func;

        Consumer {
            func: Box::new(move |t| {
                first(t);
                second(t);
            }),
        }
    }

    /// 创建 no-op consumer
    pub fn noop() -> Self {
        Consumer::new(|_| {})
    }

    /// 创建打印 consumer
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        Consumer::new(|t| {
            println!("{:?}", t);
        })
    }

    /// 创建条件 consumer
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        Consumer::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }

    /// 创建条件分支 consumer
    pub fn if_then_else<P, C1, C2>(
        predicate: P,
        then_consumer: C1,
        else_consumer: C2,
    ) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&mut T) + 'static,
        C2: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        Consumer::new(move |t| {
            if pred(t) {
                then_cons(t);
            } else {
                else_cons(t);
            }
        })
    }
}

// SharedConsumer（基于 Arc + Mutex）
pub struct SharedConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&mut T) + Send>>,
}

impl<T> SharedConsumer<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        SharedConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn accept(&self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    pub fn and_then(&self, next: &SharedConsumer<T>) -> Self {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        SharedConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

// 实现 Clone（Arc 可以克隆）
impl<T> Clone for SharedConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}
```

### 使用示例

```rust
// 创建 Consumer
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);

// 调用需要使用 .accept()
let mut value = 5;
consumer.accept(&mut value);
assert_eq!(value, 10);

// 方法链式调用
let mut chained = Consumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x| *x += 10)
    .and_then(|x| println!("Result: {}", x));

let mut value = 5;
chained.accept(&mut value); // Prints: Result: 20
assert_eq!(value, 20);

// 使用工厂方法
let mut print = Consumer::<i32>::print();
let mut value = 42;
print.accept(&mut value); // Prints: 42

// 条件 consumer
let mut conditional = Consumer::if_then(
    |x: &i32| *x > 0,
    |x: &mut i32| *x += 1,
);

let mut positive = 5;
conditional.accept(&mut positive);
assert_eq!(positive, 6);

let mut negative = -5;
conditional.accept(&mut negative);
assert_eq!(negative, -5); // 未修改

// SharedConsumer 可以克隆
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
let cloned1 = shared.clone();
let cloned2 = shared.clone();

// 可以在多个地方使用
let mut value1 = 5;
shared.accept(&mut value1);
assert_eq!(value1, 10);

let mut value2 = 7;
cloned1.accept(&mut value2);
assert_eq!(value2, 14);
```

### 作为函数参数使用

方案二需要定义辅助 trait 来统一接受不同类型的参数：

```rust
// 方式 1：定义 Consumable trait（推荐）
pub trait Consumable<T> {
    fn accept(&mut self, value: &mut T);
}

// 为闭包实现 Consumable
impl<T, F> Consumable<T> for F
where
    F: FnMut(&mut T),
{
    fn accept(&mut self, value: &mut T) {
        self(value)
    }
}

// 为 Consumer 实现 Consumable
impl<T> Consumable<T> for Consumer<T> {
    fn accept(&mut self, value: &mut T) {
        self.accept(value)
    }
}

// 定义接受 consumer 参数的函数
fn for_each<T, C>(values: &mut [T], consumer: &mut C)
where
    C: Consumable<T>,
{
    for value in values.iter_mut() {
        consumer.accept(value);
    }
}

// 使用示例
let mut values = vec![1, 2, 3, 4, 5];

// 1. 传入闭包引用
let mut closure = |x: &mut i32| *x *= 2;
for_each(&mut values, &mut closure);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. 传入 Consumer 对象引用
let mut consumer = Consumer::new(|x: &mut i32| *x += 1);
for_each(&mut values, &mut consumer);
assert_eq!(values, vec![3, 5, 7, 9, 11]);
// consumer 仍然可用（只是借用）

// 3. 传入组合后的 consumer
let mut combined = Consumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, &mut combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### 优点

#### 1. **优雅的方法链**
- ✅ **流式 API**：`.and_then()` 的链式调用更加自然
- ✅ **可读性好**：复杂组合更加清晰易读
- ✅ **符合面向对象习惯**：类似 Java、C# 等语言的风格

```rust
// 方法链比嵌套调用更清晰
let mut complex = Consumer::new(|x| *x *= 2)
    .and_then(|x| *x += 10)
    .and_then(|x| println!("Result: {}", x));
```

#### 2. **强大的扩展性**
- ✅ **可添加字段**：可以为 Consumer 添加名称、统计信息等元数据
- ✅ **可实现 trait**：Display、Debug、Serialize 等
- ✅ **可添加方法**：任何自定义的实例方法和工厂方法

```rust
pub struct Consumer<T> {
    func: Box<dyn FnMut(&mut T)>,
    name: Option<String>,           // 名称
    call_count: Arc<AtomicUsize>,   // 调用统计
}

impl<T> Consumer<T> {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}
```

#### 3. **类型安全**
- ✅ **独立的类型**：`Consumer<T>` 是明确的类型，不会与 `Box<dyn FnMut>` 混淆
- ✅ **更好的类型检查**：编译器可以提供更好的错误信息
- ✅ **类型语义清晰**：类型名称直接反映用途

#### 4. **丰富的工厂方法**
- ✅ **便捷的构造函数**：`noop()`、`print()`、`if_then()` 等
- ✅ **提高开发效率**：常用模式开箱即用
- ✅ **代码复用**：避免重复编写相同逻辑

```rust
// 便捷的工厂方法
let mut noop = Consumer::<i32>::noop();
let mut print = Consumer::<i32>::print();
let mut conditional = Consumer::if_then(|x| *x > 0, |x| *x += 1);
```

### 缺点

#### 1. **无法直接调用**
- ❌ **必须使用 `.accept()`**：`consumer.accept(&mut value)` 而不是 `consumer(&mut value)`
- ❌ **与标准库集成不够自然**：在 `for_each` 中需要额外的适配
- ❌ **代码略显冗长**：每次调用都多一个 `.accept()`

```rust
// 不能直接调用
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);
// consumer(&mut value);  // ❌ 编译错误

// 必须这样
consumer.accept(&mut value);  // ✅

// 在标准库中使用略显笨拙
values.iter_mut().for_each(|x| consumer.accept(x)); // ⚠️ 但 consumer 的可变借用会有问题
```

#### 2. **仍需要多个实现**
- ⚠️ **Box 和 Arc 需要分别实现**：`Consumer` 和 `SharedConsumer`
- ⚠️ **代码重复**：`and_then` 等方法需要在两个 struct 中重复实现
- ⚠️ **SharedConsumer 必须使用 Mutex**：由于 `FnMut` 需要可变性，Arc 必须配合 Mutex
- ⚠️ **维护成本增加**：修改一个需要同时修改另一个

```rust
// 需要实现两遍相同的逻辑
impl<T> Consumer<T> {
    pub fn and_then(self, other: ...) -> Self { /* 实现 */ }
}

impl<T> SharedConsumer<T> {
    pub fn and_then(&self, other: ...) -> Self { /* 类似的实现，但需要处理锁 */ }
}
```

#### 3. **所有权问题**
- ⚠️ **方法链消耗 self**：每次调用都会移动所有权
- ⚠️ **无法重用中间结果**：Consumer 不能克隆（`Box<dyn FnMut>` 不能克隆）
- ⚠️ **SharedConsumer 需要显式克隆**：即使是共享所有权，也需要 `.clone()`

```rust
let consumer = Consumer::new(|x: &mut i32| *x *= 2);
let combined = consumer.and_then(|x| *x += 1);
// consumer 已经被移动，无法再使用

// SharedConsumer 需要显式克隆
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
let combined1 = shared.clone().and_then(...);
let combined2 = shared.clone().and_then(...);
```

#### 4. **SharedConsumer 的性能开销**
- ⚠️ **每次调用都要加锁**：Mutex 的锁开销无法避免
- ⚠️ **可能导致锁竞争**：多线程场景下性能可能受影响
- ⚠️ **错误处理复杂**：`lock().unwrap()` 可能导致 panic

```rust
// 每次调用都需要加锁
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
shared.accept(&mut value); // 内部需要 lock().unwrap()

// 组合时锁嵌套，可能导致死锁或性能问题
let combined = shared1.and_then(&shared2); // 内部创建嵌套的 Mutex
```

#### 5. **可变借用的限制**
- ⚠️ **accept 需要 &mut self**：导致在某些场景下难以使用
- ⚠️ **不能在迭代器中直接使用**：因为需要可变借用

```rust
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);

// ❌ 编译错误：不能在闭包中可变借用 consumer
// values.iter_mut().for_each(|x| consumer.accept(x));

// ✅ 必须手动循环
for value in values.iter_mut() {
    consumer.accept(value);
}
```

### 适用场景

✅ **最适合以下场景：**

1. **需要方法链**：复杂的操作组合，希望使用流式 API
2. **需要元数据**：为 consumer 添加名称、统计、调试信息
3. **需要实现 trait**：Display、Debug、Serialize 等
4. **面向对象风格**：团队更熟悉 OOP 风格的 API
5. **提供丰富的工厂方法**：noop、print、if_then 等便捷构造函数

❌ **不适合以下场景：**

1. 追求极简 API，不需要额外功能
2. 需要直接调用（如 `consumer(&mut value)`）
3. 需要在迭代器链式调用中使用
4. 多线程高频调用（SharedConsumer 的锁开销）

---

## 方案三：Trait 抽象 + 多种实现

### 设计概述

这是最灵活和最优雅的方案，类似于 Predicate 的方案三设计。它结合了 trait 的统一抽象能力和 struct 的具体实现能力。

**核心思想**：
1. **定义最小化的 `Consumer<T>` Trait**：只包含核心的 `accept(&mut self, &mut T)` 方法和 `into_*` 类型转换方法
2. **提供三种具体的 Struct 实现**：
   - `BoxConsumer<T>`：基于 `Box`，用于单一所有权的场景
   - `ArcConsumer<T>`：基于 `Arc<Mutex<>>`，用于线程安全的共享所有权场景
   - `RcConsumer<T>`：基于 `Rc<RefCell<>>`，用于单线程的共享所有权场景
3. **在 Struct 上实现特例化的组合方法**：每种 Struct 都实现自己的 `and_then` 等固有方法
4. **为闭包提供扩展 Trait**：通过扩展 trait，为所有闭包提供 `.and_then()` 等方法
5. **统一实现 `Consumer<T>` Trait**：所有闭包和三种 Struct 都实现 `Consumer<T>` Trait

### 核心设计

```rust
// ============================================================================
// 1. 定义最小化的 Consumer trait
// ============================================================================

/// Consumer trait - 统一的消费者接口
pub trait Consumer<T> {
    /// 执行消费操作
    fn accept(&mut self, value: &mut T);

    /// 转换为 BoxConsumer
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 RcConsumer
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 ArcConsumer
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
}

// ============================================================================
// 2. 为闭包实现 Consumer trait
// ============================================================================

/// 为所有 FnMut(&mut T) 实现 Consumer
impl<T, F> Consumer<T> for F
where
    F: FnMut(&mut T),
{
    fn accept(&mut self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(self)
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcConsumer::new(self)
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcConsumer::new(self)
    }
}

// ============================================================================
// 3. 为闭包提供逻辑组合方法的扩展 trait
// ============================================================================

/// 为闭包提供组合方法的扩展 trait
pub trait FnConsumerOps<T>: FnMut(&mut T) + Sized {
    /// AND_THEN 组合 - 消耗闭包，返回 BoxConsumer
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first.accept(t);
            second.accept(t);
        })
    }
}

/// 为所有闭包类型实现 FnConsumerOps
impl<T, F> FnConsumerOps<T> for F where F: FnMut(&mut T) {}

// ============================================================================
// 4. BoxConsumer - 单一所有权实现
// ============================================================================

pub struct BoxConsumer<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> BoxConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        BoxConsumer { func: Box::new(f) }
    }

    /// AND_THEN 组合 - 消耗 self，返回 BoxConsumer
    pub fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    // 工厂方法
    pub fn noop() -> Self
    where
        T: 'static,
    {
        BoxConsumer::new(|_| {})
    }

    pub fn print() -> Self
    where
        T: std::fmt::Debug + 'static,
    {
        BoxConsumer::new(|t| println!("{:?}", t))
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        RcConsumer::new(move |t| self.func(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        ArcConsumer::new(move |t| self.func(t))
    }
}

// ============================================================================
// 5. ArcConsumer - 线程安全的共享所有权实现
// ============================================================================

pub struct ArcConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&mut T) + Send>>,
}

impl<T> ArcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        ArcConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// AND_THEN 组合 - 借用 &self，返回 ArcConsumer
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| self.func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        RcConsumer::new(move |t| self.func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        self
    }
}

impl<T> Clone for ArcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. RcConsumer - 单线程的共享所有权实现
// ============================================================================

pub struct RcConsumer<T> {
    func: Rc<RefCell<dyn FnMut(&mut T)>>,
}

impl<T> RcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        RcConsumer {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// AND_THEN 组合 - 借用 &self，返回 RcConsumer
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T>
    where
        T: 'static,
    {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcConsumer {
            func: Rc::new(RefCell::new(move |t: &mut T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.borrow_mut())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| self.func.borrow_mut()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcConsumer to ArcConsumer (not Send)")
    }
}

impl<T> Clone for RcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
```

### 使用示例

```rust
// ============================================================================
// 1. 闭包自动拥有 .accept() 和逻辑组合方法
// ============================================================================

let mut closure = |x: &mut i32| *x *= 2;
let mut value = 5;
closure.accept(&mut value); // 直接使用 .accept()
assert_eq!(value, 10);

// 闭包使用方法链，返回 BoxConsumer
let mut chained = (|x: &mut i32| *x *= 2).and_then(|x| *x += 10);
let mut value = 5;
chained.accept(&mut value);
assert_eq!(value, 20);

// ============================================================================
// 2. BoxConsumer - 一次性使用场景，消耗 self
// ============================================================================

let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
let mut combined = consumer.and_then(|x| *x += 10); // consumer 被消耗
let mut value = 5;
combined.accept(&mut value);
assert_eq!(value, 20);

// ============================================================================
// 3. ArcConsumer - 多线程共享场景，借用 &self
// ============================================================================

let shared = ArcConsumer::new(|x: &mut i32| *x *= 2);

// ✅ 使用方法链组合，不需要显式 clone
let combined = shared.and_then(&ArcConsumer::new(|x| *x += 10));

// ✅ shared 仍然可用，可以继续组合
let another_combined = shared.and_then(&ArcConsumer::new(|x| *x -= 5));

let mut value = 5;
let mut shared_clone = shared.clone();
shared_clone.accept(&mut value);
assert_eq!(value, 10);

// ✅ 组合结果仍然是 ArcConsumer，可以克隆和跨线程使用
let combined_clone = combined.clone();
use std::thread;
let handle = thread::spawn(move || {
    let mut val = 5;
    let mut c = combined_clone;
    c.accept(&mut val);
    val
});
assert_eq!(handle.join().unwrap(), 20);

// ============================================================================
// 4. RcConsumer - 单线程复用场景，借用 &self
// ============================================================================

let rc_consumer = RcConsumer::new(|x: &mut i32| *x *= 2);

// ✅ 使用方法链，不需要显式 clone
let combined1 = rc_consumer.and_then(&RcConsumer::new(|x| *x += 10));
let combined2 = rc_consumer.and_then(&RcConsumer::new(|x| *x -= 5));

// ✅ 原始 consumer 仍然可用
let mut value = 5;
let mut rc_clone = rc_consumer.clone();
rc_clone.accept(&mut value);
assert_eq!(value, 10);

// ============================================================================
// 5. 统一的接口 - 所有类型都实现了 Consumer trait
// ============================================================================

fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: i32) -> i32 {
    let mut val = value;
    consumer.accept(&mut val);
    val
}

// 所有类型都可以传入
let mut box_con = BoxConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut box_con, 5), 10);

let mut arc_con = ArcConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut arc_con, 5), 10);

let mut rc_con = RcConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut rc_con, 5), 10);

let mut closure = |x: &mut i32| *x *= 2;
assert_eq!(apply_consumer(&mut closure, 5), 10);
```

### 作为函数参数使用

方案三的统一 trait 接口使得函数参数使用非常自然：

```rust
// 定义接受 consumer 参数的函数（通过可变借用）
fn for_each<T, C>(values: &mut [T], consumer: &mut C)
where
    C: Consumer<T>,
{
    for value in values.iter_mut() {
        consumer.accept(value);
    }
}

// 使用示例
let mut values = vec![1, 2, 3, 4, 5];

// 1. 传入闭包引用
let mut closure = |x: &mut i32| *x *= 2;
for_each(&mut values, &mut closure);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. 传入 BoxConsumer 对象引用
let mut box_con = BoxConsumer::new(|x: &mut i32| *x += 1);
for_each(&mut values, &mut box_con);
assert_eq!(values, vec![3, 5, 7, 9, 11]);

// 3. 传入 ArcConsumer 对象引用
let mut arc_con = ArcConsumer::new(|x: &mut i32| *x *= 2);
for_each(&mut values, &mut arc_con);
assert_eq!(values, vec![6, 10, 14, 18, 22]);

// 4. 传入组合后的 consumer
let mut combined = (|x: &mut i32| *x *= 2).and_then(|x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, &mut combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### 优点

#### 1. **完美的语义清晰度**

- ✅ **名称即文档**：`BoxConsumer`、`ArcConsumer`、`RcConsumer` 直接表达底层实现和所有权模型
- ✅ **对称的设计**：三个类型功能对称，易于理解和使用
- ✅ **与标准库一致**：命名模式与 Rust 标准库的智能指针 `Box`, `Arc`, `Rc` 一致

#### 2. **统一的 trait 接口**

- ✅ **统一抽象**：所有类型通过 `Consumer<T>` trait 统一，都可以使用 `.accept()`
- ✅ **多态支持**：可以编写接受 `&mut dyn Consumer<T>` 或 `impl Consumer<T>` 的泛型函数
- ✅ **闭包自动支持**：所有闭包自动实现 `Consumer<T>`，无需任何转换

#### 3. **完整的所有权模型覆盖**

三种实现对应三种典型场景：

| 类型 | 所有权 | 克隆 | 线程安全 | 内部可变性 | API | 适用场景 |
|:---|:---|:---|:---:|:---:|:---|:---|
| `BoxConsumer` | 单一 | ❌ | ❌ | FnMut | `self` | 一次性使用、构建器模式 |
| `ArcConsumer` | 共享 | ✅ | ✅ | Arc<Mutex<>> | `&self` | 多线程共享、并发任务 |
| `RcConsumer` | 共享 | ✅ | ❌ | Rc<RefCell<>> | `&self` | 单线程复用、事件处理 |

#### 4. **特例化带来的类型保持和优雅的 API**

这是此方案最核心的优势：

- ✅ **类型保持**：`ArcConsumer` 的组合方法返回的仍然是 `ArcConsumer`，保持了其可克隆和线程安全的特性
- ✅ **优雅的 API**：`ArcConsumer` 和 `RcConsumer` 的组合方法使用 `&self`，调用时无需显式 `.clone()`
- ✅ **无需静态组合方法**：所有操作都通过方法链完成，API 更内聚和简洁

```rust
// ArcConsumer → ArcConsumer（借用 &self，可重复使用）
let arc_con = ArcConsumer::new(|x| *x *= 2);
let arc_result = arc_con.and_then(&another_arc);   // ✅ 不需要 clone
let arc_result2 = arc_con.and_then(&third_arc);    // ✅ arc_con 仍然可用
let cloned = arc_result.clone();                   // ✅ 组合结果也可以克隆

// BoxConsumer → BoxConsumer（消耗所有权，使用 self）
let box_con = BoxConsumer::new(|x| *x *= 2);
let box_result = box_con.and_then(another);        // ⚠️ box_con 被移动，不可再用
```

#### 5. **解决了内部可变性问题**

- ✅ **ArcConsumer 使用 Arc<Mutex<>>**：线程安全的内部可变性
- ✅ **RcConsumer 使用 Rc<RefCell<>>**：单线程的内部可变性，无锁开销
- ✅ **清晰的语义**：类型名称明确表达了内部可变性的实现方式

#### 6. **最强的扩展性**

- ✅ **可添加新实现**：未来可以轻松添加新的 consumer 类型
- ✅ **可添加字段**：每个实现都可以有自己的元数据（名称、统计等）
- ✅ **可实现 trait**：`Display`、`Debug`、`Serialize` 等

#### 7. **与 Rust 标准库设计哲学一致**

该设计模式（一个 trait + 多种 struct 实现）与 Rust 标准库中的智能指针设计完全一致，符合 Rust 的设计哲学。

### 缺点

#### 1. **仍然无法直接调用**

与方案二相同，这是使用上的不便之处。

```rust
let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);

// ❌ 不能直接调用
// consumer(&mut value);

// ✅ 必须使用 .accept()
consumer.accept(&mut value);
```

#### 2. **学习成本略高**

用户需要理解：
- ⚠️ `Consumer` trait 作为统一接口
- ⚠️ `BoxConsumer`、`ArcConsumer`、`RcConsumer` 三种实现的区别和适用场景
- ⚠️ 闭包组合默认返回 `BoxConsumer`
- ⚠️ 为什么 `BoxConsumer` 的组合方法消耗 `self`，而 `Arc/RcConsumer` 使用 `&self`
- ⚠️ `ArcConsumer` 使用 `Mutex`，`RcConsumer` 使用 `RefCell` 的原因

**缓解方案**：提供清晰的文档和使用指南（正是本文档的目的）。

#### 3. **实现成本**

- ⚠️ 需要为三个 Struct 分别实现所有的方法，代码量较大
- ⚠️ 但由于架构清晰，逻辑重复性强，长期维护成本反而更低

#### 4. **内部可变性的开销**

- ⚠️ **ArcConsumer**：Mutex 加锁开销（但对于多线程共享是必要的）
- ⚠️ **RcConsumer**：RefCell 运行时借用检查开销（但比 Mutex 轻量）

#### 5. **Trait Object 限制**

`Consumer<T>` trait 本身不是 object-safe 的（如果包含 `into_*` 方法且有 `where Self: Sized` 约束），这意味着不能创建 `Box<dyn Consumer<T>>`。

```rust
// ❌ 可能编译错误：trait 不是 object-safe
// let consumers: Vec<Box<dyn Consumer<i32>>> = vec![...];

// ✅ 解决方案：使用具体类型或 Enum 包装器
// 方案 A：使用具体类型
let consumers: Vec<BoxConsumer<i32>> = vec![...];

// 方案 B：使用 Enum 包装
enum AnyConsumer<T> {
    Box(BoxConsumer<T>),
    Arc(ArcConsumer<T>),
    Rc(RcConsumer<T>),
}
```

### 适用场景

✅ **最适合以下场景：**

1. **库开发**：为用户提供清晰、灵活、强大的 API
2. **大型项目**：代码库规模大，需要清晰的架构来保证可维护性
3. **团队协作**：提供统一的接口规范和清晰的语义
4. **多场景支持**：同时存在一次性使用、单线程复用、多线程共享等多种场景
5. **需要内部可变性**：需要在不同场景下选择合适的内部可变性实现

✅ **强烈推荐用于 `prism3-rust-function` 这样的基础库项目。**

---

## 三种方案对比总结

### 核心特性对比表

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 |
|:---|:---|:---|:---|
| **调用方式** | `consumer(&mut x)` ✅ | `consumer.accept(&mut x)` ❌ | `consumer.accept(&mut x)` ❌ |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 **极好** ✨ |
| **所有权模型** | Box + Arc<Mutex>（两种）| Box + Arc<Mutex>（两种）| Box + Arc<Mutex> + Rc<RefCell>（三种）✅ |
| **类型名称** | Consumer / SharedConsumer | Consumer / SharedConsumer | BoxConsumer / ArcConsumer / RcConsumer ✅ |
| **统一接口** | ❌ 两套独立 API | ❌ 两套独立 struct | ✅ **统一的 Consumer trait** |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ **支持（且类型保持）**✨ |
| **扩展性** | ❌ 无法扩展 | ✅ 可扩展 | ✅ **极易扩展** |
| **元数据支持**| ❌ 不支持 | ✅ 支持 | ✅ 支持 |
| **工厂方法** | 🟡 可添加静态方法 | ✅ 丰富的工厂方法 | ✅ 丰富的工厂方法 |
| **泛型支持** | ✅ 完美（FnMut trait）| 🟡 中等（需额外抽象）| ✅ **完美（Consumer trait）**|
| **内部可变性**| ⚠️ SharedConsumer 必须 Mutex | ⚠️ SharedConsumer 必须 Mutex | ✅ **三种方式（无/Mutex/RefCell）**|
| **代码简洁度** | ✅ 极简 | 🟡 中等 | 🟡 略复杂 |
| **学习成本** | ✅ 最低 | 🟡 中等 | 🟡 略高 |
| **维护成本** | 🟡 中等（两套 API）| 🟡 中等（代码重复）| ✅ **低（架构清晰）**|
| **与标准库一致性**| 🟡 中等 | 🟡 中等 | ✅ **完美** ✨ |

### 使用场景对比

| 场景 | 方案一 | 方案二 | 方案三 |
|:---|:---|:---|:---|
| **快速原型开发** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **简单操作组合** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **复杂方法链** | ❌ 不适合 | ✅ 适合 | ✅ **最佳** |
| **需要元数据/调试**| ❌ 不支持 | ✅ 支持 | ✅ **最佳** |
| **多线程共享** | 🟡 SharedConsumer（有锁）| 🟡 SharedConsumer（有锁）| ✅ **ArcConsumer（清晰）**|
| **单线程复用** | ❌ 不支持 | ❌ 不支持 | ✅ **RcConsumer（无锁）**|
| **库开发** | 🟡 可以 | ✅ 适合 | ✅ **最佳** |
| **大型项目** | 🟡 可以 | ✅ 适合 | ✅ **最佳** |
| **长期维护** | 🟡 中等 | 🟡 中等 | ✅ **最佳** |

### Consumer 与 Predicate 的关键差异

| 差异点 | Predicate | Consumer |
|:---|:---|:---|
| **函数签名** | `Fn(&T) -> bool` | `FnMut(&mut T)` |
| **可变性** | 不可变（Fn） | 可变（FnMut）|
| **共享所有权** | Arc 可直接共享 | Arc 必须配合 Mutex/RefCell |
| **组合方式** | `and`/`or`/`not`（逻辑运算）| `and_then`（序列执行）|
| **返回值** | 有返回值（bool）| 无返回值（副作用）|
| **并发难度** | 低（无可变性）| 高（需要内部可变性）|

---

## 结论

### Consumer 的特殊性

与 Predicate 相比，Consumer 的实现面临额外的挑战：

1. **可变性需求**：Consumer 需要 `FnMut`，这意味着必须处理内部可变性
2. **共享困难**：由于可变性，共享所有权必须使用 `Mutex`（多线程）或 `RefCell`（单线程）
3. **性能权衡**：需要在安全性和性能之间做出选择

### 方案选择建议

对于 `prism3-rust-function` 这样的库项目：

#### 当前方案（方案二）
当前的实现采用方案二是一个**合理的中间选择**：
- ✅ 提供了方法链和扩展性
- ✅ 有丰富的工厂方法
- ⚠️ 但缺少统一的 trait 抽象
- ⚠️ 需要维护两套独立的实现

#### 推荐方案（方案三）
如果希望达到与 Predicate 同等的架构优雅度，**强烈建议升级到方案三**：
- ✅ 统一的 `Consumer` trait 接口
- ✅ 三种清晰的实现覆盖所有场景
- ✅ `RcConsumer` 提供单线程无锁共享（这是方案二缺失的）
- ✅ 类型名称语义明确（`BoxConsumer`/`ArcConsumer`/`RcConsumer`）
- ✅ 与 `Predicate` 的设计保持一致，降低学习成本

#### 实施路径

如果决定升级到方案三，建议采用渐进式迁移：

1. **第一步**：保留当前的 `Consumer` 结构，将其重命名为 `BoxConsumer`
2. **第二步**：添加 `ArcConsumer` 和 `RcConsumer` 实现
3. **第三步**：引入统一的 `Consumer` trait
4. **第四步**：为所有类型实现 trait 和组合方法
5. **第五步**：更新文档和示例

这样可以保持向后兼容，同时逐步引入新的架构。

### 最终建议

对于 Consumer 的实现：

- **快速开发/原型项目**：选择方案一或保持当前的方案二
- **长期维护的库项目**：**强烈推荐方案三**，理由如下：
  - 提供最清晰的架构和语义
  - 完整覆盖三种所有权模型（特别是 RcConsumer）
  - 与 Predicate 的设计保持一致性
  - 长期维护成本最低
  - 为用户提供最灵活和强大的 API

方案三虽然实现成本较高，但它带来的结构性优势和优雅的 API 设计完全值得这份投入，尤其是对于像 `prism3-rust-function` 这样的基础库项目。

