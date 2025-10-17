/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ConsumerOnce 类型
//!
//! 提供一次性消费者接口的实现，用于执行接受单个输入参数但不返回结果的一次性操作。
//!
//! 本模块提供统一的 `ConsumerOnce` trait 和一种具体实现:
//!
//! - **`BoxConsumerOnce<T>`**: 基于 Box 的单一所有权实现
//!
//! # 为什么没有 Arc/Rc 变体？
//!
//! 与 `Consumer` 和 `ReadonlyConsumer` 不同，本模块**不**提供 `ArcConsumerOnce`
//! 或 `RcConsumerOnce` 实现。这是基于 `FnOnce` 语义与共享所有权根本不兼容的
//! 设计决策。详见设计文档。
//!
//! # 设计理念
//!
//! ConsumerOnce 使用 `FnOnce(&T)` 语义，用于真正的一次性消费操作。
//! 与 Consumer 不同，ConsumerOnce 在首次调用时消耗自身。适用于初始化回调、
//! 清理回调等场景。
//!
//! # 作者
//!
//! 胡海星

use std::fmt;

// ============================================================================
// 1. ConsumerOnce Trait - 统一的 ConsumerOnce 接口
// ============================================================================

/// ConsumerOnce trait - 统一的一次性消费者接口
///
/// 定义所有一次性消费者类型的核心行为。类似于实现 `FnOnce(&T)` 的消费者，
/// 执行接受一个值引用但不返回结果的操作(仅产生副作用)，并在过程中消耗自身。
///
/// # 自动实现
///
/// - 所有实现 `FnOnce(&T)` 的闭包
/// - `BoxConsumerOnce<T>`
///
/// # 特性
///
/// - **统一接口**: 所有消费者类型共享相同的 `accept` 方法签名
/// - **自动实现**: 闭包自动实现此 trait，零开销
/// - **类型转换**: 可以转换为 BoxConsumerOnce
/// - **泛型编程**: 编写可用于任何一次性消费者类型的函数
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: ConsumerOnce<i32>>(consumer: C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let box_con = BoxConsumerOnce::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// apply_consumer(box_con, &5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # 作者
///
/// 胡海星
pub trait ConsumerOnce<T> {
    /// 执行一次性消费操作
    ///
    /// 对给定的引用执行操作。操作通常读取输入值或产生副作用，
    /// 但不修改输入值本身。消耗 self。
    ///
    /// # 参数
    ///
    /// * `value` - 要消费的值的引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let consumer = BoxConsumerOnce::new(|x: &i32| println!("{}", x));
    /// consumer.accept(&5);
    /// ```
    fn accept(self, value: &T);

    /// 转换为 BoxConsumerOnce
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `BoxConsumerOnce<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::ConsumerOnce;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// };
    /// let box_consumer = closure.into_box();
    /// box_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box(self) -> BoxConsumerOnce<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为闭包
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// 将一次性消费者转换为闭包，可以直接用于标准库中需要 `FnOnce` 的地方。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `FnOnce(&T)` 的闭包
    fn into_fn(self) -> impl FnOnce(&T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxConsumerOnce - 单一所有权实现
// ============================================================================

/// BoxConsumerOnce 结构体
///
/// 基于 `Box<dyn FnOnce(&T)>` 的一次性消费者实现，用于单一所有权场景。
/// 这是真正一次性使用的最简单消费者类型。
///
/// # 特性
///
/// - **单一所有权**: 不可克隆，使用时转移所有权
/// - **零开销**: 无引用计数或锁开销
/// - **一次性使用**: 首次调用时消耗 self
/// - **构建器模式**: 方法链自然地消耗 `self`
///
/// # 使用场景
///
/// 选择 `BoxConsumerOnce` 当:
/// - 消费者真正只使用一次
/// - 构建流水线，所有权自然流动
/// - 消费者捕获应该被消耗的值
/// - 性能关键且无法接受共享开销
///
/// # 性能
///
/// `BoxConsumerOnce` 性能最好:
/// - 无引用计数开销
/// - 无锁获取或运行时借用检查
/// - 通过 vtable 直接调用函数
/// - 最小内存占用(单个指针)
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
///
/// let consumer = BoxConsumerOnce::new(|x: &i32| {
///     println!("Value: {}", x);
/// });
/// consumer.accept(&5);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct BoxConsumerOnce<T> {
    function: Box<dyn FnOnce(&T)>,
    name: Option<String>,
}

impl<T> BoxConsumerOnce<T>
where
    T: 'static,
{
    /// 创建新的 BoxConsumerOnce
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
    /// 返回新的 `BoxConsumerOnce<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxConsumerOnce::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce {
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

    /// 顺序链接另一个一次性消费者
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
    /// 返回新的组合 `BoxConsumerOnce<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = BoxConsumerOnce::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: ConsumerOnce<T> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxConsumerOnce::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// 创建空操作消费者
    ///
    /// # 返回值
    ///
    /// 返回空操作消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let noop = BoxConsumerOnce::<i32>::noop();
    /// noop.accept(&42);
    /// // 值未改变
    /// ```
    pub fn noop() -> Self {
        BoxConsumerOnce::new(|_| {})
    }

    /// 创建打印消费者
    ///
    /// 返回一个消费者，该消费者打印输入值。
    ///
    /// # 返回值
    ///
    /// 返回打印消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let print = BoxConsumerOnce::<i32>::print();
    /// print.accept(&42); // Prints: 42
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        BoxConsumerOnce::new(|t| {
            println!("{:?}", t);
        })
    }

    /// 创建带前缀的打印消费者
    ///
    /// 返回一个消费者，该消费者使用指定前缀打印输入值。
    ///
    /// # 参数
    ///
    /// * `prefix` - 前缀字符串
    ///
    /// # 返回值
    ///
    /// 返回打印消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let print = BoxConsumerOnce::<i32>::print_with("Value: ");
    /// print.accept(&42); // Prints: Value: 42
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxConsumerOnce::new(move |t| {
            println!("{}{:?}", prefix, t);
        })
    }

    /// 创建条件消费者
    ///
    /// 返回一个消费者，仅在谓词为 true 时执行操作。
    ///
    /// # 类型参数
    ///
    /// * `P` - 谓词类型
    /// * `C` - 消费者类型
    ///
    /// # 参数
    ///
    /// * `predicate` - 谓词函数
    /// * `consumer` - 要执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回条件消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let conditional = BoxConsumerOnce::if_then(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l.lock().unwrap().push(*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnOnce(&T) -> bool + 'static,
        C: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce::new(move |t| {
            if predicate(t) {
                consumer(t);
            }
        })
    }

    /// 创建条件分支消费者
    ///
    /// 返回一个消费者，根据谓词执行不同的操作。
    ///
    /// # 类型参数
    ///
    /// * `P` - 谓词类型
    /// * `C1` - then 消费者类型
    /// * `C2` - else 消费者类型
    ///
    /// # 参数
    ///
    /// * `predicate` - 谓词函数
    /// * `then_consumer` - 谓词为 true 时执行的消费者
    /// * `else_consumer` - 谓词为 false 时执行的消费者
    ///
    /// # 返回值
    ///
    /// 返回条件分支消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let conditional = BoxConsumerOnce::if_then_else(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l1.lock().unwrap().push(*x);
    ///     },
    ///     move |x: &i32| {
    ///         l2.lock().unwrap().push(-*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnOnce(&T) -> bool + 'static,
        C1: FnOnce(&T) + 'static,
        C2: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce::new(move |t| {
            if predicate(t) {
                then_consumer(t);
            } else {
                else_consumer(t);
            }
        })
    }
}

impl<T> ConsumerOnce<T> for BoxConsumerOnce<T> {
    fn accept(self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        self.function
    }
}

impl<T> fmt::Debug for BoxConsumerOnce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxConsumerOnce")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxConsumerOnce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxConsumerOnce({})", name),
            None => write!(f, "BoxConsumerOnce"),
        }
    }
}

// ============================================================================
// 3. 为闭包实现 ConsumerOnce trait
// ============================================================================

/// 为所有 FnOnce(&T) 实现 ConsumerOnce
impl<T, F> ConsumerOnce<T> for F
where
    F: FnOnce(&T),
{
    fn accept(self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumerOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumerOnce::new(self)
    }

    fn into_fn(self) -> impl FnOnce(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 4. 为闭包提供扩展方法
// ============================================================================

/// 为闭包提供一次性消费者组合方法的扩展 trait
///
/// 为所有实现 `FnOnce(&T)` 的闭包提供 `and_then` 和其他组合方法，
/// 使闭包无需显式包装类型即可直接进行方法链接。
///
/// # 特性
///
/// - **自然语法**: 直接在闭包上链接操作
/// - **返回 BoxConsumerOnce**: 组合结果可继续链接
/// - **零成本**: 组合闭包时无开销
/// - **自动实现**: 所有 `FnOnce(&T)` 闭包自动获得这些方法
///
/// # 示例
///
/// ```rust
/// use prism3_function::{ConsumerOnce, FnConsumerOnceOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let chained = (move |x: &i32| {
///     l1.lock().unwrap().push(*x * 2);
/// }).and_then(move |x: &i32| {
///     l2.lock().unwrap().push(*x + 10);
/// });
/// chained.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
/// ```
///
/// # 作者
///
/// 胡海星
pub trait FnConsumerOnceOps<T>: FnOnce(&T) + Sized {
    /// 顺序链接另一个一次性消费者
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。
    /// 消耗当前闭包并返回 `BoxConsumerOnce<T>`。
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
    /// 返回组合的 `BoxConsumerOnce<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, FnConsumerOnceOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// }).and_then(|x: &i32| println!("Result: {}", x));
    ///
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumerOnce<T>
    where
        Self: 'static,
        C: ConsumerOnce<T> + 'static,
        T: 'static,
    {
        let first = self;
        let second = next;
        BoxConsumerOnce::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// 为所有闭包类型实现 FnConsumerOnceOps
impl<T, F> FnConsumerOnceOps<T> for F where F: FnOnce(&T) {}
