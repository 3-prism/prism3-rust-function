/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer 类型
//!
//! 提供消费者接口的实现，用于执行接受单个输入参数但不返回结果的操作。
//!
//! 本模块提供统一的 `Consumer` trait 和三种基于不同所有权模型的具体实现:
//!
//! - **`BoxConsumer<T>`**: 基于 Box 的单一所有权实现，用于一次性使用场景
//! - **`ArcConsumer<T>`**: 基于 Arc<Mutex<>> 的线程安全共享所有权实现
//! - **`RcConsumer<T>`**: 基于 Rc<RefCell<>> 的单线程共享所有权实现
//!
//! # 设计理念
//!
//! Consumer 使用 `FnMut(&T)` 语义，可以修改自身状态但不修改输入值。适用于
//! 统计、累积、事件处理等场景。
//!
//! # 作者
//!
//! 胡海星

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Type alias for consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and returns nothing.
/// It is used to reduce type complexity in struct definitions.
type ConsumerFn<T> = dyn FnMut(&T);

/// Type alias for thread-safe consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and returns nothing,
/// with Send bound for thread-safe usage. It is used to reduce type complexity
/// in Arc-based struct definitions.
type SendConsumerFn<T> = dyn FnMut(&T) + Send;

// ============================================================================
// 1. Consumer Trait - 统一的 Consumer 接口
// ============================================================================

/// Consumer trait - 统一的消费者接口
///
/// 定义所有消费者类型的核心行为。类似于 Java 的 `Consumer<T>` 接口，
/// 执行接受一个值但不返回结果的操作(仅产生副作用)。
///
/// Consumer 可以修改自身状态(如累积、计数)，但不应该修改被消费的值本身。
///
/// # 自动实现
///
/// - 所有实现 `FnMut(&T)` 的闭包
/// - `BoxConsumer<T>`, `ArcConsumer<T>`, `RcConsumer<T>`
///
/// # 特性
///
/// - **统一接口**: 所有消费者类型共享相同的 `accept` 方法签名
/// - **自动实现**: 闭包自动实现此 trait，零开销
/// - **类型转换**: 可以方便地在不同所有权模型间转换
/// - **泛型编程**: 编写可用于任何消费者类型的函数
///
/// # 示例
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer, ArcConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// // 适用于任何消费者类型
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut box_con = BoxConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// apply_consumer(&mut box_con, &5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # 作者
///
/// 胡海星
pub trait Consumer<T> {
    /// 执行消费操作
    ///
    /// 对给定的引用执行操作。操作通常读取输入值或产生副作用，
    /// 但不修改输入值本身。可以修改消费者自身的状态。
    ///
    /// # 参数
    ///
    /// * `value` - 要消费的值的引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
    /// let value = 5;
    /// consumer.accept(&value);
    /// ```
    fn accept(&mut self, value: &T);

    /// 转换为 BoxConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// 将当前消费者转换为 `BoxConsumer<T>`。
    ///
    /// # 所有权
    ///
    /// 此方法**消耗**消费者(获取 `self` 的所有权)。
    /// 调用此方法后，原始消费者不再可用。
    ///
    /// **提示**: 对于可克隆的消费者([`ArcConsumer`], [`RcConsumer`])，
    /// 如果需要保留原始对象，可以先调用 `.clone()`。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `BoxConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::Consumer;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// };
    /// let mut box_consumer = closure.into_box();
    /// box_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 RcConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `RcConsumer<T>`
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// 转换为 ArcConsumer
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// # 返回值
    ///
    /// 返回包装后的 `ArcConsumer<T>`
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;

    /// 转换为闭包
    ///
    /// **⚠️ 消耗 `self`**: 调用此方法后原始消费者将不可用。
    ///
    /// 将消费者转换为闭包，可以直接用于标准库中需要 `FnMut` 的地方。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `FnMut(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut func = consumer.into_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_fn(self) -> impl FnMut(&T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxConsumer - 单一所有权实现
// ============================================================================

/// BoxConsumer 结构体
///
/// 基于 `Box<dyn FnMut(&T)>` 的消费者实现，用于单一所有权场景。
/// 当不需要共享时，这是最简单、最高效的消费者类型。
///
/// # 特性
///
/// - **单一所有权**: 不可克隆，使用时转移所有权
/// - **零开销**: 无引用计数或锁开销
/// - **可变状态**: 可通过 `FnMut` 修改捕获的环境
/// - **构建器模式**: 方法链自然地消耗 `self`
///
/// # 使用场景
///
/// 选择 `BoxConsumer` 当:
/// - 消费者只使用一次或呈线性流
/// - 构建流水线，所有权自然流动
/// - 不需要跨上下文共享消费者
/// - 性能关键且无法接受共享开销
///
/// # 性能
///
/// `BoxConsumer` 在三种消费者类型中性能最好:
/// - 无引用计数开销
/// - 无锁获取或运行时借用检查
/// - 通过 vtable 直接调用函数
/// - 最小内存占用(单个指针)
///
/// # 示例
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct BoxConsumer<T> {
    function: Box<dyn FnMut(&T)>,
    name: Option<String>,
}

impl<T> BoxConsumer<T>
where
    T: 'static,
{
    /// 创建新的 BoxConsumer
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
    /// 返回新的 `BoxConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// 获取消费者的名称
    ///
    /// # 返回值
    ///
    /// 返回消费者的名称，如果未设置则返回 `None`
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    ///
    /// # 参数
    ///
    /// * `name` - 要设置的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 顺序链接另一个消费者
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
    /// 返回新的组合 `BoxConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = BoxConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: Consumer<T> + 'static,
    {
        let mut first = self.function;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// 创建空操作消费者
    ///
    /// 返回不执行任何操作的消费者。
    ///
    /// # 返回值
    ///
    /// 返回空操作消费者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut noop = BoxConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // 值未改变
    /// ```
    pub fn noop() -> Self {
        BoxConsumer::new(|_| {})
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
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print();
    /// print.accept(&42); // Prints: 42
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        BoxConsumer::new(|t| {
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
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print_with("Value: ");
    /// print.accept(&42); // Prints: Value: 42
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxConsumer::new(move |t| {
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
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut conditional = BoxConsumer::if_then(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l.lock().unwrap().push(*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// conditional.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                cons(t);
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
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut conditional = BoxConsumer::if_then_else(
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
    ///
    /// conditional.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 5]); // -(-5) = 5
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&T) + 'static,
        C2: FnMut(&T) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                then_cons(t);
            } else {
                else_cons(t);
            }
        })
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function)(value)
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
        let mut func = self.function;
        RcConsumer::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        // 注意：BoxConsumer 的 function 不一定是 Send，所以无法安全转换为 ArcConsumer
        // 这里我们 panic，因为这个转换在类型系统上是不安全的
        panic!("Cannot convert BoxConsumer to ArcConsumer: BoxConsumer's inner function may not be Send")
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        self.function
    }
}

impl<T> fmt::Debug for BoxConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxConsumer({})", name),
            None => write!(f, "BoxConsumer"),
        }
    }
}

// ============================================================================
// 3. ArcConsumer - 线程安全的共享所有权实现
// ============================================================================

/// ArcConsumer 结构体
///
/// 基于 `Arc<Mutex<dyn FnMut(&T) + Send>>` 的消费者实现，
/// 用于线程安全的共享所有权场景。此消费者可以安全地克隆并在多个线程间共享。
///
/// # 特性
///
/// - **共享所有权**: 通过 `Arc` 可克隆，允许多个所有者
/// - **线程安全**: 实现 `Send + Sync`，可安全地并发使用
/// - **内部可变性**: 使用 `Mutex` 实现安全的可变访问
/// - **非消耗 API**: `and_then` 借用 `&self`，原始对象仍可使用
/// - **跨线程共享**: 可发送到其他线程并使用
///
/// # 使用场景
///
/// 选择 `ArcConsumer` 当:
/// - 需要在多个线程间共享消费者
/// - 并发任务处理(如线程池)
/// - 在多个地方同时使用相同的消费者
/// - 需要线程安全(Send + Sync)
///
/// # 性能考虑
///
/// `ArcConsumer` 相比 `BoxConsumer` 有一些性能开销:
/// - **引用计数**: clone/drop 时的原子操作
/// - **Mutex 锁定**: 每次 `accept` 调用需要获取锁
/// - **锁竞争**: 高并发可能导致竞争
///
/// 但这些开销对于安全的并发访问是必要的。如果不需要线程安全，
/// 考虑使用 `RcConsumer` 以获得更少的单线程共享开销。
///
/// # 示例
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10]);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct ArcConsumer<T> {
    function: Arc<Mutex<SendConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> ArcConsumer<T>
where
    T: Send + 'static,
{
    /// 创建新的 ArcConsumer
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
    /// 返回新的 `ArcConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcConsumer {
            function: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// 获取消费者的名称
    ///
    /// # 返回值
    ///
    /// 返回消费者的名称，如果未设置则返回 `None`
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    ///
    /// # 参数
    ///
    /// * `name` - 要设置的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 转换为闭包（不消费自身）
    ///
    /// 创建一个新的闭包，通过 Arc 调用底层函数。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `FnMut(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = Arc::clone(&self.function);
        move |t: &T| {
            func.lock().unwrap()(t);
        }
    }

    /// 顺序链接另一个 ArcConsumer
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
    /// 返回新的组合 `ArcConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = ArcConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// });
    /// let second = ArcConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first 和 second 在链接后仍可使用
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]); // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcConsumer {
            function: Arc::new(Mutex::new(move |t: &T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
            name: None,
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        RcConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func.lock().unwrap()(t);
        }
    }
}

impl<T> Clone for ArcConsumer<T> {
    /// 克隆 ArcConsumer
    ///
    /// 创建与原始实例共享底层函数的新 ArcConsumer。
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for ArcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for ArcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcConsumer({})", name),
            None => write!(f, "ArcConsumer"),
        }
    }
}

// ============================================================================
// 4. RcConsumer - 单线程共享所有权实现
// ============================================================================

/// RcConsumer 结构体
///
/// 基于 `Rc<RefCell<dyn FnMut(&T)>>` 的消费者实现，用于单线程共享所有权场景。
/// 此消费者提供共享所有权的好处，而无需线程安全的开销。
///
/// # 特性
///
/// - **共享所有权**: 通过 `Rc` 可克隆，允许多个所有者
/// - **单线程**: 非线程安全，不能跨线程发送
/// - **内部可变性**: 使用 `RefCell` 进行运行时借用检查
/// - **无锁开销**: 比 `ArcConsumer` 在单线程使用时更高效
/// - **非消耗 API**: `and_then` 借用 `&self`，原始对象仍可使用
///
/// # 使用场景
///
/// 选择 `RcConsumer` 当:
/// - 需要在单线程内共享消费者
/// - 不需要线程安全
/// - 性能重要(避免锁开销)
/// - 单线程框架中的 UI 事件处理
/// - 构建复杂的单线程状态机
///
/// # 性能考虑
///
/// `RcConsumer` 在单线程场景下比 `ArcConsumer` 性能更好:
/// - **非原子计数**: clone/drop 比 `Arc` 更便宜
/// - **无锁开销**: `RefCell` 使用运行时检查，不用锁
/// - **更好的缓存局部性**: 无原子操作意味着更好的 CPU 缓存行为
///
/// 但相比 `BoxConsumer` 仍有轻微开销:
/// - **引用计数**: 虽非原子但仍存在
/// - **运行时借用检查**: `RefCell` 在运行时检查
///
/// # 安全性
///
/// `RcConsumer` 不是线程安全的，未实现 `Send` 或 `Sync`。
/// 尝试将其发送到另一个线程将导致编译错误。
/// 对于线程安全的共享，请改用 `ArcConsumer`。
///
/// # 示例
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcConsumer::new(move |x: &i32| {
///     l.borrow_mut().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.borrow(), vec![10]);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct RcConsumer<T> {
    function: Rc<RefCell<ConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> RcConsumer<T>
where
    T: 'static,
{
    /// 创建新的 RcConsumer
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
    /// 返回新的 `RcConsumer<T>` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcConsumer {
            function: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// 获取消费者的名称
    ///
    /// # 返回值
    ///
    /// 返回消费者的名称，如果未设置则返回 `None`
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 设置消费者的名称
    ///
    /// # 参数
    ///
    /// * `name` - 要设置的名称
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// 转换为闭包（不消费自身）
    ///
    /// 创建一个新的闭包，通过 Rc 调用底层函数。
    ///
    /// # 返回值
    ///
    /// 返回实现了 `FnMut(&T)` 的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T| {
            func.borrow_mut()(t);
        }
    }

    /// 顺序链接另一个 RcConsumer
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
    /// 返回新的组合 `RcConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = RcConsumer::new(move |x: &i32| {
    ///     l1.borrow_mut().push(*x * 2);
    /// });
    /// let second = RcConsumer::new(move |x: &i32| {
    ///     l2.borrow_mut().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first 和 second 在链接后仍可使用
    /// chained.accept(&5);
    /// assert_eq!(*log.borrow(), vec![10, 15]); // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcConsumer {
            function: Rc::new(RefCell::new(move |t: &T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
            name: None,
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.borrow_mut())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxConsumer::new(move |t| func.borrow_mut()(t))
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

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func.borrow_mut()(t);
        }
    }
}

impl<T> Clone for RcConsumer<T> {
    /// 克隆 RcConsumer
    ///
    /// 创建与原始实例共享底层函数的新 RcConsumer。
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for RcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for RcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcConsumer({})", name),
            None => write!(f, "RcConsumer"),
        }
    }
}

// ============================================================================
// 5. 为闭包实现 Consumer trait
// ============================================================================

/// 为所有 FnMut(&T) 实现 Consumer
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

    fn into_fn(self) -> impl FnMut(&T)
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

/// 为闭包提供消费者组合方法的扩展 trait
///
/// 为所有实现 `FnMut(&T)` 的闭包提供 `and_then` 和其他组合方法，
/// 使闭包无需显式包装类型即可直接进行方法链接。
///
/// # 设计理念
///
/// 此 trait 允许闭包使用方法语法自然地组合，类似于迭代器组合器。
/// 组合方法消耗闭包并返回 `BoxConsumer<T>`，可以继续链接。
///
/// # 特性
///
/// - **自然语法**: 直接在闭包上链接操作
/// - **返回 BoxConsumer**: 组合结果是 `BoxConsumer<T>`，可继续链接
/// - **零成本**: 组合闭包时无开销
/// - **自动实现**: 所有 `FnMut(&T)` 闭包自动获得这些方法
///
/// # 示例
///
/// ```rust
/// use prism3_function::{Consumer, FnConsumerOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut chained = (move |x: &i32| {
///     l1.lock().unwrap().push(*x * 2);
/// }).and_then(move |x: &i32| {
///     l2.lock().unwrap().push(*x + 10);
/// });
/// chained.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10, 15]); // (5 * 2), (5 + 10)
/// ```
///
/// # 作者
///
/// 胡海星
pub trait FnConsumerOps<T>: FnMut(&T) + Sized {
    /// 顺序链接另一个消费者
    ///
    /// 返回一个新的消费者，先执行当前操作，然后执行下一个操作。
    /// 消耗当前闭包并返回 `BoxConsumer<T>`。
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
    /// 返回组合的 `BoxConsumer<T>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnConsumerOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// }).and_then(|x: &i32| println!("Result: {}", x));
    ///
    /// chained.accept(&5); // Prints: Result: 5
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// 为所有闭包类型实现 FnConsumerOps
impl<T, F> FnConsumerOps<T> for F where F: FnMut(&T) {}
