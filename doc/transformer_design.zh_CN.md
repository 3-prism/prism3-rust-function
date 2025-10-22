# Transformer 设计方案分析

## 概述

本文档从 Transformer（转换器）的本质语义出发，分析其主要用途和核心价值，探讨合理的设计方案。

Transformer 的核心功能是**将一个类型的值转换为另一个类型的值**，类似于 Java 中的 `Function<T, R>` 接口和 Rust 标准库中的 `Fn(T) -> R`。本文将深入分析 Transformer 的设计决策，并提出符合实际业务需求的解决方案。

---

## 一、Transformer 的本质语义

### 1.1 什么是 Transformer？

**Transformer（转换器）的核心语义**：

> **将一个类型的值转换为另一个类型的值。这是一个"转换"操作，消耗输入产生输出，通常应该是纯函数（无副作用）。**

这类似于数学中的函数映射：
- ✅ **类型转换**：从一个类型映射到另一个类型
- ✅ **值消耗**：转换过程中消耗输入值的所有权
- ✅ **纯函数性**：相同输入应产生相同输出（从用户角度）
- ✅ **无副作用**：不修改外部状态（或通过内部可变性隐藏）

**对比其他函数式抽象**：

| 类型 | 输入 | 输出 | 修改输入？ | 修改自己？ | 典型用途 |
|------|------|------|-----------|-----------|---------|
| **Transformer** | `T` | `R` | ❌ | ❌ | 类型转换、映射、计算 |
| **Predicate** | `&T` | `bool` | ❌ | ❌ | 过滤、验证、判断 |
| **Consumer** | `&T` | `()` | ❌ | ✅ | 观察、日志、统计 |
| **Supplier** | 无 | `T` | N/A | ✅ | 工厂、生成器 |

**关键洞察**：
- Transformer 的输入是 `T`（所有权转移），而非 `&T`（借用）
- Transformer 应该是"纯函数"，不应该修改自己的状态
- 如果需要状态（如缓存），使用内部可变性

### 1.2 Transformer 的主要用途

| 用途 | 描述 | 示例 |
|------|------|------|
| **类型转换** | 将一个类型转换为另一个类型 | `String -> i32`, `Vec<u8> -> String` |
| **数据映射** | 配合 `map()` 等迭代器方法 | `vec.into_iter().map(transformer)` |
| **管道处理** | 构建数据处理管道 | `parse.and_then(validate).and_then(transform)` |
| **策略模式** | 将转换逻辑作为策略保存 | `transformers.insert("json", parser)` |
| **延迟计算** | 保存转换逻辑，稍后执行 | `let result = transformer.apply(input)` |

### 1.3 Transformer 的核心价值

**临时转换 vs 保存逻辑**：

```rust
// ❌ 不需要 Transformer：临时转换一次
let result = input.to_string();

// ✅ 需要 Transformer：保存转换逻辑以便复用
let to_string = BoxTransformer::new(|x: i32| x.to_string());
let result1 = values1.into_iter().map(|x| to_string.apply(x));
let result2 = values2.into_iter().map(|x| to_string.apply(x));
```

**Transformer 的价值在于**：
1. **保存转换逻辑**：将转换操作封装为可复用的对象
2. **延迟执行**：在需要的时候才执行转换
3. **逻辑组合**：通过 `and_then`、`compose` 构建复杂转换
4. **简化接口**：作为类型约束提高代码可读性

---

## 二、核心设计决策

### 2.1 输入参数：T vs &T？

这是 Transformer 设计中最关键的问题。

#### 方案 A：接受所有权 `T`（推荐）

```rust
pub trait Transformer<T, R> {
    fn transform(&self, input: T) -> R;  // 消耗输入的所有权
}

// 使用场景：类型转换
let parse = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));
let result = parse.apply("42".to_string());  // String 被消耗
```

**优点**：
- ✅ **符合转换语义**：转换通常消耗输入（如 `String` 转为 `Vec<u8>`）
- ✅ **与标准库一致**：`Option::map(FnOnce(T) -> U)` 消耗 T
- ✅ **最大灵活性**：可以移动输入，避免不必要的克隆
- ✅ **语义清晰**：Transformer 是"消耗并转换"

**缺点**：
- ⚠️ **每次调用需要新输入**：如果想复用输入，需要克隆

#### 方案 B：接受引用 `&T`

```rust
pub trait Transformer<T, R> {
    fn transform(&self, input: &T) -> R;  // 借用输入
}

// 使用场景：非破坏性转换
let length = BoxTransformer::new(|s: &String| s.len());
let s = "hello".to_string();
let len1 = length.apply(&s);
let len2 = length.apply(&s);  // 可以多次使用同一个输入
```

**优点**：
- ✅ **可重复使用输入**：同一个输入可以传给多个 Transformer
- ✅ **避免克隆**：不需要克隆输入值

**缺点**：
- ❌ **不符合转换语义**：真正的类型转换通常需要消耗输入
- ❌ **与标准库不一致**：Rust 标准库的 `map` 消耗输入
- ❌ **限制灵活性**：无法在闭包中获取输入的所有权

#### 推荐方案：使用 `T`（消耗输入）

**理由**：
1. **符合 Transformer 的本质**：转换器就是将输入"转换"为输出
2. **与 Rust 标准库一致**：`Iterator::map`、`Option::map` 都消耗输入
3. **最大灵活性**：用户可以选择移动或克隆
4. **语义明确**：Transformer 是"转换器"，而非"计算器"

**如果需要借用输入的场景**：
- 使用闭包时在内部借用：`BoxTransformer::new(|x: String| x.len())`
- 或者传入引用类型：`BoxTransformer::new(|s: &str| s.len())`

### 2.2 self 的可变性：&self vs &mut self？

Transformer 应该是纯函数，不应该修改自己的状态。

```rust
// ✅ 推荐：使用 &self
pub trait Transformer<T, R> {
    fn transform(&self, input: T) -> R;  // 不修改自己
}

// ❌ 不推荐：使用 &mut self
pub trait TransformerMut<T, R> {
    fn transform(&mut self, input: T) -> R;  // 可修改自己
}
```

**为什么不需要 TransformerMut？**

与 Predicate 的分析一致，内部可变性足以解决所有"需要状态"的场景：

```rust
// 场景：缓存转换结果
use std::cell::RefCell;
use std::collections::HashMap;

let cache = RefCell::new(HashMap::new());
let cached_parse = BoxTransformer::new(move |s: String| {
    let mut cache = cache.borrow_mut();
    if let Some(&result) = cache.get(&s) {
        result
    } else {
        let result = s.parse::<i32>().unwrap_or(0);
        cache.insert(s, result);
        result
    }
});

// 用户不需要 mut
cached_parse.apply("42".to_string());
```

**为什么内部可变性更好？**

| 特性 | TransformerMut (`&mut self`) | Transformer + RefCell (`&self`) |
|------|------------------------------|----------------------------------|
| **用户代码** | `let mut transformer = ...` | `let transformer = ...` |
| **调用方式** | `transformer.transform_mut(x)` | `transformer.apply(x)` |
| **语义** | "这个转换器会改变" ❌ | "这是一个纯转换"（内部优化）✅ |
| **灵活性** | 不能在不可变上下文使用 | 可以在任何地方使用 |
| **实现复杂度** | 需要额外的 trait | 统一使用 Transformer |

**结论**：只提供 `Transformer<T, R>` (使用 `&self`)，不需要 `TransformerMut`。

### 2.3 为什么需要 TransformerOnce？✅

与 ConsumerOnce、SupplierOnce 类似，TransformerOnce 的价值在于：

1. **保存 FnOnce 闭包**：闭包可以移动捕获的变量
2. **一次性转换**：某些转换操作本身就是一次性的
3. **延迟执行**：保存转换逻辑，稍后执行一次

```rust
pub trait TransformerOnce<T, R> {
    fn transform(self, input: T) -> R;  // 消费 self 和 input
}

// 使用场景 1：捕获只能移动一次的资源
let resource = acquire_expensive_resource();
let transformer = BoxTransformerOnce::new(move |input: String| {
    // 使用 resource 和 input 进行转换
    process_with_resource(resource, input)
});
let result = transformer.apply("data".to_string());  // transformer 被消耗

// 使用场景 2：延迟初始化转换
struct Processor {
    initializer: Option<BoxTransformerOnce<Config, Processor>>,
}

impl Processor {
    fn initialize(mut self, config: Config) -> Processor {
        if let Some(init) = self.initializer.take() {
            init.apply(config)
        } else {
            self
        }
    }
}
```

**TransformerOnce vs Transformer**：

| | Transformer | TransformerOnce |
|---|---|---|
| **self 签名** | `&self` | `self` |
| **可调用次数** | 多次 | 一次 |
| **闭包类型** | `Fn(T) -> R` | `FnOnce(T) -> R` |
| **适用场景** | 可复用的转换 | 一次性转换、延迟计算 |

**结论**：TransformerOnce 是**必要的**，与 Transformer 形成互补。

### 2.4 输出参数：R vs &R？

用户建议"输出应该是具体的值而非引用"，这是完全正确的。

```rust
// ✅ 推荐：返回所有权
pub trait Transformer<T, R> {
    fn transform(&self, input: T) -> R;  // 返回值的所有权
}

// ❌ 不推荐：返回引用（会有生命周期问题）
pub trait RefTransformer<'a, T, R> {
    fn transform(&'a self, input: T) -> &'a R;  // 生命周期复杂
}
```

**为什么返回 `R`？**

1. **避免生命周期问题**：返回引用会引入复杂的生命周期约束
2. **符合转换语义**：Transformer 生成新值，而非返回已有值的引用
3. **灵活性**：用户可以选择返回 `Arc<T>`、`Rc<T>` 等智能指针
4. **与标准库一致**：`Option::map` 返回值而非引用

### 2.5 简化后的核心设计

基于以上分析，Transformer 模块只需要：

```rust
/// 转换器 - 将输入转换为输出
pub trait Transformer<T, R> {
    /// 转换输入，产生输出
    ///
    /// 使用 &self，可以多次调用（但每次需要新的输入）。
    /// 如果需要内部状态（如缓存），使用 RefCell、Cell 或 Mutex。
    fn transform(&self, input: T) -> R;

    // 类型转换方法
    fn into_box(self) -> BoxTransformer<T, R> where ...;
    fn into_rc(self) -> RcTransformer<T, R> where ...;
    fn into_arc(self) -> ArcTransformer<T, R> where ...;

    /// 转换为标准闭包，用于与标准库集成
    fn into_fn(self) -> impl Fn(T) -> R where ...;
}

/// 一次性转换器 - 只能调用一次
pub trait TransformerOnce<T, R> {
    /// 转换输入，消耗 self
    fn transform(self, input: T) -> R;

    fn into_box(self) -> BoxTransformerOnce<T, R> where ...;

    /// 转换为标准闭包，用于与标准库集成
    fn into_fn(self) -> impl FnOnce(T) -> R where ...;
}
```

**就这两个 trait！** 简单、清晰、符合语义。

---

## 三、实现方案：Trait 抽象 + 多种实现（推荐）

参考 Consumer、Supplier、Predicate 的设计，采用统一的 Trait + 多种实现方案。

### 3.1 核心架构

```rust
// ============================================================================
// 1. 最小化的 Transformer trait
// ============================================================================

/// 转换器 - 将输入转换为输出（可重复调用）
pub trait Transformer<T, R> {
    /// 转换输入值
    fn transform(&self, input: T) -> R;

    // 类型转换方法
    fn into_box(self) -> BoxTransformer<T, R> where Self: Sized + 'static, T: 'static, R: 'static;
    fn into_rc(self) -> RcTransformer<T, R> where Self: Sized + 'static, T: 'static, R: 'static;
    fn into_arc(self) -> ArcTransformer<T, R>
        where Self: Sized + Send + Sync + 'static, T: Send + 'static, R: Send + 'static;

    /// 转换为标准闭包，便于与标准库集成
    fn into_fn(self) -> impl Fn(T) -> R
        where Self: Sized + 'static, T: 'static, R: 'static;
}

/// 一次性转换器 - 只能调用一次
pub trait TransformerOnce<T, R> {
    /// 转换输入值（消耗 self）
    fn transform(self, input: T) -> R;

    fn into_box(self) -> BoxTransformerOnce<T, R> where Self: Sized + 'static, T: 'static, R: 'static;

    /// 转换为标准闭包，便于与标准库集成
    fn into_fn(self) -> impl FnOnce(T) -> R
        where Self: Sized + 'static, T: 'static, R: 'static;
}

// ============================================================================
// 2. 为闭包提供扩展能力
// ============================================================================

/// 为闭包实现 Transformer trait
impl<T, R, F> Transformer<T, R> for F
where
    F: Fn(T) -> R
{
    fn transform(&self, input: T) -> R {
        self(input)
    }
    // ... into_* 实现
}

/// 为闭包提供组合方法的扩展 trait
pub trait FnTransformerOps<T, R>: Fn(T) -> R + Sized {
    /// 链式组合：self -> after
    fn and_then<S, G>(self, after: G) -> BoxTransformer<T, S>
    where
        G: Fn(R) -> S + 'static,
        T: 'static,
        S: 'static;

    /// 反向组合：before -> self
    fn compose<S, G>(self, before: G) -> BoxTransformer<S, R>
    where
        G: Fn(S) -> T + 'static,
        S: 'static,
        R: 'static;
}

// ============================================================================
// 3. BoxTransformer - 单一所有权，可重复调用
// ============================================================================

pub struct BoxTransformer<T, R> {
    function: Box<dyn Fn(T) -> R>,
}

impl<T, R> BoxTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static
    {
        BoxTransformer { function: Box::new(f) }
    }

    /// 恒等转换
    pub fn identity() -> BoxTransformer<T, T> {
        BoxTransformer::new(|x| x)
    }

    /// 常量转换
    pub fn constant(value: R) -> BoxTransformer<T, R>
    where
        R: Clone
    {
        BoxTransformer::new(move |_| value.clone())
    }

    /// 链式组合：self -> after
    /// 消耗 self，返回新的 BoxTransformer
    pub fn and_then<S, G>(self, after: G) -> BoxTransformer<T, S>
    where
        G: Transformer<R, S> + 'static,
        S: 'static,
    {
        let func = self.function;
        BoxTransformer::new(move |x| after.apply(func(x)))
    }

    /// 反向组合：before -> self
    pub fn compose<S, G>(self, before: G) -> BoxTransformer<S, R>
    where
        G: Transformer<S, T> + 'static,
        S: 'static,
    {
        let func = self.function;
        BoxTransformer::new(move |x| func(before.apply(x)))
    }
}

impl<T, R> Transformer<T, R> for BoxTransformer<T, R> {
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }
    // ... into_* 实现
}

// ============================================================================
// 4. BoxTransformerOnce - 一次性转换器
// ============================================================================

pub struct BoxTransformerOnce<T, R> {
    function: Option<Box<dyn FnOnce(T) -> R>>,
}

impl<T, R> BoxTransformerOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static
    {
        BoxTransformerOnce {
            function: Some(Box::new(f))
        }
    }

    /// 链式组合：self -> after
    pub fn and_then<S, G>(self, after: G) -> BoxTransformerOnce<T, S>
    where
        G: TransformerOnce<R, S> + 'static,
        S: 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = self.apply(x);
            after.apply(intermediate)
        })
    }

    /// 反向组合：before -> self
    pub fn compose<S, G>(self, before: G) -> BoxTransformerOnce<S, R>
    where
        G: TransformerOnce<S, T> + 'static,
        S: 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = before.apply(x);
            self.apply(intermediate)
        })
    }
}

impl<T, R> TransformerOnce<T, R> for BoxTransformerOnce<T, R> {
    fn transform(mut self, input: T) -> R {
        (self.function.take().unwrap())(input)
    }
}

// ============================================================================
// 5. ArcTransformer - 线程安全的共享所有权
// ============================================================================

pub struct ArcTransformer<T, R> {
    function: Arc<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> ArcTransformer<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static
    {
        ArcTransformer {
            function: Arc::new(f)
        }
    }

    /// 恒等转换
    pub fn identity() -> ArcTransformer<T, T>
    where
        T: Send + Sync
    {
        ArcTransformer::new(|x| x)
    }

    /// 链式组合：self -> after
    /// 借用 &self，返回新的 ArcTransformer
    pub fn and_then<S, F>(&self, after: F) -> ArcTransformer<T, S>
    where
        S: Send + Sync + 'static,
        F: Transformer<R, S> + Send + Sync + 'static,
    {
        let self_func = Arc::clone(&self.function);
        ArcTransformer {
            function: Arc::new(move |x| after.apply(self_func(x))),
        }
    }

    /// 反向组合：before -> self
    pub fn compose<S, F>(&self, before: F) -> ArcTransformer<S, R>
    where
        S: Send + Sync + 'static,
        F: Transformer<S, T> + Send + Sync + 'static,
    {
        let self_func = Arc::clone(&self.function);
        ArcTransformer {
            function: Arc::new(move |x| self_func(before.apply(x))),
        }
    }
}

impl<T, R> Transformer<T, R> for ArcTransformer<T, R>
where
    T: Send,
    R: Send,
{
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }
    // ... into_* 实现
}

impl<T, R> Clone for ArcTransformer<T, R> {
    fn clone(&self) -> Self {
        ArcTransformer {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// 6. RcTransformer - 单线程的共享所有权
// ============================================================================

pub struct RcTransformer<T, R> {
    function: Rc<dyn Fn(T) -> R>,
}

impl<T, R> RcTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static
    {
        RcTransformer {
            function: Rc::new(f)
        }
    }

    /// 恒等转换
    pub fn identity() -> RcTransformer<T, T> {
        RcTransformer::new(|x| x)
    }

    /// 链式组合：self -> after
    /// 借用 &self，返回新的 RcTransformer
    pub fn and_then<S, F>(&self, after: F) -> RcTransformer<T, S>
    where
        S: 'static,
        F: Transformer<R, S> + 'static,
    {
        let self_func = Rc::clone(&self.function);
        RcTransformer {
            function: Rc::new(move |x| after.apply(self_func(x))),
        }
    }

    /// 反向组合：before -> self
    pub fn compose<S, F>(&self, before: F) -> RcTransformer<S, R>
    where
        S: 'static,
        F: Transformer<S, T> + 'static,
    {
        let self_func = Rc::clone(&self.function);
        RcTransformer {
            function: Rc::new(move |x| self_func(before.apply(x))),
        }
    }
}

impl<T, R> Transformer<T, R> for RcTransformer<T, R> {
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }
    // ... into_* 实现
}

impl<T, R> Clone for RcTransformer<T, R> {
    fn clone(&self) -> Self {
        RcTransformer {
            function: Rc::clone(&self.function),
        }
    }
}
```

### 3.2 使用示例

```rust
// ============================================================================
// 1. 闭包自动拥有 Transformer 能力
// ============================================================================

let double = |x: i32| x * 2;
assert_eq!(double.apply(21), 42);  // 闭包自动实现 Transformer

// 闭包可以直接组合
let add_one = |x: i32| x + 1;
let pipeline = double.and_then(add_one);  // 返回 BoxTransformer
assert_eq!(pipeline.apply(5), 11);  // (5 * 2) + 1

// ============================================================================
// 2. BoxTransformer - 可重复调用，单一所有权
// ============================================================================

let parse = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));

// ✅ 可以多次调用（每次需要新的输入）
assert_eq!(parse.apply("42".to_string()), 42);
assert_eq!(parse.apply("100".to_string()), 100);

// 方法链
let pipeline = BoxTransformer::new(|s: String| s.len())
    .and_then(|len| len * 2)
    .and_then(|x| format!("Length: {}", x));

assert_eq!(pipeline.apply("hello".to_string()), "Length: 10");

// ============================================================================
// 3. BoxTransformerOnce - 一次性使用
// ============================================================================

// 捕获只能移动一次的资源
let resource = vec![1, 2, 3];
let transformer = BoxTransformerOnce::new(move |multiplier: i32| {
    resource.into_iter().map(|x| x * multiplier).collect::<Vec<_>>()
});

let result = transformer.apply(10);
assert_eq!(result, vec![10, 20, 30]);
// transformer 已被消耗，不能再次使用

// ============================================================================
// 4. ArcTransformer - 多线程共享，不消耗所有权
// ============================================================================

let parse = ArcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));

// ✅ 可以克隆
let parse_clone = parse.clone();

// ✅ 组合时消耗参数但不消耗 self（使用 &self，参数 F）
let double = ArcTransformer::new(|x: i32| x * 2);
let pipeline = parse.and_then(double);

// 原始 parse 转换器仍然可用（double 已被消耗）
assert_eq!(parse.apply("42".to_string()), 42);
assert_eq!(pipeline.apply("21".to_string()), 42);

// ✅ 可以跨线程使用
use std::thread;
let handle = thread::spawn(move || {
    parse_clone.apply("100".to_string())
});
assert_eq!(handle.join().unwrap(), 100);

// ============================================================================
// 5. RcTransformer - 单线程复用，性能更好
// ============================================================================

let parse = RcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));
let double = RcTransformer::new(|x: i32| x * 2);

// ✅ 可以克隆
let parse_clone = parse.clone();

// ✅ 组合时消耗参数但不消耗 self
let pipeline1 = parse.and_then(double);
let to_string = RcTransformer::new(|x: i32| x.to_string());
let pipeline2 = parse.and_then(to_string);

// 原始 parse 转换器仍然可用（double 和 to_string 已被消耗）
assert_eq!(parse.apply("42".to_string()), 42);

// ============================================================================
// 6. 统一的接口 - 泛型编程
// ============================================================================

fn transform_vec<T, R, F>(transformer: &F, vec: Vec<T>) -> Vec<R>
where
    F: Transformer<T, R>,
{
    vec.into_iter().map(|x| transformer.apply(x)).collect()
}

let arc_transformer = ArcTransformer::new(|x: i32| x * 2);
let results = transform_vec(&arc_transformer, vec![1, 2, 3]);
assert_eq!(results, vec![2, 4, 6]);

// ============================================================================
// 7. 使用内部可变性实现缓存
// ============================================================================

use std::cell::RefCell;
use std::collections::HashMap;

let cache = RefCell::new(HashMap::new());
let cached_expensive = BoxTransformer::new(move |x: i32| {
    let mut cache = cache.borrow_mut();
    *cache.entry(x).or_insert_with(|| {
        // 模拟昂贵计算
        println!("Computing for {}", x);
        x * x
    })
});

// 第一次调用：计算
assert_eq!(cached_expensive.apply(5), 25);  // 打印 "Computing for 5"
// 第二次调用：使用缓存
assert_eq!(cached_expensive.apply(5), 25);  // 不打印（使用缓存）

// ============================================================================
// 8. 转换为标准闭包 - 与标准库深度集成
// ============================================================================

let transformer = BoxTransformer::new(|x: i32| x * 2);

// 转换为标准闭包，可以直接在 map 等方法中使用
let func = transformer.into_fn_once();
let results: Vec<_> = vec![1, 2, 3].into_iter().map(func).collect();
assert_eq!(results, vec![2, 4, 6]);

// 也可以直接使用
let transformer = BoxTransformer::new(|s: String| s.len());
let lengths: Vec<_> = vec!["hello".to_string(), "world".to_string()]
    .into_iter()
    .map(transformer.into_fn_once())
    .collect();
assert_eq!(lengths, vec![5, 5]);

// TransformerOnce 也可以转换为 FnOnce
let once_transformer = BoxTransformerOnce::new(|data: Vec<i32>| {
    data.into_iter().sum::<i32>()
});

let func_once = once_transformer.into_fn_once();
let result = func_once(vec![1, 2, 3, 4, 5]);
assert_eq!(result, 15);
```

### 3.3 类型选择指南

| 需求 | 推荐类型 | 理由 |
|------|---------|------|
| 可重复调用，单一所有权 | `BoxTransformer` | 单一所有权，可多次调用 |
| 一次性使用 | `BoxTransformerOnce` | 消耗 self，保存 FnOnce |
| 多线程共享 | `ArcTransformer` | 线程安全，可克隆 |
| 单线程复用 | `RcTransformer` | 无原子操作，性能更好 |
| 需要内部状态（缓存） | 任意类型 + RefCell/Mutex | 内部可变性 |

---

## 四、与其他函数式抽象的对比

### 4.1 核心差异

| | Transformer | Predicate | Consumer | Supplier |
|---|---|---|---|---|
| **输入** | `T` | `&T` | `&T` | 无 |
| **输出** | `R` | `bool` | `()` | `T` |
| **self 签名** | `&self` | `&self` | `&mut self` | `&mut self` |
| **消耗输入** | ✅ | ❌ | ❌ | N/A |
| **修改自己** | ❌（内部可变性）| ❌（内部可变性）| ✅ | ✅ |
| **Once 变体** | ✅ 有价值 | ❌ 无意义 | ✅ 有价值 | ✅ 有价值 |
| **核心用途** | 类型转换、映射 | 过滤、验证 | 观察、累积 | 工厂、生成 |

### 4.2 为什么 Transformer 消耗输入而 Predicate 不消耗？

| | Transformer `T -> R` | Predicate `&T -> bool` |
|---|---|---|
| **语义** | "转换" - 将输入变为输出 | "判断" - 检查输入是否满足条件 |
| **典型场景** | `String -> Vec<u8>` | `&i32 -> bool` (是否 > 0) |
| **输入后续** | 输入已转换，通常不再需要 | 判断后输入仍然存在 |
| **所有权** | 需要所有权进行转换 | 只需要读取，不需要所有权 |

**实际业务中的区别**：

```rust
// Transformer: 转换 - 消耗输入
let parse_json = BoxTransformer::new(|json_str: String| {
    serde_json::from_str(&json_str).unwrap()  // 消耗 String
});

// Predicate: 判断 - 借用输入
let is_valid_json = BoxPredicate::new(|json_str: &String| {
    serde_json::from_str::<serde_json::Value>(json_str).is_ok()  // 只借用
});

let json = r#"{"key": "value"}"#.to_string();

// 判断后 json 仍然可用
if is_valid_json.test(&json) {
    let data = parse_json.apply(json);  // json 被消耗
    // json 不再可用
}
```

### 4.3 设计一致性

所有函数式抽象遵循统一的设计模式：

1. **统一的 trait 接口**：每种抽象都有核心 trait
2. **三种实现**：Box（单一）、Arc（共享+线程安全）、Rc（共享+单线程）
3. **类型保持的方法链**：组合方法返回相同类型
4. **闭包自动实现 trait**：无缝集成
5. **扩展 trait 提供组合能力**：如 `FnTransformerOps`

---

## 五、真实业务场景示例

### 5.1 数据转换管道

```rust
// 构建复杂的数据处理管道
let pipeline = BoxTransformer::new(|raw: String| raw.trim().to_string())
    .and_then(|s| s.parse::<i32>().ok())
    .and_then(|opt| opt.unwrap_or(0))
    .and_then(|x| x * 2)
    .and_then(|x| format!("Result: {}", x));

let result = pipeline.apply("  42  ".to_string());
assert_eq!(result, "Result: 84");
```

### 5.2 配置转换器

```rust
use std::collections::HashMap;

struct ConfigManager {
    transformers: HashMap<String, BoxTransformer<String, String>>,
}

impl ConfigManager {
    fn new() -> Self {
        let mut transformers = HashMap::new();

        // 注册各种转换器
        transformers.insert(
            "uppercase".to_string(),
            BoxTransformer::new(|s: String| s.to_uppercase()),
        );

        transformers.insert(
            "trim".to_string(),
            BoxTransformer::new(|s: String| s.trim().to_string()),
        );

        ConfigManager { transformers }
    }

    fn transform(&self, key: &str, value: String) -> String {
        if let Some(transformer) = self.transformers.get(key) {
            transformer.apply(value)
        } else {
            value
        }
    }
}
```

### 5.3 多线程数据处理

```rust
use std::thread;

// 创建可在多线程间共享的转换器
let heavy_transform = ArcTransformer::new(|data: Vec<u8>| {
    // 模拟耗时的转换操作
    data.into_iter().map(|b| b.wrapping_mul(2)).collect::<Vec<_>>()
});

let mut handles = vec![];

for i in 0..4 {
    let transformer = heavy_transform.clone();
    let handle = thread::spawn(move || {
        let data = vec![i; 100];
        transformer.apply(data)
    });
    handles.push(handle);
}

let results: Vec<_> = handles.into_iter()
    .map(|h| h.join().unwrap())
    .collect();
```

### 5.4 延迟计算

```rust
// 保存昂贵的转换逻辑，延迟执行
struct LazyComputation<T, R> {
    input: Option<T>,
    transformer: BoxTransformerOnce<T, R>,
}

impl<T, R> LazyComputation<T, R> {
    fn new<F>(input: T, transformer: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
        T: 'static,
        R: 'static,
    {
        LazyComputation {
            input: Some(input),
            transformer: BoxTransformerOnce::new(transformer),
        }
    }

    fn compute(mut self) -> R {
        let input = self.input.take().unwrap();
        self.transformer.apply(input)
    }
}

// 使用
let lazy = LazyComputation::new(
    "large dataset".to_string(),
    |data| {
        // 只在 compute() 被调用时才执行
        expensive_analysis(data)
    },
);

// 稍后执行
let result = lazy.compute();
```

---

## 六、总结

### 6.1 核心设计原则

1. **Transformer 消耗输入 `T`**：符合转换语义，最大灵活性
2. **Transformer 返回所有权 `R`**：避免生命周期问题，语义明确
3. **Transformer 使用 `&self`**：纯函数，不修改自己（使用内部可变性）
4. **保留 TransformerOnce**：一次性转换、延迟计算
5. **不需要 TransformerMut**：内部可变性完全够用
6. **类型名称语义明确**：Box/Arc/Rc 表达所有权模型

### 6.2 为什么这个设计最好？

**与过度设计的对比**：

| | 过度设计 | 简化设计（推荐）|
|---|---|---|
| **Trait 数量** | 多个（Function、FunctionMut、RefFunction）| 2 个（Transformer、TransformerOnce）✅ |
| **输入类型** | 混乱（T、&T、&mut T）| 清晰（T）✅ |
| **用户心智负担** | 高（何时用哪个？）| 低（语义明确）✅ |
| **状态管理** | 需要 `&mut self` | 内部可变性 ✅ |
| **API 一致性** | 多套方法 | 统一的 transform ✅ |

**与其他模块设计的一致性**：

- Consumer **观察**输入（`&T`），**可修改**自己（累积）
- Predicate **判断**输入（`&T`），**不修改**自己（纯函数）
- Transformer **转换**输入（`T`），**不修改**自己（纯函数）
- Supplier **生成**输出（无输入），**可修改**自己（状态递增）

### 6.3 为什么 Transformer 比 Function 更好？

| 方面 | Function | Transformer |
|------|----------|-------------|
| **语义精确度** | "函数" - 太宽泛 ❌ | "转换器" - 精确表达转换 ✅ |
| **避免混淆** | 与 Fn/FnMut/FnOnce 混淆 ❌ | 完全区分 ✅ |
| **命名对称性** | 与其他模块不一致 ❌ | 与 Consumer、Supplier 对称 ✅ |
| **可读性** | `BoxFunction<String, User>` ❌ | `BoxTransformer<String, User>` ✅ |
| **业界实践** | 不够明确 ❌ | 符合 Kotlin、ReactiveX 等 ✅ |

### 6.4 最终结论

对于 `prism3-rust-function` 这样的库项目：

1. **采用 Trait + 多种实现方案**：统一接口，灵活实现
2. **提供 Transformer 和 TransformerOnce**：覆盖可重复调用和一次性使用场景
3. **三种实现**：BoxTransformer、ArcTransformer、RcTransformer
4. **使用内部可变性**：需要状态时用 RefCell/Cell/Mutex
5. **文档说明最佳实践**：指导用户何时使用哪种类型

这个设计：
- ✅ **符合转换语义**：Transformer 就是消耗输入产生输出
- ✅ **与 Rust 标准库一致**：`Iterator::map` 等都消耗输入
- ✅ **最大灵活性**：可重复调用（Transformer）和一次性使用（TransformerOnce）
- ✅ **简洁优雅**：只有两个核心 trait，清晰明了
- ✅ **命名精确**：Transformer 比 Function 更能表达"转换"的语义
- ✅ **长期可维护**：架构清晰，语义明确

**这是一个从真实业务需求出发、经过深思熟虑、符合 Rust 惯例的优雅方案。**

