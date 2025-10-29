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
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::consumers::bi_consumer_once::{
    BiConsumerOnce,
    BoxBiConsumerOnce,
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
    ///
    /// let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + 'static,
    {
        BoxBiConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// Creates a no-op readonly bi-consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op readonly bi-consumer
    pub fn noop() -> Self {
        BoxBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another readonly bi-consumer in sequence
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
    ///   - An `RcBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
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
    ///
    /// let first = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Second: sum = {}", x + y);
    /// });
    ///
    /// // second is moved here
    /// let chained = first.and_then(second);
    /// chained.accept(&5, &3);
    /// // second.accept(&2, &3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer, RcBiConsumer};
    ///
    /// let first = BoxBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Second: sum = {}", x + y);
    /// });
    ///
    /// // Clone to preserve original
    /// let chained = first.and_then(second.clone());
    /// chained.accept(&5, &3);
    ///
    /// // Original still usable
    /// second.accept(&2, &3);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumer<T, U> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
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

/// Implements BiConsumerOnce for BoxBiConsumer
///
/// Allows BoxBiConsumer to be used in contexts requiring
/// BiConsumerOnce. Since Fn implements FnOnce, this is a natural
/// conversion that enables BoxBiConsumer to work seamlessly with
/// one-time consumer APIs.
impl<T, U> BiConsumerOnce<T, U> for BoxBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept_once(self, first: &T, second: &U) {
        self.accept(first, second)
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U> {
        BoxBiConsumerOnce::new(move |t, u| self.accept(t, u))
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U) {
        move |t, u| self.accept(t, u)
    }
}

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
    ///
    /// let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + Send + Sync + 'static,
    {
        ArcBiConsumer {
            function: Arc::new(f),
            name: None,
        }
    }

    /// Creates a no-op readonly bi-consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op readonly bi-consumer
    pub fn noop() -> Self {
        ArcBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation. Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
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
    ///
    /// let first = ArcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = ArcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Second: sum = {}", x + y);
    /// });
    ///
    /// // second is passed by reference, so it remains usable
    /// let chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// first.accept(&2, &3); // Still usable
    /// second.accept(&7, &8); // Still usable
    /// ```
    pub fn and_then<C>(&self, next: C) -> ArcBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + Send + Sync + 'static,
    {
        let first = Arc::clone(&self.function);
        ArcBiConsumer::new(move |t: &T, u: &U| {
            first(t, u);
            next.accept(t, u);
        })
    }
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

impl<T, U> Clone for ArcBiConsumer<T, U> {
    /// Clones the ArcBiConsumer
    ///
    /// Creates a new ArcBiConsumer sharing the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
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

/// Implements BiConsumerOnce for ArcBiConsumer
///
/// Allows ArcBiConsumer to be used in contexts requiring
/// BiConsumerOnce. Since Fn implements FnOnce, this is a natural
/// conversion that enables ArcBiConsumer to work seamlessly with
/// one-time consumer APIs.
impl<T, U> BiConsumerOnce<T, U> for ArcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept_once(self, first: &T, second: &U) {
        self.accept(first, second)
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U> {
        BoxBiConsumerOnce::new(move |t, u| self.accept(t, u))
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U) {
        move |t, u| self.accept(t, u)
    }
}

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
    ///
    /// let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + 'static,
    {
        RcBiConsumer {
            function: Rc::new(f),
            name: None,
        }
    }

    /// Creates a no-op readonly bi-consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op readonly bi-consumer
    pub fn noop() -> Self {
        RcBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation. Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumer<T, U>`
    ///   - An `RcBiConsumer<T, U>`
    ///   - An `ArcBiConsumer<T, U>`
    ///   - Any type implementing `BiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let first = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = RcBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Second: sum = {}", x + y);
    /// });
    ///
    /// // second is passed by reference, so it remains usable
    /// let chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// first.accept(&2, &3); // Still usable
    /// second.accept(&7, &8); // Still usable
    /// ```
    pub fn and_then<C>(&self, next: C) -> RcBiConsumer<T, U>
    where
        C: BiConsumer<T, U> + 'static,
    {
        let first = Rc::clone(&self.function);
        RcBiConsumer::new(move |t: &T, u: &U| {
            first(t, u);
            next.accept(t, u);
        })
    }
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

impl<T, U> Clone for RcBiConsumer<T, U> {
    /// Clones the RcBiConsumer
    ///
    /// Creates a new RcBiConsumer sharing the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
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

/// Implements BiConsumerOnce for RcBiConsumer
///
/// Allows RcBiConsumer to be used in contexts requiring
/// BiConsumerOnce. Since Fn implements FnOnce, this is a natural
/// conversion that enables RcBiConsumer to work seamlessly with
/// one-time consumer APIs.
impl<T, U> BiConsumerOnce<T, U> for RcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept_once(self, first: &T, second: &U) {
        self.accept(first, second)
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U> {
        BoxBiConsumerOnce::new(move |t, u| self.accept(t, u))
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U) {
        move |t, u| self.accept(t, u)
    }
}

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
