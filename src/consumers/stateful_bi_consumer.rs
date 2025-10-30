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
//! - **`BoxStatefulBiConsumer<T, U>`**: Box-based single ownership for one-time use
//! - **`ArcStatefulBiConsumer<T, U>`**: Arc<Mutex<>>-based thread-safe shared
//!   ownership
//! - **`RcStatefulBiConsumer<T, U>`**: Rc<RefCell<>>-based single-threaded shared
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
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

use crate::consumers::macros::{
    impl_box_consumer_methods,
    impl_conditional_consumer_clone,
    impl_conditional_consumer_debug_display,
    impl_consumer_clone,
    impl_consumer_debug_display,
    impl_consumer_common_methods,
    impl_shared_consumer_methods,
};
use crate::predicates::bi_predicate::{
    ArcBiPredicate,
    BiPredicate,
    BoxBiPredicate,
    RcBiPredicate,
};

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
/// - `BoxStatefulBiConsumer<T, U>`, `ArcStatefulBiConsumer<T, U>`, `RcStatefulBiConsumer<T, U>`
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
/// use prism3_function::{BiConsumer, BoxStatefulBiConsumer, ArcStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_bi_consumer<C: StatefulBiConsumer<i32, i32>>(
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
/// let mut box_con = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// apply_bi_consumer(&mut box_con, &5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait StatefulBiConsumer<T, U> {
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
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn accept(&mut self, first: &T, second: &U);

    /// Converts to BoxStatefulBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the current bi-consumer to `BoxStatefulBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling, the original consumer is no longer available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcStatefulBiConsumer`],
    /// [`RcStatefulBiConsumer`]), call `.clone()` first if you need to keep the
    /// original.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxStatefulBiConsumer<T, U>`
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
    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        BoxStatefulBiConsumer::new(move |t, u| consumer.accept(t, u))
    }

    /// Converts to RcStatefulBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcStatefulBiConsumer<T, U>`
    fn into_rc(self) -> RcStatefulBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        RcStatefulBiConsumer::new(move |t, u| consumer.accept(t, u))
    }

    /// Converts to ArcStatefulBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcStatefulBiConsumer<T, U>`
    fn into_arc(self) -> ArcStatefulBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: 'static,
        U: 'static,
    {
        let mut consumer = self;
        ArcStatefulBiConsumer::new(move |t, u| consumer.accept(t, u))
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
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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

    /// Converts to BoxStatefulBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone**: Original consumer must implement Clone.
    ///
    /// Converts the current bi-consumer to `BoxStatefulBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `BoxStatefulBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxStatefulBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut box_consumer = consumer.to_box();
    /// box_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    /// ```
    fn to_box(&self) -> BoxStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_box()
    }

    /// Converts to RcStatefulBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone**: Original consumer must implement Clone.
    ///
    /// Converts the current bi-consumer to `RcStatefulBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `RcStatefulBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcStatefulBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut rc_consumer = consumer.to_rc();
    /// rc_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    /// ```
    fn to_rc(&self) -> RcStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts to ArcStatefulBiConsumer (non-consuming)
    ///
    /// **⚠️ Requires Clone + Send**: Original consumer must implement Clone +
    /// Send.
    ///
    /// Converts the current bi-consumer to `ArcStatefulBiConsumer<T, U>` by cloning
    /// it first.
    ///
    /// # Ownership
    ///
    /// This method does **not consume** the consumer. It clones the consumer
    /// and then converts the clone to `ArcStatefulBiConsumer<T, U>`. The original
    /// consumer remains available after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcStatefulBiConsumer<T, U>` from the clone
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcStatefulBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    /// let mut arc_consumer = consumer.to_arc();
    /// arc_consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// // Original consumer still usable
    /// consumer.accept(&2, &1);
    /// assert_eq!(*log.borrow(), vec![8, 3]);
    /// ```
    fn to_arc(&self) -> ArcStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + Send + 'static,
        T: 'static,
        U: 'static,
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
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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

    /// Convert to BiConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts a reusable stateful bi-consumer to a one-time consumer that consumes itself on use.
    /// This enables passing `StatefulBiConsumer` to functions that require `BiConsumerOnce`.
    ///
    /// # Returns
    ///
    /// Returns a `BoxBiConsumerOnce<T, U>`
    fn into_once(self) -> crate::consumers::bi_consumer_once::BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        crate::consumers::bi_consumer_once::BoxBiConsumerOnce::new(move |t, u| {
            let mut consumer = self;
            consumer.accept(t, u);
        })
    }

    /// Convert to BiConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement `Clone`.
    /// Clones the current consumer and converts the clone to a one-time consumer.
    ///
    /// # Returns
    ///
    /// Returns a `BoxBiConsumerOnce<T, U>`
    fn to_once(&self) -> crate::consumers::bi_consumer_once::BoxBiConsumerOnce<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_once()
    }
}

// =======================================================================
// 2. BoxStatefulBiConsumer - Single Ownership Implementation
// =======================================================================

/// BoxStatefulBiConsumer struct
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
/// Choose `BoxStatefulBiConsumer` when:
/// - The bi-consumer is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the consumer across contexts
/// - Performance is critical and sharing overhead is unacceptable
///
/// # Performance
///
/// `BoxStatefulBiConsumer` has the best performance among the three bi-consumer
/// types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulBiConsumer<T, U> {
    function: Box<BiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxStatefulBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        BoxStatefulBiConsumer<T, U>,
        (FnMut(&T, &U) + 'static),
        |f| Box::new(f)
    );

    // Generates: when() and and_then() methods that consume self
    impl_box_consumer_methods!(
        BoxStatefulBiConsumer<T, U>,
        BoxConditionalStatefulBiConsumer,
        StatefulBiConsumer
    );
}

impl<T, U> StatefulBiConsumer<T, U> for BoxStatefulBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let mut func = self.function;
        RcStatefulBiConsumer::new(move |t, u| func(t, u))
    }

    // do NOT override BiConsumer::into_arc() because BoxStatefulBiConsumer is not Send + Sync
    // and calling BoxStatefulBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }

    // do NOT override BiConsumer::to_xxx() because BoxStatefulBiConsumer is not Clone
    // and calling BoxStatefulBiConsumer::to_xxx() will cause a compile error
}

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(BoxStatefulBiConsumer<T, U>);

// =======================================================================
// 3. ArcStatefulBiConsumer - Thread-Safe Shared Ownership Implementation
// =======================================================================

/// ArcStatefulBiConsumer struct
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
/// Choose `ArcStatefulBiConsumer` when:
/// - Need to share bi-consumer across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Thread safety (Send + Sync) is required
///
/// # Performance Considerations
///
/// `ArcStatefulBiConsumer` has some overhead compared to `BoxStatefulBiConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread
/// safety is not needed, consider using `RcStatefulBiConsumer` for lower
/// overhead in single-threaded sharing.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
pub struct ArcStatefulBiConsumer<T, U> {
    function: Arc<Mutex<SendBiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> ArcStatefulBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        ArcStatefulBiConsumer<T, U>,
        (FnMut(&T, &U) + Send + 'static),
        |f| Arc::new(Mutex::new(f))
    );

    // Generates: when() and and_then() methods that borrow &self (Arc can clone)
    impl_shared_consumer_methods!(
        ArcStatefulBiConsumer<T, U>,
        ArcConditionalStatefulBiConsumer,
        into_arc,
        StatefulBiConsumer,
        Send + Sync + 'static
    );
}

impl<T, U> StatefulBiConsumer<T, U> for ArcStatefulBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.lock().unwrap())(first, second)
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        BoxStatefulBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        RcStatefulBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn into_arc(self) -> ArcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
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

    fn to_box(&self) -> BoxStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn to_rc(&self) -> RcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        RcStatefulBiConsumer::new(move |t, u| self_fn.lock().unwrap()(t, u))
    }

    fn to_arc(&self) -> ArcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
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

// Use macro to generate Clone implementation
impl_consumer_clone!(ArcStatefulBiConsumer<T, U>, Arc::clone);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(ArcStatefulBiConsumer<T, U>);

// =======================================================================
// 4. RcStatefulBiConsumer - Single-Threaded Shared Ownership Implementation
// =======================================================================

/// RcStatefulBiConsumer struct
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
/// - **No Lock Overhead**: More efficient than `ArcStatefulBiConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Use Cases
///
/// Choose `RcStatefulBiConsumer` when:
/// - Need to share bi-consumer within a single thread
/// - Thread safety is not needed
/// - Performance matters (avoiding lock overhead)
/// - Single-threaded UI framework event handling
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcStatefulBiConsumer` performs better than `ArcStatefulBiConsumer` in single-threaded
/// scenarios:
/// - **Non-Atomic Counting**: clone/drop cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checking, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU
///   cache behavior
///
/// But still has slight overhead compared to `BoxStatefulBiConsumer`:
/// - **Reference Counting**: Though non-atomic, still exists
/// - **Runtime Borrow Checking**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcStatefulBiConsumer` is not thread-safe and does not implement `Send` or
/// `Sync`. Attempting to send it to another thread will result in a
/// compile error. For thread-safe sharing, use `ArcStatefulBiConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcStatefulBiConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
pub struct RcStatefulBiConsumer<T, U> {
    function: Rc<RefCell<BiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> RcStatefulBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        RcStatefulBiConsumer<T, U>,
        (FnMut(&T, &U) + 'static),
        |f| Rc::new(RefCell::new(f))
    );

    // Generates: when() and and_then() methods that borrow &self (Rc can clone)
    impl_shared_consumer_methods!(
        RcStatefulBiConsumer<T, U>,
        RcConditionalStatefulBiConsumer,
        into_rc,
        StatefulBiConsumer,
        'static
    );
}

impl<T, U> StatefulBiConsumer<T, U> for RcStatefulBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.borrow_mut())(first, second)
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        BoxStatefulBiConsumer::new(move |t, u| self_fn.borrow_mut()(t, u))
    }

    fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T, u: &U| {
            func.borrow_mut()(t, u);
        }
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(RcStatefulBiConsumer<T, U>, Rc::clone);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(RcStatefulBiConsumer<T, U>);

// =======================================================================
// 5. Implement BiConsumer trait for closures
// =======================================================================

/// Implements BiConsumer for all FnMut(&T, &U)
impl<T, U, F> StatefulBiConsumer<T, U> for F
where
    F: FnMut(&T, &U),
{
    fn accept(&mut self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxStatefulBiConsumer::new(self)
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcStatefulBiConsumer::new(self)
    }

    fn into_arc(self) -> ArcStatefulBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: 'static,
        U: 'static,
    {
        ArcStatefulBiConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let cloned = self.clone();
        BoxStatefulBiConsumer::new(cloned)
    }

    fn to_rc(&self) -> RcStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let cloned = self.clone();
        RcStatefulBiConsumer::new(cloned)
    }

    fn to_arc(&self) -> ArcStatefulBiConsumer<T, U>
    where
        Self: Sized + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        let cloned = self.clone();
        ArcStatefulBiConsumer::new(cloned)
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
// 6. Provide extension methods for closures
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
/// the closure and return `BoxStatefulBiConsumer<T, U>`, which can be further
/// chained.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxStatefulBiConsumer**: Composition results are
///   `BoxStatefulBiConsumer<T, U>` for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&T, &U)` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, FnStatefulBiConsumerOps};
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
pub trait FnStatefulBiConsumerOps<T, U>: FnMut(&T, &U) + Sized {
    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes the current closure and returns
    /// `BoxStatefulBiConsumer<T, U>`.
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
    ///   - A `BoxStatefulBiConsumer<T, U>`
    ///   - An `ArcStatefulBiConsumer<T, U>`
    ///   - An `RcStatefulBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, FnStatefulBiConsumerOps};
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
    fn and_then<C>(self, next: C) -> BoxStatefulBiConsumer<T, U>
    where
        Self: 'static,
        C: StatefulBiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxStatefulBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnStatefulBiConsumerOps for all closure types
impl<T, U, F> FnStatefulBiConsumerOps<T, U> for F where F: FnMut(&T, &U) {}

// =======================================================================
// 7. BoxConditionalBiConsumer - Box-based Conditional BiConsumer
// =======================================================================

/// BoxConditionalBiConsumer struct
///
/// A conditional bi-consumer that only executes when a predicate is satisfied.
/// Uses `BoxStatefulBiConsumer` and `BoxBiPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxStatefulBiConsumer::when()` and is
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
/// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
/// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
pub struct BoxConditionalStatefulBiConsumer<T, U> {
    consumer: BoxStatefulBiConsumer<T, U>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U> BoxConditionalStatefulBiConsumer<T, U>
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
    ///   - A `BoxStatefulBiConsumer<T, U>`
    ///   - An `ArcStatefulBiConsumer<T, U>`
    ///   - An `RcStatefulBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new `BoxStatefulBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let cond = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    /// let second = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let cond = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    /// let second = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
    pub fn and_then<C>(self, next: C) -> BoxStatefulBiConsumer<T, U>
    where
        C: StatefulBiConsumer<T, U> + 'static,
    {
        let mut first = self;
        let mut second = next.into_fn();
        BoxStatefulBiConsumer::new(move |t, u| {
            first.accept(t, u);
            second(t, u);
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
    ///   - A `BoxStatefulBiConsumer<T, U>`
    ///   - An `RcStatefulBiConsumer<T, U>`
    ///   - An `ArcStatefulBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
    pub fn or_else<C>(self, else_consumer: C) -> BoxStatefulBiConsumer<T, U>
    where
        C: StatefulBiConsumer<T, U> + 'static,
    {
        let pred = self.predicate;
        let mut then_cons = self.consumer;
        let mut else_cons = else_consumer;
        BoxStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

impl<T, U> StatefulBiConsumer<T, U> for BoxConditionalStatefulBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U> {
        let pred = self.predicate.into_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcStatefulBiConsumer::new(move |t, u| {
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

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(BoxConditionalStatefulBiConsumer<T, U>);

// =======================================================================
// 8. ArcConditionalStatefulBiConsumer - Arc-based Conditional BiConsumer
// =======================================================================

/// ArcConditionalStatefulBiConsumer struct
///
/// A thread-safe conditional bi-consumer that only executes when a predicate is
/// satisfied. Uses `ArcStatefulBiConsumer` and `ArcBiPredicate` for shared ownership across
/// threads.
///
/// This type is typically created by calling `ArcStatefulBiConsumer::when()` and is
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
/// use prism3_function::{BiConsumer, ArcStatefulBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let conditional = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
pub struct ArcConditionalStatefulBiConsumer<T, U> {
    consumer: ArcStatefulBiConsumer<T, U>,
    predicate: ArcBiPredicate<T, U>,
}

impl<T, U> ArcConditionalStatefulBiConsumer<T, U>
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
    ///   - An `ArcStatefulBiConsumer<T, U>`
    ///   - A `BoxStatefulBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcStatefulBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcStatefulBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
    pub fn or_else<C>(&self, else_consumer: C) -> ArcStatefulBiConsumer<T, U>
    where
        C: StatefulBiConsumer<T, U> + Send + 'static,
        T: Send + Sync,
        U: Send + Sync,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        ArcStatefulBiConsumer::new(move |t: &T, u: &U| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

impl<T, U> StatefulBiConsumer<T, U> for ArcConditionalStatefulBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let pred = self.predicate.to_rc();
        let consumer = self.consumer.into_rc();
        let mut consumer_fn = consumer;
        RcStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer_fn.accept(t, u);
            }
        })
    }

    fn into_arc(self) -> ArcStatefulBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        ArcStatefulBiConsumer::new(move |t, u| {
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

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(ArcConditionalStatefulBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(ArcConditionalStatefulBiConsumer<T, U>);

// =======================================================================
// 9. RcConditionalStatefulBiConsumer - Rc-based Conditional BiConsumer
// =======================================================================

/// RcConditionalStatefulBiConsumer struct
///
/// A single-threaded conditional bi-consumer that only executes when a predicate is
/// satisfied. Uses `RcStatefulBiConsumer` and `RcBiPredicate` for shared ownership within a
/// single thread.
///
/// This type is typically created by calling `RcStatefulBiConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalStatefulBiConsumer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcStatefulBiConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let conditional = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
pub struct RcConditionalStatefulBiConsumer<T, U> {
    consumer: RcStatefulBiConsumer<T, U>,
    predicate: RcBiPredicate<T, U>,
}

impl<T, U> RcConditionalStatefulBiConsumer<T, U>
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
    ///   - An `RcStatefulBiConsumer<T, U>`
    ///   - A `BoxStatefulBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcStatefulBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcStatefulBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
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
    pub fn or_else<C>(&self, else_consumer: C) -> RcStatefulBiConsumer<T, U>
    where
        C: StatefulBiConsumer<T, U> + 'static,
    {
        let pred = self.predicate.clone();
        let mut then_cons = self.consumer.clone();
        let mut else_cons = else_consumer;

        RcStatefulBiConsumer::new(move |t: &T, u: &U| {
            if pred.test(t, u) {
                then_cons.accept(t, u);
            } else {
                else_cons.accept(t, u);
            }
        })
    }
}

impl<T, U> StatefulBiConsumer<T, U> for RcConditionalStatefulBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&mut self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    fn into_box(self) -> BoxStatefulBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        BoxStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    fn into_rc(self) -> RcStatefulBiConsumer<T, U> {
        let pred = self.predicate;
        let mut consumer = self.consumer;
        RcStatefulBiConsumer::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept(t, u);
            }
        })
    }

    // do NOT override BiConsumer::into_arc() because RcConditionalStatefulBiConsumer is not Send + Sync
    // and calling RcConditionalStatefulBiConsumer::into_arc() will cause a compile error

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

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(RcConditionalStatefulBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(RcConditionalStatefulBiConsumer<T, U>);