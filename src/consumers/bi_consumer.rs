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
//! Provides readonly bi-consumer interface implementations for operations
//! that accept two input parameters without modifying their own state or
//! the input values.
//!
//! This module provides a unified `BiConsumer` trait and three
//! concrete implementations based on different ownership models:
//!
//! - **`BoxBiConsumer<T, U>`**: Box-based single ownership
//! - **`ArcBiConsumer<T, U>`**: Arc-based thread-safe shared
//!   ownership
//! - **`RcBiConsumer<T, U>`**: Rc-based single-threaded shared
//!   ownership
//!
//! # Design Philosophy
//!
//! BiConsumer uses `Fn(&T, &U)` semantics: neither modifies its
//! own state nor the input values. Suitable for pure observation, logging,
//! and notification scenarios with two parameters. Compared to BiConsumer,
//! BiConsumer does not require interior mutability
//! (Mutex/RefCell), thus more efficient and easier to share.
//!
//! # Author
//!
//! Haixing Hu
use std::rc::Rc;
use std::sync::Arc;

use crate::consumers::bi_consumer_once::BoxBiConsumerOnce;
use crate::consumers::macros::{
    impl_box_conditional_consumer,
    impl_box_consumer_methods,
    impl_conditional_consumer_clone,
    impl_conditional_consumer_conversions,
    impl_conditional_consumer_debug_display,
    impl_consumer_clone,
    impl_consumer_common_methods,
    impl_consumer_debug_display,
    impl_shared_conditional_consumer,
    impl_shared_consumer_methods,
};
use crate::predicates::bi_predicate::{
    ArcBiPredicate,
    BiPredicate,
    BoxBiPredicate,
    RcBiPredicate,
};

// ==========================================================================
// Type Aliases
// ==========================================================================

/// Type alias for readonly bi-consumer function signature.
type BiConsumerFn<T, U> = dyn Fn(&T, &U);

/// Type alias for thread-safe readonly bi-consumer function signature.
type ThreadSafeBiConsumerFn<T, U> = dyn Fn(&T, &U) + Send + Sync;

// =======================================================================
// 1. BiConsumer Trait - Unified Interface
// =======================================================================

/// BiConsumer trait - Unified readonly bi-consumer interface
///
/// Defines core behavior for all readonly bi-consumer types. Unlike
/// `BiConsumer`, `BiConsumer` neither modifies its own state nor
/// the input values, making it a fully immutable operation.
///
/// # Automatic Implementations
///
/// - All closures implementing `Fn(&T, &U)`
/// - `BoxBiConsumer<T, U>`, `ArcBiConsumer<T, U>`,
///   `RcBiConsumer<T, U>`
///
/// # Features
///
/// - **Unified Interface**: All readonly bi-consumer types share the same
///   `accept` method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions accepting any readonly
///   bi-consumer type
/// - **No Interior Mutability**: No need for Mutex or RefCell, more
///   efficient
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// fn apply_consumer<C: BiConsumer<i32, i32>>(
///     consumer: &C,
///     a: &i32,
///     b: &i32
/// ) {
///     consumer.accept(a, b);
/// }
///
/// let box_con = BoxBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// apply_consumer(&box_con, &5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait BiConsumer<T, U> {
    /// Performs the readonly consumption operation
    ///
    /// Executes an operation on the given two references. The operation
    /// typically reads input values or produces side effects, but neither
    /// modifies the input values nor the consumer's own state.
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
    ///
    /// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Values: {}, {}", x, y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    fn accept(&self, first: &T, second: &U);

    /// Converts to BoxBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumer<T, U>`
    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(move |t, u| self.accept(t, u))
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
        RcBiConsumer::new(move |t, u| self.accept(t, u))
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
        Self: Sized + Send + Sync + 'static,
        T: 'static,
        U: 'static,
    {
        ArcBiConsumer::new(move |t, u| self.accept(t, u))
    }

    /// Converts readonly bi-consumer to a closure
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the readonly bi-consumer to a closure usable with standard
    /// library methods requiring `Fn`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let func = consumer.into_fn();
    /// func(&5, &3);
    /// ```
    fn into_fn(self) -> impl Fn(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        move |t, u| self.accept(t, u)
    }

    /// Converts to BoxBiConsumer (without consuming self)
    ///
    /// Creates a new `BoxBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let box_consumer = consumer.to_box();
    /// box_consumer.accept(&5, &3);
    /// // Original consumer still usable
    /// consumer.accept(&10, &20);
    /// ```
    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_box()
    }

    /// Converts to RcBiConsumer (without consuming self)
    ///
    /// Creates a new `RcBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    ///
    /// let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let rc_consumer = consumer.to_rc();
    /// rc_consumer.accept(&5, &3);
    /// // Original consumer still usable
    /// consumer.accept(&10, &20);
    /// ```
    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts to ArcBiConsumer (without consuming self)
    ///
    /// Creates a new `ArcBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// // Note: This will only compile if the closure is Send + Sync
    /// // For demonstration, we use a simple closure
    /// let arc_consumer = consumer.to_arc();
    /// arc_consumer.accept(&5, &3);
    /// ```
    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        Self: Clone + Send + Sync + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_arc()
    }

    /// Converts to a closure (without consuming self)
    ///
    /// Creates a new closure by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let func = consumer.to_fn();
    /// func(&5, &3);
    /// // Original consumer still usable
    /// consumer.accept(&10, &20);
    /// ```
    fn to_fn(&self) -> impl Fn(&T, &U)
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_fn()
    }

    /// Convert to BiConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts a reusable readonly bi-consumer to a one-time consumer that consumes itself on use.
    /// This enables passing `BiConsumer` to functions that require `BiConsumerOnce`.
    ///
    /// # Returns
    ///
    /// Returns a `BoxBiConsumerOnce<T, U>`
    fn into_once(self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumerOnce::new(move |t, u| self.accept(t, u))
    }

    /// Convert to BiConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement `Clone`.
    /// Clones the current consumer and converts the clone to a one-time consumer.
    ///
    /// # Returns
    ///
    /// Returns a `BoxBiConsumerOnce<T, U>`
    fn to_once(&self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_once()
    }
}

// =======================================================================
// 2. BoxBiConsumer - Single Ownership Implementation
// =======================================================================

/// BoxBiConsumer struct
///
/// A readonly bi-consumer implementation based on `Box<dyn Fn(&T, &U)>`
/// for single ownership scenarios.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Fully Immutable**: Neither modifies itself nor input values
/// - **No Interior Mutability**: No need for Mutex or RefCell
///
/// # Use Cases
///
/// Choose `BoxBiConsumer` when:
/// - The readonly bi-consumer is used only once or in a linear flow
/// - No need to share the consumer across contexts
/// - Pure observation operations like logging
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// consumer.accept(&5, &3);
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
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        BoxBiConsumer<T, U>,
        (Fn(&T, &U) + 'static),
        |f| Box::new(f)
    );

    // Generates: when() and and_then() methods that consume self
    impl_box_consumer_methods!(
        BoxBiConsumer<T, U>,
        BoxConditionalBiConsumer,
        BiConsumer
    );
}

impl<T, U> BiConsumer<T, U> for BoxBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
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
        RcBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    // do NOT override ReadonlyConsumer::into_arc() because ArcBiConsumer is not Send + Sync
    // and calling ArcBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }
}

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(BoxBiConsumer<T, U>);

// =======================================================================
// 3. ArcBiConsumer - Thread-Safe Shared Ownership
// =======================================================================

/// ArcBiConsumer struct
///
/// A readonly bi-consumer implementation based on
/// `Arc<dyn Fn(&T, &U) + Send + Sync>` for thread-safe shared ownership
/// scenarios. No need for Mutex because operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **No Locks**: Because readonly, no need for Mutex protection
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Use Cases
///
/// Choose `ArcBiConsumer` when:
/// - Need to share readonly bi-consumer across multiple threads
/// - Pure observation operations like logging, monitoring, notifications
/// - Need high-concurrency reads without lock overhead
///
/// # Performance Advantages
///
/// Compared to `ArcBiConsumer`, `ArcBiConsumer` has no Mutex
/// locking overhead, resulting in better performance in high-concurrency
/// scenarios.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
///
/// let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// clone.accept(&10, &20);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcBiConsumer<T, U> {
    function: Arc<ThreadSafeBiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> ArcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        ArcBiConsumer<T, U>,
        (Fn(&T, &U) + Send + Sync + 'static),
        |f| Arc::new(f)
    );

    // Generates: when() and and_then() methods that borrow &self (Arc can clone)
    impl_shared_consumer_methods!(
        ArcBiConsumer<T, U>,
        ArcConditionalBiConsumer,
        into_arc,
        BiConsumer,
        Send + Sync + 'static
    );
}

impl<T, U> BiConsumer<T, U> for ArcBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        RcBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        move |t, u| (self.function)(t, u)
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        RcBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        move |t, u| self_fn(t, u)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(ArcBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(ArcBiConsumer<T, U>);

// =======================================================================
// 4. RcBiConsumer - Single-Threaded Shared Ownership
// =======================================================================

/// RcBiConsumer struct
///
/// A readonly bi-consumer implementation based on `Rc<dyn Fn(&T, &U)>`
/// for single-threaded shared ownership scenarios. No need for RefCell
/// because operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot send across threads
/// - **No Interior Mutability Overhead**: No need for RefCell because
///   readonly
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Use Cases
///
/// Choose `RcBiConsumer` when:
/// - Need to share readonly bi-consumer within a single thread
/// - Pure observation operations, performance critical
/// - Single-threaded UI framework event handling
///
/// # Performance Advantages
///
/// `RcBiConsumer` has neither Arc's atomic operation overhead nor
/// RefCell's runtime borrow checking overhead, making it the best
/// performing among the three readonly bi-consumer types.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
///
/// let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// clone.accept(&10, &20);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcBiConsumer<T, U> {
    function: Rc<BiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> RcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(
        RcBiConsumer<T, U>,
        (Fn(&T, &U) + 'static),
        |f| Rc::new(f)
    );

    // Generates: when() and and_then() methods that borrow &self (Rc can clone)
    impl_shared_consumer_methods!(
        RcBiConsumer<T, U>,
        RcConditionalBiConsumer,
        into_rc,
        BiConsumer,
        'static
    );
}

impl<T, U> BiConsumer<T, U> for RcBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(move |t, u| (self.function)(t, u))
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

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        move |t, u| (self.function)(t, u)
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiConsumer::new(move |t, u| self_fn(t, u))
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

    fn to_fn(&self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        move |t, u| self_fn(t, u)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(RcBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(RcBiConsumer<T, U>);

// =======================================================================
// 5. Implement BiConsumer trait for closures
// =======================================================================

/// Implements BiConsumer for all Fn(&T, &U)
impl<T, U, F> BiConsumer<T, U> for F
where
    F: Fn(&T, &U),
{
    fn accept(&self, first: &T, second: &U) {
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
        Self: Sized + Send + Sync + 'static,
        T: 'static,
        U: 'static,
    {
        ArcBiConsumer::new(self)
    }

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.clone();
        BoxBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.clone();
        RcBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_arc(&self) -> ArcBiConsumer<T, U>
    where
        Self: Clone + Send + Sync + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.clone();
        ArcBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_fn(&self) -> impl Fn(&T, &U)
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone()
    }
}

// =======================================================================
// 6. Provide extension methods for closures
// =======================================================================

/// Extension trait providing readonly bi-consumer composition methods for
/// closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `Fn(&T, &U)`, enabling direct method chaining on closures
/// without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxBiConsumer**: Composition results can be
///   further chained
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `Fn(&T, &U)` closures get these
///   methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, FnBiConsumerOps};
///
/// let chained = (|x: &i32, y: &i32| {
///     println!("First: {}, {}", x, y);
/// }).and_then(|x: &i32, y: &i32| {
///     println!("Second: sum = {}", x + y);
/// });
/// chained.accept(&5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnBiConsumerOps<T, U>: Fn(&T, &U) + Sized {
    /// Chains another readonly bi-consumer in sequence
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
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, FnBiConsumerOps};
    ///
    /// let chained = (|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// }).and_then(|x: &i32, y: &i32| {
    ///     println!("Second: sum = {}", x + y);
    /// }).and_then(|x: &i32, y: &i32| {
    ///     println!("Third: product = {}", x * y);
    /// });
    ///
    /// chained.accept(&5, &3);
    /// ```
    fn and_then<C>(self, next: C) -> BoxBiConsumer<T, U>
    where
        Self: 'static,
        C: BiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let first = self;
        let second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnBiConsumerOps for all closure types
impl<T, U, F> FnBiConsumerOps<T, U> for F where F: Fn(&T, &U) {}

// =======================================================================
// 7. BoxConditionalBiConsumer - Box-based Conditional BiConsumer
// =======================================================================

/// BoxConditionalBiConsumer struct
///
/// A conditional readonly bi-consumer that only executes when a predicate is satisfied.
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
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Both positive: {} + {} = {}", x, y, x + y);
/// });
/// let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
///
/// conditional.accept(&5, &3);  // Prints: Both positive: 5 + 3 = 8
/// conditional.accept(&-5, &3); // Does nothing
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Both positive: {} + {} = {}", x, y, x + y);
/// })
/// .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
/// .or_else(|x: &i32, y: &i32| {
///     println!("Not both positive: {} and {}", x, y);
/// });
///
/// consumer.accept(&5, &3);  // Prints: Both positive: 5 + 3 = 8
/// consumer.accept(&-5, &3); // Prints: Not both positive: -5 and 3
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalBiConsumer<T, U> {
    consumer: BoxBiConsumer<T, U>,
    predicate: BoxBiPredicate<T, U>,
}

// Use macro to generate conditional bi-consumer implementations
impl_box_conditional_consumer!(
    BoxConditionalBiConsumer<T, U>,
    BoxBiConsumer,
    BiConsumer
);

// Hand-written BiConsumer trait implementation
impl<T, U> BiConsumer<T, U> for BoxConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(
        BoxBiConsumer<T, U>,
        RcBiConsumer,
        Fn
    );
}

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(BoxConditionalBiConsumer<T, U>);

// =======================================================================
// 8. ArcConditionalBiConsumer - Arc-based Conditional BiConsumer
// =======================================================================

/// ArcConditionalBiConsumer struct
///
/// A conditional bi-consumer that wraps an `ArcBiConsumer` and only executes
/// when a predicate is satisfied. Based on `Arc` for thread-safe shared ownership.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allows multiple owners
/// - **Thread Safe**: Implements `Send + Sync`, can be safely used concurrently
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Implements BiConsumer**: Can be used anywhere a `BiConsumer` is expected
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalBiConsumer<T, U> {
    consumer: ArcBiConsumer<T, U>,
    predicate: ArcBiPredicate<T, U>,
}

// Use macro to generate conditional bi-consumer implementations
impl_shared_conditional_consumer!(
    ArcConditionalBiConsumer<T, U>,
    ArcBiConsumer,
    BiConsumer,
    into_arc,
    Send + Sync + 'static
);

// Hand-written BiConsumer trait implementation
impl<T, U> BiConsumer<T, U> for ArcConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(
        BoxBiConsumer<T, U>,
        RcBiConsumer,
        Fn
    );
}

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(ArcConditionalBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(ArcConditionalBiConsumer<T, U>);

// =======================================================================
// 9. RcConditionalBiConsumer - Rc-based Conditional BiConsumer
// =======================================================================

/// RcConditionalBiConsumer struct
///
/// A conditional bi-consumer that wraps an `RcBiConsumer` and only executes
/// when a predicate is satisfied. Based on `Rc` for single-threaded shared ownership.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allows multiple owners
/// - **Single-Threaded**: Not thread-safe, more efficient than Arc in single-threaded contexts
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Implements BiConsumer**: Can be used anywhere a `BiConsumer` is expected
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalBiConsumer<T, U> {
    consumer: RcBiConsumer<T, U>,
    predicate: RcBiPredicate<T, U>,
}

// Use macro to generate conditional bi-consumer implementations
impl_shared_conditional_consumer!(
    RcConditionalBiConsumer<T, U>,
    RcBiConsumer,
    BiConsumer,
    into_rc,
    'static
);

// Hand-written BiConsumer trait implementation
impl<T, U> BiConsumer<T, U> for RcConditionalBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept(&self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept(first, second);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(
        BoxBiConsumer<T, U>,
        RcBiConsumer,
        Fn
    );
}

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(RcConditionalBiConsumer<T, U>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(RcConditionalBiConsumer<T, U>);
