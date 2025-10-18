/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Types
//!
//! Provides Java-style one-time `Mutator` interface implementations for performing
//! operations that consume self and modify the input value.
//!
//! This module provides a unified `MutatorOnce` trait and a Box-based single
//! ownership implementation:
//!
//! - **`BoxMutatorOnce<T>`**: Box-based single ownership implementation for
//!   one-time use scenarios
//!
//! # Design Philosophy
//!
//! The key difference between `MutatorOnce` and `Mutator`:
//!
//! - **Mutator**: `&mut self`, can be called multiple times, uses `FnMut(&mut T)`
//! - **MutatorOnce**: `self`, can only be called once, uses `FnOnce(&mut T)`
//!
//! ## MutatorOnce vs Mutator
//!
//! | Feature | Mutator | MutatorOnce |
//! |---------|---------|-------------|
//! | **Self Parameter** | `&mut self` | `self` |
//! | **Call Count** | Multiple | Once |
//! | **Closure Type** | `FnMut(&mut T)` | `FnOnce(&mut T)` |
//! | **Use Cases** | Repeatable modifications | One-time resource transfers, init callbacks |
//!
//! # Why MutatorOnce?
//!
//! Core value of MutatorOnce:
//!
//! 1. **Store FnOnce closures**: Allows moving captured variables
//! 2. **Delayed execution**: Store in data structures, execute later
//! 3. **Resource transfer**: Suitable for scenarios requiring ownership transfer
//!
//! # Why Only Box Variant?
//!
//! - **Arc/Rc conflicts with FnOnce semantics**: FnOnce can only be called once,
//!   while shared ownership implies multiple references
//! - **Box is perfect match**: Single ownership aligns perfectly with one-time
//!   call semantics
//!
//! # Use Cases
//!
//! ## BoxMutatorOnce
//!
//! - Post-initialization callbacks (moving data)
//! - Resource transfer (moving Vec, String, etc.)
//! - One-time complex operations (requiring moved capture variables)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxMutatorOnce, MutatorOnce};
//!
//! let data = vec![1, 2, 3];
//! let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
//!     x.extend(data); // Move data
//! });
//!
//! let mut target = vec![0];
//! mutator.mutate(&mut target);
//! assert_eq!(target, vec![0, 1, 2, 3]);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{BoxMutatorOnce, MutatorOnce};
//!
//! let data1 = vec![1, 2];
//! let data2 = vec![3, 4];
//!
//! let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
//!     x.extend(data1);
//! })
//! .and_then(move |x: &mut Vec<i32>| {
//!     x.extend(data2);
//! });
//!
//! let mut target = vec![0];
//! chained.mutate(&mut target);
//! assert_eq!(target, vec![0, 1, 2, 3, 4]);
//! ```
//!
//! ## Initialization Callback
//!
//! ```rust
//! use prism3_function::BoxMutatorOnce;
//!
//! struct Initializer {
//!     on_complete: Option<BoxMutatorOnce<Vec<i32>>>,
//! }
//!
//! impl Initializer {
//!     fn new<F>(callback: F) -> Self
//!     where
//!         F: FnOnce(&mut Vec<i32>) + 'static
//!     {
//!         Self {
//!             on_complete: Some(BoxMutatorOnce::new(callback))
//!         }
//!     }
//!
//!     fn run(mut self, data: &mut Vec<i32>) {
//!         // Execute initialization logic
//!         data.push(42);
//!
//!         // Call callback
//!         if let Some(callback) = self.on_complete.take() {
//!             callback.mutate(data);
//!         }
//!     }
//! }
//!
//! let data_to_add = vec![1, 2, 3];
//! let init = Initializer::new(move |x| {
//!     x.extend(data_to_add); // Move data_to_add
//! });
//!
//! let mut result = Vec::new();
//! init.run(&mut result);
//! assert_eq!(result, vec![42, 1, 2, 3]);
//! ```
//!
//! # Author
//!
//! Haixing Hu

// ============================================================================
// 1. MutatorOnce Trait - One-time Mutator Interface
// ============================================================================

/// MutatorOnce trait - One-time mutator interface
///
/// Defines the core behavior of all one-time mutator types. Performs operations
/// that consume self and modify the input value.
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnOnce(&mut T)`
/// - `BoxMutatorOnce<T>`
///
/// # Design Rationale
///
/// This trait provides a unified abstraction for one-time mutation operations.
/// The key difference from `Mutator`:
/// - `Mutator` uses `&mut self`, can be called multiple times
/// - `MutatorOnce` uses `self`, can only be called once
///
/// # Features
///
/// - **Unified Interface**: All one-time mutators share the same `mutate`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Provides `into_box` method for type conversion
/// - **Generic Programming**: Write functions that work with any one-time
///   mutator type
///
/// # Examples
///
/// ## Generic Function
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// fn apply_once<M: MutatorOnce<Vec<i32>>>(
///     mutator: M,
///     initial: Vec<i32>
/// ) -> Vec<i32> {
///     let mut val = initial;
///     mutator.mutate(&mut val);
///     val
/// }
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data);
/// });
/// let result = apply_once(mutator, vec![0]);
/// assert_eq!(result, vec![0, 1, 2, 3]);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::MutatorOnce;
///
/// let data = vec![1, 2, 3];
/// let closure = move |x: &mut Vec<i32>| x.extend(data);
/// let box_mutator = closure.into_box();
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait MutatorOnce<T> {
    /// Performs the one-time mutation operation
    ///
    /// Consumes self and executes an operation on the given mutable reference.
    /// The operation typically modifies the input value or produces side effects,
    /// and can only be called once.
    ///
    /// # Parameters
    ///
    /// * `value` - A mutable reference to the value to be mutated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data = vec![1, 2, 3];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data);
    /// });
    ///
    /// let mut target = vec![0];
    /// mutator.mutate(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    /// ```
    fn mutate(self, value: &mut T);

    /// Converts to BoxMutatorOnce
    ///
    /// **⚠️ Consumes `self`**: The original mutator becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current mutator to `BoxMutatorOnce<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the mutator (takes ownership of `self`).
    /// After calling this method, the original mutator is no longer
    /// available.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::MutatorOnce;
    ///
    /// let data = vec![1, 2, 3];
    /// let closure = move |x: &mut Vec<i32>| x.extend(data);
    /// let box_mutator = closure.into_box();
    /// ```
    fn into_box(self) -> BoxMutatorOnce<T>
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxMutatorOnce - Single Ownership Implementation
// ============================================================================

/// BoxMutatorOnce struct
///
/// A one-time mutator implementation based on `Box<dyn FnOnce(&mut T)>` for
/// single ownership scenarios. This is the only MutatorOnce implementation type
/// because FnOnce conflicts with shared ownership semantics.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes self on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Move Semantics**: Can capture and move variables
/// - **Method Chaining**: Compose multiple operations via `and_then`
///
/// # Use Cases
///
/// Choose `BoxMutatorOnce` when:
/// - Need to store FnOnce closures (with moved captured variables)
/// - One-time resource transfer operations
/// - Post-initialization callbacks
/// - Complex operations requiring ownership transfer
///
/// # Performance
///
/// `BoxMutatorOnce` performance characteristics:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Why No Arc/Rc Variants?
///
/// FnOnce can only be called once, which conflicts with Arc/Rc shared ownership
/// semantics:
/// - Arc/Rc implies multiple owners might need to call
/// - FnOnce is consumed after calling, cannot be called again
/// - This semantic incompatibility makes Arc/Rc variants meaningless
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data); // Move data
/// });
///
/// let mut target = vec![0];
/// mutator.mutate(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data1);
/// })
/// .and_then(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
///
/// let mut target = vec![0];
/// chained.mutate(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3, 4]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxMutatorOnce<T> {
    func: Box<dyn FnOnce(&mut T)>,
}

impl<T> BoxMutatorOnce<T>
where
    T: 'static,
{
    /// Creates a new BoxMutatorOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatorOnce<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data = String::from("world");
    /// let mutator = BoxMutatorOnce::new(move |x: &mut String| {
    ///     x.push_str(" ");
    ///     x.push_str(&data); // Move data
    /// });
    ///
    /// let mut target = String::from("hello");
    /// mutator.mutate(&mut target);
    /// assert_eq!(target, "hello world");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut T) + 'static,
    {
        BoxMutatorOnce { func: Box::new(f) }
    }

    /// Creates a no-op mutator
    ///
    /// Returns a mutator that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let noop = BoxMutatorOnce::<i32>::noop();
    /// let mut value = 42;
    /// noop.mutate(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxMutatorOnce::new(|_| {})
    }

    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    /// let data3 = vec![5, 6];
    ///
    /// let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data1);
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data3);
    /// });
    ///
    /// let mut target = vec![0];
    /// chained.mutate(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: MutatorOnce<T> + 'static,
    {
        let first = self.func;
        BoxMutatorOnce::new(move |t| {
            first(t);
            next.mutate(t);
        })
    }
}

impl<T> MutatorOnce<T> for BoxMutatorOnce<T> {
    fn mutate(self, value: &mut T) {
        (self.func)(value)
    }

    fn into_box(self) -> BoxMutatorOnce<T>
    where
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 3. Implement MutatorOnce trait for closures
// ============================================================================

impl<T, F> MutatorOnce<T> for F
where
    F: FnOnce(&mut T),
{
    fn mutate(self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxMutatorOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxMutatorOnce::new(self)
    }
}

// ============================================================================
// 4. Provide extension methods for closures
// ============================================================================

/// Extension trait providing one-time mutator composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnOnce(&mut T)`, enabling direct method chaining on closures
/// without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxMutatorOnce**: Composition results are `BoxMutatorOnce<T>`
///   for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnOnce(&mut T)` closures get these
///   methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{MutatorOnce, FnMutatorOnceOps};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = (move |x: &mut Vec<i32>| x.extend(data1))
///     .and_then(move |x: &mut Vec<i32>| x.extend(data2));
///
/// let mut target = vec![0];
/// chained.mutate(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3, 4]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnMutatorOnceOps<T>: FnOnce(&mut T) + Sized {
    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxMutatorOnce<T>`.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, FnMutatorOnceOps};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    ///
    /// let chained = (move |x: &mut Vec<i32>| x.extend(data1))
    ///     .and_then(move |x: &mut Vec<i32>| x.extend(data2));
    ///
    /// let mut target = vec![0];
    /// chained.mutate(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxMutatorOnce<T>
    where
        Self: 'static,
        C: MutatorOnce<T> + 'static,
        T: 'static,
    {
        BoxMutatorOnce::new(move |t| {
            self(t);
            next.mutate(t);
        })
    }
}

/// Implements FnMutatorOnceOps for all closure types
impl<T, F> FnMutatorOnceOps<T> for F where F: FnOnce(&mut T) {}
