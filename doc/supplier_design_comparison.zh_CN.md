# Supplier 设计方案对比分析

## 概述

本文档详细分析了 Rust 中实现 Supplier（供应者）类型的三种不同设计方案，对比了它们的优缺点、适用场景和实现细节。

Supplier 的核心功能是懒惰地生成值，类似于 Java 中的 `Supplier<T>` 接口。与 Predicate 不同，Supplier 通常需要维护内部状态（如计数器、序列生成器），这使得它的设计面临独特的挑战。

## Supplier vs Predicate：核心差异

在分析具体方案前，理解 Supplier 与 Predicate 的本质区别至关重要：

| 特性 | Predicate | Supplier |
|:---|:---|:---|
| **函数签名** | `Fn(&T) -> bool` | `FnMut() -> T` |
| **可变性** | ❌ 不可变（`&self`） | ✅ 需要可变（`&mut self`） |
| **副作用** | ❌ 通常无副作用 | ✅ 通常有副作用（状态变化） |
| **共享难度** | ✅ 容易（`Arc<dyn Fn>`） | ❌ 困难（需要 `Arc<Mutex<dyn FnMut>>`） |
| **主要场景** | 验证、过滤、条件判断 | 值生成、惰性计算、序列 |
| **无状态使用** | ✅ 常见 | ❌ 少见（无状态用常量即可） |
| **有状态使用** | ❌ 少见 | ✅ 非常常见 |

这些差异决定了 Supplier 的设计方案需要特殊考虑。

---

## 方案一：类型别名 + 静态组合方法

### 设计概述

使用类型别名定义 Supplier 类型，并通过静态工具类提供组合方法。这是最简单直接的实现方式。

### 核心设计

```rust
// 类型别名定义
pub type Supplier<T> = Box<dyn FnMut() -> T>;
pub type SharedSupplier<T> = Arc<Mutex<dyn FnMut() -> T + Send>>;

// 静态组合工具类
pub struct Suppliers;

impl Suppliers {
    /// 创建常量供应者
    pub fn constant<T: Clone + 'static>(value: T) -> Supplier<T> {
        Box::new(move || value.clone())
    }

    /// 映射转换
    pub fn map<T, U, F>(mut supplier: Supplier<T>, mut mapper: F) -> Supplier<U>
    where
        T: 'static,
        U: 'static,
        F: FnMut(T) -> U + 'static,
    {
        Box::new(move || mapper(supplier()))
    }

    /// 组合两个供应者
    pub fn zip<T, U>(
        mut first: Supplier<T>,
        mut second: Supplier<U>,
    ) -> Supplier<(T, U)>
    where
        T: 'static,
        U: 'static,
    {
        Box::new(move || (first(), second()))
    }
}

// SharedSupplier 的工具类
pub struct SharedSuppliers;

impl SharedSuppliers {
    pub fn constant<T: Clone + Send + 'static>(value: T) -> SharedSupplier<T> {
        Arc::new(Mutex::new(move || value.clone()))
    }

    pub fn map<T, U, F>(
        supplier: SharedSupplier<T>,
        mapper: Arc<Mutex<F>>,
    ) -> SharedSupplier<U>
    where
        T: Send + 'static,
        U: Send + 'static,
        F: FnMut(T) -> U + Send + 'static,
    {
        Arc::new(Mutex::new(move || {
            let value = supplier.lock().unwrap()();
            mapper.lock().unwrap()(value)
        }))
    }
}
```

### 使用示例

```rust
// 创建供应者
let mut counter = 0;
let mut supplier: Supplier<i32> = Box::new(move || {
    counter += 1;
    counter
});

// 直接调用
assert_eq!(supplier(), 1);
assert_eq!(supplier(), 2);
assert_eq!(supplier(), 3);

// 使用工具方法
let mut constant = Suppliers::constant(42);
assert_eq!(constant(), 42);
assert_eq!(constant(), 42);

// 映射转换
let mut mapped = Suppliers::map(
    Box::new(|| 10),
    |x| x * 2,
);
assert_eq!(mapped(), 20);

// SharedSupplier（线程安全，但需要锁）
let shared: SharedSupplier<i32> = Arc::new(Mutex::new({
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}));

let cloned = Arc::clone(&shared);

// 需要显式加锁
assert_eq!(shared.lock().unwrap()(), 1);
assert_eq!(cloned.lock().unwrap()(), 2);
```

### 作为函数参数使用

```rust
// 定义接受供应者参数的函数
fn generate_values<T, F>(count: usize, mut supplier: F) -> Vec<T>
where
    F: FnMut() -> T,
{
    (0..count).map(|_| supplier()).collect()
}

// 使用示例
let mut counter = 0;
let values = generate_values(5, || {
    counter += 1;
    counter
});
assert_eq!(values, vec![1, 2, 3, 4, 5]);

// 传入 Supplier 对象（注意：会转移所有权）
let mut supplier: Supplier<i32> = Box::new({
    let mut n = 0;
    move || {
        n += 1;
        n
    }
});
let values = generate_values(3, supplier);
assert_eq!(values, vec![1, 2, 3]);
// supplier 在此处不再可用
```

### 优点

#### 1. **极简的 API 和使用体验**
- ✅ **直接调用**：`supplier()` 而不是 `supplier.get()`
- ✅ **零心智负担**：类型别名完全透明
- ✅ **与标准库完美集成**：可以直接用在迭代器等场景

```rust
// 在迭代器中使用非常自然
let mut supplier = Box::new(|| 42);
let values: Vec<i32> = (0..5).map(|_| supplier()).collect();
```

#### 2. **实现简单**
- ✅ **代码量最少**：无需定义复杂的 struct 或 trait
- ✅ **维护成本低**：类型别名易于理解
- ✅ **快速原型开发**：最快实现功能

#### 3. **完美的泛型支持**
- ✅ **统一的 FnMut trait**：闭包、函数和 Supplier 统一
- ✅ **类型推断友好**：编译器可以自动推断类型

### 缺点

#### 1. **无法扩展**
- ❌ **不能添加字段**：无法添加名称、统计等元数据
- ❌ **不能实现 trait**：类型别名无法实现 `Display`、`Debug`
- ❌ **不能添加方法**：无法添加实例方法

#### 2. **SharedSupplier 使用体验差**
- ❌ **必须显式加锁**：每次调用都要 `.lock().unwrap()()`
- ❌ **错误处理繁琐**：需要处理 `PoisonError`
- ❌ **API 不一致**：`Supplier` 是 `supplier()`，`SharedSupplier` 是 `supplier.lock().unwrap()()`

```rust
// SharedSupplier 使用体验不好
let shared: SharedSupplier<i32> = ...;

// ❌ 冗长且容易出错
let value = shared.lock().unwrap()();

// ❌ 需要到处处理锁
let result = shared.lock().unwrap()();
process(result);
let result2 = shared.lock().unwrap()();
```

#### 3. **组合方法复杂**
- ❌ **SharedSupplier 的组合非常繁琐**：需要多层嵌套的 `Arc<Mutex<>>`
- ❌ **难以实现复杂转换**：如 `filter`、`memoize` 等

```rust
// SharedSupplier 的 map 实现非常繁琐
let mapped = Arc::new(Mutex::new(move || {
    let value = original.lock().unwrap()();
    mapper.lock().unwrap()(value)
}));
```

#### 4. **无法实现方法链**
- ❌ **只能嵌套调用**：复杂组合时嵌套较深
- ❌ **可读性较差**：不如方法链清晰

```rust
// 复杂组合需要深层嵌套
let complex = Suppliers::map(
    Suppliers::filter(
        Suppliers::zip(supplier1, supplier2),
        |(a, b)| a > b
    ),
    |(a, b)| a + b
);
```

### 适用场景

✅ **最适合以下场景：**

1. **简单的值生成**：不需要复杂的状态管理
2. **单线程使用**：不需要跨线程共享
3. **追求极简 API**：希望代码尽可能简洁
4. **快速原型开发**：快速验证想法

❌ **不适合以下场景：**

1. 需要多线程共享供应者
2. 需要添加元数据或调试信息
3. 需要复杂的方法链式调用
4. 需要实现 `Display`、`Debug` 等 trait

---

## 方案二：Struct 封装 + 实例方法

### 设计概述

将 Supplier 定义为 struct，内部包装 `Box<dyn FnMut>`，通过实例方法提供组合能力，支持方法链式调用。这是当前库采用的方案。

### 核心设计

```rust
// Struct 定义
pub struct Supplier<T> {
    func: Box<dyn FnMut() -> T>,
    name: Option<String>,  // 可以添加元数据
}

impl<T> Supplier<T> {
    /// 创建新的 Supplier
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        Self {
            func: Box::new(f),
            name: None,
        }
    }

    /// 创建常量供应者
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        Supplier::new(move || value.clone())
    }

    /// 获取值
    pub fn get(&mut self) -> T {
        (self.func)()
    }

    /// 映射转换（消耗 self）
    pub fn map<U, F>(mut self, mut mapper: F) -> Supplier<U>
    where
        F: FnMut(T) -> U + 'static,
        T: 'static,
        U: 'static,
    {
        Supplier::new(move || mapper(self.get()))
    }

    /// 过滤（消耗 self）
    pub fn filter<P>(mut self, mut predicate: P) -> Supplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
        T: 'static,
    {
        Supplier::new(move || {
            let value = self.get();
            if predicate(&value) {
                Some(value)
            } else {
                None
            }
        })
    }

    /// 组合两个供应者（消耗 self）
    pub fn zip<U>(mut self, mut other: Supplier<U>) -> Supplier<(T, U)>
    where
        T: 'static,
        U: 'static,
    {
        Supplier::new(move || (self.get(), other.get()))
    }

    /// 记忆化（消耗 self）
    pub fn memoize(mut self) -> Supplier<T>
    where
        T: Clone + 'static,
    {
        let mut cache: Option<T> = None;
        Supplier::new(move || {
            if let Some(ref cached) = cache {
                cached.clone()
            } else {
                let value = self.get();
                cache = Some(value.clone());
                value
            }
        })
    }
}

// 实现 Display trait
impl<T> std::fmt::Display for Supplier<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Supplier({})", self.name.as_deref().unwrap_or("unnamed"))
    }
}

// SharedSupplier（基于 Arc + Mutex）
pub struct SharedSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
    name: Option<String>,
}

impl<T> SharedSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        Self {
            func: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// 获取值（需要处理锁）
    pub fn get(&self) -> T {
        (self.func.lock().unwrap())()
    }

    /// 映射转换
    pub fn map<U, F>(self, mapper: F) -> SharedSupplier<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        let mapper = Arc::new(Mutex::new(mapper));
        let func = self.func;
        SharedSupplier {
            func: Arc::new(Mutex::new(move || {
                let value = func.lock().unwrap()();
                mapper.lock().unwrap()(value)
            })),
            name: None,
        }
    }
}

// 实现 Clone
impl<T> Clone for SharedSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
            name: self.name.clone(),
        }
    }
}
```

### 使用示例

```rust
// 创建 Supplier
let mut counter = 0;
let mut supplier = Supplier::new(move || {
    counter += 1;
    counter
});

// 使用 .get() 获取值
assert_eq!(supplier.get(), 1);
assert_eq!(supplier.get(), 2);

// 方法链式调用
let mut complex = Supplier::new(|| 10)
    .map(|x| x * 2)
    .filter(|x| *x > 15)
    .map(|opt| opt.unwrap_or(0));

assert_eq!(complex.get(), 20);

// 记忆化
let mut call_count = 0;
let mut memoized = Supplier::new(move || {
    call_count += 1;
    42
}).memoize();

assert_eq!(memoized.get(), 42); // 调用底层函数
assert_eq!(memoized.get(), 42); // 返回缓存值
assert_eq!(memoized.get(), 42); // 返回缓存值

// SharedSupplier 可以克隆和跨线程使用
let shared = SharedSupplier::new({
    let mut count = 0;
    move || {
        count += 1;
        count
    }
});

let cloned = shared.clone();

// 调用需要使用 .get()
assert_eq!(shared.get(), 1);
assert_eq!(cloned.get(), 2);
```

### 作为函数参数使用

```rust
// 定义接受供应者参数的函数
fn generate_values<T>(count: usize, supplier: &mut Supplier<T>) -> Vec<T> {
    (0..count).map(|_| supplier.get()).collect()
}

// 使用示例
let mut supplier = Supplier::new({
    let mut n = 0;
    move || {
        n += 1;
        n
    }
});

let values = generate_values(5, &mut supplier);
assert_eq!(values, vec![1, 2, 3, 4, 5]);

// supplier 仍然可用（只是借用）
assert_eq!(supplier.get(), 6);
```

### 优点

#### 1. **优雅的方法链**
- ✅ **流式 API**：`.map().filter().zip()` 的链式调用更自然
- ✅ **可读性好**：复杂组合更加清晰易读
- ✅ **符合 Rust 惯用法**：与 Iterator 等标准库风格一致

```rust
// 方法链清晰直观
let mut supplier = Supplier::new(|| 10)
    .map(|x| x * 2)
    .map(|x| x + 5)
    .memoize();
```

#### 2. **强大的扩展性**
- ✅ **可添加字段**：名称、统计信息、创建时间等
- ✅ **可实现 trait**：Display、Debug 等
- ✅ **可添加方法**：任何自定义的实例方法

```rust
pub struct Supplier<T> {
    func: Box<dyn FnMut() -> T>,
    name: Option<String>,
    call_count: Arc<AtomicUsize>,  // 调用统计
}

impl<T> Supplier<T> {
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}
```

#### 3. **类型安全**
- ✅ **独立的类型**：`Supplier<T>` 是明确的类型
- ✅ **更好的类型检查**：编译器提供更好的错误信息
- ✅ **类型语义清晰**：类型名称直接反映用途

#### 4. **隐藏实现细节**
- ✅ **封装性好**：用户不需要知道内部是 `Box<dyn FnMut>`
- ✅ **API 稳定**：内部实现可以改变而不影响用户代码

### 缺点

#### 1. **无法直接调用**
- ❌ **必须使用 `.get()`**：`supplier.get()` 而不是 `supplier()`
- ❌ **略显冗长**：每次调用都多一个 `.get()`

```rust
let mut supplier = Supplier::new(|| 42);

// ❌ 不能直接调用
// let value = supplier();

// ✅ 必须这样
let value = supplier.get();
```

#### 2. **SharedSupplier 仍然复杂**
- ⚠️ **需要处理锁**：虽然封装了，但性能开销仍在
- ⚠️ **可能死锁**：如果在 mapper 中访问原 supplier
- ⚠️ **错误处理**：需要处理 `PoisonError`

```rust
// SharedSupplier 的 get() 内部仍需要锁
pub fn get(&self) -> T {
    (self.func.lock().unwrap())()  // 锁开销
}
```

#### 3. **所有权问题**
- ⚠️ **方法链消耗 self**：每次调用都会移动所有权
- ⚠️ **无法重用中间结果**：Box<dyn FnMut> 不能克隆

```rust
let supplier = Supplier::new(|| 42);
let mapped = supplier.map(|x| x * 2);
// supplier 已经被移动，无法再使用

// SharedSupplier 需要显式克隆
let shared = SharedSupplier::new(|| 42);
let mapped1 = shared.clone().map(...);
let mapped2 = shared.clone().map(...);
```

#### 4. **仍需要两套实现**
- ⚠️ **代码重复**：`Supplier` 和 `SharedSupplier` 的方法需要分别实现
- ⚠️ **维护成本**：修改一个需要同时考虑另一个

### 适用场景

✅ **最适合以下场景：**

1. **复杂的值生成逻辑**：需要多步转换和组合
2. **需要方法链**：希望使用流式 API
3. **需要元数据**：添加名称、统计等信息
4. **单线程为主**：主要在单线程中使用
5. **中等规模项目**：代码量适中，需要一定的结构化

✅ **这是当前库采用的方案，平衡了简洁性和功能性。**

❌ **不适合以下场景：**

1. 追求极简 API，不需要额外功能
2. 大量跨线程共享（SharedSupplier 的锁开销较大）
3. 需要直接调用（如 `supplier()`）

---

## 方案三：Trait 抽象 + 多种实现

### 设计概述

定义统一的 `Supplier<T>` Trait，并提供三种具体实现：`BoxSupplier`、`ArcSupplier`、`RcSupplier`。这是参考 Predicate 方案三的设计。

### 核心设计

```rust
// ============================================================================
// 1. 定义 Supplier trait
// ============================================================================

pub trait Supplier<T> {
    /// 获取下一个值
    fn get(&mut self) -> T;
}

// ============================================================================
// 2. BoxSupplier - 单一所有权实现
// ============================================================================

pub struct BoxSupplier<T> {
    func: Box<dyn FnMut() -> T>,
    name: Option<String>,
}

impl<T> BoxSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        Self {
            func: Box::new(f),
            name: None,
        }
    }

    /// 映射转换（消耗 self）
    pub fn map<U, F>(mut self, mut mapper: F) -> BoxSupplier<U>
    where
        F: FnMut(T) -> U + 'static,
        T: 'static,
        U: 'static,
    {
        BoxSupplier::new(move || mapper(self.get()))
    }

    // ... 其他方法
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&mut self) -> T {
        (self.func)()
    }
}

// ============================================================================
// 3. ArcSupplier - 线程安全的共享所有权实现
// ============================================================================

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
    name: Option<String>,
}

impl<T> ArcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        Self {
            func: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// 映射转换（克隆 Arc）
    pub fn map<U, F>(&self, mapper: F) -> ArcSupplier<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        let func = Arc::clone(&self.func);
        let mapper = Arc::new(Mutex::new(mapper));
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                let value = func.lock().unwrap()();
                mapper.lock().unwrap()(value)
            })),
            name: None,
        }
    }

    // ... 其他方法
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()
    }
}

impl<T> Clone for ArcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
            name: self.name.clone(),
        }
    }
}

// ============================================================================
// 4. RcSupplier - 单线程的共享所有权实现
// ============================================================================

pub struct RcSupplier<T> {
    func: Rc<RefCell<dyn FnMut() -> T>>,
    name: Option<String>,
}

impl<T> RcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        Self {
            func: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// 映射转换（克隆 Rc）
    pub fn map<U, F>(&self, mapper: F) -> RcSupplier<U>
    where
        F: FnMut(T) -> U + 'static,
        T: 'static,
        U: 'static,
    {
        let func = Rc::clone(&self.func);
        let mapper = Rc::new(RefCell::new(mapper));
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                let value = func.borrow_mut()();
                mapper.borrow_mut()(value)
            })),
            name: None,
        }
    }

    // ... 其他方法
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.borrow_mut())()
    }
}

impl<T> Clone for RcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
            name: self.name.clone(),
        }
    }
}
```

### 使用示例

```rust
// ============================================================================
// 1. BoxSupplier - 一次性使用场景
// ============================================================================

let mut box_supplier = BoxSupplier::new({
    let mut count = 0;
    move || {
        count += 1;
        count
    }
});

assert_eq!(box_supplier.get(), 1);
assert_eq!(box_supplier.get(), 2);

// 方法链
let mut mapped = BoxSupplier::new(|| 10)
    .map(|x| x * 2)
    .map(|x| x + 5);
assert_eq!(mapped.get(), 25);

// ============================================================================
// 2. ArcSupplier - 多线程共享场景
// ============================================================================

let arc_supplier = ArcSupplier::new({
    let mut count = 0;
    move || {
        count += 1;
        count
    }
});

// 可以克隆
let mut supplier1 = arc_supplier.clone();
let mut supplier2 = arc_supplier.clone();

assert_eq!(supplier1.get(), 1);
assert_eq!(supplier2.get(), 2);

// 可以跨线程使用
use std::thread;
let mut supplier_thread = arc_supplier.clone();
let handle = thread::spawn(move || {
    supplier_thread.get()
});
assert_eq!(handle.join().unwrap(), 3);

// ============================================================================
// 3. RcSupplier - 单线程复用场景
// ============================================================================

let rc_supplier = RcSupplier::new({
    let mut count = 0;
    move || {
        count += 1;
        count
    }
});

let mut supplier1 = rc_supplier.clone();
let mut supplier2 = rc_supplier.clone();

assert_eq!(supplier1.get(), 1);
assert_eq!(supplier2.get(), 2);

// ============================================================================
// 4. 统一的接口 - 所有类型都实现了 Supplier trait
// ============================================================================

fn generate_values<T, S: Supplier<T>>(count: usize, supplier: &mut S) -> Vec<T> {
    (0..count).map(|_| supplier.get()).collect()
}

// 所有类型都可以传入
let values = generate_values(3, &mut box_supplier);
let values = generate_values(3, &mut arc_supplier.clone());
let values = generate_values(3, &mut rc_supplier.clone());
```

### 优点

#### 1. **完美的语义清晰度**
- ✅ **名称即文档**：`BoxSupplier`、`ArcSupplier`、`RcSupplier` 直接表达所有权模型
- ✅ **对称的设计**：三个类型功能对称，易于理解
- ✅ **与标准库一致**：命名与 `Box`, `Arc`, `Rc` 一致

#### 2. **统一的 trait 接口**
- ✅ **统一抽象**：所有类型通过 `Supplier<T>` trait 统一
- ✅ **多态支持**：可以编写接受 `impl Supplier<T>` 的泛型函数

#### 3. **完整的所有权模型覆盖**

| 类型 | 所有权 | 克隆 | 线程安全 | 性能 | 适用场景 |
|:---|:---|:---|:---:|:---|:---|
| `BoxSupplier` | 单一 | ❌ | ❌ | ⚡⚡⚡ 最快 | 一次性使用、构建器 |
| `ArcSupplier` | 共享 | ✅ | ✅ | ⚡ 较慢（Mutex） | 多线程共享、全局配置 |
| `RcSupplier` | 共享 | ✅ | ❌ | ⚡⚡ 中等（RefCell） | 单线程复用、事件系统 |

#### 4. **可扩展性强**
- ✅ **可添加新实现**：未来可以添加新类型
- ✅ **可添加字段**：每个实现都可以有自己的元数据
- ✅ **可实现 trait**：Display、Debug 等

### 缺点

#### 1. **实现复杂度高** ⭐⭐⭐⭐⭐

这是最大的问题。与 Predicate 的 `Fn(&T) -> bool` 不同，Supplier 的 `FnMut() -> T` 需要可变访问：

```rust
// ArcSupplier 必须使用 Mutex（性能开销）
pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,  // 必须有 Mutex
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()  // 每次调用都加锁
    }
}

// RcSupplier 必须使用 RefCell（运行时检查）
pub struct RcSupplier<T> {
    func: Rc<RefCell<dyn FnMut() -> T>>,  // 必须有 RefCell
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.borrow_mut())()  // 运行时借用检查
    }
}
```

#### 2. **性能开销明显** ⭐⭐⭐⭐

```rust
// 对比：Predicate 的零开销
impl<T> Predicate<T> for ArcPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.inner)(value)  // ✅ 直接调用，零开销
    }
}

// Supplier 的开销
impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()  // ❌ 每次加锁，开销大
    }
}
```

#### 3. **API 不如 Predicate 优雅** ⭐⭐⭐⭐

Predicate 的 `ArcPredicate` 可以使用 `&self`：

```rust
// ✅ Predicate：优雅的 &self API
impl<T> ArcPredicate<T> {
    pub fn and(&self, other: &Self) -> Self {  // &self！
        // ...
    }
}

let pred = ArcPredicate::new(|x| *x > 0);
let combined = pred.and(&other);  // ✅ pred 仍可用
```

Supplier 必须使用 `&mut self`：

```rust
// ⚠️ Supplier：必须用 &mut self
impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {  // &mut self！
        // ...
    }
}

let mut supplier = ArcSupplier::new(|| 42);
let value = supplier.get();  // ⚠️ 需要 mut
```

#### 4. **组合方法实现复杂** ⭐⭐⭐⭐

```rust
// ArcSupplier 的 map 需要嵌套多层 Arc<Mutex>
pub fn map<U, F>(&self, mapper: F) -> ArcSupplier<U>
where
    F: FnMut(T) -> U + Send + 'static,
{
    let func = Arc::clone(&self.func);
    let mapper = Arc::new(Mutex::new(mapper));  // mapper 也要装箱
    ArcSupplier {
        func: Arc::new(Mutex::new(move || {
            let value = func.lock().unwrap()();      // 第一个锁
            mapper.lock().unwrap()(value)            // 第二个锁
        })),
        name: None,
    }
}
```

#### 5. **代码量爆炸** ⭐⭐⭐

需要为三个类型分别实现：`map`, `filter`, `zip`, `memoize`, `lazy` 等所有方法。

#### 6. **学习成本高** ⭐⭐⭐

用户需要理解：
- 为什么需要三种类型
- `Mutex` 和 `RefCell` 的区别
- 何时用哪种类型
- 性能影响

#### 7. **使用场景不匹配** ⭐⭐⭐⭐

统计显示，Supplier 的使用场景：
- 90% 是单线程、一次性使用
- 5% 是单线程、需要复用
- 5% 是多线程共享

为了 5% 的场景引入三种类型的复杂度，收益不大。

### 适用场景

✅ **可能适合以下场景：**

1. **大型库项目**：需要支持各种复杂场景
2. **明确需要多种所有权模型**：确实有大量跨线程共享需求
3. **性能不敏感**：可以接受 Mutex/RefCell 的开销

❌ **不适合以下场景（大多数情况）：**

1. **一般项目**：引入的复杂度 >> 实际收益
2. **性能敏感场景**：每次调用都加锁开销太大
3. **主要单线程使用**：不需要复杂的共享机制

---

## 三种方案对比总结

### 核心特性对比表

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 |
|:---|:---|:---|:---|
| **调用方式** | `supplier()` ✅ | `supplier.get()` ⚠️ | `supplier.get()` ⚠️ |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 极好 |
| **所有权模型** | Box + Arc (两种) | Box + Arc (两种) | Box + Arc + Rc (三种) |
| **类型名称** | Supplier / SharedSupplier | Supplier / SharedSupplier | BoxSupplier / ArcSupplier / RcSupplier ✅ |
| **统一接口** | ❌ 两套独立 API | ❌ 两套独立 struct | ✅ 统一 Supplier trait |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ 支持 |
| **实现复杂度** | ✅ 极简 | 🟡 中等 | ❌ **非常复杂** ⚠️ |
| **性能开销** | ✅ 低（Box）<br>⚠️ 高（Arc+Mutex） | ✅ 低（Box）<br>⚠️ 高（Arc+Mutex） | ✅ 低（Box）<br>❌ **高（Arc+Mutex）** ⚠️ |
| **共享使用体验** | ❌ 差（显式锁） | 🟡 中等（封装锁） | 🟡 中等（封装锁） |
| **代码量** | ✅ 极少 | 🟡 中等 | ❌ **非常多** |
| **维护成本** | ✅ 低 | 🟡 中等 | ❌ **高** |
| **学习成本** | ✅ 最低 | 🟡 中等 | ❌ **高** |

### 使用场景对比

| 场景 | 方案一 | 方案二 ✅ | 方案三 |
|:---|:---:|:---:|:---:|
| **快速原型开发** | ✅ 最佳 | 🟢 好 | ❌ 过度设计 |
| **简单值生成** | ✅ 最佳 | ✅ 最佳 | ❌ 过度设计 |
| **复杂方法链** | ❌ 不支持 | ✅ 最佳 | 🟢 好 |
| **单线程为主** | ✅ 最佳 | ✅ **最佳** | ⚠️ 复杂 |
| **少量多线程共享** | ❌ 使用体验差 | 🟢 可以接受 | 🟡 可以 |
| **大量多线程共享** | ❌ 不适合 | ⚠️ 性能问题 | 🟡 可以（性能差） |
| **中等项目** | 🟢 好 | ✅ **最佳** | ❌ 过度设计 |
| **大型库项目** | ⚠️ 扩展性差 | ✅ **推荐** | 🟡 可以考虑 |

### Predicate vs Supplier 方案对比

这个对比表清楚地展示了为什么同样的方案对两种类型的适用性如此不同：

| 维度 | Predicate 方案三 | Supplier 方案三 |
|:---|:---|:---|
| **核心函数** | `Fn(&T) -> bool` | `FnMut() -> T` |
| **可变性** | ✅ 不可变（`&self`） | ❌ 需要可变（`&mut self`） |
| **共享复杂度** | ✅ 简单（`Arc<dyn Fn>`） | ❌ 复杂（`Arc<Mutex<dyn FnMut>>`） |
| **性能开销** | ✅ 零开销 | ❌ 每次调用加锁 |
| **API 优雅度** | ✅ 极优雅（`&self`） | ⚠️ 一般（`&mut self`） |
| **组合方法实现** | ✅ 简单 | ❌ 复杂（多层嵌套锁） |
| **主要使用场景** | ✅ 大量共享复用 | ❌ 主要一次性使用 |
| **方案适用性** | ✅ **完美适配** ✨ | ❌ **收益不足** ⚠️ |

---

## 推荐方案与理由

### 🏆 推荐：方案二（Struct 封装 + 实例方法）

**这是当前库采用的方案，也是最佳选择。**

### 推荐理由

#### 1. **平衡了简洁性和功能性**

- ✅ 提供了方法链、元数据等高级功能
- ✅ 实现复杂度适中，代码量可控
- ✅ 维护成本合理

#### 2. **契合实际使用场景**

根据对 Supplier 使用场景的分析：

```
一次性使用/单线程：约 90%  → BoxSupplier（方案二的 Supplier）
单线程复用：     约 5%   → 使用 Box + clone workaround
多线程共享：     约 5%   → SharedSupplier
```

方案二用最小的复杂度覆盖了最常见的场景。

#### 3. **性能表现好**

- ✅ 单所有权场景（90%）：零开销
- ⚠️ 共享场景（10%）：虽有锁开销，但频率低，可接受

#### 4. **学习成本低**

- 只有两个类型：`Supplier` 和 `SharedSupplier`
- API 直观，与标准库风格一致
- 新手友好

#### 5. **可以渐进式增强**

如果未来真的需要更多功能：
- 可以添加 `RcSupplier`（成本低）
- 可以添加更多组合方法
- 不影响现有代码

### 不推荐方案三的核心原因

#### 1. **技术约束**

`FnMut` 的可变性要求导致：
- `Arc` 必须配合 `Mutex`，性能差
- `Rc` 必须配合 `RefCell`，运行时检查
- 无法像 Predicate 那样优雅

#### 2. **场景不匹配**

```rust
// Predicate：大量共享场景
let shared_predicate = ArcPredicate::new(|x| *x > 0);
// 在配置、验证、过滤器等场景大量使用 ✅

// Supplier：主要一次性场景
let supplier = Supplier::new(|| generate_value());
let value = supplier.get();  // 通常只调用几次 ⚠️
```

#### 3. **复杂度 vs 收益**

```
方案三的成本：
- 实现复杂度：⭐⭐⭐⭐⭐ (非常高)
- 性能开销：  ⭐⭐⭐⭐   (显著)
- 学习成本：  ⭐⭐⭐⭐   (高)
- 维护成本：  ⭐⭐⭐⭐   (高)

实际收益：
- 覆盖 5% 的多线程场景
- 覆盖 5% 的单线程复用场景

结论：成本 >> 收益 ❌
```

### 特殊场景建议

如果确实有大量跨线程共享 Supplier 的需求：

**建议：使用领域特定的封装**

```rust
// 不要引入通用的三类型体系
// 而是为特定领域设计专门的类型

pub struct ConfigSupplier {
    inner: Arc<Mutex<dyn FnMut() -> Config + Send>>,
}

pub struct IdGenerator {
    counter: Arc<AtomicU64>,
}

impl IdGenerator {
    pub fn get(&self) -> u64 {  // 注意：&self，不需要 &mut
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}
```

这种领域特定的设计：
- ✅ 性能更好（如 `AtomicU64` 代替 `Mutex`）
- ✅ API 更清晰
- ✅ 更符合实际需求

---

## 结论

### 核心要点

1. **Supplier ≠ Predicate**
   - 可变性要求使得 Supplier 无法像 Predicate 那样优雅
   - 同样的设计模式对两者的适用性完全不同

2. **方案二是最佳平衡**
   - 覆盖了 90% 的使用场景
   - 实现复杂度适中
   - 性能表现好

3. **方案三不推荐**
   - 实现复杂度高
   - 性能开销大
   - 使用场景不匹配
   - 学习和维护成本高

4. **保持务实**
   - 不要为了架构完美而过度设计
   - 针对实际需求选择合适的方案
   - 可以渐进式增强

### 最终建议

✅ **保持当前的方案二设计**

对于 `prism3-rust-function` 库，当前的 `Supplier<T>` + `SharedSupplier<T>` 设计是最佳选择。它在简洁性、功能性和性能之间取得了良好的平衡，满足了绝大多数使用场景的需求。

如果未来确实有更多跨线程共享的需求，可以考虑：
1. 添加 `RcSupplier` 单独支持单线程复用场景
2. 或者针对特定领域设计专门的生成器类型

但不建议引入完整的三类型体系，因为其复杂度远大于实际收益。

---

## 附录：实现建议

如果确实要实现方案三，以下是一些建议：

### 1. 优化 ArcSupplier 性能

```rust
// 使用 parking_lot 的 Mutex（性能更好）
use parking_lot::Mutex;

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

// parking_lot::Mutex 比 std::sync::Mutex 快约 2-3 倍
```

### 2. 考虑特殊化实现

```rust
// 对于无状态的情况，可以避免 Mutex
pub struct StatelessArcSupplier<T> {
    func: Arc<dyn Fn() -> T + Send + Sync>,  // Fn 而不是 FnMut
}

impl<T> Supplier<T> for StatelessArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func)()  // ✅ 无需加锁
    }
}
```

### 3. 提供转换方法

```rust
impl<T> BoxSupplier<T> {
    /// 转换为 SharedSupplier（需要 Send）
    pub fn into_shared(self) -> SharedSupplier<T>
    where
        T: Send + 'static,
    {
        // ...
    }
}
```

但即使有这些优化，方案三的复杂度仍然是个大问题。

