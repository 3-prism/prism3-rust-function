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
//! Provides implementations of readonly consumer interfaces for executing
//! operations that neither modify their own state nor modify input values.
//!
//! This module provides a unified `Consumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxConsumer<T>`**: Box-based single ownership implementation
//! - **`ArcConsumer<T>`**: Arc-based thread-safe shared ownership
//!   implementation
//! - **`RcConsumer<T>`**: Rc-based single-threaded shared ownership
//!   implementation
//!
//! # Design Philosophy
//!
//! Consumer uses `Fn(&T)` semantics, neither modifying its own state nor
//! modifying input values.
//! Suitable for pure observation, logging, notification and other scenarios.
//! Compared to Consumer, Consumer does not require interior mutability
//! (Mutex/RefCell), making it more efficient and easier to share.
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

use crate::consumers::consumer_once::BoxConsumerOnce;
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
use crate::predicates::predicate::{
    ArcPredicate,
    BoxPredicate,
    Predicate,
    RcPredicate,
};

// ============================================================================
// 1. Consumer Trait - Unified Consumer Interface
// ============================================================================

/// Consumer trait - Unified readonly consumer interface
///
/// Defines the core behavior of all readonly consumer types. Unlike `Consumer`,
/// `Consumer` neither modifies its own state nor modifies input values,
/// making it a completely immutable operation.
///
/// # Auto-implementation
///
/// - All closures implementing `Fn(&T)`
/// - `BoxConsumer<T>`, `ArcConsumer<T>`,
///   `RcConsumer<T>`
///
/// # Features
///
/// - **Unified Interface**: All readonly consumer types share the same `accept`
///   method signature
/// - **Auto-implementation**: Closures automatically implement this trait with
///   zero overhead
/// - **Type Conversion**: Easy conversion between different ownership models
/// - **Generic Programming**: Write functions that work with any readonly
///   consumer type
/// - **No Interior Mutability**: No need for Mutex or RefCell, more efficient
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// fn apply_consumer<C: Consumer<i32>>(consumer: &C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// let box_con = BoxConsumer::new(|x: &i32| {
///     println!("Value: {}", x);
/// });
/// apply_consumer(&box_con, &5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Consumer<T> {
    /// Execute readonly consumption operation
    ///
    /// Performs an operation on the given reference. The operation typically
    /// reads input values or produces side effects, but neither modifies the
    /// input value nor the consumer's own state.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to consume
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
    /// consumer.accept(&5);
    /// ```
    fn accept(&self, value: &T);

    /// Convert to BoxConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxConsumer<T>`
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(move |t| self.accept(t))
    }

    /// Convert to RcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcConsumer<T>`
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcConsumer::new(move |t| self.accept(t))
    }

    /// Convert to ArcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcConsumer<T>`
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: 'static,
    {
        ArcConsumer::new(move |t| self.accept(t))
    }

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after
    /// calling this method.
    ///
    /// Converts a readonly consumer to a closure that can be used directly in
    /// places where the standard library requires `Fn`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let consumer = BoxConsumer::new(|x: &i32| {
    ///     println!("Value: {}", x);
    /// });
    /// let func = consumer.into_fn();
    /// func(&5);
    /// ```
    fn into_fn(self) -> impl Fn(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t| self.accept(t)
    }

    /// Non-consuming conversion to `BoxConsumer`
    ///
    /// **⚠️ Does NOT consume `self`**: This method clones `self` and returns a
    /// boxed readonly consumer that calls the cloned consumer. Requires
    /// `Self: Clone` so it can be called through an immutable reference.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxConsumer<T>`
    fn to_box(&self) -> BoxConsumer<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Non-consuming conversion to `RcConsumer`
    ///
    /// **⚠️ Does NOT consume `self`**: Clones `self` and returns an
    /// `RcConsumer` that forwards to the cloned consumer. Requires
    /// `Self: Clone`.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcConsumer<T>`
    fn to_rc(&self) -> RcConsumer<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Non-consuming conversion to `ArcConsumer`
    ///
    /// **⚠️ Does NOT consume `self`**: Clones `self` and returns an
    /// `ArcConsumer`. Requires `Self: Clone + Send + Sync` so the result
    /// is thread-safe.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcConsumer<T>`
    fn to_arc(&self) -> ArcConsumer<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: 'static,
    {
        self.clone().into_arc()
    }

    /// Non-consuming conversion to a boxed closure
    ///
    /// **⚠️ Does NOT consume `self`**: Returns a closure which calls a cloned
    /// copy of the consumer. Requires `Self: Clone`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T)` which forwards to the cloned
    /// consumer.
    fn to_fn(&self) -> impl Fn(&T)
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_fn()
    }

    /// Convert to ConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts a reusable readonly consumer to a one-time consumer that consumes itself on use.
    /// This enables passing `Consumer` to functions that require `ConsumerOnce`.
    ///
    /// # Returns
    ///
    /// Returns a `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    ///
    /// fn takes_once<C: ConsumerOnce<i32>>(consumer: C, value: &i32) {
    ///     consumer.accept(value);
    /// }
    ///
    /// let consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
    /// takes_once(consumer.into_once(), &5);
    /// ```
    fn into_once(self) -> BoxConsumerOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumerOnce::new(move |t| self.accept(t))
    }

    /// Convert to ConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement `Clone`.
    /// Clones the current consumer and converts the clone to a one-time consumer.
    ///
    /// # Returns
    ///
    /// Returns a `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    ///
    /// fn takes_once<C: ConsumerOnce<i32>>(consumer: C, value: &i32) {
    ///     consumer.accept(value);
    /// }
    ///
    /// let consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
    /// takes_once(consumer.to_once(), &5);
    /// ```
    fn to_once(&self) -> BoxConsumerOnce<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_once()
    }
}

// ============================================================================
// 2. BoxConsumer - Single Ownership Implementation
// ============================================================================

/// BoxConsumer struct
///
/// Readonly consumer implementation based on `Box<dyn Fn(&T)>` for single
/// ownership scenarios.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership when used
/// - **Zero Overhead**: No reference counting or lock overhead
/// - **Completely Immutable**: Neither modifies itself nor input
/// - **No Interior Mutability**: No need for Mutex or RefCell
///
/// # Use Cases
///
/// Choose `BoxConsumer` when:
/// - Readonly consumer is used once or in a linear flow
/// - No need to share consumer across contexts
/// - Pure observation operations, such as logging
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Observed value: {}", x);
/// });
/// consumer.accept(&5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConsumer<T> {
    function: Box<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> BoxConsumer<T>
where
    T: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(BoxConsumer<T>, (Fn(&T) + 'static), |f| Box::new(f));

    // Generates: when() and and_then() methods that consume self
    impl_box_consumer_methods!(BoxConsumer<T>, BoxConditionalConsumer, Consumer);
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&self, value: &T) {
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
        let func = self.function;
        RcConsumer::new(move |t| func(t))
    }

    // do NOT override Consumer::into_arc() because BoxConsumer is not Send + Sync
    // and calling BoxConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        self.function
    }
}

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(BoxConsumer<T>);

// ============================================================================
// 3. ArcConsumer - Thread-safe Shared Ownership Implementation
// ============================================================================

/// ArcConsumer struct
///
/// Readonly consumer implementation based on `Arc<dyn Fn(&T) + Send + Sync>`,
/// for thread-safe shared ownership scenarios. No Mutex needed because
/// operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allows multiple owners
/// - **Thread Safe**: Implements `Send + Sync`, can be safely used concurrently
/// - **Lock-free**: No Mutex protection needed because it's readonly
/// - **Non-consuming API**: `and_then` borrows `&self`, original object remains
///   usable
///
/// # Use Cases
///
/// Choose `ArcConsumer` when:
/// - Need to share readonly consumer across multiple threads
/// - Pure observation operations, such as logging, monitoring, notifications
/// - Need high-concurrency reads with no lock overhead
///
/// # Performance Advantages
///
/// Compared to `ArcConsumer`, `ArcConsumer` has no Mutex lock overhead,
/// performing better in high-concurrency scenarios.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
///
/// let consumer = ArcConsumer::new(|x: &i32| {
///     println!("Observed: {}", x);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5);
/// clone.accept(&10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConsumer<T> {
    function: Arc<dyn Fn(&T) + Send + Sync>,
    name: Option<String>,
}

impl<T> ArcConsumer<T>
where
    T: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(ArcConsumer<T>, (Fn(&T) + Send + Sync + 'static), |f| {
        Arc::new(f)
    });

    // Generates: when() and and_then() methods that borrow &self (Arc can clone)
    impl_shared_consumer_methods!(
        ArcConsumer<T>,
        ArcConditionalConsumer,
        into_arc,
        Consumer,
        Send + Sync + 'static
    );
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| (self.function)(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        RcConsumer::new(move |t| (self.function)(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        move |t| (self.function)(t)
    }

    fn to_box(&self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxConsumer::new(move |t| self_fn(t))
    }

    fn to_rc(&self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        RcConsumer::new(move |t| self_fn(t))
    }

    fn to_arc(&self) -> ArcConsumer<T>
    where
        T: 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        move |t| self_fn(t)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(ArcConsumer<T>, Arc::clone);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(ArcConsumer<T>);

// ============================================================================
// 4. RcConsumer - Single-threaded Shared Ownership Implementation
// ============================================================================

/// RcConsumer struct
///
/// Readonly consumer implementation based on `Rc<dyn Fn(&T)>` for
/// single-threaded shared ownership scenarios. No RefCell needed because
/// operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allows multiple owners
/// - **Single-threaded**: Not thread-safe, cannot be sent across threads
/// - **No Interior Mutability Overhead**: No RefCell needed because it's readonly
/// - **Non-consuming API**: `and_then` borrows `&self`, original object remains
///   usable
///
/// # Use Cases
///
/// Choose `RcConsumer` when:
/// - Need to share readonly consumer within a single thread
/// - Pure observation operations, performance critical
/// - Event handling in single-threaded UI frameworks
///
/// # Performance Advantages
///
/// `RcConsumer` has neither Arc's atomic operation overhead nor
/// RefCell's runtime borrow checking overhead, making it the most performant of
/// the three readonly consumers.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let consumer = RcConsumer::new(|x: &i32| {
///     println!("Observed: {}", x);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5);
/// clone.accept(&10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConsumer<T> {
    function: Rc<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> RcConsumer<T>
where
    T: 'static,
{
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(RcConsumer<T>, (Fn(&T) + 'static), |f| Rc::new(f));

    // Generates: when() and and_then() methods that borrow &self (Rc can clone)
    impl_shared_consumer_methods!(
        RcConsumer<T>,
        RcConditionalConsumer,
        into_rc,
        Consumer,
        'static
    );
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| (self.function)(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    // do NOT override Consumer::into_arc() because RcConsumer is not Send + Sync
    // and calling RcConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T)
    where
        T: 'static,
    {
        move |t| (self.function)(t)
    }

    fn to_box(&self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxConsumer::new(move |t| self_fn(t))
    }

    fn to_rc(&self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self.clone()
    }

    // do NOT override Consumer::to_arc() because RcConsumer is not Send + Sync
    // and calling RcConsumer::to_arc() will cause a compile error

    fn to_fn(&self) -> impl Fn(&T)
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        move |t| self_fn(t)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(RcConsumer<T>, Rc::clone);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(RcConsumer<T>);

// ============================================================================
// 5. Implement Consumer trait for closures
// ============================================================================

/// Implement Consumer for all Fn(&T)
impl<T, F> Consumer<T> for F
where
    F: Fn(&T),
{
    fn accept(&self, value: &T) {
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
        Self: Sized + Send + Sync + 'static,
        T: 'static,
    {
        ArcConsumer::new(self)
    }

    fn into_fn(self) -> impl Fn(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxConsumer<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.clone();
        BoxConsumer::new(self_fn)
    }

    fn to_rc(&self) -> RcConsumer<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.clone();
        RcConsumer::new(self_fn)
    }

    fn to_arc(&self) -> ArcConsumer<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: 'static,
    {
        let self_fn = self.clone();
        ArcConsumer::new(self_fn)
    }

    fn to_fn(&self) -> impl Fn(&T)
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone()
    }
}

// ============================================================================
// 6. Provide extension methods for closures
// ============================================================================

/// Extension trait providing readonly consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `Fn(&T)`, allowing closures to directly chain methods without
/// explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxConsumer**: Combined results can continue chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Auto-implementation**: All `Fn(&T)` closures automatically get these
///   methods
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, FnConsumerOps};
///
/// let chained = (|x: &i32| {
///     println!("First: {}", x);
/// }).and_then(|x: &i32| {
///     println!("Second: {}", x);
/// });
/// chained.accept(&5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnConsumerOps<T>: Fn(&T) + Sized {
    /// Sequentially chain another readonly consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the
    /// next operation. Consumes the current closure and returns
    /// `BoxConsumer<T>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a combined `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnConsumerOps};
    ///
    /// let chained = (|x: &i32| {
    ///     println!("First: {}", x);
    /// }).and_then(|x: &i32| {
    ///     println!("Second: {}", x);
    /// }).and_then(|x: &i32| println!("Third: {}", x));
    ///
    /// chained.accept(&5);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let first = self;
        let second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// Implement FnConsumerOps for all closure types
impl<T, F> FnConsumerOps<T> for F where F: Fn(&T) {}

// ============================================================================
// 7. BoxConditionalConsumer - Box-based Conditional Consumer
// ============================================================================

/// BoxConditionalConsumer struct
///
/// A conditional readonly consumer that only executes when a predicate is satisfied.
/// Uses `BoxConsumer` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Consumer**: Can be used anywhere a `Consumer` is expected
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// });
/// let conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);  // Prints: Positive: 5
/// conditional.accept(&-5); // Does nothing
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(|x: &i32| {
///     println!("Non-positive: {}", x);
/// });
///
/// consumer.accept(&5);  // Prints: Positive: 5
/// consumer.accept(&-5); // Prints: Non-positive: -5
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalConsumer<T> {
    consumer: BoxConsumer<T>,
    predicate: BoxPredicate<T>,
}

// Use macro to generate conditional consumer implementations
impl_box_conditional_consumer!(BoxConditionalConsumer<T>, BoxConsumer, Consumer);

// Hand-written Consumer trait implementation
impl<T> Consumer<T> for BoxConditionalConsumer<T>
where
    T: 'static,
{
    fn accept(&self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(BoxConsumer<T>, RcConsumer, Fn);
}

// ============================================================================
// 8. ArcConditionalConsumer - Arc-based Conditional Consumer
// ============================================================================

/// ArcConditionalConsumer struct
///
/// A conditional readonly consumer that only executes when a predicate is satisfied.
/// Uses `ArcConsumer` and `ArcPredicate` for thread-safe shared ownership semantics.
///
/// This type is typically created by calling `ArcConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allows multiple owners
/// - **Thread Safe**: Implements `Send + Sync`, can be safely used concurrently
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Consumer**: Can be used anywhere a `Consumer` is expected
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
///
/// let consumer = ArcConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// });
/// let conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);  // Prints: Positive: 5
/// conditional.accept(&-5); // Does nothing
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
///
/// let consumer = ArcConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(|x: &i32| {
///     println!("Non-positive: {}", x);
/// });
///
/// consumer.accept(&5);  // Prints: Positive: 5
/// consumer.accept(&-5); // Prints: Non-positive: -5
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalConsumer<T> {
    consumer: ArcConsumer<T>,
    predicate: ArcPredicate<T>,
}

// Use macro to generate conditional consumer implementations
impl_shared_conditional_consumer!(
    ArcConditionalConsumer<T>,
    ArcConsumer,
    Consumer,
    into_arc,
    Send + Sync + 'static
);

// Hand-written Consumer trait implementation
impl<T> Consumer<T> for ArcConditionalConsumer<T>
where
    T: 'static,
{
    fn accept(&self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(BoxConsumer<T>, RcConsumer, Fn);
}

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(ArcConditionalConsumer<T>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(ArcConditionalConsumer<T>);

// ============================================================================
// 9. RcConditionalConsumer - Rc-based Conditional Consumer
// ============================================================================

/// RcConditionalConsumer struct
///
/// A conditional readonly consumer that only executes when a predicate is satisfied.
/// Uses `RcConsumer` and `RcPredicate` for single-threaded shared ownership semantics.
///
/// This type is typically created by calling `RcConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allows multiple owners
/// - **Single-threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Consumer**: Can be used anywhere a `Consumer` is expected
/// - **Readonly**: Neither modifies itself nor input values
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let consumer = RcConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// });
/// let conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);  // Prints: Positive: 5
/// conditional.accept(&-5); // Does nothing
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let consumer = RcConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(|x: &i32| {
///     println!("Non-positive: {}", x);
/// });
///
/// consumer.accept(&5);  // Prints: Positive: 5
/// consumer.accept(&-5); // Prints: Non-positive: -5
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalConsumer<T> {
    consumer: RcConsumer<T>,
    predicate: RcPredicate<T>,
}

// Use macro to generate conditional consumer implementations
impl_shared_conditional_consumer!(
    RcConditionalConsumer<T>,
    RcConsumer,
    Consumer,
    into_rc,
    'static
);

// Hand-written Consumer trait implementation
impl<T> Consumer<T> for RcConditionalConsumer<T>
where
    T: 'static,
{
    fn accept(&self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }

    // Generates: into_box(), into_rc(), into_fn()
    impl_conditional_consumer_conversions!(BoxConsumer<T>, RcConsumer, Fn);
}

// Use macro to generate Clone implementation
impl_conditional_consumer_clone!(RcConditionalConsumer<T>);

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(RcConditionalConsumer<T>);

// ============================================================================
// 10. BoxConditionalBiConsumer - Box-based Conditional BiConsumer
// ============================================================================
