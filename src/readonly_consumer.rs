/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ReadonlyConsumer 类型
//!
//! 提供只读消费者接口的实现，用于执行不修改自身状态且不修改输入值的操作。
//!
//! 本模块提供统一的 `ReadonlyConsumer` trait 和三种基于不同所有权模型的具体实现:
//!
//! - **`BoxReadonlyConsumer<T>`**: 基于 Box 的单一所有权实现
//! - **`ArcReadonlyConsumer<T>`**: 基于 Arc 的线程安全共享所有权实现
//! - **`RcReadonlyConsumer<T>`**: 基于 Rc 的单线程共享所有权实现
//!
//! # 设计理念
//!
//! ReadonlyConsumer 使用 `Fn(&T)` 语义，既不修改自身状态也不修改输入值。
//! 适用于纯观察、日志记录、通知等场景。与 Consumer 相比，ReadonlyConsumer
//! 不需要内部可变性(Mutex/RefCell)，因此更高效且更易于共享。
//!
//! # 作者
//!
//! 胡海星

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// 1. ReadonlyConsumer Trait - 统一的 ReadonlyConsumer 接口
// ============================================================================

/// ReadonlyConsumer trait - 统一的只读消费者接口
///
/// 定义所有只读消费者类型的核心行为。与 `Consumer` 不同，`ReadonlyConsumer`
/// 既不修改自身状态也不修改输入值，是完全不可变的操作。
///
/// # 自动实现
///
/// - 所有实现 `Fn(&T)` 的闭包
/// - `BoxReadonlyConsumer<T>`, `ArcReadonlyConsumer<T>`, `RcReadonlyConsumer<T>`
///
/// # 特性
///
/// - **统一接口**: 所有只读消费者类型共享相同的 `accept` 方法签名
/// - **自动实现**: 闭包自动实现此 trait，零开销
/// - **类型转换**: 可以方便地在不同所有权模型间转换
/// - **泛型编程**: 编写可用于任何只读消费者类型的函数
/// - **无需内部可变性**: 不需要 Mutex 或 RefCell，更高效
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
///
/// fn apply_consumer<C: ReadonlyConsumer<i32>>(consumer: &C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// let box_con = BoxReadonlyConsumer::new(|x: &i32| {
///     println!("Value: {}", x);
/// });
/// apply_consumer(&box_con, &5);
/// ```
///
/// # 作者
///
/// 胡海星
pub trait ReadonlyConsumer<T> {
    /// 执行只读消费操作
    ///
    /// 对给定的引用执行操作。操作通常读取输入值或产生副作用，
    /// 但既不修改输入值也不修改消费者自身的状态。
    ///
    /// # 参数
    ///
    /// * `value` - 要消费的值的引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let consumer = BoxReadonlyConsumer::new(|x: &i32| println!("{}", x));
    /// consumer.accept(&5);
    /// ```
    fn accept(&self, value: &T);

    /// 转换为 BoxReadonlyConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `BoxReadonlyConsumer<T>`
    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 RcReadonlyConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `RcReadonlyConsumer<T>`
    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 ArcReadonlyConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `ArcReadonlyConsumer<T>`
    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static;

    /// 转换为闭包
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// 将只读消费者转换为闭包，可以直接用于标准库中需要 `Fn` 的地方。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `Fn(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let consumer = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    /// let func = consumer.into_fn();
    /// func(&5);
    /// ```
    fn into_fn(self) -> impl Fn(&T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxReadonlyConsumer - 单一所有权实现
// ============================================================================

/// BoxReadonlyConsumer 结构体
///
/// 基于 `Box<dyn Fn(&T)>` 的只读消费者实现，用于单一所有权场景。
///
/// # 特性
///
/// - **单一所有权**: 不可克隆，使用时转移所有权
/// - **零开销**: 无引用计数或锁开销
/// - **完全不可变**: 既不修改自身也不修改输入
/// - **无内部可变性**: 不需要 Mutex 或 RefCell
///
/// # 使用场景
///
/// 选择 `BoxReadonlyConsumer` 当:
/// - 只读消费者只使用一次或呈线性流
/// - 不需要跨上下文共享消费者
/// - 纯观察操作，如日志记录
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
///
/// let consumer = BoxReadonlyConsumer::new(|x: &i32| {
///     println!("Observed value: {}", x);
/// });
/// consumer.accept(&5);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct BoxReadonlyConsumer<T> {
    function: Box<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> BoxReadonlyConsumer<T>
where
    T: 'static,
{
    /// 创建新的 BoxReadonlyConsumer
    ///
    /// # 类型参数
    ///
    /// * `F` - 闭包类型
    ///
    /// # 参数
    ///
    /// * `f` - 要包装的闭包
    ///
    /// # 返回值
    ///
    /// 返回新的 `BoxReadonlyConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let consumer = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    /// consumer.accept(&5);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        BoxReadonlyConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// 获取消费者的名称
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 顺序链接另一个只读消费者
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。消耗 self。
    ///
    /// # 类型参数
    ///
    /// * `C` - 下一个消费者的类型
    ///
    /// # 参数
    ///
    /// * `next` - 当前操作之后要执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回新的组合 `BoxReadonlyConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let chained = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("First: {}", x);
    /// }).and_then(|x: &i32| {
    ///     println!("Second: {}", x);
    /// });
    /// chained.accept(&5);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: ReadonlyConsumer<T> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxReadonlyConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// 创建空操作消费者
    ///
    /// # 返回值
    ///
    /// 返回空操作消费者
    pub fn noop() -> Self {
        BoxReadonlyConsumer::new(|_| {})
    }
}

impl<T> ReadonlyConsumer<T> for BoxReadonlyConsumer<T> {
    fn accept(&self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        RcReadonlyConsumer::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        T: Send + Sync + 'static,
    {
        panic!("Cannot convert BoxReadonlyConsumer to ArcReadonlyConsumer: inner function may not be Send+Sync")
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        self.function
    }
}

impl<T> fmt::Debug for BoxReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxReadonlyConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxReadonlyConsumer({})", name),
            None => write!(f, "BoxReadonlyConsumer"),
        }
    }
}

// ============================================================================
// 3. ArcReadonlyConsumer - 线程安全的共享所有权实现
// ============================================================================

/// ArcReadonlyConsumer 结构体
///
/// 基于 `Arc<dyn Fn(&T) + Send + Sync>` 的只读消费者实现，
/// 用于线程安全的共享所有权场景。不需要 Mutex，因为操作是只读的。
///
/// # 特性
///
/// - **共享所有权**: 通过 `Arc` 可克隆，允许多个所有者
/// - **线程安全**: 实现 `Send + Sync`，可安全地并发使用
/// - **无锁**: 因为是只读的，不需要 Mutex 保护
/// - **非消耗 API**: `and_then` 借用 `&self`，原始对象仍可使用
///
/// # 使用场景
///
/// 选择 `ArcReadonlyConsumer` 当:
/// - 需要在多个线程间共享只读消费者
/// - 纯观察操作，如日志、监控、通知
/// - 需要高并发读取，无锁开销
///
/// # 性能优势
///
/// 相比 `ArcConsumer`，`ArcReadonlyConsumer` 没有 Mutex 锁开销，
/// 在高并发场景下性能更好。
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ReadonlyConsumer, ArcReadonlyConsumer};
///
/// let consumer = ArcReadonlyConsumer::new(|x: &i32| {
///     println!("Observed: {}", x);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5);
/// clone.accept(&10);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct ArcReadonlyConsumer<T> {
    function: Arc<dyn Fn(&T) + Send + Sync>,
    name: Option<String>,
}

impl<T> ArcReadonlyConsumer<T>
where
    T: Send + Sync + 'static,
{
    /// 创建新的 ArcReadonlyConsumer
    ///
    /// # 类型参数
    ///
    /// * `F` - 闭包类型
    ///
    /// # 参数
    ///
    /// * `f` - 要包装的闭包
    ///
    /// # 返回值
    ///
    /// 返回新的 `ArcReadonlyConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, ArcReadonlyConsumer};
    ///
    /// let consumer = ArcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    /// consumer.accept(&5);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        ArcReadonlyConsumer {
            function: Arc::new(f),
            name: None,
        }
    }

    /// 获取消费者的名称
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 转换为闭包（不消费自身）
    ///
    /// 创建一个新的闭包，通过 Arc 调用底层函数。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `Fn(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, ArcReadonlyConsumer};
    ///
    /// let consumer = ArcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    ///
    /// let func = consumer.to_fn();
    /// func(&5);
    /// ```
    pub fn to_fn(&self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let func = Arc::clone(&self.function);
        move |t: &T| {
            func(t);
        }
    }

    /// 顺序链接另一个 ArcReadonlyConsumer
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。
    /// 借用 &self，不消耗原始消费者。
    ///
    /// # 参数
    ///
    /// * `next` - 当前操作之后要执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回新的组合 `ArcReadonlyConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, ArcReadonlyConsumer};
    ///
    /// let first = ArcReadonlyConsumer::new(|x: &i32| {
    ///     println!("First: {}", x);
    /// });
    /// let second = ArcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Second: {}", x);
    /// });
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first 和 second 在链接后仍可使用
    /// chained.accept(&5);
    /// ```
    pub fn and_then(&self, next: &ArcReadonlyConsumer<T>) -> ArcReadonlyConsumer<T> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcReadonlyConsumer {
            function: Arc::new(move |t: &T| {
                first(t);
                second(t);
            }),
            name: None,
        }
    }
}

impl<T> ReadonlyConsumer<T> for ArcReadonlyConsumer<T> {
    fn accept(&self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxReadonlyConsumer::new(move |t| func(t))
    }

    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        RcReadonlyConsumer::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        T: Send + Sync + 'static,
    {
        self
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func(t);
        }
    }
}

impl<T> Clone for ArcReadonlyConsumer<T> {
    /// 克隆 ArcReadonlyConsumer
    ///
    /// 创建与原始实例共享底层函数的新 ArcReadonlyConsumer。
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for ArcReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcReadonlyConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for ArcReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcReadonlyConsumer({})", name),
            None => write!(f, "ArcReadonlyConsumer"),
        }
    }
}

// ============================================================================
// 4. RcReadonlyConsumer - 单线程共享所有权实现
// ============================================================================

/// RcReadonlyConsumer 结构体
///
/// 基于 `Rc<dyn Fn(&T)>` 的只读消费者实现，用于单线程共享所有权场景。
/// 不需要 RefCell，因为操作是只读的。
///
/// # 特性
///
/// - **共享所有权**: 通过 `Rc` 可克隆，允许多个所有者
/// - **单线程**: 非线程安全，不能跨线程发送
/// - **无内部可变性开销**: 不需要 RefCell，因为是只读的
/// - **非消耗 API**: `and_then` 借用 `&self`，原始对象仍可使用
///
/// # 使用场景
///
/// 选择 `RcReadonlyConsumer` 当:
/// - 需要在单线程内共享只读消费者
/// - 纯观察操作，性能关键
/// - 单线程 UI 框架中的事件处理
///
/// # 性能优势
///
/// `RcReadonlyConsumer` 既没有 Arc 的原子操作开销，也没有 RefCell 的
/// 运行时借用检查开销，是三种只读消费者中性能最好的。
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ReadonlyConsumer, RcReadonlyConsumer};
///
/// let consumer = RcReadonlyConsumer::new(|x: &i32| {
///     println!("Observed: {}", x);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5);
/// clone.accept(&10);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct RcReadonlyConsumer<T> {
    function: Rc<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> RcReadonlyConsumer<T>
where
    T: 'static,
{
    /// 创建新的 RcReadonlyConsumer
    ///
    /// # 类型参数
    ///
    /// * `F` - 闭包类型
    ///
    /// # 参数
    ///
    /// * `f` - 要包装的闭包
    ///
    /// # 返回值
    ///
    /// 返回新的 `RcReadonlyConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, RcReadonlyConsumer};
    ///
    /// let consumer = RcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    /// consumer.accept(&5);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        RcReadonlyConsumer {
            function: Rc::new(f),
            name: None,
        }
    }

    /// 获取消费者的名称
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 转换为闭包（不消费自身）
    ///
    /// 创建一个新的闭包，通过 Rc 调用底层函数。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `Fn(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, RcReadonlyConsumer};
    ///
    /// let consumer = RcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    ///
    /// let func = consumer.to_fn();
    /// func(&5);
    /// ```
    pub fn to_fn(&self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T| {
            func(t);
        }
    }

    /// 顺序链接另一个 RcReadonlyConsumer
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。
    /// 借用 &self，不消耗原始消费者。
    ///
    /// # 参数
    ///
    /// * `next` - 当前操作之后要执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回新的组合 `RcReadonlyConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, RcReadonlyConsumer};
    ///
    /// let first = RcReadonlyConsumer::new(|x: &i32| {
    ///     println!("First: {}", x);
    /// });
    /// let second = RcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Second: {}", x);
    /// });
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first 和 second 在链接后仍可使用
    /// chained.accept(&5);
    /// ```
    pub fn and_then(&self, next: &RcReadonlyConsumer<T>) -> RcReadonlyConsumer<T> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcReadonlyConsumer {
            function: Rc::new(move |t: &T| {
                first(t);
                second(t);
            }),
            name: None,
        }
    }
}

impl<T> ReadonlyConsumer<T> for RcReadonlyConsumer<T> {
    fn accept(&self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxReadonlyConsumer::new(move |t| func(t))
    }

    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        T: Send + Sync + 'static,
    {
        panic!("Cannot convert RcReadonlyConsumer to ArcReadonlyConsumer (not Send+Sync)")
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func(t);
        }
    }
}

impl<T> Clone for RcReadonlyConsumer<T> {
    /// 克隆 RcReadonlyConsumer
    ///
    /// 创建与原始实例共享底层函数的新 RcReadonlyConsumer。
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for RcReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcReadonlyConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for RcReadonlyConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcReadonlyConsumer({})", name),
            None => write!(f, "RcReadonlyConsumer"),
        }
    }
}

// ============================================================================
// 5. 为闭包实现 ReadonlyConsumer trait
// ============================================================================

/// 为所有 Fn(&T) 实现 ReadonlyConsumer
impl<T, F> ReadonlyConsumer<T> for F
where
    F: Fn(&T),
{
    fn accept(&self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxReadonlyConsumer::new(self)
    }

    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcReadonlyConsumer::new(self)
    }

    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        ArcReadonlyConsumer::new(self)
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 6. 为闭包提供扩展方法
// ============================================================================

/// 为闭包提供只读消费者组合方法的扩展 trait
///
/// 为所有实现 `Fn(&T)` 的闭包提供 `and_then` 和其他组合方法，
/// 使闭包无需显式包装类型即可直接进行方法链接。
///
/// # 特性
///
/// - **自然语法**: 直接在闭包上链接操作
/// - **返回 BoxReadonlyConsumer**: 组合结果可继续链接
/// - **零成本**: 组合闭包时无开销
/// - **自动实现**: 所有 `Fn(&T)` 闭包自动获得这些方法
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ReadonlyConsumer, FnReadonlyConsumerOps};
///
/// let chained = (|x: &i32| {
///     println!("First: {}", x);
/// }).and_then(|x: &i32| {
///     println!("Second: {}", x);
/// });
/// chained.accept(&5);
/// ```
///
/// # 作者
///
/// 胡海星
pub trait FnReadonlyConsumerOps<T>: Fn(&T) + Sized {
    /// 顺序链接另一个只读消费者
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。
    /// 消耗当前闭包并返回 `BoxReadonlyConsumer<T>`。
    ///
    /// # 类型参数
    ///
    /// * `C` - 下一个消费者的类型
    ///
    /// # 参数
    ///
    /// * `next` - 当前操作之后要执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回组合的 `BoxReadonlyConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, FnReadonlyConsumerOps};
    ///
    /// let chained = (|x: &i32| {
    ///     println!("First: {}", x);
    /// }).and_then(|x: &i32| {
    ///     println!("Second: {}", x);
    /// }).and_then(|x: &i32| println!("Third: {}", x));
    ///
    /// chained.accept(&5);
    /// ```
    fn and_then<C>(self, next: C) -> BoxReadonlyConsumer<T>
    where
        Self: 'static,
        C: ReadonlyConsumer<T> + 'static,
        T: 'static,
    {
        let first = self;
        let second = next;
        BoxReadonlyConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// 为所有闭包类型实现 FnReadonlyConsumerOps
impl<T, F> FnReadonlyConsumerOps<T> for F where F: Fn(&T) {}
