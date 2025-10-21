/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ReadonlyBiConsumer Types
//!
//! Provides readonly bi-consumer interface implementations for operations
//! that accept two input parameters without modifying their own state or
//! the input values.
//!
//! This module provides a unified `ReadonlyBiConsumer` trait and three
//! concrete implementations based on different ownership models:
//!
//! - **`BoxReadonlyBiConsumer<T, U>`**: Box-based single ownership
//! - **`ArcReadonlyBiConsumer<T, U>`**: Arc-based thread-safe shared
//!   ownership
//! - **`RcReadonlyBiConsumer<T, U>`**: Rc-based single-threaded shared
//!   ownership
//!
//! # Design Philosophy
//!
//! ReadonlyBiConsumer uses `Fn(&T, &U)` semantics: neither modifies its
//! own state nor the input values. Suitable for pure observation, logging,
//! and notification scenarios with two parameters. Compared to BiConsumer,
//! ReadonlyBiConsumer does not require interior mutability
//! (Mutex/RefCell), thus more efficient and easier to share.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

// ==========================================================================
// Type Aliases
// ==========================================================================

/// Type alias for readonly bi-consumer function signature.
type ReadonlyBiConsumerFn<T, U> = dyn Fn(&T, &U);

/// Type alias for thread-safe readonly bi-consumer function signature.
type ThreadSafeReadonlyBiConsumerFn<T, U> = dyn Fn(&T, &U) + Send + Sync;

// =======================================================================
// 1. ReadonlyBiConsumer Trait - Unified Interface
// =======================================================================

/// ReadonlyBiConsumer trait - Unified readonly bi-consumer interface
///
/// Defines core behavior for all readonly bi-consumer types. Unlike
/// `BiConsumer`, `ReadonlyBiConsumer` neither modifies its own state nor
/// the input values, making it a fully immutable operation.
///
/// # Automatic Implementations
///
/// - All closures implementing `Fn(&T, &U)`
/// - `BoxReadonlyBiConsumer<T, U>`, `ArcReadonlyBiConsumer<T, U>`,
///   `RcReadonlyBiConsumer<T, U>`
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
/// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
///
/// fn apply_consumer<C: ReadonlyBiConsumer<i32, i32>>(
///     consumer: &C,
///     a: &i32,
///     b: &i32
/// ) {
///     consumer.accept(a, b);
/// }
///
/// let box_con = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// apply_consumer(&box_con, &5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait ReadonlyBiConsumer<T, U> {
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
    /// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
    ///
    /// let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Values: {}, {}", x, y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    fn accept(&self, first: &T, second: &U);

    /// Converts to BoxReadonlyBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxReadonlyBiConsumer<T, U>`
    fn into_box(self) -> BoxReadonlyBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxReadonlyBiConsumer::new(move |t, u| self.accept(t, u))
    }

    /// Converts to RcReadonlyBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcReadonlyBiConsumer<T, U>`
    fn into_rc(self) -> RcReadonlyBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcReadonlyBiConsumer::new(move |t, u| self.accept(t, u))
    }

    /// Converts to ArcReadonlyBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcReadonlyBiConsumer<T, U>`
    fn into_arc(self) -> ArcReadonlyBiConsumer<T, U>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        ArcReadonlyBiConsumer::new(move |t, u| self.accept(t, u))
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
    /// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
    ///
    /// let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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

    /// Converts to BoxReadonlyBiConsumer (without consuming self)
    ///
    /// Creates a new `BoxReadonlyBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `BoxReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let box_consumer = consumer.to_box();
    /// box_consumer.accept(&5, &3);
    /// // Original consumer still usable
    /// consumer.accept(&10, &20);
    /// ```
    fn to_box(&self) -> BoxReadonlyBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_box()
    }

    /// Converts to RcReadonlyBiConsumer (without consuming self)
    ///
    /// Creates a new `RcReadonlyBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `RcReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, ArcReadonlyBiConsumer};
    ///
    /// let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// let rc_consumer = consumer.to_rc();
    /// rc_consumer.accept(&5, &3);
    /// // Original consumer still usable
    /// consumer.accept(&10, &20);
    /// ```
    fn to_rc(&self) -> RcReadonlyBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts to ArcReadonlyBiConsumer (without consuming self)
    ///
    /// Creates a new `ArcReadonlyBiConsumer` by cloning the current consumer.
    /// The original consumer remains usable after this call.
    ///
    /// # Returns
    ///
    /// Returns a new `ArcReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// // Note: This will only compile if the closure is Send + Sync
    /// // For demonstration, we use a simple closure
    /// let arc_consumer = consumer.to_arc();
    /// arc_consumer.accept(&5, &3);
    /// ```
    fn to_arc(&self) -> ArcReadonlyBiConsumer<T, U>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
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
    /// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
// 2. BoxReadonlyBiConsumer - Single Ownership Implementation
// =======================================================================

/// BoxReadonlyBiConsumer struct
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
/// Choose `BoxReadonlyBiConsumer` when:
/// - The readonly bi-consumer is used only once or in a linear flow
/// - No need to share the consumer across contexts
/// - Pure observation operations like logging
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
///
/// let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// consumer.accept(&5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxReadonlyBiConsumer<T, U> {
    function: Box<ReadonlyBiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxReadonlyBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new BoxReadonlyBiConsumer
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
    /// Returns a new `BoxReadonlyBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
    ///
    /// let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + 'static,
    {
        BoxReadonlyBiConsumer {
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
        BoxReadonlyBiConsumer::new(|_, _| {})
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
    ///   - A `BoxReadonlyBiConsumer<T, U>`
    ///   - An `RcReadonlyBiConsumer<T, U>`
    ///   - An `ArcReadonlyBiConsumer<T, U>`
    ///   - Any type implementing `ReadonlyBiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer};
    ///
    /// let first = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
    /// use prism3_function::{ReadonlyBiConsumer, BoxReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let first = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
        C: ReadonlyBiConsumer<T, U> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxReadonlyBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

impl<T, U> ReadonlyBiConsumer<T, U> for BoxReadonlyBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_rc(self) -> RcReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        RcReadonlyBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    // do NOT override ReadonlyConsumer::into_arc() because ArcReadonlyBiConsumer is not Send + Sync
    // and calling ArcReadonlyBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }
}

impl<T, U> fmt::Debug for BoxReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxReadonlyBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for BoxReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxReadonlyBiConsumer({})", name),
            None => write!(f, "BoxReadonlyBiConsumer"),
        }
    }
}

// =======================================================================
// 3. ArcReadonlyBiConsumer - Thread-Safe Shared Ownership
// =======================================================================

/// ArcReadonlyBiConsumer struct
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
/// Choose `ArcReadonlyBiConsumer` when:
/// - Need to share readonly bi-consumer across multiple threads
/// - Pure observation operations like logging, monitoring, notifications
/// - Need high-concurrency reads without lock overhead
///
/// # Performance Advantages
///
/// Compared to `ArcBiConsumer`, `ArcReadonlyBiConsumer` has no Mutex
/// locking overhead, resulting in better performance in high-concurrency
/// scenarios.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ReadonlyBiConsumer, ArcReadonlyBiConsumer};
///
/// let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
pub struct ArcReadonlyBiConsumer<T, U> {
    function: Arc<ThreadSafeReadonlyBiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> ArcReadonlyBiConsumer<T, U>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
{
    /// Creates a new ArcReadonlyBiConsumer
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
    /// Returns a new `ArcReadonlyBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, ArcReadonlyBiConsumer};
    ///
    /// let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + Send + Sync + 'static,
    {
        ArcReadonlyBiConsumer {
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
        ArcReadonlyBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another ArcReadonlyBiConsumer in sequence
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
    ///   - An `ArcReadonlyBiConsumer<T, U>` (passed by reference)
    ///   - Any type implementing `ReadonlyBiConsumer<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, ArcReadonlyBiConsumer};
    ///
    /// let first = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
    pub fn and_then(&self, next: &ArcReadonlyBiConsumer<T, U>) -> ArcReadonlyBiConsumer<T, U> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcReadonlyBiConsumer {
            function: Arc::new(move |t: &T, u: &U| {
                first(t, u);
                second(t, u);
            }),
            name: None,
        }
    }
}

impl<T, U> ReadonlyBiConsumer<T, U> for ArcReadonlyBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxReadonlyBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    fn into_rc(self) -> RcReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        RcReadonlyBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    fn into_arc(self) -> ArcReadonlyBiConsumer<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
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

    fn to_box(&self) -> BoxReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        RcReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_arc(&self) -> ArcReadonlyBiConsumer<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
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

impl<T, U> Clone for ArcReadonlyBiConsumer<T, U> {
    /// Clones the ArcReadonlyBiConsumer
    ///
    /// Creates a new ArcReadonlyBiConsumer sharing the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for ArcReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcReadonlyBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for ArcReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcReadonlyBiConsumer({})", name),
            None => write!(f, "ArcReadonlyBiConsumer"),
        }
    }
}

// =======================================================================
// 4. RcReadonlyBiConsumer - Single-Threaded Shared Ownership
// =======================================================================

/// RcReadonlyBiConsumer struct
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
/// Choose `RcReadonlyBiConsumer` when:
/// - Need to share readonly bi-consumer within a single thread
/// - Pure observation operations, performance critical
/// - Single-threaded UI framework event handling
///
/// # Performance Advantages
///
/// `RcReadonlyBiConsumer` has neither Arc's atomic operation overhead nor
/// RefCell's runtime borrow checking overhead, making it the best
/// performing among the three readonly bi-consumer types.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
///
/// let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
pub struct RcReadonlyBiConsumer<T, U> {
    function: Rc<ReadonlyBiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> RcReadonlyBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new RcReadonlyBiConsumer
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
    /// Returns a new `RcReadonlyBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("Product: {}", x * y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) + 'static,
    {
        RcReadonlyBiConsumer {
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
        RcReadonlyBiConsumer::new(|_, _| {})
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another RcReadonlyBiConsumer in sequence
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
    ///   - An `RcReadonlyBiConsumer<T, U>` (passed by reference)
    ///   - Any type implementing `ReadonlyBiConsumer<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, RcReadonlyBiConsumer};
    ///
    /// let first = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
    ///     println!("First: {}, {}", x, y);
    /// });
    /// let second = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
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
    pub fn and_then(&self, next: &RcReadonlyBiConsumer<T, U>) -> RcReadonlyBiConsumer<T, U> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcReadonlyBiConsumer {
            function: Rc::new(move |t: &T, u: &U| {
                first(t, u);
                second(t, u);
            }),
            name: None,
        }
    }
}

impl<T, U> ReadonlyBiConsumer<T, U> for RcReadonlyBiConsumer<T, U> {
    fn accept(&self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxReadonlyBiConsumer::new(move |t, u| (self.function)(t, u))
    }

    fn into_rc(self) -> RcReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    // do NOT override ReadonlyBiConsumer::into_arc() because RcReadonlyBiConsumer is not Send + Sync
    // and calling RcReadonlyBiConsumer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        move |t, u| (self.function)(t, u)
    }

    fn to_box(&self) -> BoxReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        BoxReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcReadonlyBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self.clone()
    }

    // do NOT override ReadonlyBiConsumer::to_arc() because RcReadonlyBiConsumer is not Send + Sync
    // and calling RcReadonlyBiConsumer::to_arc() will cause a compile error

    fn to_fn(&self) -> impl Fn(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function.clone();
        move |t, u| self_fn(t, u)
    }
}

impl<T, U> Clone for RcReadonlyBiConsumer<T, U> {
    /// Clones the RcReadonlyBiConsumer
    ///
    /// Creates a new RcReadonlyBiConsumer sharing the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for RcReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcReadonlyBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for RcReadonlyBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcReadonlyBiConsumer({})", name),
            None => write!(f, "RcReadonlyBiConsumer"),
        }
    }
}

// =======================================================================
// 5. Implement ReadonlyBiConsumer trait for closures
// =======================================================================

/// Implements ReadonlyBiConsumer for all Fn(&T, &U)
impl<T, U, F> ReadonlyBiConsumer<T, U> for F
where
    F: Fn(&T, &U),
{
    fn accept(&self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box(self) -> BoxReadonlyBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxReadonlyBiConsumer::new(self)
    }

    fn into_rc(self) -> RcReadonlyBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcReadonlyBiConsumer::new(self)
    }

    fn into_arc(self) -> ArcReadonlyBiConsumer<T, U>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        ArcReadonlyBiConsumer::new(self)
    }

    fn into_fn(self) -> impl Fn(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxReadonlyBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.clone();
        BoxReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcReadonlyBiConsumer<T, U>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.clone();
        RcReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
    }

    fn to_arc(&self) -> ArcReadonlyBiConsumer<T, U>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        let self_fn = self.clone();
        ArcReadonlyBiConsumer::new(move |t, u| self_fn(t, u))
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
/// - **Returns BoxReadonlyBiConsumer**: Composition results can be
///   further chained
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `Fn(&T, &U)` closures get these
///   methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ReadonlyBiConsumer, FnReadonlyBiConsumerOps};
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
pub trait FnReadonlyBiConsumerOps<T, U>: Fn(&T, &U) + Sized {
    /// Chains another readonly bi-consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes the current closure and returns
    /// `BoxReadonlyBiConsumer<T, U>`.
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
    /// Returns the composed `BoxReadonlyBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyBiConsumer, FnReadonlyBiConsumerOps};
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
    fn and_then<C>(self, next: C) -> BoxReadonlyBiConsumer<T, U>
    where
        Self: 'static,
        C: ReadonlyBiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let first = self;
        let second = next;
        BoxReadonlyBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnReadonlyBiConsumerOps for all closure types
impl<T, U, F> FnReadonlyBiConsumerOps<T, U> for F where F: Fn(&T, &U) {}
