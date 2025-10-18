# Supplier 设计方案对比分析

## 概述

本文档分析 Rust 中实现 Supplier（供应者）类型的设计方案，阐明核心语义和设计决策。

## 什么是 Supplier？

### Supplier 的本质语义

在函数式编程中，**Supplier（供应者）**的核心语义是：

> **生成并提供一个值，不接受输入参数。可能每次生成新值（如工厂），也可能返回固定值（如常量）。**

这类似于现实生活中的"供应"行为：
- ✅ **工厂生产产品**：每次调用生成新的实例
- ✅ **仓库提供库存**：返回已有的值（或其引用）
- ✅ **计数器生成序列号**：每次调用状态递增，返回不同值
- ✅ **配置提供默认值**：返回固定的默认配置

### Supplier vs 其他函数式抽象

基于这个语义理解，我们需要明确 Supplier 与其他类型的区别：

| 类型 | 输入 | 输出 | 修改自己？| 典型用途 | Java 对应 |
|------|------|------|----------|---------|-----------|
| **Supplier** | 无 | `T` | ✅ | 工厂、生成器、延迟初始化 | `Supplier<T>` |
| **Function** | `&T` | `R` | ❌ | 转换、映射、计算 | `Function<T, R>` |
| **Consumer** | `&T` | `()` | ✅ | 观察、日志、统计 | `Consumer<T>` |
| **Predicate** | `&T` | `bool` | ❌ | 过滤、验证、判断 | `Predicate<T>` |

**关键洞察**：
- Supplier 是**唯一不需要输入的**函数式抽象
- Supplier **可以修改自身状态**（生成不同的值）
- Supplier 必须返回**所有权 `T`**（不返回引用，避免生命周期问题）

### Supplier 的主要用途

Supplier 类型的核心价值在于：

1. **延迟初始化**：将昂贵的计算推迟到真正需要时
2. **工厂模式**：封装对象创建逻辑
3. **依赖注入**：提供可配置的值源
4. **生成器模式**：按需生成序列值
5. **默认值提供**：为可选参数提供默认值

**如果只是获取一个固定值，直接用变量更简单**：
```rust
// ❌ 不需要 Supplier：直接用变量
let default_config = Config::default();

// ✅ 需要 Supplier：延迟初始化，避免不必要的计算
struct Service {
    config_supplier: BoxSupplier<Config>,  // 只在需要时创建
}

// ✅ 需要 Supplier：每次生成新值
let id_generator = BoxSupplier::new(|| generate_uuid());
```

## 核心设计决策

### 1. 返回值的所有权

Supplier 应该返回 `T` 还是 `&T`？这是最核心的设计问题。

#### 方案 A：返回所有权 `T`

```rust
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // 返回所有权
}

// 使用场景：工厂模式
let mut factory = BoxSupplier::new(|| User::new("Alice"));
let user1 = factory.get();  // 每次生成新实例
let user2 = factory.get();  // 独立的新实例
```

**优点**：
- ✅ 语义清晰：每次"生产"新值
- ✅ 灵活性高：可以生成不同的实例
- ✅ 无生命周期问题：返回值独立存在
- ✅ 符合 Java `Supplier<T>` 语义

**缺点**：
- ❌ 无法返回引用类型
- ❌ 必须每次克隆或重新创建（成本可能高）

#### 方案 B：返回引用 `&T`

```rust
pub trait RefSupplier<T> {
    fn get(&self) -> &T;  // 返回引用
}

// 使用场景：提供已有值的引用
let config = Config::default();
let supplier = BoxRefSupplier::new(move || &config);  // ❌ 生命周期问题！
```

**问题**：生命周期约束极其复杂，几乎无法实现通用的 `RefSupplier`！

```rust
// 生命周期问题示例
pub trait RefSupplier<'a, T> {
    fn get(&'a self) -> &'a T;  // 'a 必须固定
}

// 用户代码
let supplier = create_supplier();
let ref1 = supplier.get();
let ref2 = supplier.get();  // ref1 和 ref2 互相干扰！
```

**结论**：返回引用的设计在 Rust 中几乎不可行（除非有明确的生命周期保证）。

#### 推荐方案：只支持返回所有权 `T`

```rust
/// 供应者 - 生成并返回值
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // 返回所有权
}

// 如果需要提供引用，包装为返回 Arc<T> 或 Rc<T>
let arc_config = Arc::new(Config::default());
let supplier = BoxSupplier::new(move || Arc::clone(&arc_config));
let config = supplier.get();  // 返回 Arc<Config>
```

**理由**：
1. **避免生命周期陷阱**：返回 `T` 没有生命周期问题
2. **语义明确**：Supplier 是"生产者"，每次返回新值
3. **灵活性**：用户可以选择返回 `Arc<T>`、`Rc<T>` 或克隆的值
4. **与 Java 一致**：Java 的 `Supplier<T>` 也是返回值而非引用

### 2. self 的可变性

Supplier 自己是否需要可变？这涉及是否可以生成不同的值：

```rust
// 方案 A：ReadonlySupplier（不可变 self）
pub trait ReadonlySupplier<T> {
    fn get(&self) -> T;  // 不修改自己
}

// 方案 B：Supplier（可变 self）
pub trait Supplier<T> {
    fn get(&mut self) -> T;  // 可修改自己的状态
}
```

**场景对比**：

| 场景 | 需要修改状态？| 适合的类型 |
|------|------------|-----------|
| 固定默认值 | ❌ | ReadonlySupplier |
| 计数器生成器 | ✅ | Supplier |
| 随机数生成 | ✅ | Supplier |
| 工厂（每次新实例）| 🟡 可能需要 | Supplier |
| 迭代器模式 | ✅ | Supplier |

**关键问题**：ReadonlySupplier 真的有价值吗？

#### ReadonlySupplier 的场景分析

```rust
// 场景 1：返回固定值
let supplier = BoxReadonlySupplier::new(|| 42);
let value1 = supplier.get();  // 42
let value2 = supplier.get();  // 42

// ❌ 没意义：直接用常量不更好吗？
const DEFAULT_VALUE: i32 = 42;
let value1 = DEFAULT_VALUE;
let value2 = DEFAULT_VALUE;

// 场景 2：工厂模式（每次创建新对象）
let factory = BoxReadonlySupplier::new(|| User::new("Alice"));
let user1 = factory.get();  // 新对象
let user2 = factory.get();  // 又一个新对象

// 🟡 可行：闭包本身不修改状态，但每次返回新对象
// 但问题是：工厂场景很少见，大多数 Supplier 场景需要状态

// 场景 3：延迟计算（只计算一次）
let cached = {
    let mut cache = None;
    BoxSupplier::new(move || {
        if cache.is_none() {
            cache = Some(expensive_computation());
        }
        cache.clone().unwrap()
    })
};
let v1 = cached.get();  // 第一次：计算
let v2 = cached.get();  // 第二次：返回缓存

// ✅ 用 Supplier (`&mut self`) 直接实现，不需要内部可变性！
```

#### 与 Consumer/Predicate 的对比

| 类型 | `&self` 变体价值 | 理由 |
|------|-----------------|------|
| **Consumer** | ✅ 高（ReadonlyConsumer）| 主要场景（日志、通知）确实不需要修改状态 |
| **Predicate** | N/A（只有 `&self`）| 判断操作天然不应该修改状态 |
| **Supplier** | ❌ 低（ReadonlySupplier）| 主要场景（计数器、生成器、有状态工厂）都需要修改状态 |

#### 为什么 Supplier 不需要 ReadonlySupplier？

**关键差异**：Supplier 本身使用 `&mut self`，已经可以修改状态，**不需要**内部可变性：

```rust
// Supplier：直接修改状态，无需内部可变性
let mut counter = {
    let mut count = 0;
    BoxSupplier::new(move || {
        count += 1;  // 直接修改，因为 get(&mut self)
        count
    })
};

// Predicate：需要内部可变性才能修改状态
let counter_pred = {
    let count = Cell::new(0);  // ❗ 必须用 Cell
    BoxPredicate::new(move |x: &i32| {
        count.set(count.get() + 1);  // 通过 Cell 修改
        *x > 0
    })
};
```

**结论**：
- ✅ **只提供 `Supplier<T>` (使用 `&mut self`)**：覆盖所有场景
- ❌ **不需要 ReadonlySupplier**：价值极低，增加复杂度

### 3. SupplierOnce 的价值

**关键理解**：SupplierOnce 与 Supplier 的区别不仅在于 `self` 的所有权，更在于**一次性资源消耗**。

```rust
pub trait SupplierOnce<T> {
    fn get(self) -> T;  // 消费 self，返回值
}

// 使用场景 1：延迟初始化（只初始化一次）
let initializer = BoxSupplierOnce::new(|| {
    expensive_initialization()
});
let value = initializer.get();  // 消费 supplier

// 使用场景 2：消耗资源生成值
let resource = acquire_resource();
let supplier = BoxSupplierOnce::new(move || {
    consume_resource(resource)  // resource 被移动
});

// 使用场景 3：配合 Option 实现延迟计算
struct LazyValue<T> {
    supplier: Option<BoxSupplierOnce<T>>,
    value: Option<T>,
}

impl<T> LazyValue<T> {
    fn get_or_init(&mut self) -> &T {
        if self.value.is_none() {
            let supplier = self.supplier.take().unwrap();
            self.value = Some(supplier.get());
        }
        self.value.as_ref().unwrap()
    }
}
```

**对比 Supplier**：

```rust
// Supplier：可以多次调用（但需要 &mut self）
let mut counter = BoxSupplier::new(|| next_id());
let id1 = counter.get();
let id2 = counter.get();

// SupplierOnce：只能调用一次，消耗 self
let once = BoxSupplierOnce::new(|| initialize_db());
let db = once.get();  // once 被消耗
```

**SupplierOnce 的真实价值**：

1. **类型系统保证一次性**：编译期防止多次调用
2. **保存 FnOnce 闭包**：闭包可以移动捕获的变量
3. **延迟初始化模式**：配合 Option 实现懒加载
4. **资源消耗场景**：生成值时消耗不可克隆的资源

**结论**：SupplierOnce 是**必要的**，与 Supplier 形成互补。

---

## 三种实现方案对比

### 方案一：类型别名 + 静态组合方法

使用类型别名定义 Supplier 类型，并通过静态工具类提供辅助方法。

```rust
// 类型别名定义
pub type Supplier<T> = Box<dyn FnMut() -> T>;
pub type SupplierOnce<T> = Box<dyn FnOnce() -> T>;
pub type ArcSupplier<T> = Arc<Mutex<dyn FnMut() -> T + Send>>;

// 静态工具类
pub struct Suppliers;

impl Suppliers {
    pub fn constant<T: Clone + 'static>(value: T) -> Supplier<T> {
        Box::new(move || value.clone())
    }

    pub fn lazy<T, F>(f: F) -> SupplierOnce<T>
    where
        F: FnOnce() -> T + 'static,
    {
        Box::new(f)
    }
}
```

**使用示例**：
```rust
// 创建 supplier
let mut supplier: Supplier<i32> = Box::new(|| 42);
let value = supplier();  // ✅ 可以直接调用

// 使用工具方法
let constant = Suppliers::constant(100);
let lazy = Suppliers::lazy(|| expensive_init());
```

**优点**：
- ✅ 极简的 API，直接调用 `supplier()`
- ✅ 与标准库完美集成
- ✅ 零成本抽象，单次装箱
- ✅ 实现简单，代码量少

**缺点**：
- ❌ 无法扩展（不能添加字段、实现 trait）
- ❌ 类型区分度低（与 `Box<dyn FnMut>` 等价）
- ❌ 无法实现方法链
- ❌ 需要维护多套 API（Supplier、ArcSupplier 等）

---

### 方案二：Struct 封装 + 实例方法

将 Supplier 定义为 struct，内部包装 `Box<dyn FnMut>`，通过实例方法提供功能。

```rust
pub struct Supplier<T> {
    func: Box<dyn FnMut() -> T>,
}

impl<T> Supplier<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        Supplier { func: Box::new(f) }
    }

    pub fn get(&mut self) -> T {
        (self.func)()
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone,
    {
        Supplier::new(move || value.clone())
    }

    pub fn map<R, F>(self, mapper: F) -> Supplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
    {
        let mut func = self.func;
        let mut mapper = mapper;
        Supplier::new(move || mapper(func()))
    }
}

pub struct SupplierOnce<T> {
    func: Option<Box<dyn FnOnce() -> T>>,
}

impl<T> SupplierOnce<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        SupplierOnce {
            func: Some(Box::new(f)),
        }
    }

    pub fn get(mut self) -> T {
        (self.func.take().unwrap())()
    }
}

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcSupplier {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn get(&self) -> T {
        (self.func.lock().unwrap())()
    }
}

impl<T> Clone for ArcSupplier<T> {
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
let mut supplier = Supplier::new(|| 42);
let value = supplier.get();  // 必须使用 .get()

// 工厂方法
let constant = Supplier::constant(100);
let mut counter = {
    let mut count = 0;
    Supplier::new(move || {
        count += 1;
        count
    })
};

// 方法链
let mut mapped = Supplier::new(|| 5)
    .map(|x| x * 2)
    .map(|x| x + 1);
assert_eq!(mapped.get(), 11);

// ArcSupplier 可以跨线程共享
let arc_supplier = ArcSupplier::new(|| generate_id());
let clone = arc_supplier.clone();
std::thread::spawn(move || {
    let id = clone.get();
    println!("Generated: {}", id);
});
```

**优点**：
- ✅ 优雅的方法链（`.map()` 等）
- ✅ 强大的扩展性（可添加字段、实现 trait）
- ✅ 类型安全，独立的类型
- ✅ 丰富的工厂方法

**缺点**：
- ❌ 无法直接调用（必须用 `.get()`）
- ❌ 需要维护多套独立实现（Supplier、ArcSupplier 等）
- ❌ 代码重复（工厂方法需要分别实现）

---

### 方案三：Trait 抽象 + 多种实现（推荐，当前采用）

定义统一的 `Supplier` trait，提供三种具体实现（Box/Arc/Rc），在 struct 上实现特例化的方法。

```rust
// ============================================================================
// 1. 统一的 Supplier trait
// ============================================================================

pub trait Supplier<T> {
    fn get(&mut self) -> T;

    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
}

pub trait SupplierOnce<T> {
    fn get(self) -> T;

    fn into_box(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. 为闭包实现 Supplier trait
// ============================================================================

impl<T, F> Supplier<T> for F
where
    F: FnMut() -> T,
{
    fn get(&mut self) -> T {
        self()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplier::new(self)
    }

    // ... 其他 into_* 方法
}

// ============================================================================
// 3. BoxSupplier - 单一所有权实现
// ============================================================================

pub struct BoxSupplier<T> {
    func: Box<dyn FnMut() -> T>,
}

impl<T> BoxSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        BoxSupplier { func: Box::new(f) }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        BoxSupplier::new(move || value.clone())
    }

    /// 映射：转换 Supplier 的输出
    pub fn map<R, F>(self, mapper: F) -> BoxSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
    {
        let mut func = self.func;
        let mut mapper = mapper;
        BoxSupplier::new(move || mapper(func()))
    }
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&mut self) -> T {
        (self.func)()
    }

    // ... into_* 方法实现
}

// ============================================================================
// 4. BoxSupplierOnce - 一次性供应者
// ============================================================================

pub struct BoxSupplierOnce<T> {
    func: Option<Box<dyn FnOnce() -> T>>,
}

impl<T> BoxSupplierOnce<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        BoxSupplierOnce {
            func: Some(Box::new(f)),
        }
    }
}

impl<T> SupplierOnce<T> for BoxSupplierOnce<T> {
    fn get(mut self) -> T {
        (self.func.take().unwrap())()
    }
}

// ============================================================================
// 5. ArcSupplier - 线程安全的共享所有权实现
// ============================================================================

pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcSupplier {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + Send + 'static,
    {
        ArcSupplier::new(move || value.clone())
    }

    /// ArcSupplier 的 map：借用 &self，返回新的 ArcSupplier
    pub fn map<R, F>(&self, mapper: F) -> ArcSupplier<R>
    where
        F: FnMut(T) -> R + Send + 'static,
        R: Send + 'static,
        T: 'static,
    {
        let func = Arc::clone(&self.func);
        let mut mapper = mapper;
        ArcSupplier::new(move || mapper((func.lock().unwrap())()))
    }
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()
    }

    // ... into_* 方法实现
}

impl<T> Clone for ArcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. RcSupplier - 单线程的共享所有权实现
// ============================================================================

pub struct RcSupplier<T> {
    func: Rc<RefCell<dyn FnMut() -> T>>,
}

impl<T> RcSupplier<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        RcSupplier {
            func: Rc::new(RefCell::new(f)),
        }
    }

    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        RcSupplier::new(move || value.clone())
    }

    /// RcSupplier 的 map：借用 &self，返回新的 RcSupplier
    pub fn map<R, F>(&self, mapper: F) -> RcSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static,
        T: 'static,
    {
        let func = Rc::clone(&self.func);
        let mut mapper = mapper;
        RcSupplier::new(move || mapper((func.borrow_mut())()))
    }
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.borrow_mut())()
    }

    // ... into_* 方法实现
}

impl<T> Clone for RcSupplier<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
```

**使用示例**：
```rust
// 1. 闭包自动拥有 .get() 方法
let mut closure = || 42;
let value = closure.get();  // ✅ 直接使用

// 2. BoxSupplier - 一次性使用
let mut counter = {
    let mut count = 0;
    BoxSupplier::new(move || {
        count += 1;
        count
    })
};
assert_eq!(counter.get(), 1);
assert_eq!(counter.get(), 2);

// 3. BoxSupplier 方法链
let mut mapped = BoxSupplier::new(|| 5)
    .map(|x| x * 2)
    .map(|x| x + 1);
assert_eq!(mapped.get(), 11);

// 4. BoxSupplierOnce - 延迟初始化
let once = BoxSupplierOnce::new(|| {
    println!("Expensive initialization");
    expensive_init()
});
let value = once.get();  // 只初始化一次

// 5. ArcSupplier - 多线程共享，不需要显式 clone
let shared = ArcSupplier::new(|| generate_uuid());
let mapped = shared.map(|id| format!("ID: {}", id));
// shared 仍然可用
let clone = shared.clone();
std::thread::spawn(move || {
    let mut c = clone;
    let id = c.get();
    println!("{}", id);
});

// 6. RcSupplier - 单线程复用
let rc = RcSupplier::constant(100);
let mapped1 = rc.map(|x| x * 2);
let mapped2 = rc.map(|x| x + 10);
// rc 仍然可用

// 7. 统一的接口
fn use_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
    supplier.get()
}

let mut box_sup = BoxSupplier::new(|| 42);
use_supplier(&mut box_sup);

let mut arc_sup = ArcSupplier::new(|| 100);
use_supplier(&mut arc_sup);
```

**优点**：
- ✅ 统一的 trait 接口（所有类型实现 `Supplier<T>`）
- ✅ 语义清晰（`BoxSupplier`/`ArcSupplier`/`RcSupplier` 名称即文档）
- ✅ 完整的所有权模型覆盖（Box/Arc/Rc 三种）
- ✅ 类型保持（`ArcSupplier.map()` 返回 `ArcSupplier`）
- ✅ 优雅的 API（Arc/Rc 的方法使用 `&self`，无需显式 clone）
- ✅ 解决内部可变性（Arc 用 Mutex，Rc 用 RefCell）
- ✅ 最强的扩展性（可添加新实现、字段、trait）
- ✅ 与 Rust 标准库设计哲学一致

**缺点**：
- ❌ 仍然无法直接调用（必须用 `.get()`）
- ❌ 学习成本略高（需要理解三种实现的区别）
- ❌ 实现成本高（需要为三个 struct 分别实现）

---

## 三种方案对比总结

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 ⭐ |
|:---|:---:|:---:|:---:|
| **调用方式** | `supplier()` ✅ | `supplier.get()` | `supplier.get()` |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 **极好** ✨ |
| **统一接口** | ❌ 无 | ❌ 两套独立 | ✅ **统一 trait** ✨ |
| **所有权模型** | Box + Arc（两种）| Box + Arc（两种）| Box + Arc + Rc（三种）✅ |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ **支持（且类型保持）** ✨ |
| **扩展性** | ❌ 无法扩展 | ✅ 可扩展 | ✅ **极易扩展** |
| **代码简洁度** | ✅ **极简** | 🟡 中等 | 🟡 略复杂 |
| **学习成本** | ✅ **最低** | 🟡 中等 | 🟡 略高 |
| **维护成本** | 🟡 中等 | 🟡 中等 | ✅ **低（架构清晰）** |
| **与标准库一致** | 🟡 中等 | 🟡 中等 | ✅ **完美** ✨ |

### 适用场景对比

| 场景 | 方案一 | 方案二 | 方案三 ⭐ |
|:---|:---:|:---:|:---:|
| **快速原型开发** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **复杂方法链** | ❌ 不适合 | ✅ 适合 | ✅ **最佳** |
| **多线程共享** | 🟡 手动 Arc | 🟡 ArcSupplier | ✅ **ArcSupplier（清晰）** |
| **单线程复用** | ❌ 不支持 | ❌ 不支持 | ✅ **RcSupplier（无锁）** |
| **库开发** | 🟡 可以 | ✅ 适合 | ✅ **最佳** |
| **长期维护** | 🟡 中等 | 🟡 中等 | ✅ **最佳** |

---

## 推荐的完整设计

### 核心 Trait 定义

```rust
// === Supplier 系列（生成值）===

/// 供应者：生成并返回值
pub trait Supplier<T> {
    /// 获取值（可以多次调用）
    fn get(&mut self) -> T;
}

/// 一次性供应者：生成并返回值，只能调用一次
pub trait SupplierOnce<T> {
    /// 获取值（消耗 self，只能调用一次）
    fn get(self) -> T;
}
```

**当前实现状态**：
- ✅ `Supplier` - 需要实现
- ✅ `SupplierOnce` - 需要实现
- ❌ `ReadonlySupplier` - 不需要（主要场景都需要状态修改，价值极低）

### 具体实现

```rust
// Box 实现（单一所有权）
pub struct BoxSupplier<T> { func: Box<dyn FnMut() -> T> }
pub struct BoxSupplierOnce<T> { func: Option<Box<dyn FnOnce() -> T>> }

// Arc 实现（线程安全共享）
pub struct ArcSupplier<T> { func: Arc<Mutex<dyn FnMut() -> T + Send>> }

// Rc 实现（单线程共享）
pub struct RcSupplier<T> { func: Rc<RefCell<dyn FnMut() -> T>> }
```

### 类型选择指南

| 需求 | 推荐类型 | 理由 |
|------|---------|------|
| 一次性使用 | `BoxSupplier` | 单一所有权，无开销 |
| 延迟初始化（只计算一次）| `BoxSupplierOnce` | 消耗 self，保存 FnOnce |
| 多线程共享 | `ArcSupplier` | 线程安全，Mutex 保护 |
| 单线程复用 | `RcSupplier` | RefCell 无锁开销 |
| 固定常量 | `BoxSupplier::constant()` | 工厂方法 |
| 计数器/生成器 | `BoxSupplier` | 可修改状态 |

### 常用工厂方法

```rust
impl<T> BoxSupplier<T> {
    /// 创建常量供应者（每次返回相同值的克隆）
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static;

    /// 创建递增计数器
    pub fn counter(start: i32) -> BoxSupplier<i32> {
        let mut count = start;
        BoxSupplier::new(move || {
            let result = count;
            count += 1;
            result
        })
    }

    /// 映射供应者的输出
    pub fn map<R, F>(self, mapper: F) -> BoxSupplier<R>
    where
        F: FnMut(T) -> R + 'static,
        R: 'static;
}

impl<T> BoxSupplierOnce<T> {
    /// 创建延迟初始化供应者
    pub fn lazy<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static;
}
```

---

## 总结

### 为什么选择方案三？

**`prism3-rust-function` 采用方案三**，原因如下：

1. **统一的 trait 抽象**
   - 提供 `Supplier<T>` 和 `SupplierOnce<T>` trait
   - 所有类型通过统一接口使用
   - 支持泛型编程

2. **完整的所有权模型覆盖**
   - Box：单一所有权，零开销
   - Arc：线程安全共享，Mutex 保护
   - Rc：单线程共享，RefCell 优化

3. **优雅的 API 设计**
   - 类型保持：`ArcSupplier.map()` 返回 `ArcSupplier`
   - 无需显式 clone：Arc/Rc 的方法使用 `&self`
   - 方法链：流式 API

4. **与 Rust 生态一致**
   - 命名模式与标准库智能指针一致（Box/Arc/Rc）
   - 设计哲学符合 Rust 惯例

5. **长期可维护性**
   - 清晰的架构
   - 易于扩展（添加新实现、trait、元数据）
   - 类型名称即文档

### 核心设计原则

1. **Supplier 返回所有权 `T`**：避免生命周期问题，语义明确
2. **Supplier 使用 `&mut self`**：典型场景（计数器、生成器）都需要修改状态
3. **保留 SupplierOnce**：延迟初始化、一次性资源消耗
4. **不需要 ReadonlySupplier**：主要场景都需要状态修改，价值极低
5. **类型名称语义明确**：Box/Arc/Rc 表达所有权模型

### Supplier 与其他函数式抽象的对比

| | Supplier | Consumer | Predicate | Function |
|---|---|---|---|---|
| **输入** | 无 | `&T` | `&T` | `&T` |
| **输出** | `T` | `()` | `bool` | `R` |
| **self 签名** | `&mut self` | `&mut self` | `&self` | `&self` |
| **修改自己** | ✅ 典型场景 | ✅ 典型场景 | ❌ 不应该 | ❌ 不应该 |
| **Once 变体** | ✅ 有价值 | ✅ 有价值 | ❌ 无意义 | 🟡 边缘场景 |
| **Readonly 变体** | ❌ 不需要 | ✅ 有价值 | N/A（只有 `&self`）| N/A（只有 `&self`）|
| **核心用途** | 工厂、生成器 | 观察、累积 | 过滤、验证 | 转换、映射 |

### 设计一致性

所有函数式抽象遵循统一的设计模式：

1. **统一的 trait 接口**：每种抽象都有核心 trait
2. **三种实现**：Box（单一）、Arc（共享+线程安全）、Rc（共享+单线程）
3. **类型保持的方法链**：组合方法返回相同类型
4. **闭包自动实现 trait**：无缝集成
5. **扩展 trait 提供组合能力**：如 `FnSupplierOps`

这个设计为用户提供了最灵活、最强大、最清晰的 API，是库项目的最佳选择。

---

## 附录：常见使用模式

### 1. 延迟初始化

```rust
struct Database {
    connection: OnceCell<Connection>,
    supplier: BoxSupplierOnce<Connection>,
}

impl Database {
    fn new<F>(init: F) -> Self
    where
        F: FnOnce() -> Connection + 'static,
    {
        Database {
            connection: OnceCell::new(),
            supplier: BoxSupplierOnce::new(init),
        }
    }

    fn get_connection(&mut self) -> &Connection {
        self.connection.get_or_init(|| self.supplier.get())
    }
}
```

### 2. 工厂模式

```rust
struct UserFactory {
    id_generator: BoxSupplier<u64>,
}

impl UserFactory {
    fn new() -> Self {
        let mut id = 0;
        UserFactory {
            id_generator: BoxSupplier::new(move || {
                id += 1;
                id
            }),
        }
    }

    fn create_user(&mut self, name: &str) -> User {
        User {
            id: self.id_generator.get(),
            name: name.to_string(),
        }
    }
}
```

### 3. 配置默认值

```rust
struct Config {
    timeout: Duration,
    max_retries: u32,
}

impl Config {
    fn default_timeout() -> BoxSupplier<Duration> {
        BoxSupplier::constant(Duration::from_secs(30))
    }

    fn default_max_retries() -> BoxSupplier<u32> {
        BoxSupplier::constant(3)
    }
}
```

### 4. 随机数生成器

```rust
use rand::Rng;

fn random_supplier() -> BoxSupplier<u32> {
    BoxSupplier::new(|| rand::thread_rng().gen())
}

fn random_range_supplier(min: i32, max: i32) -> BoxSupplier<i32> {
    BoxSupplier::new(move || rand::thread_rng().gen_range(min..max))
}
```

### 5. 多线程共享供应者

```rust
let id_gen = ArcSupplier::new({
    let mut id = AtomicU64::new(0);
    move || id.fetch_add(1, Ordering::SeqCst)
});

let handles: Vec<_> = (0..10)
    .map(|_| {
        let gen = id_gen.clone();
        std::thread::spawn(move || {
            let mut g = gen;
            g.get()
        })
    })
    .collect();
```

