/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Types
//!
//! Provides implementations of consumer interfaces for executing operations
//! that accept a single input parameter but return no result.
//!
//! This module provides a unified `Consumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxStatefulConsumer<T>`**: Box-based single ownership implementation for
//!   one-time use scenarios
//! - **`ArcStatefulConsumer<T>`**: Thread-safe shared ownership implementation
//!   based on Arc<Mutex<>>
//! - **`RcStatefulConsumer<T>`**: Single-threaded shared ownership implementation
//!   based on Rc<RefCell<>>
//!
//! # Design Philosophy
//!
//! Consumer uses `FnMut(&T)` semantics, allowing modification of its own state
//! but not the input value. Suitable for statistics, accumulation, event
//! handling, and other scenarios.
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

use crate::consumers::consumer_once::{
    BoxConsumerOnce,
    ConsumerOnce,
};
use crate::predicates::predicate::{
    ArcPredicate,
    BoxPredicate,
    Predicate,
    RcPredicate,
};

/// Type alias for consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and
/// returns nothing. It is used to reduce type complexity in struct definitions.
type ConsumerFn<T> = dyn FnMut(&T);

/// Type alias for thread-safe consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and
/// returns nothing, with Send bound for thread-safe usage. It is used to
/// reduce type complexity in Arc-based struct definitions.
type SendConsumerFn<T> = dyn FnMut(&T) + Send;

// ============================================================================
// 1. Consumer Trait - Unified Consumer Interface
// ============================================================================

/// Consumer trait - Unified consumer interface
///
/// Defines the core behavior of all consumer types. Similar to Java's
/// `Consumer<T>` interface, executes operations that accept a value but return
/// no result (side effects only).
///
/// Consumer can modify its own state (such as accumulation, counting), but
/// should not modify the consumed value itself.
///
/// # Automatic Implementation
///
/// - All closures implementing `FnMut(&T)`
/// - `BoxStatefulConsumer<T>`, `ArcStatefulConsumer<T>`, `RcStatefulConsumer<T>`
///
/// # Features
///
/// - **Unified Interface**: All consumer types share the same `accept` method
///   signature
/// - **Automatic Implementation**: Closures automatically implement this trait
///   with zero overhead
/// - **Type Conversion**: Easy conversion between different ownership models
/// - **Generic Programming**: Write functions that work with any consumer type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxStatefulConsumer, ArcStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: StatefulConsumer<i32>>(consumer: &mut C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// // Works with any consumer type
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut box_con = BoxStatefulConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// apply_consumer(&mut box_con, &5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait StatefulConsumer<T> {
    /// Execute consumption operation
    ///
    /// Performs an operation on the given reference. The operation typically
    /// reads the input value or produces side effects, but does not modify the
    /// input value itself. Can modify the consumer's own state.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    ///
    /// let mut consumer = BoxStatefulConsumer::new(|x: &i32| println!("{}", x));
    /// let value = 5;
    /// consumer.accept(&value);
    /// ```
    fn accept(&mut self, value: &T);

    /// Convert to BoxStatefulConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the current consumer to `BoxStatefulConsumer<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcStatefulConsumer`], [`RcStatefulConsumer`]),
    /// if you need to preserve the original object, you can call `.clone()`
    /// first.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxStatefulConsumer<T>`
    ///
    /// # Examples
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
    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut consumer = self;
        BoxStatefulConsumer::new(move |t| consumer.accept(t))
    }

    /// Convert to RcStatefulConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `RcStatefulConsumer<T>`
    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut consumer = self;
        RcStatefulConsumer::new(move |t| consumer.accept(t))
    }

    /// Convert to ArcStatefulConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `ArcStatefulConsumer<T>`
    fn into_arc(self) -> ArcStatefulConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: 'static,
    {
        let mut consumer = self;
        ArcStatefulConsumer::new(move |t| consumer.accept(t))
    }

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the consumer to a closure that can be used directly in standard
    /// library functions requiring `FnMut`.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnMut(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut func = consumer.into_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_fn(self) -> impl FnMut(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut consumer = self;
        move |t| consumer.accept(t)
    }

    /// Convert to BoxStatefulConsumer
    ///
    /// **⚠️ Requires Clone**: The original consumer must implement Clone.
    ///
    /// Converts the current consumer to `BoxStatefulConsumer<T>` by cloning it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer and
    /// then converts the clone to `BoxStatefulConsumer<T>`. The original consumer remains
    /// available after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxStatefulConsumer<T>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut box_consumer = consumer.to_box();
    /// box_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    /// ```
    fn to_box(&self) -> BoxStatefulConsumer<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Convert to RcStatefulConsumer
    ///
    /// **⚠️ Requires Clone**: The original consumer must implement Clone.
    ///
    /// Converts the current consumer to `RcStatefulConsumer<T>` by cloning it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer and
    /// then converts the clone to `RcStatefulConsumer<T>`. The original consumer remains
    /// available after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `RcStatefulConsumer<T>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut rc_consumer = consumer.to_rc();
    /// rc_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    /// ```
    fn to_rc(&self) -> RcStatefulConsumer<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Convert to ArcStatefulConsumer
    ///
    /// **⚠️ Requires Clone + Send**: The original consumer must implement
    /// Clone + Send.
    ///
    /// Converts the current consumer to `ArcStatefulConsumer<T>` by cloning it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer and
    /// then converts the clone to `ArcStatefulConsumer<T>`. The original consumer remains
    /// available after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `ArcStatefulConsumer<T>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let mut arc_consumer = consumer.to_arc();
    /// arc_consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.borrow(), vec![5, 3]);
    /// ```
    fn to_arc(&self) -> ArcStatefulConsumer<T>
    where
        Self: Sized + Clone + Send + 'static,
        T: 'static,
    {
        self.clone().into_arc()
    }

    /// Convert to closure
    ///
    /// **⚠️ Requires Clone**: The original consumer must implement Clone.
    ///
    /// Converts the consumer to a closure that can be used directly in standard
    /// library functions requiring `FnMut`.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer and
    /// then converts the clone to a closure. The original consumer remains
    /// available after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnMut(&T)` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut func = consumer.to_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    /// ```
    fn to_fn(&self) -> impl FnMut(&T)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone().into_fn()
    }
}

// ============================================================================
// 2. BoxStatefulConsumer - Single Ownership Implementation
// ============================================================================

/// BoxStatefulConsumer struct
///
/// Consumer implementation based on `Box<dyn FnMut(&T)>` for single ownership
/// scenarios. When sharing is not needed, this is the simplest and most
/// efficient consumer type.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership when used
/// - **Zero Overhead**: No reference counting or lock overhead
/// - **Mutable State**: Can modify captured environment through `FnMut`
/// - **Builder Pattern**: Method chaining naturally consumes `self`
///
/// # Use Cases
///
/// Choose `BoxStatefulConsumer` when:
/// - Consumer is used only once or in a linear flow
/// - Building pipelines where ownership flows naturally
/// - No need to share consumers across contexts
/// - Performance critical and cannot accept sharing overhead
///
/// # Performance
///
/// `BoxStatefulConsumer` has the best performance among the three consumer types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrowing checks
/// - Direct function calls through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulConsumer<T> {
    function: Box<dyn FnMut(&T)>,
    name: Option<String>,
}

impl<T> BoxStatefulConsumer<T>
where
    T: 'static,
{
    /// Create a new BoxStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `BoxStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxStatefulConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// Create a new named BoxStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `BoxStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxStatefulConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxStatefulConsumer {
            function: Box::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    ///
    /// let mut noop = BoxStatefulConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxStatefulConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then
    /// the next operation. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If you need
    ///   to preserve the original consumer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T|`
    ///   - A `BoxStatefulConsumer<T>`
    ///   - An `RcStatefulConsumer<T>`
    ///   - An `ArcStatefulConsumer<T>`
    ///   - Any type implementing `Consumer<T>`
    ///
    /// # Return Value
    ///
    /// Returns a new combined `BoxStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// });
    /// let second = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// // second is moved here
    /// let mut chained = first.and_then(second);
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// // second.accept(&3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// });
    /// let second = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// // Clone to preserve original
    /// let mut chained = first.and_then(second.clone());
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    ///
    /// // Original still usable
    /// second.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15, 13]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: StatefulConsumer<T> + 'static,
    {
        let mut first = self.function;
        let mut second = next;
        BoxStatefulConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// Creates a conditional consumer
    ///
    /// Returns a consumer that only executes when a predicate is satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Return Value
    ///
    /// Returns `BoxConditionalStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut conditional = consumer.when(|x: &i32| *x > 0);
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// conditional.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]); // Unchanged
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    ///
    /// // Clone to preserve original predicate
    /// let mut conditional = consumer.when(is_positive.clone());
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalStatefulConsumer<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalStatefulConsumer {
            consumer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T> StatefulConsumer<T> for BoxStatefulConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        let mut self_fn = self.function;
        RcStatefulConsumer::new(move |t| self_fn(t))
    }

    // do NOT override Consumer::into_arc() because BoxStatefulConsumer is not Send + Sync
    // and calling BoxStatefulConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        self.function
    }

    // do NOT override Consumer::to_xxx() because BoxStatefulConsumer is not Clone
    // and calling BoxStatefulConsumer::to_xxx() will cause a compile error
}

impl<T> fmt::Debug for BoxStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxConsumer({})", name),
            None => write!(f, "BoxConsumer"),
        }
    }
}

// ============================================================================
// 9. BoxStatefulConsumer ConsumerOnce Implementation
// ============================================================================

impl<T> ConsumerOnce<T> for BoxStatefulConsumer<T> {
    /// Execute one-time consumption operation
    ///
    /// Executes the consumer operation once and consumes self. This method
    /// provides a bridge between the reusable Consumer interface and the
    /// one-time ConsumerOnce interface.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// consumer.accept_once(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn accept_once(mut self, value: &T) {
        self.accept(value);
    }

    /// Convert to BoxConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the current consumer to `BoxConsumerOnce<T>` by wrapping the
    /// consumer's accept method in a FnOnce closure.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let box_consumer_once = consumer.into_box_once();
    /// box_consumer_once.accept_once(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box_once(self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        let mut consumer = self;
        BoxConsumerOnce::new(move |t| {
            consumer.accept(t);
        })
    }

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the consumer to a closure that can be used directly in places
    /// where the standard library requires `FnOnce`.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let func = consumer.into_fn_once();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_fn_once(self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        let mut consumer = self;
        move |t| consumer.accept(t)
    }

    // do NOT override ConsumerOnce::to_box_once() because BoxStatefulConsumer is not Clone
    // and calling BoxStatefulConsumer::to_box_once() will cause a compile error

    // do NOT override ConsumerOnce::to_fn_once() because BoxStatefulConsumer is not Clone
    // and calling BoxStatefulConsumer::to_fn_once() will cause a compile error
}

// ============================================================================
// 3. BoxConditionalStatefulConsumer - Box-based Conditional Consumer
// ============================================================================

/// BoxConditionalStatefulConsumer struct
///
/// A conditional consumer that only executes when a predicate is satisfied.
/// Uses `BoxStatefulConsumer` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxStatefulConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Consumer**: Can be used anywhere a `Consumer` is expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{Consumer, BoxStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let consumer = BoxStatefulConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// let mut conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![5]); // Executed
///
/// conditional.accept(&-5);
/// assert_eq!(*log.lock().unwrap(), vec![5]); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{Consumer, BoxStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
///     l1.lock().unwrap().push(*x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(move |x: &i32| {
///     l2.lock().unwrap().push(-*x);
/// });
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![5]); // when branch executed
///
/// consumer.accept(&-5);
/// assert_eq!(*log.lock().unwrap(), vec![5, 5]); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalStatefulConsumer<T> {
    consumer: BoxStatefulConsumer<T>,
    predicate: BoxPredicate<T>,
}

impl<T> StatefulConsumer<T> for BoxConditionalStatefulConsumer<T>
where
    T: 'static,
{
    fn accept(&mut self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    fn into_box(self) -> BoxStatefulConsumer<T> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer.accept(t);
            }
        })
    }

    fn into_rc(self) -> RcStatefulConsumer<T> {
        let pred = self.predicate.into_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer_fn.accept(t);
            }
        })
    }

    // do NOT override Consumer::into_arc() because BoxConditionalStatefulConsumer is not Send + Sync
    // and calling BoxConditionalStatefulConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T) {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T| {
            if pred.test(t) {
                consumer.accept(t);
            }
        }
    }

    // do NOT override Consumer::to_xxx() because BoxConditionalStatefulConsumer is not Clone
    // and calling BoxConditionalStatefulConsumer::to_xxx() will cause a compile error
}

impl<T> BoxConditionalStatefulConsumer<T>
where
    T: 'static,
{
    /// Chains another consumer in sequence
    ///
    /// Combines the current conditional consumer with another consumer into a new
    /// consumer. The current conditional consumer executes first, followed by the
    /// next consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The next consumer to execute
    ///
    /// # Returns
    ///
    /// Returns a new `BoxStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let cond1 = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).when(|x: &i32| *x > 0);
    /// let cond2 = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 100);
    /// }).when(|x: &i32| *x > 10);
    /// let mut chained = cond1.and_then(cond2);
    ///
    /// chained.accept(&6);
    /// assert_eq!(*log.lock().unwrap(), vec![12, 106]);
    /// // First *2 = 12, then +100 = 106
    /// ```
    pub fn and_then<C>(self, next: C) -> BoxStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxStatefulConsumer::new(move |t| {
            first.accept(t);
            second.accept(t);
        })
    }

    /// Adds an else branch
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch, can be:
    ///   - Closure: `|x: &T|`
    ///   - `BoxStatefulConsumer<T>`, `RcStatefulConsumer<T>`, `ArcStatefulConsumer<T>`
    ///   - Any type implementing `Consumer<T>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x);
    /// })
    /// .when(|x: &i32| *x > 0)
    /// .or_else(move |x: &i32| {
    ///     l2.lock().unwrap().push(-*x);
    /// });
    ///
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Condition satisfied, execute first
    ///
    /// consumer.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 5]);
    /// // Condition not satisfied, execute else
    /// ```
    pub fn or_else<C>(self, else_consumer: C) -> BoxStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + 'static,
    {
        let pred = self.predicate;
        let mut then_cons = self.consumer;
        let mut else_cons = else_consumer;
        BoxStatefulConsumer::new(move |t| {
            if pred.test(t) {
                then_cons.accept(t);
            } else {
                else_cons.accept(t);
            }
        })
    }
}

// ============================================================================
// 4. ArcStatefulConsumer - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcStatefulConsumer struct
///
/// Consumer implementation based on `Arc<Mutex<dyn FnMut(&T) + Send>>` for
/// thread-safe shared ownership scenarios. This consumer can be safely cloned
/// and shared across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allowing multiple owners
/// - **Thread Safety**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains
///   usable
/// - **Cross-Thread Sharing**: Can be sent to other threads and used
///
/// # Use Cases
///
/// Choose `ArcStatefulConsumer` when:
/// - Need to share consumers across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Need thread safety (Send + Sync)
///
/// # Performance Considerations
///
/// `ArcStatefulConsumer` has some performance overhead compared to `BoxStatefulConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread safety
/// is not needed, consider using `RcStatefulConsumer` for less single-threaded sharing
/// overhead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, ArcStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcStatefulConsumer<T> {
    function: Arc<Mutex<SendConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> ArcStatefulConsumer<T>
where
    T: 'static,
{
    /// Create a new ArcStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `ArcStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcStatefulConsumer {
            function: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// Create a new named ArcStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `ArcStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcStatefulConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcStatefulConsumer {
            function: Arc::new(Mutex::new(f)),
            name: Some(name.to_string()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    ///
    /// let mut noop = ArcStatefulConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        ArcStatefulConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the
    /// next operation. Borrows &self, does not consume the original consumer.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. Can be:
    ///   - A closure: `|x: &T|`
    ///   - A `BoxStatefulConsumer<T>`
    ///   - An `ArcStatefulConsumer<T>`
    ///   - An `RcStatefulConsumer<T>`
    ///   - Any type implementing `StatefulConsumer<T> + Send`
    ///
    /// # Return Value
    ///
    /// Returns a new combined `ArcStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// });
    /// let second = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then<C>(&self, mut next: C) -> ArcStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + Send + 'static,
    {
        let first = Arc::clone(&self.function);
        ArcStatefulConsumer::new(move |t: &T| {
            first.lock().unwrap()(t);
            next.accept(t);
        })
    }

    /// Creates a conditional consumer (thread-safe version)
    ///
    /// Returns a consumer that only executes when a predicate is satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, must be `Send + Sync`, can be:
    ///   - Closure: `|x: &T| -> bool` (requires `Send + Sync`)
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let conditional = consumer.when(|x: &i32| *x > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// let mut positive = 5;
    /// let mut m = conditional;
    /// m.accept(&positive);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalStatefulConsumer<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
        T: Send + Sync,
    {
        ArcConditionalStatefulConsumer {
            consumer: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T> StatefulConsumer<T> for ArcStatefulConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxStatefulConsumer::new(move |t| self_fn.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        RcStatefulConsumer::new(move |t| self_fn.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcStatefulConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let self_fn = self.function;
        move |t: &T| {
            self_fn.lock().unwrap()(t);
        }
    }

    fn to_box(&self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulConsumer::new(move |t| self_fn.lock().unwrap()(t))
    }

    fn to_rc(&self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        RcStatefulConsumer::new(move |t| self_fn.lock().unwrap()(t))
    }

    fn to_arc(&self) -> ArcStatefulConsumer<T>
    where
        T: 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut(&T) {
        let self_fn = self.function.clone();
        move |t| self_fn.lock().unwrap()(t)
    }
}

impl<T> Clone for ArcStatefulConsumer<T> {
    /// Clone ArcStatefulConsumer
    ///
    /// Creates a new ArcStatefulConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        ArcStatefulConsumer {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for ArcStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for ArcStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcConsumer({})", name),
            None => write!(f, "ArcConsumer"),
        }
    }
}

// ============================================================================
// 11. ArcStatefulConsumer ConsumerOnce Implementation
// ============================================================================

impl<T> ConsumerOnce<T> for ArcStatefulConsumer<T> {
    /// Execute one-time consumption operation
    ///
    /// Executes the consumer operation once and consumes self. This method
    /// provides a bridge between the reusable Consumer interface and the
    /// one-time ConsumerOnce interface.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// consumer.accept_once(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn accept_once(mut self, value: &T) {
        self.accept(value);
    }

    /// Convert to BoxConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the current consumer to `BoxConsumerOnce<T>` by wrapping the
    /// consumer's accept method in a FnOnce closure.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let box_consumer_once = consumer.into_box_once();
    /// box_consumer_once.accept_once(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box_once(self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        let mut consumer = self;
        BoxConsumerOnce::new(move |t| {
            consumer.accept(t);
        })
    }

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the consumer to a closure that can be used directly in places
    /// where the standard library requires `FnOnce`.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let func = consumer.into_fn_once();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_fn_once(self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        let mut consumer = self;
        move |t| consumer.accept(t)
    }

    /// Convert to BoxConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current consumer and wraps it in a
    /// `BoxConsumerOnce`.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let box_consumer_once = consumer.to_box_once();
    /// box_consumer_once.accept_once(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    /// ```
    fn to_box_once(&self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxConsumerOnce::new(move |t| {
            self_fn.lock().unwrap()(t);
        })
    }

    /// Convert to closure without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current consumer and then converts the clone
    /// to a closure.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let func = consumer.to_fn_once();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        move |t| self_fn.lock().unwrap()(t)
    }
}

// ============================================================================
// 5. ArcConditionalStatefulConsumer - Arc-based Conditional Consumer
// ============================================================================

/// ArcConditionalStatefulConsumer struct
///
/// A thread-safe conditional consumer that only executes when a predicate is
/// satisfied. Uses `ArcStatefulConsumer` and `ArcPredicate` for shared ownership across
/// threads.
///
/// This type is typically created by calling `ArcStatefulConsumer::when()` and is
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
/// use prism3_function::{Consumer, ArcStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let conditional = ArcStatefulConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// })
/// .when(|x: &i32| *x > 0);
///
/// let conditional_clone = conditional.clone();
///
/// let mut value = 5;
/// let mut m = conditional;
/// m.accept(&value);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalStatefulConsumer<T> {
    consumer: ArcStatefulConsumer<T>,
    predicate: ArcPredicate<T>,
}

impl<T> StatefulConsumer<T> for ArcConditionalStatefulConsumer<T>
where
    T: Send + 'static,
{
    fn accept(&mut self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer.accept(t);
            }
        })
    }

    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        let pred = self.predicate.to_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer_fn.accept(t);
            }
        })
    }

    fn into_arc(self) -> ArcStatefulConsumer<T>
    where
        T: Send + 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        ArcStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer.accept(t);
            }
        })
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T| {
            if pred.test(t) {
                consumer.accept(t);
            }
        }
    }

    // inherit the default implementation of to_xxx() from Consumer
}

impl<T> ArcConditionalStatefulConsumer<T>
where
    T: Send + 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch, can be:
    ///   - Closure: `|x: &T|` (must be `Send`)
    ///   - `ArcStatefulConsumer<T>`, `BoxStatefulConsumer<T>`
    ///   - Any type implementing `Consumer<T> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x);
    /// })
    /// .when(|x: &i32| *x > 0)
    /// .or_else(move |x: &i32| {
    ///     l2.lock().unwrap().push(-*x);
    /// });
    ///
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// consumer.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 5]);
    /// ```
    pub fn or_else<C>(&self, else_consumer: C) -> ArcStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + Send + 'static,
        T: Send + Sync,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        ArcStatefulConsumer::new(move |t: &T| {
            if pred.test(t) {
                then_cons.accept(t);
            } else {
                else_cons.accept(t);
            }
        })
    }
}

impl<T> Clone for ArcConditionalStatefulConsumer<T> {
    /// Clones the conditional consumer
    ///
    /// Creates a new instance that shares the underlying consumer and predicate
    /// with the original instance.
    fn clone(&self) -> Self {
        ArcConditionalStatefulConsumer {
            consumer: self.consumer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// 6. RcStatefulConsumer - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcStatefulConsumer struct
///
/// Consumer implementation based on `Rc<RefCell<dyn FnMut(&T)>>` for
/// single-threaded shared ownership scenarios. This consumer provides the
/// benefits of shared ownership without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allowing multiple owners
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrowing checks
/// - **No Lock Overhead**: More efficient than `ArcStatefulConsumer` for single-threaded
///   use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains
///   usable
///
/// # Use Cases
///
/// Choose `RcStatefulConsumer` when:
/// - Need to share consumers within a single thread
/// - Thread safety is not needed
/// - Performance is important (avoid lock overhead)
/// - UI event handling in single-threaded frameworks
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcStatefulConsumer` performs better than `ArcStatefulConsumer` in single-threaded scenarios:
/// - **Non-Atomic Counting**: clone/drop is cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checks, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU cache
///   behavior
///
/// But still has slight overhead compared to `BoxStatefulConsumer`:
/// - **Reference Counting**: Non-atomic but still exists
/// - **Runtime Borrowing Checks**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcStatefulConsumer` is not thread-safe and does not implement `Send` or `Sync`.
/// Attempting to send it to another thread will result in a compilation error.
/// For thread-safe sharing, use `ArcStatefulConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, RcStatefulConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
///     l.borrow_mut().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.borrow(), vec![10]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcStatefulConsumer<T> {
    function: Rc<RefCell<ConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> RcStatefulConsumer<T>
where
    T: 'static,
{
    /// Create a new RcStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `RcStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcStatefulConsumer {
            function: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// Create a new named RcStatefulConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `RcStatefulConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcStatefulConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.borrow_mut().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcStatefulConsumer {
            function: Rc::new(RefCell::new(f)),
            name: Some(name.to_string()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    ///
    /// let mut noop = RcStatefulConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        RcStatefulConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the
    /// next operation. Borrows &self, does not consume the original consumer.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. Can be:
    ///   - A closure: `|x: &T|`
    ///   - A `BoxStatefulConsumer<T>`
    ///   - An `RcStatefulConsumer<T>`
    ///   - An `ArcStatefulConsumer<T>`
    ///   - Any type implementing `StatefulConsumer<T>`
    ///
    /// # Return Value
    ///
    /// Returns a new combined `RcStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = RcStatefulConsumer::new(move |x: &i32| {
    ///     l1.borrow_mut().push(*x * 2);
    /// });
    /// let second = RcStatefulConsumer::new(move |x: &i32| {
    ///     l2.borrow_mut().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// assert_eq!(*log.borrow(), vec![10, 15]);
    /// // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then<C>(&self, mut next: C) -> RcStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + 'static,
    {
        let first = Rc::clone(&self.function);
        RcStatefulConsumer::new(move |t: &T| {
            first.borrow_mut()(t);
            next.accept(t);
        })
    }

    /// Creates a conditional consumer (single-threaded shared version)
    ///
    /// Returns a consumer that only executes when a predicate is satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, can be:
    ///   - Closure: `|x: &T| -> bool`
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `RcPredicate<T>`, `BoxPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `RcConditionalStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let conditional = consumer.when(|x: &i32| *x > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// let mut positive = 5;
    /// let mut m = conditional;
    /// m.accept(&positive);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    pub fn when<P>(&self, predicate: P) -> RcConditionalStatefulConsumer<T>
    where
        P: Predicate<T> + 'static,
    {
        RcConditionalStatefulConsumer {
            consumer: self.clone(),
            predicate: predicate.into_rc(),
        }
    }
}

impl<T> StatefulConsumer<T> for RcStatefulConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.borrow_mut())(value)
    }

    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxStatefulConsumer::new(move |t| self_fn.borrow_mut()(t))
    }

    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        self
    }

    //  do NOT override Consumer::into_arc() because RcStatefulConsumer is not Send + Sync
    // and calling RcStatefulConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let self_fn = self.function;
        move |t| self_fn.borrow_mut()(t)
    }

    fn to_box(&self) -> BoxStatefulConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulConsumer::new(move |t| self_fn.borrow_mut()(t))
    }

    fn to_rc(&self) -> RcStatefulConsumer<T>
    where
        T: 'static,
    {
        self.clone()
    }

    // do NOT override Consumer::to_arc() because RcStatefulConsumer is not Send + Sync
    // and calling RcStatefulConsumer::to_arc() will cause a compile error

    fn to_fn(&self) -> impl FnMut(&T) {
        let self_fn = self.function.clone();
        move |t| self_fn.borrow_mut()(t)
    }
}

impl<T> Clone for RcStatefulConsumer<T> {
    /// Clone RcStatefulConsumer
    ///
    /// Creates a new RcStatefulConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        RcStatefulConsumer {
            function: self.function.clone(),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for RcStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for RcStatefulConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcConsumer({})", name),
            None => write!(f, "RcConsumer"),
        }
    }
}

// ============================================================================
// 10. RcStatefulConsumer ConsumerOnce Implementation
// ============================================================================

impl<T> ConsumerOnce<T> for RcStatefulConsumer<T> {
    /// Execute one-time consumption operation
    ///
    /// Executes the consumer operation once and consumes self. This method
    /// provides a bridge between the reusable Consumer interface and the
    /// one-time ConsumerOnce interface.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// consumer.accept_once(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    fn accept_once(mut self, value: &T) {
        self.accept(value);
    }

    /// Convert to BoxConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the current consumer to `BoxConsumerOnce<T>` by wrapping the
    /// consumer's accept method in a FnOnce closure.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let box_consumer_once = consumer.into_box_once();
    /// box_consumer_once.accept_once(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    fn into_box_once(self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        let mut consumer = self;
        BoxConsumerOnce::new(move |t| {
            consumer.accept(t);
        })
    }

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts the consumer to a closure that can be used directly in places
    /// where the standard library requires `FnOnce`.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let func = consumer.into_fn_once();
    /// func(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    fn into_fn_once(self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        let mut consumer = self;
        move |t| consumer.accept(t)
    }

    /// Convert to BoxConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current consumer and wraps it in a
    /// `BoxConsumerOnce`.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let box_consumer_once = consumer.to_box_once();
    /// box_consumer_once.accept_once(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.borrow(), vec![5, 3]);
    /// ```
    fn to_box_once(&self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxConsumerOnce::new(move |t| {
            self_fn.borrow_mut()(t);
        })
    }

    /// Convert to closure without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current consumer and then converts the clone
    /// to a closure.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ConsumerOnce, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    /// let func = consumer.to_fn_once();
    /// func(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// // Original consumer still usable
    /// consumer.accept(&3);
    /// assert_eq!(*log.borrow(), vec![5, 3]);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        move |t| self_fn.borrow_mut()(t)
    }
}

// ============================================================================
// 7. RcConditionalStatefulConsumer - Rc-based Conditional Consumer
// ============================================================================

/// RcConditionalStatefulConsumer struct
///
/// A single-threaded conditional consumer that only executes when a predicate is
/// satisfied. Uses `RcStatefulConsumer` and `RcPredicate` for shared ownership within a
/// single thread.
///
/// This type is typically created by calling `RcStatefulConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalStatefulConsumer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, RcStatefulConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let conditional = RcStatefulConsumer::new(move |x: &i32| {
///     l.borrow_mut().push(*x);
/// })
/// .when(|x: &i32| *x > 0);
///
/// let conditional_clone = conditional.clone();
///
/// let mut value = 5;
/// let mut m = conditional;
/// m.accept(&value);
/// assert_eq!(*log.borrow(), vec![5]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalStatefulConsumer<T> {
    consumer: RcStatefulConsumer<T>,
    predicate: RcPredicate<T>,
}

impl<T> StatefulConsumer<T> for RcConditionalStatefulConsumer<T>
where
    T: 'static,
{
    fn accept(&mut self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    fn into_box(self) -> BoxStatefulConsumer<T> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer.accept(t);
            }
        })
    }

    fn into_rc(self) -> RcStatefulConsumer<T> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        RcStatefulConsumer::new(move |t| {
            if pred.test(t) {
                consumer.accept(t);
            }
        })
    }

    // do NOT override Consumer::into_arc() because RcConditionalStatefulConsumer is not Send + Sync
    // and calling RcConditionalStatefulConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T) {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        move |t: &T| {
            if pred.test(t) {
                consumer.accept(t);
            }
        }
    }

    // inherit the default implementation of to_xxx() from Consumer
}

impl<T> RcConditionalStatefulConsumer<T>
where
    T: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch, can be:
    ///   - Closure: `|x: &T|`
    ///   - `RcStatefulConsumer<T>`, `BoxStatefulConsumer<T>`
    ///   - Any type implementing `Consumer<T>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcStatefulConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
    ///     l1.borrow_mut().push(*x);
    /// })
    /// .when(|x: &i32| *x > 0)
    /// .or_else(move |x: &i32| {
    ///     l2.borrow_mut().push(-*x);
    /// });
    ///
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    ///
    /// consumer.accept(&-5);
    /// assert_eq!(*log.borrow(), vec![5, 5]);
    /// ```
    pub fn or_else<C>(&self, else_consumer: C) -> RcStatefulConsumer<T>
    where
        C: StatefulConsumer<T> + 'static,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        RcStatefulConsumer::new(move |t: &T| {
            if pred.test(t) {
                then_cons.accept(t);
            } else {
                else_cons.accept(t);
            }
        })
    }
}

impl<T> Clone for RcConditionalStatefulConsumer<T> {
    /// Clones the conditional consumer
    ///
    /// Creates a new instance that shares the underlying consumer and predicate
    /// with the original instance.
    fn clone(&self) -> Self {
        RcConditionalStatefulConsumer {
            consumer: self.consumer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// 8. Implement Consumer trait for closures
// ============================================================================

/// Implement Consumer for all FnMut(&T)
impl<T, F> StatefulConsumer<T> for F
where
    F: FnMut(&T),
{
    fn accept(&mut self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxStatefulConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxStatefulConsumer::new(self)
    }

    fn into_rc(self) -> RcStatefulConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcStatefulConsumer::new(self)
    }

    fn into_arc(self) -> ArcStatefulConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: 'static,
    {
        ArcStatefulConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxStatefulConsumer<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        BoxStatefulConsumer::new(cloned)
    }

    fn to_rc(&self) -> RcStatefulConsumer<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        RcStatefulConsumer::new(cloned)
    }

    fn to_arc(&self) -> ArcStatefulConsumer<T>
    where
        Self: Sized + Clone + Send + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        ArcStatefulConsumer::new(cloned)
    }

    fn to_fn(&self) -> impl FnMut(&T)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone()
    }
}

// ============================================================================
// 9. Extension methods for closures
// ============================================================================

/// Extension trait providing consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `FnMut(&T)`, allowing direct method chaining on closures
/// without explicit wrapper types.
///
/// # Design Philosophy
///
/// This trait allows closures to be naturally composed using method syntax,
/// similar to iterator combinators. Composition methods consume the closure and
/// return `BoxStatefulConsumer<T>`, which can continue chaining.
///
/// # Features
///
/// - **Natural Syntax**: Direct method chaining on closures
/// - **Returns BoxStatefulConsumer**: Composition results in `BoxStatefulConsumer<T>`, can
///   continue chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&T)` closures automatically get
///   these methods
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, FnStatefulConsumerOps};
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
/// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
/// // (5 * 2), (5 + 10)
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnStatefulConsumerOps<T>: FnMut(&T) + Sized {
    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the
    /// next operation. Consumes the current closure and returns `BoxStatefulConsumer<T>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If you need
    ///   to preserve the original consumer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T|`
    ///   - A `BoxStatefulConsumer<T>`
    ///   - An `RcStatefulConsumer<T>`
    ///   - An `ArcStatefulConsumer<T>`
    ///   - Any type implementing `Consumer<T>`
    ///
    /// # Return Value
    ///
    /// Returns a combined `BoxStatefulConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnStatefulConsumerOps, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let second = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// // second is moved here
    /// let mut chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(second);
    ///
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// // second.accept(&3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnStatefulConsumerOps, BoxStatefulConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let second = BoxStatefulConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// // Clone to preserve original
    /// let mut chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(second.clone());
    ///
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    ///
    /// // Original still usable
    /// second.accept(&3);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15, 13]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxStatefulConsumer<T>
    where
        Self: 'static,
        C: StatefulConsumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxStatefulConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// Implement FnStatefulConsumerOps for all closure types
impl<T, F> FnStatefulConsumerOps<T> for F where F: FnMut(&T) {}
