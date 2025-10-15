# Function 设计方案对比分析

## 概述

本文档详细分析了 Rust 中实现 Function（函数）类型的三种不同设计方案，对比了它们的优缺点、适用场景和实现细节。

Function 的核心功能是将一个类型的值转换为另一个类型的值,类似于 Java 中的 `Function<T, R>` 接口。在 Rust 中，我们需要在以下几个方面做出权衡：

- **类型表达**：类型别名 vs Struct vs Trait
- **所有权模型**：Box（单一所有权）vs Arc（共享所有权）vs Rc（单线程共享）
- **调用方式**：直接调用 vs 方法调用
- **组合能力**：静态方法 vs 实例方法 vs Trait 方法
- **可重用性**：FnOnce（一次性）vs Fn（可重复调用）

**特别注意**：Function 与 Predicate 的最大区别在于：
- **消耗性**：Function 通常消耗输入值（接受 `T` 而不是 `&T`），天然适合 `FnOnce`
- **一次性使用**：大多数转换场景都是一次性的，不需要重复调用同一个 Function 实例
- **类型转换**：输入类型 T 和输出类型 R 通常不同，使得设计更加复杂

---

## 方案一：类型别名 + 静态组合方法

### 设计概述

使用类型别名定义 Function 类型，并通过静态工具类提供组合方法。这是最简单直接的实现方式。

### 核心设计

```rust
// 类型别名定义
pub type Function<T, R> = Box<dyn FnOnce(T) -> R>;

// 静态组合工具类
pub struct Functions;

impl Functions {
    /// 创建 Function
    pub fn new<T, R, F>(f: F) -> Function<T, R>
    where
        T: 'static,
        R: 'static,
        F: FnOnce(T) -> R + 'static,
    {
        Box::new(f)
    }

    /// 创建恒等函数
    pub fn identity<T>() -> Function<T, T>
    where
        T: 'static,
    {
        Box::new(|x| x)
    }

    /// 创建常量函数
    pub fn constant<T, R>(value: R) -> Function<T, R>
    where
        T: 'static,
        R: Clone + 'static,
    {
        Box::new(move |_| value.clone())
    }

    /// 组合两个 Function (f . g)
    /// 执行顺序: input -> g -> f -> output
    pub fn compose<T, U, R, F, G>(f: F, g: G) -> Function<T, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
        F: FnOnce(U) -> R + 'static,
        G: FnOnce(T) -> U + 'static,
    {
        Box::new(move |x| f(g(x)))
    }

    /// 链式组合 (and_then)
    /// 执行顺序: input -> first -> second -> output
    pub fn and_then<T, U, R, F, G>(first: F, second: G) -> Function<T, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
        F: FnOnce(T) -> U + 'static,
        G: FnOnce(U) -> R + 'static,
    {
        Box::new(move |x| second(first(x)))
    }
}
```

### 使用示例

```rust
// 创建 Function
let double: Function<i32, i32> = Functions::new(|x| x * 2);
let to_string: Function<i32, String> = Functions::new(|x| x.to_string());

// 直接调用（因为 Box<dyn FnOnce> 实现了 FnOnce）
let result = double(21);
assert_eq!(result, 42);

let result = to_string(42);
assert_eq!(result, "42");

// 注意：调用后 Function 被消耗，无法再次使用
// let again = double(10); // ❌ 编译错误：double 已被移动

// 恒等函数
let identity: Function<i32, i32> = Functions::identity();
assert_eq!(identity(42), 42);

// 常量函数
let always_hello: Function<i32, String> = Functions::constant("hello".to_string());
assert_eq!(always_hello(123), "hello");
assert_eq!(always_hello(456), "hello"); // ❌ 错误！already moved

// 组合函数
let add_one = |x: i32| x + 1;
let double = |x: i32| x * 2;
let composed = Functions::and_then(add_one, double);
assert_eq!(composed(5), 12); // (5 + 1) * 2 = 12

// 复杂组合
let parse_int = |s: String| s.parse::<i32>().unwrap_or(0);
let double = |x: i32| x * 2;
let to_string = |x: i32| x.to_string();

let pipeline = Functions::and_then(
    Functions::and_then(parse_int, double),
    to_string,
);
assert_eq!(pipeline("21".to_string()), "42");
```

### 作为函数参数使用

```rust
// 定义接受 Function 参数的函数
fn transform<T, R, F>(value: T, func: F) -> R
where
    F: FnOnce(T) -> R,
{
    func(value)
}

// 使用示例
let result = transform(21, |x: i32| x * 2);
assert_eq!(result, 42);

// 传入 Function 对象（会转移所有权）
let double: Function<i32, i32> = Functions::new(|x| x * 2);
let result = transform(21, double);
assert_eq!(result, 42);
// double 在此处不再可用

// 对于集合的转换操作
fn map_vec<T, R, F>(vec: Vec<T>, mut func: F) -> Vec<R>
where
    F: FnMut(T) -> R,
{
    vec.into_iter().map(func).collect()
}

let numbers = vec![1, 2, 3];
let result = map_vec(numbers, |x| x * 2);
assert_eq!(result, vec![2, 4, 6]);
```

### 优点

#### 1. **极简的 API 和使用体验**
- ✅ **直接调用**：`func(value)` 而不是 `func.apply(value)`
- ✅ **零心智负担**：类型别名完全透明，用户可以直接使用 `Box<dyn FnOnce>`
- ✅ **与标准库完美集成**：可以直接用在 `map`、`and_then` 等方法中

```rust
// 在标准库中使用非常自然
let result = Some(21)
    .map(|x| x * 2)  // ✅ 直接作为闭包使用
    .map(double_func); // ✅ 也可以传入 Function
```

#### 2. **完美的泛型支持**
- ✅ **统一的 FnOnce trait**：闭包、函数指针、Function 都通过 `FnOnce(T) -> R` 统一
- ✅ **无需转换**：所有可调用类型都可以直接传入组合方法
- ✅ **类型推断友好**：编译器可以自动推断闭包类型

```rust
// 支持所有可调用类型
let func1 = Functions::and_then(|x| x + 1, |x| x * 2);           // 闭包
let func2 = Functions::and_then(add_one_fn, double_fn);          // 函数指针
let func3 = Functions::and_then(func1, |x| x.to_string());       // Function + 闭包
```

#### 3. **零成本抽象**
- ✅ **单次装箱**：每个闭包只装箱一次
- ✅ **内联优化**：编译器可以优化闭包调用
- ✅ **无额外间接调用**：直接通过 `Box::call()` 调用

#### 4. **实现简单**
- ✅ **代码量少**：无需定义复杂的 struct 或 trait
- ✅ **维护成本低**：类型别名易于理解和维护
- ✅ **文档简洁**：用户只需理解函数签名

#### 5. **符合 FnOnce 语义**
- ✅ **天然一次性**：`Box<dyn FnOnce>` 天然只能调用一次，符合大多数转换场景
- ✅ **可捕获所有权**：闭包可以捕获并消耗外部变量
- ✅ **内存效率高**：不需要 Clone，直接移动所有权

### 缺点

#### 1. **无法扩展**
- ❌ **不能添加字段**：无法为 Function 添加名称、统计信息等元数据
- ❌ **不能实现 trait**：类型别名无法实现 `Display`、`Debug` 等 trait
- ❌ **不能添加方法**：无法为 Function 添加实例方法

```rust
// ❌ 无法实现
impl<T, R> Display for Function<T, R> {  // 编译错误：类型别名无法有 impl
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Function")
    }
}
```

#### 2. **一次性使用限制**
- ❌ **无法重复调用**：调用一次后 Function 被消耗
- ❌ **无法克隆**：`Box<dyn FnOnce>` 不实现 Clone
- ❌ **不适合需要多次调用的场景**：如果需要多次使用，只能重新创建

```rust
let double = Functions::new(|x: i32| x * 2);
let r1 = double(21);
// let r2 = double(42); // ❌ 编译错误：double 已被移动
```

#### 3. **类型区分度低**
- ❌ **无法在类型系统层面区分**：`Function<T, R>` 和 `Box<dyn FnOnce(T) -> R>` 完全等价
- ❌ **容易混淆**：用户可能直接使用 `Box::new()` 而不是通过 `Functions`
- ❌ **语义不够明确**：类型名称不能反映更多信息

#### 4. **缺少可重用版本**
- ❌ **只有 FnOnce 版本**：没有提供可重复调用的 `Fn` 版本
- ❌ **无法应对多次调用场景**：某些场景确实需要多次调用同一个转换函数

```rust
// 如果需要多次调用，必须使用闭包而不是 Function
let double_fn = |x: i32| x * 2;
let r1 = double_fn(21);
let r2 = double_fn(42); // ✅ 闭包可以多次调用
```

#### 5. **无法实现方法链**
- ❌ **只能嵌套调用**：复杂组合时嵌套较深
- ❌ **可读性较差**：多层嵌套不如链式调用清晰

```rust
// 复杂组合需要嵌套
let complex = Functions::and_then(
    Functions::and_then(
        parse_string,
        validate
    ),
    transform
);

// 无法使用方法链（理想形式）：
// let complex = parse_string.and_then(validate).and_then(transform);
```

### 适用场景

✅ **最适合以下场景：**

1. **一次性转换**：值转换后不需要再次使用同一个 Function
2. **管道操作**：数据流经一系列转换，每个转换只执行一次
3. **追求极简 API**：希望代码尽可能简洁
4. **与标准库深度集成**：需要在 `map`、`and_then` 等方法中直接使用
5. **快速原型开发**：快速实现功能，不考虑长期扩展

❌ **不适合以下场景：**

1. 需要多次调用同一个转换函数
2. 需要为函数添加名称、调试信息等元数据
3. 需要实现 `Display`、`Debug` 等 trait
4. 需要复杂的方法链式调用
5. 需要克隆函数对象

---

## 方案二：Struct 封装 + 实例方法

### 设计概述

将 Function 定义为 struct，内部包装 `Box<dyn FnOnce>`，通过实例方法提供组合能力，支持方法链式调用。这是当前 `prism3-rust-function` 采用的方案。

### 核心设计

```rust
// Struct 定义
pub struct Function<T, R> {
    f: Box<dyn FnOnce(T) -> R>,
}

impl<T, R> Function<T, R>
where
    T: 'static,
    R: 'static,
{
    /// 创建新的 Function
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        Function { f: Box::new(f) }
    }

    /// 应用函数到输入值
    pub fn apply(self, input: T) -> R {
        (self.f)(input)
    }

    /// 创建恒等函数
    pub fn identity() -> Function<T, T> {
        Function::new(|x| x)
    }

    /// 链式组合 (and_then)
    /// 执行顺序: input -> self -> after -> output
    pub fn and_then<S, G>(self, after: G) -> Function<T, S>
    where
        S: 'static,
        G: FnOnce(R) -> S + 'static,
    {
        Function::new(move |x| after((self.f)(x)))
    }

    /// 反向组合 (compose)
    /// 执行顺序: input -> before -> self -> output
    pub fn compose<S, G>(self, before: G) -> Function<S, R>
    where
        S: 'static,
        G: FnOnce(S) -> T + 'static,
    {
        Function::new(move |x| (self.f)(before(x)))
    }
}

// 可以添加带名称的版本
impl<T, R> Function<T, R>
where
    T: 'static,
    R: 'static,
{
    /// 创建带名称的 Function（用于调试）
    pub fn with_name(self, _name: impl Into<String>) -> Self {
        // 由于 FnOnce 限制，无法存储名称
        // 这只是一个示例，说明 struct 的扩展性
        self
    }
}

impl<T, R> Function<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// 创建常量函数
    pub fn constant(value: R) -> Function<T, R> {
        Function::new(move |_| value.clone())
    }
}

// Option/Result 辅助方法
impl<T, R> Function<Option<T>, Option<R>>
where
    T: 'static,
    R: 'static,
{
    /// 创建 Option 映射函数
    pub fn map_option<F>(f: F) -> Function<Option<T>, Option<R>>
    where
        F: FnOnce(T) -> R + 'static,
    {
        Function::new(move |opt: Option<T>| opt.map(f))
    }
}

impl<T, E, R> Function<Result<T, E>, Result<R, E>>
where
    T: 'static,
    E: 'static,
    R: 'static,
{
    /// 创建 Result 映射函数
    pub fn map_result<F>(f: F) -> Function<Result<T, E>, Result<R, E>>
    where
        F: FnOnce(T) -> R + 'static,
    {
        Function::new(move |result: Result<T, E>| result.map(f))
    }
}
```

### 使用示例

```rust
// 创建 Function
let double = Function::new(|x: i32| x * 2);
let to_string = Function::new(|x: i32| x.to_string());

// 调用需要使用 .apply()
let result = double.apply(21);
assert_eq!(result, 42);

// 方法链式调用（优雅！）
let pipeline = Function::new(|x: i32| x + 1)
    .and_then(|x| x * 2)
    .and_then(|x| x.to_string());

assert_eq!(pipeline.apply(5), "12"); // (5 + 1) * 2 = 12

// 恒等函数
let identity: Function<i32, i32> = Function::identity();
assert_eq!(identity.apply(42), 42);

// 常量函数
let always_hello = Function::constant("hello".to_string());
assert_eq!(always_hello.apply(123), "hello");

// compose（反向组合）
let double = Function::new(|x: i32| x * 2);
let add_one = |x: i32| x + 1;
let composed = double.compose(add_one);
assert_eq!(composed.apply(5), 12); // (5 + 1) * 2 = 12

// Option 映射
let double_opt = Function::map_option(|x: i32| x * 2);
assert_eq!(double_opt.apply(Some(21)), Some(42));
assert_eq!(double_opt.apply(None), None);

// Result 映射
let double_result = Function::map_result(|x: i32| x * 2);
assert_eq!(double_result.apply(Ok::<i32, &str>(21)), Ok(42));
assert_eq!(double_result.apply(Err::<i32, &str>("error")), Err("error"));

// 复杂的类型转换管道
let parse_and_process = Function::new(|s: String| s.parse::<i32>().ok())
    .and_then(|opt: Option<i32>| opt.unwrap_or(0))
    .and_then(|x| x * 2)
    .and_then(|x| format!("Result: {}", x));

assert_eq!(parse_and_process.apply("21".to_string()), "Result: 42");
```

### 作为函数参数使用

```rust
// 方式 1：接受泛型 FnOnce（推荐）
fn transform<T, R, F>(value: T, func: F) -> R
where
    F: FnOnce(T) -> R,
{
    func(value)
}

// 使用示例
let func = Function::new(|x: i32| x * 2);
// 需要先 apply
let result = func.apply(21);
assert_eq!(result, 42);

// 或者传入闭包
let result = transform(21, |x: i32| x * 2);
assert_eq!(result, 42);

// 方式 2：定义 Transformer trait
pub trait Transformer<T, R> {
    fn transform(self, value: T) -> R;
}

impl<T, R> Transformer<T, R> for Function<T, R>
where
    T: 'static,
    R: 'static,
{
    fn transform(self, value: T) -> R {
        self.apply(value)
    }
}

impl<T, R, F> Transformer<T, R> for F
where
    F: FnOnce(T) -> R,
{
    fn transform(self, value: T) -> R {
        self(value)
    }
}

fn transform_with_trait<T, R, F>(value: T, func: F) -> R
where
    F: Transformer<T, R>,
{
    func.transform(value)
}
```

### 优点

#### 1. **优雅的方法链**
- ✅ **流式 API**：`.and_then().compose()` 的链式调用更加自然
- ✅ **可读性好**：复杂组合更加清晰易读
- ✅ **符合面向对象习惯**：类似 Java、C# 等语言的风格

```rust
// 方法链比嵌套调用更清晰
let pipeline = parse_string
    .and_then(validate)
    .and_then(transform)
    .and_then(format_output);
```

#### 2. **强大的扩展性**
- ✅ **可添加辅助方法**：为 Option、Result 等类型提供专门的构造方法
- ✅ **可添加元数据**（理论上）：名称、统计信息等
- ✅ **可实现 trait**：Display、Debug（虽然受 FnOnce 限制）

```rust
// 可以为特定类型组合添加便捷方法
impl<T, E> Function<Result<T, E>, T> {
    pub fn unwrap_or_else<F>(f: F) -> Function<Result<T, E>, T>
    where
        F: FnOnce(E) -> T + 'static,
    {
        Function::new(move |result| result.unwrap_or_else(f))
    }
}
```

#### 3. **类型安全**
- ✅ **独立的类型**：`Function<T, R>` 是明确的类型，不会与 `Box<dyn FnOnce>` 混淆
- ✅ **更好的类型检查**：编译器可以提供更好的错误信息
- ✅ **类型语义清晰**：类型名称直接反映用途

#### 4. **API 一致性**
- ✅ **所有操作都是方法**：无需记忆静态函数的位置
- ✅ **IDE 友好**：自动补全可以列出所有可用方法
- ✅ **统一的调用方式**：`.apply()` 和 `.and_then()` 等方法调用一致

#### 5. **与标准库模式一致**
- ✅ **类似 Iterator**：`and_then` 等方法名与 `Option`、`Result` 一致
- ✅ **符合 Rust 惯例**：方法链式调用是 Rust 的常见模式

### 缺点

#### 1. **无法直接调用**
- ❌ **必须使用 `.apply()`**：`func.apply(value)` 而不是 `func(value)`
- ❌ **与标准库集成不够自然**：在 `map` 中需要额外的方法调用
- ❌ **代码略显冗长**：每次调用都多一个 `.apply()`

```rust
// 不能直接调用
let func = Function::new(|x: i32| x * 2);
// assert_eq!(func(21), 42);  // ❌ 编译错误

// 必须这样
assert_eq!(func.apply(21), 42);  // ✅

// 在 Option 中使用
let result = Some(21)
    .map(|x| x * 2);  // ✅ 闭包可以直接使用
    // .map(func);    // ❌ Function 不能直接用在 map 中
```

#### 2. **仍然是一次性使用**
- ❌ **apply 消耗 self**：调用后 Function 被消耗
- ❌ **无法克隆**：`Box<dyn FnOnce>` 不实现 Clone
- ❌ **不适合需要多次调用的场景**

```rust
let func = Function::new(|x: i32| x * 2);
let r1 = func.apply(21);
// let r2 = func.apply(42); // ❌ 编译错误：func 已被移动
```

#### 3. **FnOnce 的根本限制**
- ❌ **无法重复调用**：这是 `Box<dyn FnOnce>` 的固有限制
- ❌ **难以添加元数据**：由于 FnOnce 消耗 self，很难在调用前后保留元数据
- ❌ **调试困难**：无法实现有意义的 Debug（因为无法检查 FnOnce 的内容）

```rust
// 无法实现真正有用的 Debug
impl<T, R> std::fmt::Debug for Function<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Function<?, ?>") // 只能显示类型，无法显示内容
    }
}
```

#### 4. **组合时的类型复杂度**
- ⚠️ **需要大量泛型约束**：每个组合方法都需要 `'static` 约束
- ⚠️ **编译错误信息复杂**：类型不匹配时错误信息可能很长

```rust
// 泛型约束较多
pub fn and_then<S, G>(self, after: G) -> Function<T, S>
where
    S: 'static,  // 需要 'static
    G: FnOnce(R) -> S + 'static,
{
    // ...
}
```

#### 5. **缺少可重用版本**
- ❌ **只支持 FnOnce**：没有 `Fn` 或 `FnMut` 版本
- ❌ **无法满足多次调用需求**：某些场景确实需要重复使用

### 适用场景

✅ **最适合以下场景：**

1. **数据转换管道**：一系列的类型转换，每个转换只执行一次
2. **构建器模式**：链式构建复杂的转换逻辑
3. **需要方法链**：希望使用流式 API
4. **类型转换明确**：输入和输出类型明确，不需要重复调用
5. **面向对象风格**：团队更熟悉 OOP 风格的 API

✅ **当前 `prism3-rust-function` 的实现正是这个方案，适合其定位。**

❌ **不适合以下场景：**

1. 需要多次调用同一个转换函数
2. 需要直接调用（如 `func(value)`）
3. 需要与标准库深度集成（如直接用在 `map` 中）
4. 需要克隆函数对象

---

## 方案三：Trait 抽象 + 多种实现

### 设计概述

定义 `Function<T, R>` trait 作为统一接口，然后提供多种具体实现：
- `BoxFunction<T, R>`：基于 `Box<dyn FnOnce>`，单一所有权，一次性使用
- `BoxFnFunction<T, R>`：基于 `Box<dyn Fn>`，可重复调用，单一所有权
- `ArcFnFunction<T, R>`：基于 `Arc<dyn Fn + Send + Sync>`，可重复调用，多线程共享
- `RcFnFunction<T, R>`：基于 `Rc<dyn Fn>`，可重复调用，单线程共享

这是最灵活和最全面的方案，类似于 Predicate 的方案三。

### 核心设计

```rust
// ============================================================================
// 1. 定义 Function trait
// ============================================================================

/// Function trait - 统一的函数接口
pub trait Function<T, R> {
    /// 应用函数到输入值
    fn apply(self, input: T) -> R;
}

// ============================================================================
// 2. BoxFunction - 一次性使用，基于 FnOnce
// ============================================================================

pub struct BoxFunction<T, R> {
    f: Box<dyn FnOnce(T) -> R>,
}

impl<T, R> BoxFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxFunction { f: Box::new(f) }
    }

    pub fn identity() -> BoxFunction<T, T> {
        BoxFunction::new(|x| x)
    }

    pub fn and_then<S, G>(self, after: G) -> BoxFunction<T, S>
    where
        S: 'static,
        G: FnOnce(R) -> S + 'static,
    {
        BoxFunction::new(move |x| after((self.f)(x)))
    }

    pub fn compose<S, G>(self, before: G) -> BoxFunction<S, R>
    where
        S: 'static,
        G: FnOnce(S) -> T + 'static,
    {
        BoxFunction::new(move |x| (self.f)(before(x)))
    }
}

impl<T, R> Function<T, R> for BoxFunction<T, R> {
    fn apply(self, input: T) -> R {
        (self.f)(input)
    }
}

// ============================================================================
// 3. BoxFnFunction - 可重复调用，基于 Fn，单一所有权
// ============================================================================

pub struct BoxFnFunction<T, R> {
    f: Box<dyn Fn(T) -> R>,
}

impl<T, R> BoxFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        BoxFnFunction { f: Box::new(f) }
    }

    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    pub fn identity() -> BoxFnFunction<T, T> {
        BoxFnFunction::new(|x| x)
    }

    // 注意：组合方法必须消耗 self（因为 Box<dyn Fn> 不能克隆）
    // 或者返回一个新的函数，捕获 self 的所有权
    pub fn and_then<S>(self, after: BoxFnFunction<R, S>) -> BoxFnFunction<T, S>
    where
        S: 'static,
    {
        BoxFnFunction::new(move |x| after.apply((self.f)(x)))
    }
}

// ============================================================================
// 4. ArcFnFunction - 可重复调用，线程安全，共享所有权
// ============================================================================

pub struct ArcFnFunction<T, R> {
    f: Arc<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> ArcFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        ArcFnFunction { f: Arc::new(f) }
    }

    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    pub fn identity() -> ArcFnFunction<T, T>
    where
        T: Send + Sync,
    {
        ArcFnFunction::new(|x| x)
    }

    // ✅ 可以使用 &self，因为 Arc 可以克隆
    pub fn and_then<S>(&self, after: &ArcFnFunction<R, S>) -> ArcFnFunction<T, S>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcFnFunction {
            f: Arc::new(move |x| after_clone(self_clone(x))),
        }
    }
}

impl<T, R> Clone for ArcFnFunction<T, R> {
    fn clone(&self) -> Self {
        ArcFnFunction {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// 5. RcFnFunction - 可重复调用，单线程，共享所有权
// ============================================================================

pub struct RcFnFunction<T, R> {
    f: Rc<dyn Fn(T) -> R>,
}

impl<T, R> RcFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        RcFnFunction { f: Rc::new(f) }
    }

    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    pub fn identity() -> RcFnFunction<T, T> {
        RcFnFunction::new(|x| x)
    }

    // ✅ 可以使用 &self，因为 Rc 可以克隆
    pub fn and_then<S>(&self, after: &RcFnFunction<R, S>) -> RcFnFunction<T, S>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcFnFunction {
            f: Rc::new(move |x| after_clone(self_clone(x))),
        }
    }
}

impl<T, R> Clone for RcFnFunction<T, R> {
    fn clone(&self) -> Self {
        RcFnFunction {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// 6. 为闭包实现 Function trait 和扩展方法
// ============================================================================

impl<T, R, F> Function<T, R> for F
where
    F: FnOnce(T) -> R,
{
    fn apply(self, input: T) -> R {
        self(input)
    }
}

// 为闭包提供组合方法的扩展 trait
pub trait FnFunctionOps<T, R>: FnOnce(T) -> R + Sized {
    fn and_then<S, G>(self, after: G) -> BoxFunction<T, S>
    where
        Self: 'static,
        G: FnOnce(R) -> S + 'static,
        T: 'static,
        S: 'static,
    {
        BoxFunction::new(move |x| after(self(x)))
    }

    fn compose<S, G>(self, before: G) -> BoxFunction<S, R>
    where
        Self: 'static,
        G: FnOnce(S) -> T + 'static,
        S: 'static,
        R: 'static,
    {
        BoxFunction::new(move |x| self(before(x)))
    }
}

impl<T, R, F> FnFunctionOps<T, R> for F where F: FnOnce(T) -> R {}
```

### 使用示例

```rust
// ============================================================================
// 1. BoxFunction - 一次性使用场景
// ============================================================================

let func = BoxFunction::new(|x: i32| x * 2);
let result = func.apply(21);
assert_eq!(result, 42);
// func 已被消耗，不能再次使用

// 方法链
let pipeline = BoxFunction::new(|x: i32| x + 1)
    .and_then(|x| x * 2)
    .and_then(|x| x.to_string());
assert_eq!(pipeline.apply(5), "12");

// ============================================================================
// 2. BoxFnFunction - 可重复调用，单一所有权
// ============================================================================

let func = BoxFnFunction::new(|x: i32| x * 2);

// ✅ 可以多次调用（使用 &self）
let r1 = func.apply(21);
let r2 = func.apply(42);
assert_eq!(r1, 42);
assert_eq!(r2, 84);

// 但组合会消耗所有权（因为 Box<dyn Fn> 不能克隆）
let func2 = BoxFnFunction::new(|x: i32| x + 1);
let combined = func.and_then(func2); // func 和 func2 都被消耗
let result = combined.apply(5);
assert_eq!(result, 11); // (5 * 2) + 1

// ============================================================================
// 3. ArcFnFunction - 可重复调用，多线程共享
// ============================================================================

let func = ArcFnFunction::new(|x: i32| x * 2);

// ✅ 可以多次调用
let r1 = func.apply(21);
let r2 = func.apply(42);
assert_eq!(r1, 42);
assert_eq!(r2, 84);

// ✅ 可以克隆
let func_clone = func.clone();
assert_eq!(func_clone.apply(10), 20);

// ✅ 组合不消耗所有权（使用 &self）
let func2 = ArcFnFunction::new(|x: i32| x + 1);
let combined = func.and_then(&func2);

// func 和 func2 仍然可用
assert_eq!(func.apply(5), 10);
assert_eq!(func2.apply(5), 6);
assert_eq!(combined.apply(5), 11); // (5 * 2) + 1

// ✅ 可以跨线程使用
use std::thread;
let handle = thread::spawn(move || func_clone.apply(100));
assert_eq!(handle.join().unwrap(), 200);

// ============================================================================
// 4. RcFnFunction - 可重复调用，单线程复用
// ============================================================================

let func = RcFnFunction::new(|x: i32| x * 2);

// ✅ 可以多次调用
let r1 = func.apply(21);
let r2 = func.apply(42);

// ✅ 可以克隆
let func_clone = func.clone();

// ✅ 组合不消耗所有权
let func2 = RcFnFunction::new(|x: i32| x + 1);
let combined = func.and_then(&func2);

// 原始函数仍然可用
assert_eq!(func.apply(5), 10);

// ============================================================================
// 5. 闭包自动拥有组合方法
// ============================================================================

let closure = |x: i32| x + 1;
let pipeline = closure.and_then(|x| x * 2); // 返回 BoxFunction
assert_eq!(pipeline.apply(5), 12);

// ============================================================================
// 6. 统一的接口 - 所有类型都实现了 Function trait
// ============================================================================

fn apply_function<F, T, R>(func: F, value: T) -> R
where
    F: Function<T, R>,
{
    func.apply(value)
}

// 所有类型都可以传入
let box_func = BoxFunction::new(|x: i32| x * 2);
assert_eq!(apply_function(box_func, 21), 42);

let closure = |x: i32| x * 2;
assert_eq!(apply_function(closure, 21), 42);
```

### 作为函数参数使用

```rust
// 统一接口使得参数传递非常灵活

// 1. 接受实现 Function trait 的类型（消耗性）
fn transform<F, T, R>(func: F, value: T) -> R
where
    F: Function<T, R>,
{
    func.apply(value)
}

// 2. 接受可重复调用的函数引用
fn transform_ref<T, R>(func: &dyn Fn(T) -> R, value: T) -> R {
    func(value)
}

// 3. 批量转换
fn batch_transform<T, R>(func: &ArcFnFunction<T, R>, values: Vec<T>) -> Vec<R>
where
    T: Clone,
{
    values.into_iter().map(|v| func.apply(v)).collect()
}

// 使用示例
let arc_func = ArcFnFunction::new(|x: i32| x * 2);
let results = batch_transform(&arc_func, vec![1, 2, 3]);
assert_eq!(results, vec![2, 4, 6]);

// arc_func 仍然可用
assert_eq!(arc_func.apply(10), 20);
```

### 优点

#### 1. **完美的语义清晰度**

- ✅ **名称即文档**：`BoxFunction`（一次性）、`BoxFnFunction`（可重复）、`ArcFnFunction`（线程安全）、`RcFnFunction`（单线程共享）
- ✅ **对称的设计**：四个类型功能对称，易于理解和使用
- ✅ **与标准库一致**：命名模式与 Rust 标准库的智能指针一致

#### 2. **统一的 trait 接口**

- ✅ **统一抽象**：所有类型通过 `Function<T, R>` trait 统一
- ✅ **多态支持**：可以编写接受 `impl Function<T, R>` 的泛型函数
- ✅ **闭包自动支持**：所有闭包自动实现 `Function<T, R>`

#### 3. **完整的使用场景覆盖**

四种实现对应四种典型场景：

| 类型 | 所有权 | 可调用次数 | 克隆 | 线程安全 | API | 适用场景 |
|:---|:---|:---|:---|:---:|:---|:---|
| `BoxFunction` | 单一 | 一次 | ❌ | ❌ | `self` | 一次性转换、管道构建 |
| `BoxFnFunction` | 单一 | 多次 | ❌ | ❌ | `&self` | 本地重复使用 |
| `ArcFnFunction` | 共享 | 多次 | ✅ | ✅ | `&self` | 多线程共享、配置 |
| `RcFnFunction` | 共享 | 多次 | ✅ | ❌ | `&self` | 单线程复用、回调 |

#### 4. **类型保持和优雅的 API**

- ✅ **类型保持**：`ArcFnFunction` 的组合返回 `ArcFnFunction`，保持其特性
- ✅ **优雅的 API**：`ArcFnFunction` 和 `RcFnFunction` 使用 `&self`，无需显式克隆
- ✅ **方法链支持**：所有类型都支持方法链

```rust
// ArcFnFunction 的优雅使用
let func1 = ArcFnFunction::new(|x| x + 1);
let func2 = ArcFnFunction::new(|x| x * 2);

// 使用 &self，不消耗所有权
let combined = func1.and_then(&func2);

// 原始函数仍然可用
assert_eq!(func1.apply(5), 6);
assert_eq!(func2.apply(5), 10);
assert_eq!(combined.apply(5), 12);

// 组合结果也可以继续组合和克隆
let cloned = combined.clone();
```

#### 5. **最强的扩展性**

- ✅ **可添加新实现**：未来可以轻松添加新类型（如 `CowFunction`）
- ✅ **可添加字段**：每个实现可以有自己的元数据
- ✅ **可实现 trait**：`Clone`、`Send`、`Sync` 等

#### 6. **满足所有使用场景**

- ✅ **一次性转换**：`BoxFunction`
- ✅ **重复调用**：`BoxFnFunction`、`ArcFnFunction`、`RcFnFunction`
- ✅ **多线程共享**：`ArcFnFunction`
- ✅ **单线程复用**：`RcFnFunction`

### 缺点

#### 1. **仍然无法直接调用**

```rust
let func = BoxFunction::new(|x: i32| x * 2);
// assert_eq!(func(21), 42); // ❌ 不能直接调用
assert_eq!(func.apply(21), 42); // ✅ 必须使用 .apply()
```

#### 2. **学习成本最高**

用户需要理解：
- ⚠️ 四种不同的实现及其区别
- ⚠️ `FnOnce` vs `Fn` 的区别
- ⚠️ 何时使用哪种类型
- ⚠️ 为什么 `BoxFunction` 消耗 `self` 而 `ArcFnFunction` 使用 `&self`

#### 3. **实现成本最高**

- ⚠️ 需要为四个 Struct 分别实现所有方法
- ⚠️ 代码量最大
- ⚠️ 但架构清晰，长期维护成本低

#### 4. **类型选择复杂**

用户需要决策：
- 是否需要重复调用？→ `BoxFunction` vs `BoxFnFunction/ArcFnFunction/RcFnFunction`
- 是否需要跨线程？→ `ArcFnFunction` vs `RcFnFunction`
- 是否需要克隆？→ 影响组合方式

#### 5. **Fn 的输入限制**

`Fn` trait 要求参数是借用的（`&T`），但 Function 的语义通常是消耗输入（`T`）。这可能导致：
- ⚠️ 如果 `T` 不是 `Copy`，`Fn(T) -> R` 实际上只能调用一次
- ⚠️ 需要用户理解 `T` 和 `&T` 的区别

```rust
// 对于 Copy 类型，没有问题
let func = BoxFnFunction::new(|x: i32| x * 2);
let r1 = func.apply(21); // ✅ i32 是 Copy
let r2 = func.apply(42); // ✅

// 对于非 Copy 类型，会遇到问题
let func = BoxFnFunction::new(|s: String| s.len());
let s1 = "hello".to_string();
let r1 = func.apply(s1); // ✅ s1 被移动
// 如果想再次调用，需要新的 String
let s2 = "world".to_string();
let r2 = func.apply(s2); // ✅ 但每次都要创建新的输入
```

### 适用场景

✅ **最适合以下场景：**

1. **库开发**：为用户提供清晰、灵活、强大的 API
2. **大型项目**：需要清晰的架构和全面的功能覆盖
3. **多样化需求**：同时存在一次性使用、重复调用、多线程共享等场景
4. **配置和回调系统**：需要存储和重复使用转换函数
5. **长期维护**：项目规模大，需要清晰的语义和易扩展的架构

✅ **如果 `prism3-rust-function` 需要支持可重复调用的场景，这是最佳方案。**

❌ **不适合以下场景：**

1. 只需要一次性转换，不需要重复调用
2. 追求极简 API
3. 快速原型开发
4. 团队对 Rust 不够熟悉

---

## 三种方案对比总结

### 核心特性对比表

| 特性 | 方案一：类型别名 | 方案二：Struct 封装 | 方案三：Trait + 多实现 |
|:---|:---|:---|:---|
| **调用方式** | `func(x)` ✅ | `func.apply(x)` ❌ | `func.apply(x)` ❌ |
| **可重复调用** | ❌ FnOnce 限制 | ❌ FnOnce 限制 | ✅ **提供 Fn 版本** ✨ |
| **方法链** | ❌ 只能嵌套 | ✅ 支持 | ✅ 支持 |
| **所有权模型** | Box（一种） | Box（一种） | Box + Arc + Rc（三种）✅ |
| **语义清晰度** | 🟡 中等 | 🟢 好 | 🟢 **极好** ✨ |
| **统一接口** | ❌ 无 trait | ❌ 单一 struct | ✅ **统一 trait** |
| **扩展性** | ❌ 无法扩展 | ✅ 可扩展 | ✅ **极易扩展** |
| **泛型支持** | ✅ 完美（FnOnce trait）| 🟡 中等 | ✅ **完美（Function trait）**|
| **代码简洁度** | ✅ 极简 | 🟡 中等 | 🟡 略复杂 |
| **学习成本** | ✅ 最低 | 🟡 中等 | 🟡 最高 |
| **实现成本** | ✅ 最低 | 🟡 中等 | 🟡 最高 |
| **维护成本** | ✅ 低（但功能有限）| 🟡 中等 | ✅ **低（架构清晰）**|
| **多线程支持** | ❌ 不支持 | ❌ 不支持 | ✅ **ArcFnFunction** |
| **克隆支持** | ❌ 不支持 | ❌ 不支持 | ✅ **Arc/Rc 版本** |

### 使用场景对比

| 场景 | 方案一 | 方案二 | 方案三 |
|:---|:---|:---|:---|
| **一次性转换** | ✅ 最佳 | ✅ 适合 | ✅ BoxFunction |
| **重复调用** | ❌ 不支持 | ❌ 不支持 | ✅ **最佳** |
| **方法链** | ❌ 不适合 | ✅ 适合 | ✅ **最佳** |
| **多线程共享** | ❌ 不支持 | ❌ 不支持 | ✅ **ArcFnFunction** |
| **单线程复用** | ❌ 不支持 | ❌ 不支持 | ✅ **RcFnFunction** |
| **快速原型** | ✅ 最佳 | 🟡 可以 | 🟡 可以 |
| **库开发** | 🟡 可以 | ✅ **当前选择** | ✅ **全面覆盖** |
| **大型项目** | 🟡 功能有限 | ✅ 适合 | ✅ **最佳** |

### FnOnce vs Fn 的权衡

| 特性 | FnOnce 版本 | Fn 版本 |
|:---|:---|:---|
| **适用场景** | 一次性转换 | 重复调用 |
| **输入值** | 消耗 T | 消耗 T（但可多次提供）|
| **性能** | 最优（无克隆开销）| 可能需要克隆输入 |
| **灵活性** | 可捕获非 Copy 值 | 闭包捕获的值必须可多次借用 |
| **组合难度** | 简单 | 稍复杂（需要保留所有权）|

---

## 结论

### 当前实现评估

`prism3-rust-function` 当前采用的是**方案二：Struct 封装 + 实例方法**，这是一个**非常合理的选择**，因为：

✅ **优势：**
1. 提供了优雅的方法链 API
2. 类型安全，语义清晰
3. 符合大多数一次性转换的场景
4. 与 Java 的 `Function<T, R>` 语义一致
5. 实现复杂度适中

⚠️ **局限：**
1. 只支持 FnOnce，无法重复调用
2. 无法克隆，无法跨线程共享
3. 只有 Box 一种所有权模型

### 升级建议

如果未来需要支持更多场景（重复调用、多线程共享等），建议：

**选项 A：保持当前方案**
- 适合：如果 Function 的主要使用场景确实是一次性转换
- 优点：简单、清晰、够用
- 建议：在文档中明确说明 Function 是一次性使用的

**选项 B：渐进式升级到方案三**
- 第一步：保留当前 `Function<T, R>` 作为 `BoxFunction` 的别名
- 第二步：添加 `ArcFnFunction` 和 `RcFnFunction` 用于重复调用场景
- 第三步：定义统一的 `Transformer<T, R>` trait
- 优点：向后兼容，功能全面
- 适合：库的长期演进

**选项 C：简化为方案一**
- 如果发现大多数用户直接使用闭包，不需要方法链
- 优点：最简单，与标准库集成最好
- 缺点：失去方法链和扩展性

### 最终建议

对于 `prism3-rust-function` 这样的库项目：

1. **短期**：保持方案二，完善文档，明确其一次性使用的特点
2. **中期**：如果用户反馈需要重复调用，考虑添加 `ArcFnFunction` 等类型
3. **长期**：如果库的使用场景足够多样化，升级到方案三的完整架构

**核心原则**：先满足主要场景（一次性转换），再根据实际需求逐步扩展，避免过度设计。

