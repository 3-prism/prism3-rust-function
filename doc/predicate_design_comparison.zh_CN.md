# Predicate 设计方案对比分析

## 概述

本文档详细分析了 Rust 中实现 Predicate（谓词）类型的三种不同设计方案，对比了它们的优缺点、适用场景和实现细节。

Predicate 的核心功能是测试一个值是否满足特定条件，类似于 Java 中的 `Predicate<T>` 接口。在 Rust 中，我们需要在以下几个方面做出权衡：

- **类型表达**：类型别名 vs Struct vs Trait
- **所有权模型**：Box（单一所有权）vs Arc（共享所有权）vs Rc（单线程共享）
- **调用方式**：直接调用 vs 方法调用
- **组合能力**：静态方法 vs 实例方法 vs Trait 方法
- **扩展性**：是否可以添加元数据、实现其他 trait

---

## 方案一：类型别名 + 静态组合方法

### 设计概述

使用类型别名定义 Predicate 类型，并通过静态工具类提供组合方法。这是最简单直接的实现方式。

### 核心设计

```rust
// 类型别名定义
pub type Predicate<T> = Box<dyn Fn(&T) -> bool>;
pub type SharedPredicate<T> = Arc<dyn Fn(&T) -> bool + Send + Sync>;

// 静态组合工具类
pub struct Predicates;

impl Predicates {
    /// 创建 AND 组合
    pub fn and<T, F1, F2>(first: F1, second: F2) -> Predicate<T>
    where
        T: 'static,
        F1: Fn(&T) -> bool + 'static,
        F2: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| first(t) && second(t))
    }

    /// 创建 OR 组合
    pub fn or<T, F1, F2>(first: F1, second: F2) -> Predicate<T>
    where
        T: 'static,
        F1: Fn(&T) -> bool + 'static,
        F2: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| first(t) || second(t))
    }

    /// 创建 NOT 组合
    pub fn not<T, F>(predicate: F) -> Predicate<T>
    where
        T: 'static,
        F: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| !predicate(t))
    }
}

// SharedPredicate 的组合工具类
pub struct SharedPredicates;

impl SharedPredicates {
    pub fn and<T>(first: SharedPredicate<T>, second: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| first(t) && second(t))
    }

    pub fn or<T>(first: SharedPredicate<T>, second: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| first(t) || second(t))
    }

    pub fn not<T>(predicate: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| !predicate(t))
    }
}
```

### 使用示例

```rust
// 创建谓词
let is_positive: Predicate<i32> = Box::new(|x| *x > 0);
let is_even: Predicate<i32> = Box::new(|x| x % 2 == 0);

// 直接调用
assert!(is_positive(&5));
assert!(is_even(&4));

// 组合谓词（传入闭包）
let is_positive_even = Predicates::and(
    |x: &i32| *x > 0,
    |x: &i32| x % 2 == 0,
);

// 直接调用组合后的谓词
assert!(is_positive_even(&4));
assert!(!is_positive_even(&3));

// 复杂组合
let complex = Predicates::or(
    Predicates::and(|x: &i32| *x > 0, |x: &i32| x % 2 == 0),
    |x: &i32| *x > 100,
);
assert!(complex(&4));   // positive and even
assert!(complex(&150)); // large

// 使用 SharedPredicate（可克隆、线程安全）
let shared_pred: SharedPredicate<i32> = Arc::new(|x| *x > 0);
let cloned = Arc::clone(&shared_pred);

// 在多个地方使用
assert!(shared_pred(&5));
assert!(cloned(&10));
```

### 作为函数参数使用

```rust
// 定义接受谓词参数的函数
fn filter_values<T, F>(values: Vec<T>, predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    values.into_iter().filter(|v| predicate(v)).collect()
}

// 使用示例
let values = vec![1, -2, 3, -4, 5];

// 1. 传入闭包
let result = filter_values(values.clone(), |x: &i32| *x > 0);
assert_eq!(result, vec![1, 3, 5]);

// 2. 传入函数指针
fn is_positive(x: &i32) -> bool { *x > 0 }
let result = filter_values(values.clone(), is_positive);
assert_eq!(result, vec![1, 3, 5]);

// 3. 传入 Predicate 对象（注意：会转移所有权）
let pred: Predicate<i32> = Box::new(|x| *x > 0);
let result = filter_values(values.clone(), pred);  // pred 被移动
assert_eq!(result, vec![1, 3, 5]);
// pred 在此处不再可用

// 4. 传入组合后的谓词
let combined = Predicates::and(|x: &i32| *x > 0, |x: &i32| x % 2 == 0);
let result = filter_values(values, combined);
assert_eq!(result, vec![]);

// 注意：由于 Predicate<T> 就是 Box<dyn Fn(&T) -> bool>，
// 它自动实现了 Fn trait，所以可以直接传递
```

### 优点

#### 1. **极简的 API 和使用体验**
- ✅ **直接调用**：`pred(&value)` 而不是 `pred.test(&value)`
- ✅ **零心智负担**：类型别名完全透明，用户可以直接使用 `Box<dyn Fn>`
- ✅ **与标准库完美集成**：可以直接用在 `filter`、`find` 等方法中

```rust
// 在标准库中使用非常自然
let result: Vec<i32> = vec![1, -2, 3, 4]
    .into_iter()
    .filter(|x| pred(x))  // ✅ 直接作为闭包使用
    .collect();
```

#### 2. **完美的泛型支持**
- ✅ **统一的 Fn trait**：闭包、函数指针、Predicate 都通过 `Fn(&T) -> bool` 统一
- ✅ **无需转换**：所有可调用类型都可以直接传入组合方法
- ✅ **类型推断友好**：编译器可以自动推断闭包类型

```rust
// 支持所有可调用类型
let pred1 = Predicates::and(|x| *x > 0, |x| x % 2 == 0);           // 闭包
let pred2 = Predicates::and(is_positive_fn, is_even_fn);           // 函数指针
let pred3 = Predicates::and(pred1, |x| *x < 100);                  // Predicate + 闭包
```

#### 3. **零成本抽象**
- ✅ **单次装箱**：每个闭包只装箱一次
- ✅ **内联优化**：编译器可以优化闭包调用
- ✅ **无额外间接调用**：直接通过 `Box::call()` 调用

#### 4. **实现简单**
- ✅ **代码量少**：无需定义复杂的 struct 或 trait
- ✅ **维护成本低**：类型别名易于理解和维护
- ✅ **文档简洁**：用户只需理解函数签名

#### 5. **Trait Object 天然支持**
```rust
// 可以直接存储在容器中
let predicates: Vec<Predicate<i32>> = vec![
    Box::new(|x| *x > 0),
    Box::new(|x| x % 2 == 0),
];

// 可以作为 trait object 传递
fn use_predicate(pred: &dyn Fn(&i32) -> bool) {
    assert!(pred(&5));
}
```

### 缺点

#### 1. **无法扩展**
- ❌ **不能添加字段**：无法为 Predicate 添加名称、统计信息等元数据
- ❌ **不能实现 trait**：类型别名无法实现 `Display`、`Debug` 等 trait
- ❌ **不能添加方法**：无法为 Predicate 添加实例方法

```rust
// ❌ 无法实现
impl<T> Display for Predicate<T> {  // 编译错误：类型别名无法有 impl
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Predicate")
    }
}
```

#### 2. **类型区分度低**
- ❌ **无法在类型系统层面区分**：`Predicate<T>` 和 `Box<dyn Fn(&T) -> bool>` 完全等价
- ❌ **容易混淆**：用户可能直接使用 `Box::new()` 而不是通过 `Predicates`
- ❌ **语义不够明确**：类型名称不能反映更多信息

#### 3. **两套平行的 API**
- ⚠️ **Predicate vs SharedPredicate**：需要维护两套类型和组合方法
- ⚠️ **命名不够明确**："Shared" 不能明确表达是 Arc 还是 Rc
- ⚠️ **缺少 Rc 支持**：没有为单线程场景提供 Rc 实现

```rust
// 两套平行的 API
struct Predicates;           // 为 Predicate 提供组合方法
struct SharedPredicates;     // 为 SharedPredicate 提供组合方法

// 用户需要记住使用哪个
let pred1 = Predicates::and(...);           // Box 版本
let pred2 = SharedPredicates::and(...);     // Arc 版本
```

#### 4. **无法实现方法链**
- ❌ **只能嵌套调用**：复杂组合时嵌套较深
- ❌ **可读性较差**：多层嵌套不如链式调用清晰

```rust
// 复杂组合需要嵌套
let complex = Predicates::or(
    Predicates::and(
        Predicates::not(is_negative),
        is_even
    ),
    is_large
);

// 无法使用方法链（理想形式）：
// let complex = is_negative.not().and(is_even).or(is_large);
```

### 适用场景

✅ **最适合以下场景：**

1. **简单的谓词组合**：不需要复杂的元数据或方法链
2. **追求极简 API**：希望代码尽可能简洁
3. **与标准库深度集成**：需要在 `filter`、`find` 等方法中直接使用
4. **一次性使用**：谓词创建后不需要多次复用
5. **快速原型开发**：快速实现功能，不考虑长期扩展

❌ **不适合以下场景：**

1. 需要为谓词添加名称、调试信息等元数据
2. 需要实现 `Display`、`Debug` 等 trait
3. 需要复杂的方法链式调用
4. 需要在类型系统层面强制区分不同类型的谓词

---

## 方案二：Struct 封装 + 实例方法

### 设计概述

将 Predicate 定义为 struct，内部包装 `Box<dyn Fn>`，通过实例方法提供组合能力，支持方法链式调用。

### 核心设计

```rust
// Struct 定义
pub struct Predicate<T> {
    inner: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,  // 可以添加元数据
}

impl<T> Predicate<T> {
    /// 创建新的 Predicate
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            inner: Box::new(f),
            name: None,
        }
    }

    /// 添加名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 测试值
    pub fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    /// AND 组合（消耗 self）
    pub fn and<F>(self, other: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| inner(t) && other(t)),
            name: None,
        }
    }

    /// OR 组合（消耗 self）
    pub fn or<F>(self, other: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| inner(t) || other(t)),
            name: None,
        }
    }

    /// NOT 组合（消耗 self）
    pub fn not(self) -> Self
    where
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| !inner(t)),
            name: None,
        }
    }
}

// 实现 Display trait
impl<T> std::fmt::Display for Predicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Predicate({})", self.name.as_deref().unwrap_or("unnamed"))
    }
}

// 实现 Debug trait
impl<T> std::fmt::Debug for Predicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Predicate")
            .field("name", &self.name)
            .finish()
    }
}

// SharedPredicate（基于 Arc）
pub struct SharedPredicate<T> {
    inner: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    name: Option<String>,
}

impl<T> SharedPredicate<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(f),
            name: None,
        }
    }

    pub fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    pub fn and(self, other: Self) -> Self
    where
        T: 'static,
    {
        let first = self.inner;
        let second = other.inner;
        Self {
            inner: Arc::new(move |t| first(t) && second(t)),
            name: None,
        }
    }

    // ... 其他方法类似
}

// 实现 Clone（Arc 可以克隆）
impl<T> Clone for SharedPredicate<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            name: self.name.clone(),
        }
    }
}
```

### 使用示例

```rust
// 创建 Predicate
let pred = Predicate::new(|x: &i32| *x > 0)
    .with_name("is_positive");

// 调用需要使用 .test()
assert!(pred.test(&5));
assert!(!pred.test(&-3));

// 方法链式调用
let complex = Predicate::new(|x: &i32| *x > 0)
    .with_name("positive")
    .and(|x: &i32| x % 2 == 0)
    .or(|x: &i32| *x > 100);

assert!(complex.test(&4));
assert!(complex.test(&150));

// 可以打印和调试
println!("Predicate: {}", pred);
println!("Debug: {:?}", pred);

// SharedPredicate 可以克隆
let shared = SharedPredicate::new(|x: &i32| *x > 0);
let cloned1 = shared.clone();
let cloned2 = shared.clone();

// 可以在多个地方使用
assert!(shared.test(&5));
assert!(cloned1.test(&10));
assert!(cloned2.test(&15));
```

### 作为函数参数使用

方案二需要定义辅助 trait 来统一接受不同类型的参数：

```rust
// 方式 1：定义 Testable trait（推荐）
pub trait Testable<T> {
    fn test(&self, value: &T) -> bool;
}

// 为闭包实现 Testable
impl<T, F> Testable<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }
}

// 为 Predicate 实现 Testable
impl<T> Testable<T> for Predicate<T> {
    fn test(&self, value: &T) -> bool {
        self.test(value)
    }
}

// 定义接受谓词参数的函数
fn filter_values<T, P>(values: Vec<T>, predicate: &P) -> Vec<T>
where
    T: Clone,
    P: Testable<T>,
{
    values.into_iter().filter(|v| predicate.test(v)).collect()
}

// 使用示例
let values = vec![1, -2, 3, -4, 5];

// 1. 传入闭包引用
let closure = |x: &i32| *x > 0;
let result = filter_values(values.clone(), &closure);
assert_eq!(result, vec![1, 3, 5]);

// 2. 传入函数指针
fn is_positive(x: &i32) -> bool { *x > 0 }
let result = filter_values(values.clone(), &is_positive);
assert_eq!(result, vec![1, 3, 5]);

// 3. 传入 Predicate 对象引用
let pred = Predicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &pred);
assert_eq!(result, vec![1, 3, 5]);
// pred 仍然可用（只是借用）

// 4. 传入组合后的谓词
let combined = Predicate::new(|x: &i32| *x > 0).and(|x: &i32| x % 2 == 0);
let result = filter_values(values, &combined);
assert_eq!(result, vec![]);

// 方式 2：使用 Into<Predicate>（有性能问题，不推荐）
impl<T, F> From<F> for Predicate<T>
where
    F: Fn(&T) -> bool + 'static,
{
    fn from(f: F) -> Self {
        Predicate::new(f)
    }
}

fn filter_values_v2<T, P>(values: Vec<T>, predicate: P) -> Vec<T>
where
    T: 'static,
    P: Into<Predicate<T>>,
{
    let pred = predicate.into();
    values.into_iter().filter(|v| pred.test(v)).collect()
}

// 注意：这种方式会导致 Predicate 对象被二次装箱
```

### 优点

#### 1. **优雅的方法链**
- ✅ **流式 API**：`.and().or().not()` 的链式调用更加自然
- ✅ **可读性好**：复杂组合更加清晰易读
- ✅ **符合面向对象习惯**：类似 Java、C# 等语言的风格

```rust
// 方法链比嵌套调用更清晰
let complex = is_positive
    .and(is_even)
    .or(is_large)
    .not();
```

#### 2. **强大的扩展性**
- ✅ **可添加字段**：名称、统计信息、创建时间等
- ✅ **可实现 trait**：Display、Debug、Serialize 等
- ✅ **可添加方法**：任何自定义的实例方法

```rust
pub struct Predicate<T> {
    inner: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,
    call_count: Arc<AtomicUsize>,  // 调用统计
    created_at: SystemTime,        // 创建时间
}

impl<T> Predicate<T> {
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }

    pub fn test(&self, value: &T) -> bool {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        (self.inner)(value)
    }
}
```

#### 3. **类型安全**
- ✅ **独立的类型**：`Predicate<T>` 是明确的类型，不会与 `Box<dyn Fn>` 混淆
- ✅ **更好的类型检查**：编译器可以提供更好的错误信息
- ✅ **类型语义清晰**：类型名称直接反映用途

#### 4. **泛型参数支持**

通过定义统一的 trait 或使用 `Into` 转换，可以支持多种输入类型：

```rust
// 方式 1：使用 Into<Predicate>
impl<T, F> From<F> for Predicate<T>
where
    F: Fn(&T) -> bool + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

pub fn and<T, P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Into<Predicate<T>>,
    P2: Into<Predicate<T>>,
{
    let pred1 = first.into();
    let pred2 = second.into();
    // ... 组合逻辑
}

// 方式 2：定义 Testable trait
pub trait Testable<T> {
    fn test(&self, value: &T) -> bool;
}

impl<T, F> Testable<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }
}

pub fn and<T, P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Testable<T> + 'static,
    P2: Testable<T> + 'static,
{
    Predicate::new(move |t| first.test(t) && second.test(t))
}
```

### 缺点

#### 1. **无法直接调用**
- ❌ **必须使用 `.test()`**：`pred.test(&value)` 而不是 `pred(&value)`
- ❌ **与标准库集成不够自然**：在 `filter` 中需要额外的方法调用
- ❌ **代码略显冗长**：每次调用都多一个 `.test()`

```rust
// 不能直接调用
// assert!(pred(&5));  // ❌ 编译错误

// 必须这样
assert!(pred.test(&5));  // ✅

// 在 filter 中使用
let result: Vec<i32> = vec![1, -2, 3, 4]
    .into_iter()
    .filter(|x| pred.test(x))  // ⚠️ 不如直接调用自然
    .collect();
```

#### 2. **仍需要多个实现**
- ⚠️ **Box 和 Arc 需要分别实现**：`Predicate` 和 `SharedPredicate`
- ⚠️ **代码重复**：`and`、`or`、`not` 等方法需要在两个 struct 中重复实现
- ⚠️ **维护成本增加**：修改一个需要同时修改另一个

```rust
// 需要实现两遍相同的逻辑
impl<T> Predicate<T> {
    pub fn and(self, other: ...) -> Self { /* 实现 */ }
    pub fn or(self, other: ...) -> Self { /* 实现 */ }
    pub fn not(self) -> Self { /* 实现 */ }
}

impl<T> SharedPredicate<T> {
    pub fn and(self, other: ...) -> Self { /* 几乎相同的实现 */ }
    pub fn or(self, other: ...) -> Self { /* 几乎相同的实现 */ }
    pub fn not(self) -> Self { /* 几乎相同的实现 */ }
}
```

#### 3. **所有权问题**
- ⚠️ **方法链消耗 self**：每次调用都会移动所有权
- ⚠️ **无法重用中间结果**：除非实现 Clone（但 Box<dyn Fn> 不能克隆）
- ⚠️ **需要显式克隆 SharedPredicate**：即使是共享所有权，也需要 `.clone()`

```rust
let pred = Predicate::new(|x: &i32| *x > 0);
let combined1 = pred.and(|x: &i32| x % 2 == 0);
// pred 已经被移动，无法再使用

// SharedPredicate 需要显式克隆
let shared = SharedPredicate::new(|x: &i32| *x > 0);
let combined1 = shared.clone().and(...);
let combined2 = shared.clone().or(...);
```

#### 4. **Trait Object 限制**
- ❌ **无法使用 trait object 存储不同类型**：因为 Predicate 是具体的 struct

```rust
// 可以存储同一类型
let predicates: Vec<Predicate<i32>> = vec![
    Predicate::new(|x| *x > 0),
    Predicate::new(|x| x % 2 == 0),
];

// 但无法混合存储（如果想同时存储 Predicate 和 SharedPredicate）
// 需要定义一个统一的 trait
```

#### 5. **潜在的性能问题（取决于实现）**

使用 `Into<Predicate>` 可能导致双重装箱：

```rust
// 如果使用 Into 转换
pub fn and<P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Into<Predicate<T>>,
    P2: Into<Predicate<T>>,
{
    let pred1 = first.into();  // 如果 first 已经是 Predicate，再次装箱
    let pred2 = second.into();
    // 组合时又装箱一次
}
```

### 适用场景

✅ **最适合以下场景：**

1. **需要方法链**：复杂的谓词组合，希望使用流式 API
2. **需要元数据**：为谓词添加名称、统计、调试信息
3. **需要实现 trait**：Display、Debug、Serialize 等
4. **面向对象风格**：团队更熟悉 OOP 风格的 API
5. **类型安全要求高**：希望在类型系统层面区分不同的谓词类型

❌ **不适合以下场景：**

1. 追求极简 API，不需要额外功能
2. 需要直接调用（如 `pred(&value)`）
3. 需要与标准库深度集成
4. 不希望代码中到处都是 `.test()`

---

## 方案三：Trait 抽象 + 多种实现

### 设计概述

这是最灵活和最优雅的方案，也是当前库最终采用的方案。它结合了 trait 的统一抽象能力和 struct 的具体实现能力，达到了语义清晰、类型安全和 API 灵活的平衡。

**核心思想**：
1.  **定义最小化的 `Predicate<T>` Trait**：这个 trait 只包含最核心的 `test(&self, &T) -> bool` 方法和 `into_*` 类型转换方法。它不包含 `and`/`or` 等逻辑组合方法。
2.  **提供三种具体的 Struct 实现**：
    -   `BoxPredicate<T>`：基于 `Box`，用于单一所有权的场景。
    -   `ArcPredicate<T>`：基于 `Arc`，用于线程安全的共享所有权场景。
    -   `RcPredicate<T>`：基于 `Rc`，用于单线程的共享所有权场景。
3.  **在 Struct 上实现特例化的组合方法**：每种 Struct 都实现自己的 `and`/`or`/`not` 等**固有方法**。这使得组合方法可以返回**具体的类型**，从而保持各自的特性（如 `ArcPredicate` 组合后返回的还是 `ArcPredicate`，依然可克隆和线程安全）。
4.  **为闭包提供扩展 Trait**：通过 `FnPredicateOps<T>` 扩展 trait，为所有闭包和函数指针提供 `.and()`、`.or()` 等方法，组合后统一返回 `BoxPredicate<T>`，从而启动方法链。
5.  **统一实现 `Predicate<T>` Trait**：所有闭包和三种 Struct 都实现 `Predicate<T>` Trait，使得它们都可以被泛型函数统一处理。

这种设计将"是什么"（`Predicate` trait）和"怎么做"（各个 Struct 的具体实现）完美分离。

### 核心设计

```rust
// ============================================================================
// 1. 定义最小化的 Predicate trait
// ============================================================================

/// Predicate trait - 统一的谓词接口
///
/// 只定义 test 和 into_* 方法，不包含逻辑组合。
pub trait Predicate<T> {
    /// 测试值是否满足谓词条件
    fn test(&self, value: &T) -> bool;

    /// 转换为 BoxPredicate
    fn into_box(self) -> BoxPredicate<T> where Self: Sized + 'static, T: 'static;

    /// 转换为 RcPredicate
    fn into_rc(self) -> RcPredicate<T> where Self: Sized + 'static, T: 'static;

    /// 转换为 ArcPredicate
    fn into_arc(self) -> ArcPredicate<T> where Self: Sized + Send + Sync + 'static, T: Send + Sync + 'static;
}

// ============================================================================
// 2. 为闭包实现 Predicate trait 和 FnPredicateOps 扩展
// ============================================================================

/// 为所有 Fn(&T) -> bool 实现 Predicate
impl<T, F> Predicate<T> for F where F: Fn(&T) -> bool {
    fn test(&self, value: &T) -> bool { self(value) }
    // ... into_* 方法的实现 ...
}

/// 为闭包提供逻辑组合方法的扩展 trait
pub trait FnPredicateOps<T>: Fn(&T) -> bool + Sized {
    /// AND 组合 - 消耗闭包，返回 BoxPredicate
    fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self(t) && other.test(t))
    }
    // ... or, not 等方法类似, 都返回 BoxPredicate ...
}

/// 为所有闭包类型实现 FnPredicateOps
impl<T, F> FnPredicateOps<T> for F where F: Fn(&T) -> bool {}


// ============================================================================
// 3. BoxPredicate - 单一所有权实现
// ============================================================================

pub struct BoxPredicate<T> { /* ... */ }

impl<T> BoxPredicate<T> {
    /// AND 组合 - 消耗 self，返回 BoxPredicate
    pub fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self.test(t) && other.test(t))
    }
    // ... or, not 等方法类似 ...
}

// ============================================================================
// 4. ArcPredicate - 线程安全的共享所有权实现
// ============================================================================

pub struct ArcPredicate<T> { /* ... */ }

impl<T> ArcPredicate<T> {
    /// AND 组合 - 借用 &self，返回 ArcPredicate
    pub fn and(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate::new(move |t| self_clone.test(t) && other_clone.test(t))
    }
    // ... or, not 等方法类似 ...
}

// ============================================================================
// 5. RcPredicate - 单线程的共享所有权实现
// ============================================================================

pub struct RcPredicate<T> { /* ... */ }

impl<T> RcPredicate<T> {
    /// AND 组合 - 借用 &self，返回 RcPredicate
    pub fn and(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate::new(move |t| self_clone.test(t) && other_clone.test(t))
    }
    // ... or, not 等方法类似 ...
}
```

### 使用示例

```rust
// ============================================================================
// 1. 闭包自动拥有 .test() 和逻辑组合方法
// ============================================================================

let is_positive = |x: &i32| *x > 0;
assert!(is_positive.test(&5)); // 直接使用 .test()

// 闭包使用方法链，返回 BoxPredicate
let positive_even = is_positive.and(|x: &i32| x % 2 == 0); // is_positive 被消耗
assert!(positive_even.test(&4));
// positive_even 是 BoxPredicate，不可克隆

// ============================================================================
// 2. BoxPredicate - 一次性使用场景，消耗 self
// ============================================================================

let pred = BoxPredicate::new(|x: &i32| *x > 0);
let combined = pred.and(|x: &i32| x % 2 == 0); // pred 被消耗
assert!(combined.test(&4));

// ============================================================================
// 3. ArcPredicate - 多线程共享场景，借用 &self
// ============================================================================

let shared = ArcPredicate::new(|x: &i32| *x > 0);

// ✅ 使用方法链组合，不需要显式 clone（使用 &self）
let combined = shared.and(&ArcPredicate::new(|x| x % 2 == 0));

// ✅ shared 仍然可用，可以继续组合
let another_combined = shared.or(&ArcPredicate::new(|x| *x < -100));
assert!(shared.test(&5));

// ✅ 组合结果仍然是 ArcPredicate，可以克隆和跨线程使用
let combined_clone = combined.clone();
use std::thread;
let handle = thread::spawn(move || combined_clone.test(&4));
assert!(handle.join().unwrap());


// ============================================================================
// 4. RcPredicate - 单线程复用场景，借用 &self
// ============================================================================

let rc_pred = RcPredicate::new(|x: &i32| *x > 0);

// ✅ 使用方法链，不需要显式 clone
let combined1 = rc_pred.and(&RcPredicate::new(|x| x % 2 == 0));
let combined2 = rc_pred.or(&RcPredicate::new(|x| *x > 100));

// ✅ 原始谓词仍然可用
assert!(rc_pred.test(&7));

// ============================================================================
// 5. 统一的接口 - 所有类型都实现了 Predicate trait
// ============================================================================

fn use_any_predicate<P: Predicate<i32>>(pred: &P, value: i32) -> bool {
    pred.test(&value)
}

// 所有类型都可以传入
assert!(use_any_predicate(&positive_even, 4));
assert!(use_any_predicate(&shared, 5));
assert!(use_any_predicate(&rc_pred, 6));
assert!(use_any_predicate(&(|x: &i32| *x < 0), -1));
```

### 作为函数参数使用

方案三的统一 trait 接口使得函数参数使用非常自然：

```rust
// 定义接受谓词参数的函数（通过借用）
fn filter_values<T, P>(values: Vec<T>, predicate: &P) -> Vec<T>
where
    T: Clone,
    P: Predicate<T> + ?Sized, // ?Sized 允许传入 trait object
{
    values.into_iter().filter(|v| predicate.test(v)).collect()
}

// 使用示例
let values = vec![1, -2, 3, -4, 5];

// 1. 传入闭包引用
let closure = |x: &i32| *x > 0;
let result = filter_values(values.clone(), &closure);
assert_eq!(result, vec![1, 3, 5]);

// 2. 传入 BoxPredicate 对象引用
let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &box_pred);
assert_eq!(result, vec![1, 3, 5]);

// 3. 传入 ArcPredicate 对象引用
let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &arc_pred);
assert_eq!(result, vec![1, 3, 5]);

// 4. 传入组合后的谓词
let combined = (|x: &i32| *x > 0).and(|x: &i32| x % 2 == 0);
let result = filter_values(values, &combined);
assert_eq!(result, vec![]);
```

### 优点

#### 1. **完美的语义清晰度**

- ✅ **名称即文档**：`BoxPredicate`、`ArcPredicate`、`RcPredicate` 直接表达底层实现和所有权模型。
- ✅ **对称的设计**：三个类型功能对称，易于理解和使用。
- ✅ **与标准库一致**：命名模式与 Rust 标准库的智能指针 `Box`, `Arc`, `Rc` 一致。

#### 2. **统一的 trait 接口**

- ✅ **统一抽象**：所有类型通过 `Predicate<T>` trait 统一，都可以使用 `.test()`。
- ✅ **多态支持**：可以编写接受 `&dyn Predicate<T>` 或 `impl Predicate<T>` 的泛型函数。
- ✅ **闭包自动支持**：所有闭包自动实现 `Predicate<T>`，无需任何转换。

#### 3. **完整的所有权模型覆盖**

三种实现对应三种典型场景：

| 类型 | 所有权 | 克隆 | 线程安全 | API | 适用场景 |
|:---|:---|:---|:---:|:---|:---|
| `BoxPredicate` | 单一 | ❌ | ❌ | `self` | 一次性使用、构建器模式 |
| `ArcPredicate` | 共享 | ✅ | ✅ | `&self` | 多线程共享、配置中心 |
| `RcPredicate` | 共享 | ✅ | ❌ | `&self` | 单线程复用、UI 验证 |

#### 4. **特例化带来的类型保持和优雅的 API**

这是此方案最核心的优势。通过在**具体 Struct 上实现各自的组合方法**，而不是在 Trait 中定义，实现了：

- ✅ **类型保持**：`ArcPredicate` 的组合方法返回的仍然是 `ArcPredicate`，保持了其可克隆和线程安全的特性。`RcPredicate` 同理。
- ✅ **优雅的 API**：`ArcPredicate` 和 `RcPredicate` 的组合方法使用 `&self`，调用时无需显式 `.clone()`，使用体验非常自然，也符合引用计数类型的设计惯例。
- ✅ **无需静态组合方法**：所有操作都通过方法链完成，API 更内聚和简洁。

```rust
// ArcPredicate → ArcPredicate（借用 &self，可重复使用）
let arc_pred = ArcPredicate::new(|x| *x > 0);
let arc_result = arc_pred.and(&another_arc_pred);   // ✅ 不需要 clone，直接使用
let arc_result2 = arc_pred.or(&third_arc_pred);     // ✅ arc_pred 仍然可用
let cloned = arc_result.clone();                    // ✅ 组合结果也可以克隆

// BoxPredicate → BoxPredicate（消耗所有权，使用 self）
let box_pred = BoxPredicate::new(|x| *x > 0);
let box_result = box_pred.and(another);             // ⚠️ box_pred 被移动，不可再用
```

#### 5. **最强的扩展性**

- ✅ **可添加新实现**：未来可以轻松添加新的谓词类型（如 `CowPredicate`）。
- ✅ **可添加字段**：每个实现都可以有自己的元数据（名称、统计等）。
- ✅ **可实现 trait**：`Display`、`Debug`、`Serialize` 等。

#### 6. **与 Rust 标准库设计哲学一致**

该设计模式（一个 trait + 多种 struct 实现）与 Rust 标准库中 `Deref` trait 和 `Box/Rc/Arc` 的关系非常相似，符合 Rust 的设计哲学。

### 缺点

#### 1. **无法直接调用**

与方案二相同，这是最大的使用不便之处。

```rust
let pred = BoxPredicate::new(|x: &i32| *x > 0);

// ❌ 不能直接调用
// assert!(pred(&5));

// ✅ 必须使用 .test()
assert!(pred.test(&5));
```

#### 2. **学习成本略高**

用户需要理解：
- ⚠️ `Predicate` trait 作为统一接口。
- ⚠️ `BoxPredicate`、`ArcPredicate`、`RcPredicate` 三种实现的区别和适用场景。
- ⚠️ 闭包组合默认返回 `BoxPredicate`。
- ⚠️ 为什么 `BoxPredicate` 的组合方法消耗 `self`，而 `Arc/RcPredicate` 使用 `&self`。

**缓解方案**：提供清晰的文档和使用指南（正是本文档的目的）。

#### 3. **实现成本**

- ⚠️ 需要为三个 Struct 分别实现所有的逻辑组合方法（`and`、`or`、`not`、`xor` 等），代码量较大。
- ⚠️ 但由于架构清晰，逻辑重复性强，长期维护成本反而更低。

#### 4. **Trait Object 限制**

`Predicate<T>` trait 本身不是 object-safe 的，因为它的 `into_*` 方法上存在 `where Self: Sized` 约束。这意味着不能创建 `Box<dyn Predicate<T>>`。

```rust
// ❌ 编译错误：trait 不是 object-safe
// let predicates: Vec<Box<dyn Predicate<i32>>> = vec![...];

// ✅ 解决方案：使用具体类型或 Enum 包装器
// 方案 A：使用具体类型
let predicates: Vec<BoxPredicate<i32>> = vec![...];
// 方案 B: 使用 Enum 包装
enum AnyPredicate<T> {
    Box(BoxPredicate<T>),
    Arc(ArcPredicate<T>),
    Rc(RcPredicate<T>),
}
```
在大多数场景下，直接使用 `BoxPredicate` 或 `ArcPredicate` 作为集合的元素类型通常就足够了。

### 适用场景

✅ **最适合以下场景：**

1. **库开发**：为用户提供清晰、灵活、强大的 API。
2. **大型项目**：代码库规模大，需要清晰的架构来保证可维护性。
3. **团队协作**：提供统一的接口规范和清晰的语义。
4. **多场景支持**：同时存在一次性使用、单线程复用、多线程共享等多种场景。

✅ **强烈推荐用于 `prism3-rust-function` 这样的基础库项目。**

---

## 三种方案对比总结

### 核心特性对比表

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 |
|:---|:---|:---|:---|
| **调用方式** | `pred(&x)` ✅ | `pred.test(&x)` ❌ | `pred.test(&x)` ❌ |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 **极好** ✨ |
| **所有权模型** | Box + Arc（两种） | Box + Arc（两种） | Box + Arc + Rc（三种）✅ |
| **类型名称** | Predicate / SharedPredicate | Predicate / SharedPredicate | BoxPredicate / ArcPredicate / RcPredicate ✅ |
| **统一接口** | ❌ 两套独立 API | ❌ 两套独立 struct | ✅ **统一的 Predicate trait** |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ **支持（且类型保持）**✨ |
| **扩展性** | ❌ 无法扩展 | ✅ 可扩展 | ✅ **极易扩展** |
| **元数据支持**| ❌ 不支持 | ✅ 支持 | ✅ 支持 |
| **泛型支持** | ✅ 完美（Fn trait） | 🟡 中等（需额外抽象）| ✅ **完美（Predicate trait）**|
| **代码简洁度** | ✅ 极简 | 🟡 中等 | 🟡 略复杂 |
| **学习成本** | ✅ 最低 | 🟡 中等 | 🟡 略高 |
| **维护成本** | 🟡 中等（两套 API）| 🟡 中等（代码重复）| ✅ **低（架构清晰）**|
| **与标准库一致性**| 🟡 中等 | 🟡 中等 | ✅ **完美** ✨ |

### 使用场景对比

| 场景 | 方案一 | 方案二 | 方案三 |
|:---|:---|:---|:---|
| **快速原型开发** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **简单谓词组合** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **复杂方法链** | ❌ 不适合 | ✅ 适合 | ✅ **最佳** |
| **需要元数据/调试**| ❌ 不支持 | ✅ 支持 | ✅ **最佳** |
| **多线程共享** | ✅ SharedPredicate | ✅ SharedPredicate | ✅ **ArcPredicate** |
| **单线程复用** | ❌ 不支持 | ❌ 不支持 | ✅ **RcPredicate** |
| **库开发** | 🟡 可以 | 🟡 可以 | ✅ **最佳** |
| **大型项目** | 🟡 可以 | ✅ 适合 | ✅ **最佳** |
| **长期维护** | 🟡 中等 | 🟡 中等 | ✅ **最佳** |

---

## 结论

对于 `prism3-rust-function` 这样的库项目，**最终选择方案三是完全正确的**。它提供了无与伦比的语义清晰度、架构灵活性和长期可维护性，完美契合了 Rust 的设计哲学。虽然在实现和学习上有一点成本，但其带来的结构性优势和优雅的 API 设计完全值得这份投入。

