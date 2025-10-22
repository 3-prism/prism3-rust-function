/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiConsumer Types
//!
//! Provides bi-consumer interface implementations for operations accepting
//! two input parameters without returning a result.
//!
//! This module provides a unified `BiConsumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxBiConsumer<T, U>`**: Box-based single ownership for one-time use
//! - **`ArcBiConsumer<T, U>`**: Arc<Mutex<>>-based thread-safe shared
//!   ownership
//! - **`RcBiConsumer<T, U>`**: Rc<RefCell<>>-based single-threaded shared
//!   ownership
//!
//! # Design Philosophy
//!
//! BiConsumer uses `FnMut(&T, &U)` semantics: can modify its own state but
//! does NOT modify input values. Suitable for statistics, accumulation, and
//! event processing scenarios involving two parameters.
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::bi_predicate::{ArcBiPredicate, BiPredicate, BoxBiPredicate, RcBiPredicate};

/// Type alias for bi-consumer function to simplify complex types.
///
/// Represents a mutable function taking two references and returning
/// nothing. Used to reduce type complexity in struct definitions.
type BiConsumerFn<T, U> = dyn FnMut(&T, &U);

/// Type alias for thread-safe bi-consumer function.
///
/// Represents a mutable function with Send bound for thread-safe usage.
type SendBiConsumerFn<T, U> = dyn FnMut(&T, &U) + Send;

// =======================================================================
// 1. BiConsumer Trait - Unified BiConsumer Interface
// =======================================================================

/// BiConsumer trait - Unified bi-consumer interface
///
/// Defines core behavior for all bi-consumer types. Similar to Java's
/// `BiConsumer<T, U>` interface, performs operations accepting two values
/// but returning no result (side effects only).
///
/// BiConsumer can modify its own state (e.g., accumulate, count) but
/// should NOT modify the consumed values themselves.
///
/// # Automatic Implementations
///
/// - All closures implementing `FnMut(&T, &U)`
/// - `BoxBiConsumer<T, U>`, `ArcBiConsumer<T, U>`, `RcBiConsumer<T, U>`
///
/// # Features
///
/// - **Unified Interface**: All bi-consumer types share the same `accept`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions accepting any bi-consumer
///   type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer, ArcBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_bi_consumer<C: BiConsumer<i32, i32>>(
///     consumer: &mut C,
///     a: &i32,
///     b: &i32
/// ) {
///     consumer.accept(a, b);
/// }
///
/// // Works with any bi-consumer type
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut box_con = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// apply_bi_consumer(&mut box_con, &5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait BiConsumer<T, U> {
    /// Performs the consumption operation
    ///
    /// Executes an operation on the given two references. The operation
    /// typically reads input values or produces side effects, but does not
    /// modify the input values themselves. Can modify the consumer's own
    /// state.
    ///
    /// # Parameters
    ///
    /// * `first` - Reference to the first value to consume
    /// * `second` - Reference to the second value to consume
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn accept(&mut self, first: &T, second: &U);

    /// Converts to BoxBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the current bi-consumer to `BoxBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling, the original consumer is no longer available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcBiConsumer`],
    /// [`RcBiConsumer`]), call `.clone()` first if you need to keep the
    /// original.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumer;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// };
    /// let mut box_consumer = closure.into_box();
    /// box_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        BoxBiConsumer::new(move |t, u| consumer.accept(t, u))
    }

    /// Converts to RcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcBiConsumer<T, U>`
    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        RcBiConsumer::new(move |t, u| consumer.accept(t, u))
    }

    /// Converts to ArcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcBiConsumer<T, U>`
    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        let mut consumer = self;
        ArcBiConsumer::new(move |t, u| consumer.accept(t, u))
    }

    /// Converts bi-consumer to a closure
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the bi-consumer to a closure usable with standard library
    /// methods requiring `FnMut`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut func = consumer.into_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        move |t, u| consumer.accept(t, u)
    }

    /// Converts to BoxBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone**: Original consumer must implement Clone.
    ///
    /// Converts the current bi-consumer to `BoxBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `BoxBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut box_consumer = consumer.to_box();
    /// box_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    /// ```
    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_box()
    }

    /// Converts to RcBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone**: Original consumer must implement Clone.
    ///
    /// Converts the current bi-consumer to `RcBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `RcBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut rc_consumer = consumer.to_rc();
    /// rc_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    /// ```
    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts to ArcBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone + Send**: Original consumer must implement Clone +
    /// Send.
    ///
    /// Converts the current bi-consumer to `ArcBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `ArcBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    /// let mut arc_consumer = consumer.to_arc();
    /// arc_consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.borrow(), vec![8, 3]);
    /// ```
    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Clone + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Converts to closure (non-consuming)
    ///
    /// **⚠️ Requires Clone**: Original consumer must implement Clone.
    ///
    /// Converts the consumer to a closure that can be used directly in
    /// standard library functions requiring `FnMut`.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to a closure. The original consumer
    /// remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut func = consumer.to_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    /// ```
    fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_fn()
    }
}

// =======================================================================
// 2. BoxBiConsumer - Single Ownership Implementation
// =======================================================================

/// BoxBiConsumer struct
///
/// A bi-consumer implementation based on `Box<dyn FnMut(&T, &U)>` for
/// single ownership scenarios. This is the simplest and most efficient
/// bi-consumer type when sharing is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Builder Pattern**: Method chaining consumes `self` naturally
///
/// # Use Cases
///
/// Choose `BoxBiConsumer` when:
/// - The bi-consumer is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the consumer across contexts
/// - Performance is critical and sharing overhead is unacceptable
///
/// # Performance
///
/// `BoxBiConsumer` has the best performance among the three bi-consumer
/// types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxBiConsumer<T, U> {
    function: Box<BiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new BoxBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        BoxBiConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// Creates a new BoxBiConsumer with a name
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the consumer
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumer<T, U>` instance with the specified name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxBiConsumer::new_with_name("sum_logger", move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// assert_eq!(consumer.name(), Some("sum_logger"));
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        BoxBiConsumer {
            function: Box::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a no-op bi-consumer
    ///
    /// Returns a bi-consumer that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut noop = BoxBiConsumer::<i32, i32>::noop();
    /// noop.accept(&42, &10);
    /// // Values unchanged
    /// ```
    pub fn noop() -> Self {
        BoxBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation. **Note:
    ///   This parameter is passed by value and will transfer ownership.** If you
    ///   need to preserve the original consumer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// });
    /// let second = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // second is moved here
    /// let mut chained = first.and_then(second);
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// // second.accept(&2, &3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// });
    /// let second = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // Clone to preserve original
    /// let mut chained = first.and_then(second.clone());
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    ///
    /// // Original still usable
    /// second.accept(&2, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15, 6]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumer<T, U> + 'static,
    {
        let mut first = self.function;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }

    /// Creates a conditional bi-consumer
    ///
    /// Returns a bi-consumer that only executes when a predicate is satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalBiConsumer<T, U>`
    pub fn when<P>(self, predicate: P) -> BoxConditionalBiConsumer<T, U>
    where
        P: BiPredicate<T, U> + 'static,
    {
        BoxConditionalBiConsumer {
            consumer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, U> BiConsumer<T, U> for BoxBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let mut func = self.function;
        RcBiConsumer::new(move |t, u| func(t, u))
    }

    // do NOT override BiConsumer::into_arc() because BoxBiConsumer is not Send + Sync
    // and calling BoxBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }

    // do NOT override BiConsumer::to_xxx() because BoxBiConsumer is not Clone
    // and calling BoxBiConsumer::to_xxx() will cause a compile error
}

impl<T, U> fmt::Debug for BoxBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for BoxBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxBiConsumer({})", name),
            None => write!(f, "BoxBiConsumer"),
        }
    }
}

// =======================================================================
// 3. BoxConditionalBiConsumer - Box-based Conditional BiConsumer
// =======================================================================

/// BoxConditionalBiConsumer struct
///
/// A conditional bi-consumer that only executes when a predicate is satisfied.
/// Uses `BoxBiConsumer` and `BoxBiPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxBiConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements BiConsumer**: Can be used anywhere a `BiConsumer` is expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
///
/// conditional.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]); // Executed
///
/// conditional.accept(&-5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l1.lock().unwrap().push(*x + *y);
/// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
///   .or_else(move |x: &i32, y: &i32| {
///     l2.lock().unwrap().push(*x * *y);
/// });
///
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]); // when branch executed
///
/// consumer.accept(&-5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8, -15]); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalBiConsumer<T, U> {
    consumer: BoxBiConsumer<T, U>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U> BiConsumer<T, U> for BoxConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcBiConsumer<T, U> {
        let pred = self.predicate.into_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer_fn.accept(t, u);
            }
        })
    }

    // do NOT override BiConsumer::into_arc() because BoxConditionalBiConsumer is not Send + Sync
    // and calling BoxConditionalBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T, &U) {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T, u: &U| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        }
    }

    // do NOT override BiConsumer::to_xxx() because BoxConditionalBiConsumer is not Clone
    // and calling BoxConditionalBiConsumer::to_xxx() will cause a compile error
}

impl<T, U> BoxConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Chains another consumer in sequence
    ///
    /// Combines the current conditional consumer with another consumer into a new
    /// consumer. The current conditional consumer executes first, followed by the
    /// next consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The next consumer to execute. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original consumer, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let cond = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    /// let second = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // second is moved here
    /// let mut chained = cond.and_then(second);
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// // second.accept(&2, &3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let cond = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    /// let second = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // Clone to preserve original
    /// let mut chained = cond.and_then(second.clone());
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    ///
    /// // Original still usable
    /// second.accept(&2, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15, 6]);
    /// ```
    pub fn and_then<C>(self, next: C) -> BoxBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first.accept(t, u);
            second.accept(t, u);
        })
    }

    /// Adds an else branch
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to preserve
    ///   the original consumer, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///   .or_else(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]); // Condition satisfied
    ///
    /// consumer.accept(&-5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, -15]); // Condition not satisfied
    /// ```
    pub fn or_else<C>(self, else_consumer: C) -> BoxBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + 'static,
    {
        let pred = self.predicate;
        let mut then_cons = self.consumer;
        let mut else_cons = else_consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

// =======================================================================
// 4. ArcBiConsumer - Thread-Safe Shared Ownership Implementation
// =======================================================================

/// ArcBiConsumer struct
///
/// A bi-consumer implementation based on
/// `Arc<Mutex<dyn FnMut(&T, &U) + Send>>` for thread-safe shared
/// ownership scenarios. This consumer can be safely cloned and shared
/// across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
/// - **Cross-Thread Sharing**: Can be sent to and used by other threads
///
/// # Use Cases
///
/// Choose `ArcBiConsumer` when:
/// - Need to share bi-consumer across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Thread safety (Send + Sync) is required
///
/// # Performance Considerations
///
/// `ArcBiConsumer` has some overhead compared to `BoxBiConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread
/// safety is not needed, consider using `RcBiConsumer` for lower
/// overhead in single-threaded sharing.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcBiConsumer<T, U> {
    function: Arc<Mutex<SendBiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> ArcBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    /// Creates a new ArcBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + Send + 'static,
    {
        ArcBiConsumer {
            function: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// Creates a new ArcBiConsumer with a name
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the consumer
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcBiConsumer<T, U>` instance with the specified name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcBiConsumer::new_with_name("sum_logger", move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// assert_eq!(consumer.name(), Some("sum_logger"));
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T, &U) + Send + 'static,
    {
        ArcBiConsumer {
            function: Arc::new(Mutex::new(f)),
            name: Some(name.to_string()),
        }
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Converts to a closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function via Arc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = Arc::clone(&self.function);
        move |t: &T, u: &U| {
            func.lock().unwrap()(t, u);
        }
    }

    /// Chains another ArcBiConsumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation. **Note:
    ///   This parameter is passed by reference, so the original consumer remains
    ///   usable.** Can be:
    ///   - An `ArcBiConsumer<T, U>` (passed by reference)
    ///   - Any type implementing `BiConsumer<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// });
    /// let second = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // second is passed by reference, so it remains usable
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// // second.accept(&2, &3); // Still usable
    /// ```
    pub fn and_then(&self, next: &ArcBiConsumer<T, U>) -> ArcBiConsumer<T, U> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcBiConsumer {
            function: Arc::new(Mutex::new(move |t: &T, u: &U| {
                first.lock().unwrap()(t, u);
                second.lock().unwrap()(t, u);
            })),
            name: None,
        }
    }

    /// Creates a conditional bi-consumer (thread-safe version)
    ///
    /// Returns a bi-consumer that only executes when a predicate is satisfied.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`).
    ///   Must be `Send + Sync`, can be:
    ///   - A closure: `|x: &T, y: &U| -> bool` (requires `Send + Sync`)
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// let mut positive = 5;
    /// let mut m = conditional;
    /// m.accept(&positive, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalBiConsumer<T, U>
    where
        P: BiPredicate<T, U> + Send + Sync + 'static,
        T: Send + Sync,
        U: Send + Sync,
    {
        ArcConditionalBiConsumer {
            consumer: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, U> BiConsumer<T, U> for ArcBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.lock().unwrap())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        BoxBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        RcBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        move |t, u| self_fn.lock().unwrap()(t, u)
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        RcBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        move |t, u| self_fn.lock().unwrap()(t, u)
    }
}

impl<T, U> Clone for ArcBiConsumer<T, U> {
    /// Clones the ArcBiConsumer
    ///
    /// Creates a new ArcBiConsumer sharing the underlying function with
    /// the original instance.
    fn clone(&self) -> Self {
        ArcBiConsumer {
            function: self.function.clone(),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for ArcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for ArcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcBiConsumer({})", name),
            None => write!(f, "ArcBiConsumer"),
        }
    }
}

// =======================================================================
// 5. ArcConditionalBiConsumer - Arc-based Conditional BiConsumer
// =======================================================================

/// ArcConditionalBiConsumer struct
///
/// A thread-safe conditional bi-consumer that only executes when a predicate is
/// satisfied. Uses `ArcBiConsumer` and `ArcBiPredicate` for shared ownership across
/// threads.
///
/// This type is typically created by calling `ArcBiConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let conditional = ArcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
///
/// let conditional_clone = conditional.clone();
///
/// let mut value = 5;
/// let mut m = conditional;
/// m.accept(&value, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalBiConsumer<T, U> {
    consumer: ArcBiConsumer<T, U>,
    predicate: ArcBiPredicate<T, U>,
}

impl<T, U> BiConsumer<T, U> for ArcConditionalBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let pred = self.predicate.to_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer_fn.accept(t, u);
            }
        })
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        ArcBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T, u: &U| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        }
    }

    // Use the default implementation of to_xxx() from BiConsumer
}

impl<T, U> ArcConditionalBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to preserve
    ///   the original consumer, clone it first (if it implements `Clone`).
    ///   Must be `Send`, can be:
    ///   - A closure: `|x: &T, y: &U|` (must be `Send`)
    ///   - An `ArcBiConsumer<T, U>`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///   .or_else(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    ///
    /// consumer.accept(&-5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, -15]);
    /// ```
    pub fn or_else<C>(&self, else_consumer: C) -> ArcBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + Send + 'static,
        T: Send + Sync,
        U: Send + Sync,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        ArcBiConsumer::new(move |t: &T, u: &U| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

impl<T, U> Clone for ArcConditionalBiConsumer<T, U> {
    /// Clones the conditional consumer
    ///
    /// Creates a new instance that shares the underlying consumer and predicate
    /// with the original instance.
    fn clone(&self) -> Self {
        ArcConditionalBiConsumer {
            consumer: self.consumer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// =======================================================================
// 6. RcBiConsumer - Single-Threaded Shared Ownership Implementation
// =======================================================================

/// RcBiConsumer struct
///
/// A bi-consumer implementation based on `Rc<RefCell<dyn FnMut(&T, &U)>>`
/// for single-threaded shared ownership scenarios. This consumer provides
/// the benefits of shared ownership without the overhead of thread
/// safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot send across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **No Lock Overhead**: More efficient than `ArcBiConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Use Cases
///
/// Choose `RcBiConsumer` when:
/// - Need to share bi-consumer within a single thread
/// - Thread safety is not needed
/// - Performance matters (avoiding lock overhead)
/// - Single-threaded UI framework event handling
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcBiConsumer` performs better than `ArcBiConsumer` in single-threaded
/// scenarios:
/// - **Non-Atomic Counting**: clone/drop cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checking, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU
///   cache behavior
///
/// But still has slight overhead compared to `BoxBiConsumer`:
/// - **Reference Counting**: Though non-atomic, still exists
/// - **Runtime Borrow Checking**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcBiConsumer` is not thread-safe and does not implement `Send` or
/// `Sync`. Attempting to send it to another thread will result in a
/// compile error. For thread-safe sharing, use `ArcBiConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.borrow_mut().push(*x + *y);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// assert_eq!(*log.borrow(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcBiConsumer<T, U> {
    function: Rc<RefCell<BiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> RcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new RcBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        RcBiConsumer {
            function: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// Creates a new RcBiConsumer with a name
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the consumer
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcBiConsumer<T, U>` instance with the specified name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcBiConsumer::new_with_name("sum_logger", move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    /// assert_eq!(consumer.name(), Some("sum_logger"));
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        RcBiConsumer {
            function: Rc::new(RefCell::new(f)),
            name: Some(name.to_string()),
        }
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Converts to a closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function via Rc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T, u: &U| {
            func.borrow_mut()(t, u);
        }
    }

    /// Chains another RcBiConsumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.borrow_mut().push(*x + *y);
    /// });
    /// let second = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.borrow_mut().push(*x * *y);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8, 15]);
    /// ```
    pub fn and_then(&self, next: &RcBiConsumer<T, U>) -> RcBiConsumer<T, U> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcBiConsumer {
            function: Rc::new(RefCell::new(move |t: &T, u: &U| {
                first.borrow_mut()(t, u);
                second.borrow_mut()(t, u);
            })),
            name: None,
        }
    }

    /// Creates a conditional bi-consumer (single-threaded shared version)
    ///
    /// Returns a bi-consumer that only executes when a predicate is satisfied.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - An `RcBiPredicate<T, U>`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// Returns `RcConditionalBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    /// let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// let mut positive = 5;
    /// let mut m = conditional;
    /// m.accept(&positive, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// ```
    pub fn when<P>(&self, predicate: P) -> RcConditionalBiConsumer<T, U>
    where
        P: BiPredicate<T, U> + 'static,
    {
        RcConditionalBiConsumer {
            consumer: self.clone(),
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, U> BiConsumer<T, U> for RcBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.borrow_mut())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        BoxBiConsumer::new(move |t, u| self_fn.borrow_mut()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    // do NOT override BiConsumer::into_arc() because RcBiConsumer is not Send + Sync
    // and calling RcBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        move |t, u| self_fn.borrow_mut()(t, u)
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiConsumer::new(move |t, u| self_fn.borrow_mut()(t, u))
    }

    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self.clone()
    }

    // do NOT override BiConsumer::to_arc() because RcBiConsumer is not Send + Sync
    // and calling RcBiConsumer::to_arc() will cause a compile error

    fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        move |t, u| self_fn.borrow_mut()(t, u)
    }
}

impl<T, U> Clone for RcBiConsumer<T, U> {
    /// Clones the RcBiConsumer
    ///
    /// Creates a new RcBiConsumer sharing the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        RcBiConsumer {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for RcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for RcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcBiConsumer({})", name),
            None => write!(f, "RcBiConsumer"),
        }
    }
}

// =======================================================================
// 7. RcConditionalBiConsumer - Rc-based Conditional BiConsumer
// =======================================================================

/// RcConditionalBiConsumer struct
///
/// A single-threaded conditional bi-consumer that only executes when a predicate is
/// satisfied. Uses `RcBiConsumer` and `RcBiPredicate` for shared ownership within a
/// single thread.
///
/// This type is typically created by calling `RcBiConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalBiConsumer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let conditional = RcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.borrow_mut().push(*x + *y);
/// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
///
/// let conditional_clone = conditional.clone();
///
/// let mut value = 5;
/// let mut m = conditional;
/// m.accept(&value, &3);
/// assert_eq!(*log.borrow(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalBiConsumer<T, U> {
    consumer: RcBiConsumer<T, U>,
    predicate: RcBiPredicate<T, U>,
}

impl<T, U> BiConsumer<T, U> for RcConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        RcBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    // do NOT override BiConsumer::into_arc() because RcConditionalBiConsumer is not Send + Sync
    // and calling RcConditionalBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T, &U) {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T, u: &U| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        }
    }

    // Use the default implementation of to_xxx() from BiConsumer
}

impl<T, U> RcConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to preserve
    ///   the original consumer, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - An `RcBiConsumer<T, U>`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.borrow_mut().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///   .or_else(move |x: &i32, y: &i32| {
    ///     l2.borrow_mut().push(*x * *y);
    /// });
    ///
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    ///
    /// consumer.accept(&-5, &3);
    /// assert_eq!(*log.borrow(), vec![8, -15]);
    /// ```
    pub fn or_else<C>(&self, else_consumer: C) -> RcBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + 'static,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        RcBiConsumer::new(move |t: &T, u: &U| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

impl<T, U> Clone for RcConditionalBiConsumer<T, U> {
    /// Clones the conditional consumer
    ///
    /// Creates a new instance that shares the underlying consumer and predicate
    /// with the original instance.
    fn clone(&self) -> Self {
        RcConditionalBiConsumer {
            consumer: self.consumer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// =======================================================================
// 9. Implement BiConsumer trait for closures
// =======================================================================

/// Implements BiConsumer for all FnMut(&T, &U)
impl<T, U, F> BiConsumer<T, U> for F
where
    F: FnMut(&T, &U),
{
    fn accept(&mut self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(self)
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcBiConsumer::new(self)
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        ArcBiConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let cloned = self.clone();
        BoxBiConsumer::new(cloned)
    }

    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let cloned = self.clone();
        RcBiConsumer::new(cloned)
    }

    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Clone + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        let cloned = self.clone();
        ArcBiConsumer::new(cloned)
    }

    fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone()
    }
}

// =======================================================================
// 10. Provide extension methods for closures
// =======================================================================

/// Extension trait providing bi-consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `FnMut(&T, &U)`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Design Rationale
///
/// This trait allows closures to be composed naturally using method
/// syntax, similar to iterator combinators. Composition methods consume
/// the closure and return `BoxBiConsumer<T, U>`, which can be further
/// chained.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxBiConsumer**: Composition results are
///   `BoxBiConsumer<T, U>` for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&T, &U)` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, FnBiConsumerOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut chained = (move |x: &i32, y: &i32| {
///     l1.lock().unwrap().push(*x + *y);
/// }).and_then(move |x: &i32, y: &i32| {
///     l2.lock().unwrap().push(*x * *y);
/// });
/// chained.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnBiConsumerOps<T, U>: FnMut(&T, &U) + Sized {
    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes the current closure and returns
    /// `BoxBiConsumer<T, U>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation. **Note:
    ///   This parameter is passed by value and will transfer ownership.** If you
    ///   need to preserve the original consumer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, FnBiConsumerOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = (move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).and_then(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// }).and_then(|x: &i32, y: &i32| println!("Result: {}, {}", x, y));
    ///
    /// chained.accept(&5, &3); // Prints: Result: 5, 3
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxBiConsumer<T, U>
    where
        Self: 'static,
        C: BiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnBiConsumerOps for all closure types
impl<T, U, F> FnBiConsumerOps<T, U> for F where F: FnMut(&T, &U) {}
