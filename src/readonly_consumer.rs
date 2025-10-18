/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ReadonlyConsumer Types
//!
//! Provides implementations of readonly consumer interfaces for executing operations that neither modify their own state nor modify input values.
//!
//! This module provides a unified `ReadonlyConsumer` trait and three concrete implementations based on different ownership models:
//!
//! - **`BoxReadonlyConsumer<T>`**: Box-based single ownership implementation
//! - **`ArcReadonlyConsumer<T>`**: Arc-based thread-safe shared ownership implementation
//! - **`RcReadonlyConsumer<T>`**: Rc-based single-threaded shared ownership implementation
//!
//! # Design Philosophy
//!
//! ReadonlyConsumer uses `Fn(&T)` semantics, neither modifying its own state nor modifying input values.
//! Suitable for pure observation, logging, notification and other scenarios. Compared to Consumer, ReadonlyConsumer
//! does not require interior mutability (Mutex/RefCell), making it more efficient and easier to share.
//!
//! # Author
//!
//! Hu Haixing

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// 1. ReadonlyConsumer Trait - Unified ReadonlyConsumer Interface
// ============================================================================

/// ReadonlyConsumer trait - Unified readonly consumer interface
///
/// Defines the core behavior of all readonly consumer types. Unlike `Consumer`, `ReadonlyConsumer`
/// neither modifies its own state nor modifies input values, making it a completely immutable operation.
///
/// # Auto-implementation
///
/// - All closures implementing `Fn(&T)`
/// - `BoxReadonlyConsumer<T>`, `ArcReadonlyConsumer<T>`, `RcReadonlyConsumer<T>`
///
/// # Features
///
/// - **Unified Interface**: All readonly consumer types share the same `accept` method signature
/// - **Auto-implementation**: Closures automatically implement this trait with zero overhead
/// - **Type Conversion**: Easy conversion between different ownership models
/// - **Generic Programming**: Write functions that work with any readonly consumer type
/// - **No Interior Mutability**: No need for Mutex or RefCell, more efficient
///
/// # Examples
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
/// # Author
///
/// Hu Haixing
pub trait ReadonlyConsumer<T> {
    /// Execute readonly consumption operation
    ///
    /// Performs an operation on the given reference. The operation typically reads input values or produces side effects,
    /// but neither modifies the input value nor the consumer's own state.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to consume
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let consumer = BoxReadonlyConsumer::new(|x: &i32| println!("{}", x));
    /// consumer.accept(&5);
    /// ```
    fn accept(&self, value: &T);

    /// Convert to BoxReadonlyConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxReadonlyConsumer<T>`
    fn into_box(self) -> BoxReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to RcReadonlyConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcReadonlyConsumer<T>`
    fn into_rc(self) -> RcReadonlyConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to ArcReadonlyConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcReadonlyConsumer<T>`
    fn into_arc(self) -> ArcReadonlyConsumer<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static;

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts a readonly consumer to a closure that can be used directly in places where the standard library requires `Fn`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T)`
    ///
    /// # Examples
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
// 2. BoxReadonlyConsumer - Single Ownership Implementation
// ============================================================================

/// BoxReadonlyConsumer struct
///
/// Readonly consumer implementation based on `Box<dyn Fn(&T)>` for single ownership scenarios.
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
/// Choose `BoxReadonlyConsumer` when:
/// - Readonly consumer is used once or in a linear flow
/// - No need to share consumer across contexts
/// - Pure observation operations, such as logging
///
/// # Examples
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
/// # Author
///
/// Hu Haixing
pub struct BoxReadonlyConsumer<T> {
    function: Box<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> BoxReadonlyConsumer<T>
where
    T: 'static,
{
    /// Create a new BoxReadonlyConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxReadonlyConsumer<T>` instance
    ///
    /// # Examples
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

    /// Get the consumer's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another readonly consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation. Consumes self.
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
    ///   - A `BoxReadonlyConsumer<T>`
    ///   - An `RcReadonlyConsumer<T>`
    ///   - An `ArcReadonlyConsumer<T>`
    ///   - Any type implementing `ReadonlyConsumer<T>`
    ///
    /// # Returns
    ///
    /// Returns a new combined `BoxReadonlyConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer};
    ///
    /// let first = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("First: {}", x);
    /// });
    /// let second = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("Second: {}", x);
    /// });
    ///
    /// // second is moved here
    /// let chained = first.and_then(second);
    /// chained.accept(&5);
    /// // second.accept(&3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{ReadonlyConsumer, BoxReadonlyConsumer, RcReadonlyConsumer};
    ///
    /// let first = BoxReadonlyConsumer::new(|x: &i32| {
    ///     println!("First: {}", x);
    /// });
    /// let second = RcReadonlyConsumer::new(|x: &i32| {
    ///     println!("Second: {}", x);
    /// });
    ///
    /// // Clone to preserve original
    /// let chained = first.and_then(second.clone());
    /// chained.accept(&5);
    ///
    /// // Original still usable
    /// second.accept(&3);
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

    /// Create a no-op consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op consumer
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
// 3. ArcReadonlyConsumer - Thread-safe Shared Ownership Implementation
// ============================================================================

/// ArcReadonlyConsumer struct
///
/// Readonly consumer implementation based on `Arc<dyn Fn(&T) + Send + Sync>`,
/// for thread-safe shared ownership scenarios. No Mutex needed because operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allows multiple owners
/// - **Thread Safe**: Implements `Send + Sync`, can be safely used concurrently
/// - **Lock-free**: No Mutex protection needed because it's readonly
/// - **Non-consuming API**: `and_then` borrows `&self`, original object remains usable
///
/// # Use Cases
///
/// Choose `ArcReadonlyConsumer` when:
/// - Need to share readonly consumer across multiple threads
/// - Pure observation operations, such as logging, monitoring, notifications
/// - Need high-concurrency reads with no lock overhead
///
/// # Performance Advantages
///
/// Compared to `ArcConsumer`, `ArcReadonlyConsumer` has no Mutex lock overhead,
/// performing better in high-concurrency scenarios.
///
/// # Examples
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
/// # Author
///
/// Hu Haixing
pub struct ArcReadonlyConsumer<T> {
    function: Arc<dyn Fn(&T) + Send + Sync>,
    name: Option<String>,
}

impl<T> ArcReadonlyConsumer<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new ArcReadonlyConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcReadonlyConsumer<T>` instance
    ///
    /// # Examples
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

    /// Get the consumer's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Convert to closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function through Arc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T)`
    ///
    /// # Examples
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

    /// Sequentially chain another ArcReadonlyConsumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Borrows &self, does not consume the original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. **Note: This
    ///   parameter is passed by reference, so the original consumer remains
    ///   usable.** Can be:
    ///   - An `ArcReadonlyConsumer<T>` (passed by reference)
    ///   - Any type implementing `ReadonlyConsumer<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns a new combined `ArcReadonlyConsumer<T>`
    ///
    /// # Examples
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
    /// // second is passed by reference, so it remains usable
    /// let chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// first.accept(&3); // Still usable
    /// second.accept(&7); // Still usable
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
    /// Clone ArcReadonlyConsumer
    ///
    /// Creates a new ArcReadonlyConsumer that shares the underlying function with the original instance.
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
// 4. RcReadonlyConsumer - Single-threaded Shared Ownership Implementation
// ============================================================================

/// RcReadonlyConsumer struct
///
/// Readonly consumer implementation based on `Rc<dyn Fn(&T)>` for single-threaded shared ownership scenarios.
/// No RefCell needed because operations are readonly.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allows multiple owners
/// - **Single-threaded**: Not thread-safe, cannot be sent across threads
/// - **No Interior Mutability Overhead**: No RefCell needed because it's readonly
/// - **Non-consuming API**: `and_then` borrows `&self`, original object remains usable
///
/// # Use Cases
///
/// Choose `RcReadonlyConsumer` when:
/// - Need to share readonly consumer within a single thread
/// - Pure observation operations, performance critical
/// - Event handling in single-threaded UI frameworks
///
/// # Performance Advantages
///
/// `RcReadonlyConsumer` has neither Arc's atomic operation overhead nor RefCell's
/// runtime borrow checking overhead, making it the most performant of the three readonly consumers.
///
/// # Examples
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
/// # Author
///
/// Hu Haixing
pub struct RcReadonlyConsumer<T> {
    function: Rc<dyn Fn(&T)>,
    name: Option<String>,
}

impl<T> RcReadonlyConsumer<T>
where
    T: 'static,
{
    /// Create a new RcReadonlyConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcReadonlyConsumer<T>` instance
    ///
    /// # Examples
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

    /// Get the consumer's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Convert to closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function through Rc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `Fn(&T)`
    ///
    /// # Examples
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

    /// Sequentially chain another RcReadonlyConsumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Borrows &self, does not consume the original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation. **Note: This
    ///   parameter is passed by reference, so the original consumer remains
    ///   usable.** Can be:
    ///   - An `RcReadonlyConsumer<T>` (passed by reference)
    ///   - Any type implementing `ReadonlyConsumer<T>`
    ///
    /// # Returns
    ///
    /// Returns a new combined `RcReadonlyConsumer<T>`
    ///
    /// # Examples
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
    /// // second is passed by reference, so it remains usable
    /// let chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// first.accept(&3); // Still usable
    /// second.accept(&7); // Still usable
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
    /// Clone RcReadonlyConsumer
    ///
    /// Creates a new RcReadonlyConsumer that shares the underlying function with the original instance.
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
// 5. Implement ReadonlyConsumer trait for closures
// ============================================================================

/// Implement ReadonlyConsumer for all Fn(&T)
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
// 6. Provide extension methods for closures
// ============================================================================

/// Extension trait providing readonly consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures implementing `Fn(&T)`,
/// allowing closures to directly chain methods without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxReadonlyConsumer**: Combined results can continue chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Auto-implementation**: All `Fn(&T)` closures automatically get these methods
///
/// # Examples
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
/// # Author
///
/// Hu Haixing
pub trait FnReadonlyConsumerOps<T>: Fn(&T) + Sized {
    /// Sequentially chain another readonly consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Consumes the current closure and returns `BoxReadonlyConsumer<T>`.
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
    /// Returns a combined `BoxReadonlyConsumer<T>`
    ///
    /// # Examples
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

/// Implement FnReadonlyConsumerOps for all closure types
impl<T, F> FnReadonlyConsumerOps<T> for F where F: Fn(&T) {}
