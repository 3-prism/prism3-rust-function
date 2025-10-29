/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatingFunctionOnce Types
//!
//! Provides Java-like one-time `MutatingFunction` interface implementations
//! for performing operations that consume self, accept a mutable reference,
//! and return a result.
//!
//! This module provides a unified `MutatingFunctionOnce` trait and a
//! Box-based single ownership implementation:
//!
//! - **`BoxMutatingFunctionOnce<T, R>`**: Box-based single ownership
//!   implementation for one-time use scenarios
//!
//! # Design Philosophy
//!
//! The key difference between `MutatingFunctionOnce` and
//! `MutatingFunction`:
//!
//! - **MutatingFunction**: `&self`, can be called multiple times, uses
//!   `Fn(&mut T) -> R`
//! - **MutatingFunctionOnce**: `self`, can only be called once, uses
//!   `FnOnce(&mut T) -> R`
//!
//! ## MutatingFunctionOnce vs MutatingFunction
//!
//! | Feature | MutatingFunction | MutatingFunctionOnce |
//! |---------|------------------|----------------------|
//! | **Self Parameter** | `&self` | `self` |
//! | **Call Count** | Multiple | Once |
//! | **Closure Type** | `Fn(&mut T) -> R` | `FnOnce(&mut T) -> R` |
//! | **Use Cases** | Repeatable operations | One-time resource
//! transfers |
//!
//! # Why MutatingFunctionOnce?
//!
//! Core value of MutatingFunctionOnce:
//!
//! 1. **Store FnOnce closures**: Allows moving captured variables
//! 2. **Delayed execution**: Store in data structures, execute later
//! 3. **Resource transfer**: Suitable for scenarios requiring ownership
//!    transfer
//! 4. **Return results**: Unlike MutatorOnce, returns information about the
//!    operation
//!
//! # Why Only Box Variant?
//!
//! - **Arc/Rc conflicts with FnOnce semantics**: FnOnce can only be called
//!   once, while shared ownership implies multiple references
//! - **Box is perfect match**: Single ownership aligns perfectly with
//!   one-time call semantics
//!
//! # Use Cases
//!
//! ## BoxMutatingFunctionOnce
//!
//! - Post-initialization callbacks (moving data, returning status)
//! - Resource transfer with result (moving Vec, returning old value)
//! - One-time complex operations (requiring moved capture variables)
//! - Validation with fixes (fix data once, return validation result)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxMutatingFunctionOnce, MutatingFunctionOnce};
//!
//! let data = vec![1, 2, 3];
//! let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
//!     let old_len = x.len();
//!     x.extend(data); // Move data
//!     old_len
//! });
//!
//! let mut target = vec![0];
//! let old_len = func.apply_once(&mut target);
//! assert_eq!(old_len, 1);
//! assert_eq!(target, vec![0, 1, 2, 3]);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{BoxMutatingFunctionOnce, MutatingFunctionOnce};
//!
//! let data1 = vec![1, 2];
//! let data2 = vec![3, 4];
//!
//! let chained = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
//!     x.extend(data1);
//!     x.len()
//! })
//! .and_then(move |x: &mut Vec<i32>| {
//!     x.extend(data2);
//!     x.len()
//! });
//!
//! let mut target = vec![0];
//! let final_len = chained.apply_once(&mut target);
//! assert_eq!(final_len, 5);
//! assert_eq!(target, vec![0, 1, 2, 3, 4]);
//! ```
//!
//! ## Validation Pattern
//!
//! ```rust
//! use prism3_function::{BoxMutatingFunctionOnce, MutatingFunctionOnce};
//!
//! struct Data {
//!     value: i32,
//! }
//!
//! let validator = BoxMutatingFunctionOnce::new(|data: &mut Data| {
//!     if data.value < 0 {
//!         data.value = 0;
//!         Err("Fixed negative value")
//!     } else {
//!         Ok("Valid")
//!     }
//! });
//!
//! let mut data = Data { value: -5 };
//! let result = validator.apply_once(&mut data);
//! assert_eq!(data.value, 0);
//! assert!(result.is_err());
//! ```
//!
//! # Author
//!
//! Haixing Hu

// =======================================================================
// 1. MutatingFunctionOnce Trait - One-time Function Interface
// =======================================================================

/// MutatingFunctionOnce trait - One-time mutating function interface
///
/// Defines the core behavior of all one-time mutating function types.
/// Performs operations that consume self, accept a mutable reference,
/// potentially modify it, and return a result.
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnOnce(&mut T) -> R`
/// - `BoxMutatingFunctionOnce<T, R>`
///
/// # Design Rationale
///
/// This trait provides a unified abstraction for one-time mutating function
/// operations. The key difference from `MutatingFunction`:
/// - `MutatingFunction` uses `&self`, can be called multiple times
/// - `MutatingFunctionOnce` uses `self`, can only be called once
///
/// # Features
///
/// - **Unified Interface**: All one-time mutating functions share the same
///   `apply_once` method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Provides `into_box` method for type conversion
/// - **Generic Programming**: Write functions that work with any one-time
///   mutating function type
///
/// # Examples
///
/// ## Generic Function
///
/// ```rust
/// use prism3_function::{MutatingFunctionOnce, BoxMutatingFunctionOnce};
///
/// fn apply_once<F: MutatingFunctionOnce<Vec<i32>, usize>>(
///     func: F,
///     initial: Vec<i32>
/// ) -> (Vec<i32>, usize) {
///     let mut val = initial;
///     let result = func.apply_once(&mut val);
///     (val, result)
/// }
///
/// let data = vec![1, 2, 3];
/// let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
///     let old_len = x.len();
///     x.extend(data);
///     old_len
/// });
/// let (vec, old_len) = apply_once(func, vec![0]);
/// assert_eq!(vec, vec![0, 1, 2, 3]);
/// assert_eq!(old_len, 1);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::MutatingFunctionOnce;
///
/// let data = vec![1, 2, 3];
/// let closure = move |x: &mut Vec<i32>| {
///     let old_len = x.len();
///     x.extend(data);
///     old_len
/// };
/// let box_func = closure.into_box_once();
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait MutatingFunctionOnce<T, R> {
    /// Performs the one-time mutating function operation
    ///
    /// Consumes self and executes an operation on the given mutable
    /// reference, potentially modifying it, and returns a result. The
    /// operation can only be called once.
    ///
    /// # Parameters
    ///
    /// * `input` - A mutable reference to the input value
    ///
    /// # Returns
    ///
    /// The computed result value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       BoxMutatingFunctionOnce};
    ///
    /// let data = vec![1, 2, 3];
    /// let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
    ///     let old_len = x.len();
    ///     x.extend(data);
    ///     old_len
    /// });
    ///
    /// let mut target = vec![0];
    /// let old_len = func.apply_once(&mut target);
    /// assert_eq!(old_len, 1);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    /// ```
    fn apply_once(self, input: &mut T) -> R;

    /// Converts to `BoxMutatingFunctionOnce` (consuming)
    ///
    /// Consumes `self` and returns an owned `BoxMutatingFunctionOnce<T, R>`.
    /// The default implementation simply wraps the consuming
    /// `apply_once(self, &mut T)` call in a `Box<dyn FnOnce(&mut T) -> R>`.
    /// Types that can provide a cheaper or identity conversion (for example
    /// `BoxMutatingFunctionOnce` itself) should override this method.
    ///
    /// # Note
    ///
    /// - This method consumes the source value.
    /// - Implementors may return `self` directly when `Self` is already a
    ///   `BoxMutatingFunctionOnce<T, R>` to avoid the extra wrapper
    ///   allocation.
    fn into_box_once(self) -> BoxMutatingFunctionOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMutatingFunctionOnce::new(move |t| self.apply_once(t))
    }

    /// Converts to a consuming closure `FnOnce(&mut T) -> R`
    ///
    /// Consumes `self` and returns a closure that, when invoked, calls
    /// `apply_once(self, &mut T)`. This is the default, straightforward
    /// implementation; types that can produce a more direct function pointer
    /// or avoid additional captures may override it.
    fn into_fn_once(self) -> impl FnOnce(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |t| self.apply_once(t)
    }

    /// Non-consuming adapter to `BoxMutatingFunctionOnce`
    ///
    /// Creates a `BoxMutatingFunctionOnce<T, R>` that does not consume
    /// `self`. The default implementation requires `Self: Clone` and clones
    /// the receiver for the stored closure; the clone is consumed when the
    /// boxed function is invoked. Types that can provide a zero-cost adapter
    /// (for example clonable closures) should override this method to avoid
    /// unnecessary allocations.
    fn to_box_once(&self) -> BoxMutatingFunctionOnce<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box_once()
    }

    /// Non-consuming adapter to a callable `FnOnce(&mut T) -> R`
    ///
    /// Returns a closure that does not consume `self`. The default requires
    /// `Self: Clone` and clones `self` for the captured closure; the clone is
    /// consumed when the returned closure is invoked. Implementors may
    /// provide more efficient adapters for specific types.
    fn to_fn_once(&self) -> impl FnOnce(&mut T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn_once()
    }
}

// =======================================================================
// 2. BoxMutatingFunctionOnce - Single Ownership Implementation
// =======================================================================

/// BoxMutatingFunctionOnce struct
///
/// A one-time mutating function implementation based on
/// `Box<dyn FnOnce(&mut T) -> R>` for single ownership scenarios. This is
/// the only MutatingFunctionOnce implementation type because FnOnce
/// conflicts with shared ownership semantics.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes self on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Move Semantics**: Can capture and move variables
/// - **Method Chaining**: Compose multiple operations via `and_then`
/// - **Returns Results**: Unlike MutatorOnce, returns information
///
/// # Use Cases
///
/// Choose `BoxMutatingFunctionOnce` when:
/// - Need to store FnOnce closures (with moved captured variables)
/// - One-time resource transfer operations with results
/// - Post-initialization callbacks that return status
/// - Complex operations requiring ownership transfer and results
///
/// # Performance
///
/// `BoxMutatingFunctionOnce` performance characteristics:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Why No Arc/Rc Variants?
///
/// FnOnce can only be called once, which conflicts with Arc/Rc shared
/// ownership semantics:
/// - Arc/Rc implies multiple owners might need to call
/// - FnOnce is consumed after calling, cannot be called again
/// - This semantic incompatibility makes Arc/Rc variants meaningless
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{MutatingFunctionOnce, BoxMutatingFunctionOnce};
///
/// let data = vec![1, 2, 3];
/// let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
///     let old_len = x.len();
///     x.extend(data); // Move data
///     old_len
/// });
///
/// let mut target = vec![0];
/// let old_len = func.apply_once(&mut target);
/// assert_eq!(old_len, 1);
/// assert_eq!(target, vec![0, 1, 2, 3]);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{MutatingFunctionOnce, BoxMutatingFunctionOnce};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data1);
///     x.len()
/// })
/// .and_then(move |x: &mut Vec<i32>| {
///     x.extend(data2);
///     x.len()
/// });
///
/// let mut target = vec![0];
/// let final_len = chained.apply_once(&mut target);
/// assert_eq!(final_len, 5);
/// assert_eq!(target, vec![0, 1, 2, 3, 4]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxMutatingFunctionOnce<T, R> {
    function: Box<dyn FnOnce(&mut T) -> R>,
}

impl<T, R> BoxMutatingFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxMutatingFunctionOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatingFunctionOnce<T, R>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       BoxMutatingFunctionOnce};
    ///
    /// let data = String::from("world");
    /// let func = BoxMutatingFunctionOnce::new(move |x: &mut String| {
    ///     let old_len = x.len();
    ///     x.push_str(" ");
    ///     x.push_str(&data); // Move data
    ///     old_len
    /// });
    ///
    /// let mut target = String::from("hello");
    /// let old_len = func.apply_once(&mut target);
    /// assert_eq!(old_len, 5);
    /// assert_eq!(target, "hello world");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut T) -> R + 'static,
    {
        BoxMutatingFunctionOnce {
            function: Box::new(f),
        }
    }

    /// Creates an identity function
    ///
    /// Returns a function that returns a clone of the input value without
    /// modifying it. Only available when `T` and `R` are the same type.
    ///
    /// # Returns
    ///
    /// Returns an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       BoxMutatingFunctionOnce};
    ///
    /// let identity = BoxMutatingFunctionOnce::<i32, i32>::identity();
    /// let mut value = 42;
    /// let result = identity.apply_once(&mut value);
    /// assert_eq!(result, 42);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn identity() -> Self
    where
        T: Clone,
        R: From<T>,
    {
        BoxMutatingFunctionOnce::new(|t: &mut T| R::from(t.clone()))
    }

    /// Chains another mutating function in sequence
    ///
    /// Returns a new function that first executes the current operation, then
    /// executes the next operation. The result of the first operation is
    /// discarded, and the result of the second operation is returned.
    /// Consumes self.
    ///
    /// # Parameters
    ///
    /// * `next` - The function to execute after the current operation.
    ///   **Note: This parameter is passed by value and will transfer
    ///   ownership.** Since `BoxMutatingFunctionOnce` cannot be cloned, the
    ///   parameter will be consumed. Can be:
    ///   - A closure: `|x: &mut T| -> R2`
    ///   - A `BoxMutatingFunctionOnce<T, R2>`
    ///   - Any type implementing `MutatingFunctionOnce<T, R2>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxMutatingFunctionOnce<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       BoxMutatingFunctionOnce};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    /// let data3 = vec![5, 6];
    ///
    /// let chained = BoxMutatingFunctionOnce::new(
    ///     move |x: &mut Vec<i32>| {
    ///         x.extend(data1);
    ///         x.len()
    ///     }
    /// )
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    ///     x.len()
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data3);
    ///     x.len()
    /// });
    ///
    /// let mut target = vec![0];
    /// let final_len = chained.apply_once(&mut target);
    /// assert_eq!(final_len, 7);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn and_then<F, R2>(self, next: F) -> BoxMutatingFunctionOnce<T, R2>
    where
        F: MutatingFunctionOnce<T, R2> + 'static,
        R2: 'static,
    {
        let first = self.function;
        BoxMutatingFunctionOnce::new(move |t| {
            let _ = first(t);
            next.apply_once(t)
        })
    }

    /// Maps the result of this function using another function
    ///
    /// Returns a new function that applies this function and then transforms
    /// the result using the provided mapping function.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to transform the result
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatingFunctionOnce<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       BoxMutatingFunctionOnce};
    ///
    /// let data = vec![1, 2, 3];
    /// let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
    ///     let old_len = x.len();
    ///     x.extend(data);
    ///     old_len
    /// });
    /// let mapped = func.map(|old_len| format!("Old length: {}", old_len));
    ///
    /// let mut target = vec![0];
    /// let result = mapped.apply_once(&mut target);
    /// assert_eq!(result, "Old length: 1");
    /// ```
    pub fn map<F, R2>(self, mapper: F) -> BoxMutatingFunctionOnce<T, R2>
    where
        F: FnOnce(R) -> R2 + 'static,
        R2: 'static,
    {
        let func = self.function;
        BoxMutatingFunctionOnce::new(move |t| {
            let result = func(t);
            mapper(result)
        })
    }
}

impl<T, R> MutatingFunctionOnce<T, R> for BoxMutatingFunctionOnce<T, R> {
    fn apply_once(self, input: &mut T) -> R {
        (self.function)(input)
    }

    fn into_box_once(self) -> BoxMutatingFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        self
    }

    fn into_fn_once(self) -> impl FnOnce(&mut T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t| (self.function)(t)
    }
}

// =======================================================================
// 3. Implement MutatingFunctionOnce trait for closures
// =======================================================================

impl<T, R, F> MutatingFunctionOnce<T, R> for F
where
    F: FnOnce(&mut T) -> R,
{
    fn apply_once(self, input: &mut T) -> R {
        self(input)
    }

    fn into_box_once(self) -> BoxMutatingFunctionOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMutatingFunctionOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        self
    }

    // Provide specialized non-consuming conversions for closures that
    // implement `Clone`. Many simple closures are zero-sized and `Clone`,
    // allowing non-consuming adapters to be cheaply produced.
    fn to_box_once(&self) -> BoxMutatingFunctionOnce<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cloned = self.clone();
        BoxMutatingFunctionOnce::new(move |t| cloned.apply_once(t))
    }

    fn to_fn_once(&self) -> impl FnOnce(&mut T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone()
    }
}

// =======================================================================
// 4. Provide extension methods for closures
// =======================================================================

/// Extension trait providing one-time mutating function composition methods
/// for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnOnce(&mut T) -> R`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxMutatingFunctionOnce**: Composition results are
///   `BoxMutatingFunctionOnce<T, R>` for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnOnce(&mut T) -> R` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{MutatingFunctionOnce,
///                       FnOnceMutatingFunctionOps};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = (move |x: &mut Vec<i32>| {
///     x.extend(data1);
///     x.len()
/// })
/// .and_then(move |x: &mut Vec<i32>| {
///     x.extend(data2);
///     x.len()
/// });
///
/// let mut target = vec![0];
/// let final_len = chained.apply_once(&mut target);
/// assert_eq!(final_len, 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnOnceMutatingFunctionOps<T, R>: FnOnce(&mut T) -> R + Sized {
    /// Chains another mutating function in sequence
    ///
    /// Returns a new function that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxMutatingFunctionOnce<T, R2>`.
    ///
    /// # Parameters
    ///
    /// * `next` - The function to execute after the current operation.
    ///   **Note: This parameter is passed by value and will transfer
    ///   ownership.** Since `BoxMutatingFunctionOnce` cannot be cloned, the
    ///   parameter will be consumed. Can be:
    ///   - A closure: `|x: &mut T| -> R2`
    ///   - A `BoxMutatingFunctionOnce<T, R2>`
    ///   - Any type implementing `MutatingFunctionOnce<T, R2>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMutatingFunctionOnce<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       FnOnceMutatingFunctionOps};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    ///
    /// // Both closures are moved and consumed
    /// let chained = (move |x: &mut Vec<i32>| {
    ///     x.extend(data1);
    ///     x.len()
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    ///     x.len()
    /// });
    ///
    /// let mut target = vec![0];
    /// let final_len = chained.apply_once(&mut target);
    /// assert_eq!(final_len, 5);
    /// // The original closures are consumed and no longer usable
    /// ```
    fn and_then<F, R2>(self, next: F) -> BoxMutatingFunctionOnce<T, R2>
    where
        Self: 'static,
        F: MutatingFunctionOnce<T, R2> + 'static,
        T: 'static,
        R: 'static,
        R2: 'static,
    {
        BoxMutatingFunctionOnce::new(move |t| {
            let _ = self(t);
            next.apply_once(t)
        })
    }

    /// Maps the result using another function
    ///
    /// Returns a new function that applies this function and then transforms
    /// the result.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to transform the result
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatingFunctionOnce<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatingFunctionOnce,
    ///                       FnOnceMutatingFunctionOps};
    ///
    /// let data = vec![1, 2, 3];
    /// let mapped = (move |x: &mut Vec<i32>| {
    ///     let old_len = x.len();
    ///     x.extend(data);
    ///     old_len
    /// })
    /// .map(|old_len| format!("Old length: {}", old_len));
    ///
    /// let mut target = vec![0];
    /// let result = mapped.apply_once(&mut target);
    /// assert_eq!(result, "Old length: 1");
    /// ```
    fn map<F, R2>(self, mapper: F) -> BoxMutatingFunctionOnce<T, R2>
    where
        Self: 'static,
        F: FnOnce(R) -> R2 + 'static,
        T: 'static,
        R: 'static,
        R2: 'static,
    {
        BoxMutatingFunctionOnce::new(move |t| {
            let result = self(t);
            mapper(result)
        })
    }
}

/// Implements FnOnceMutatingFunctionOps for all closure types
impl<T, R, F> FnOnceMutatingFunctionOps<T, R> for F where F: FnOnce(&mut T) -> R {}
