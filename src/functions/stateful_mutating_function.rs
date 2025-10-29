/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # StatefulMutatingFunction Types
//!
//! Provides Java-like `StatefulMutatingFunction` interface implementations
//! for performing operations that accept a mutable reference, potentially
//! modify internal state, and return a result.
//!
//! This module provides a unified `StatefulMutatingFunction` trait and three
//! concrete implementations based on different ownership models:
//!
//! - **`BoxStatefulMutatingFunction<T, R>`**: Box-based single ownership
//!   implementation
//! - **`ArcStatefulMutatingFunction<T, R>`**: Arc<Mutex<>>-based thread-safe
//!   shared ownership implementation
//! - **`RcStatefulMutatingFunction<T, R>`**: Rc<RefCell<>>-based
//!   single-threaded shared ownership implementation
//!
//! # Design Philosophy
//!
//! `StatefulMutatingFunction` extends `MutatingFunction` with the ability to
//! maintain internal state:
//!
//! - **MutatingFunction**: `Fn(&mut T) -> R` - stateless, immutable self
//! - **StatefulMutatingFunction**: `FnMut(&mut T) -> R` - stateful, mutable
//!   self
//!
//! ## Comparison with Related Types
//!
//! | Type | Self | Input | Modifies Self? | Modifies Input? | Returns? |
//! |------|------|-------|----------------|-----------------|----------|
//! | **StatefulFunction** | `&mut self` | `&T` | ✅ | ❌ | ✅ |
//! | **StatefulMutator** | `&mut self` | `&mut T` | ✅ | ✅ | ❌ |
//! | **StatefulMutatingFunction** | `&mut self` | `&mut T` | ✅ | ✅ | ✅ |
//!
//! **Key Insight**: Use `StatefulMutatingFunction` when you need to:
//! - Maintain internal state (counters, accumulators, etc.)
//! - Modify the input value
//! - Return information about the operation
//!
//! # Comparison Table
//!
//! | Feature          | Box | Arc | Rc |
//! |------------------|-----|-----|----|
//! | Ownership        | Single | Shared | Shared |
//! | Cloneable        | ❌ | ✅ | ✅ |
//! | Thread-Safe      | ❌ | ✅ | ❌ |
//! | Interior Mut.    | N/A | Mutex | RefCell |
//! | `and_then` API   | `self` | `&self` | `&self` |
//! | Lock Overhead    | None | Yes | None |
//!
//! # Use Cases
//!
//! ## Common Scenarios
//!
//! - **Stateful counters**: Increment and track modification count
//! - **Accumulators**: Collect statistics while modifying data
//! - **Rate limiters**: Track calls and conditionally modify
//! - **Validators**: Accumulate errors while fixing data
//! - **Stateful transformers**: Apply transformations based on history
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxStatefulMutatingFunction,
//!                       StatefulMutatingFunction};
//!
//! // Counter that increments value and tracks calls
//! let mut counter = {
//!     let mut call_count = 0;
//!     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
//!         call_count += 1;
//!         *x += 1;
//!         call_count
//!     })
//! };
//!
//! let mut value = 5;
//! assert_eq!(counter.apply(&mut value), 1);
//! assert_eq!(value, 6);
//! assert_eq!(counter.apply(&mut value), 2);
//! assert_eq!(value, 7);
//! ```
//!
//! ## Accumulator Pattern
//!
//! ```rust
//! use prism3_function::{BoxStatefulMutatingFunction,
//!                       StatefulMutatingFunction};
//!
//! // Accumulate sum while doubling values
//! let mut accumulator = {
//!     let mut sum = 0;
//!     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
//!         *x *= 2;
//!         sum += *x;
//!         sum
//!     })
//! };
//!
//! let mut value = 5;
//! assert_eq!(accumulator.apply(&mut value), 10);
//! assert_eq!(value, 10);
//!
//! let mut value2 = 3;
//! assert_eq!(accumulator.apply(&mut value2), 16); // 10 + 6
//! assert_eq!(value2, 6);
//! ```
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

// =======================================================================
// 1. StatefulMutatingFunction Trait - Unified Interface
// =======================================================================

/// StatefulMutatingFunction trait - Unified stateful mutating function
/// interface
///
/// Defines the core behavior of all stateful mutating function types.
/// Performs operations that accept a mutable reference, potentially modify
/// both the function's internal state and the input, and return a result.
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnMut(&mut T) -> R`
/// - `BoxStatefulMutatingFunction<T, R>`,
///   `ArcStatefulMutatingFunction<T, R>`, and
///   `RcStatefulMutatingFunction<T, R>`
///
/// # Design Rationale
///
/// The trait provides a unified abstraction over different ownership models
/// for operations that need to maintain state while modifying input and
/// returning results. This is useful for scenarios where you need to:
/// - Track statistics or counts during modifications
/// - Accumulate information across multiple calls
/// - Implement stateful validators or transformers
///
/// # Features
///
/// - **Unified Interface**: All stateful mutating function types share the
///   same `apply` method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions that work with any stateful
///   mutating function type
///
/// # Examples
///
/// ## Generic Function
///
/// ```rust
/// use prism3_function::{StatefulMutatingFunction,
///                       BoxStatefulMutatingFunction};
///
/// fn apply_and_log<F: StatefulMutatingFunction<i32, i32>>(
///     func: &mut F,
///     value: i32
/// ) -> i32 {
///     let mut val = value;
///     let result = func.apply(&mut val);
///     println!("Modified: {} -> {}, returned: {}", value, val, result);
///     result
/// }
///
/// let mut counter = {
///     let mut count = 0;
///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x += 1;
///         count
///     })
/// };
/// assert_eq!(apply_and_log(&mut counter, 5), 1);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::StatefulMutatingFunction;
///
/// let mut count = 0;
/// let closure = move |x: &mut i32| {
///     count += 1;
///     *x *= 2;
///     count
/// };
///
/// // Convert to different ownership models
/// let mut box_func = closure.into_box();
/// // let mut rc_func = closure.into_rc();  // closure moved
/// // let mut arc_func = closure.into_arc(); // closure moved
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait StatefulMutatingFunction<T, R> {
    /// Applies the function to the mutable reference and returns a result
    ///
    /// Executes an operation on the given mutable reference, potentially
    /// modifying both the function's internal state and the input, and
    /// returns a result value.
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
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let mut counter = {
    ///     let mut count = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         let old = *x;
    ///         *x += 1;
    ///         (old, count)
    ///     })
    /// };
    ///
    /// let mut value = 5;
    /// let (old_value, call_count) = counter.apply(&mut value);
    /// assert_eq!(old_value, 5);
    /// assert_eq!(call_count, 1);
    /// assert_eq!(value, 6);
    /// ```
    fn apply(&mut self, input: &mut T) -> R;

    /// Convert this function into a `BoxStatefulMutatingFunction<T, R>`.
    ///
    /// This consuming conversion takes ownership of `self` and returns a
    /// boxed implementation that forwards calls to the original function.
    /// Types that can provide a more efficient conversion may override the
    /// default implementation.
    ///
    /// # Consumption
    ///
    /// This method consumes the function: the original value will no longer
    /// be available after the call. For cloneable functions call `.clone()`
    /// before converting if you need to retain the original instance.
    ///
    /// # Returns
    ///
    /// A `BoxStatefulMutatingFunction<T, R>` that forwards to the original
    /// function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::StatefulMutatingFunction;
    ///
    /// let mut count = 0;
    /// let closure = move |x: &mut i32| {
    ///     count += 1;
    ///     *x *= 2;
    ///     count
    /// };
    /// let mut boxed = closure.into_box();
    /// let mut value = 5;
    /// assert_eq!(boxed.apply(&mut value), 1);
    /// ```
    fn into_box(mut self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulMutatingFunction::new(move |t| self.apply(t))
    }

    /// Convert this function into an `RcStatefulMutatingFunction<T, R>`.
    ///
    /// This consuming conversion takes ownership of `self` and returns an
    /// `Rc`-backed function that forwards calls to the original. Override to
    /// provide a more direct or efficient conversion when available.
    ///
    /// # Consumption
    ///
    /// This method consumes the function. If you need to keep the original
    /// instance, clone it prior to calling this method.
    ///
    /// # Returns
    ///
    /// An `RcStatefulMutatingFunction<T, R>` forwarding to the original
    /// function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::StatefulMutatingFunction;
    ///
    /// let mut count = 0;
    /// let closure = move |x: &mut i32| {
    ///     count += 1;
    ///     *x *= 2;
    ///     count
    /// };
    /// let mut rc = closure.into_rc();
    /// let mut value = 5;
    /// assert_eq!(rc.apply(&mut value), 1);
    /// ```
    fn into_rc(mut self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        RcStatefulMutatingFunction::new(move |t| self.apply(t))
    }

    /// Convert this function into an `ArcStatefulMutatingFunction<T, R>`.
    ///
    /// This consuming conversion takes ownership of `self` and returns an
    /// `Arc`-wrapped, thread-safe function. Types may override the default
    /// implementation to provide a more efficient conversion.
    ///
    /// # Consumption
    ///
    /// This method consumes the function. Clone the instance first if you
    /// need to retain the original for further use.
    ///
    /// # Returns
    ///
    /// An `ArcStatefulMutatingFunction<T, R>` that forwards to the original
    /// function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::StatefulMutatingFunction;
    ///
    /// let mut count = 0;
    /// let closure = move |x: &mut i32| {
    ///     count += 1;
    ///     *x *= 2;
    ///     count
    /// };
    /// let mut arc = closure.into_arc();
    /// let mut value = 5;
    /// assert_eq!(arc.apply(&mut value), 1);
    /// ```
    fn into_arc(mut self) -> ArcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        ArcStatefulMutatingFunction::new(move |t| self.apply(t))
    }

    /// Consume the function and return an `FnMut(&mut T) -> R` closure.
    ///
    /// The returned closure forwards calls to the original function and is
    /// suitable for use with iterator adapters or other contexts expecting
    /// closures.
    ///
    /// # Consumption
    ///
    /// This method consumes the function. The original instance will not be
    /// available after calling this method.
    ///
    /// # Returns
    ///
    /// A closure implementing `FnMut(&mut T) -> R` which forwards to the
    /// original function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let func = {
    ///     let mut sum = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         *x *= 2;
    ///         sum += *x;
    ///         sum
    ///     })
    /// };
    /// let mut closure = func.into_fn();
    /// let mut value = 5;
    /// assert_eq!(closure(&mut value), 10);
    /// ```
    fn into_fn(mut self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |t| self.apply(t)
    }

    /// Create a non-consuming `BoxStatefulMutatingFunction<T, R>` that
    /// forwards to `self`.
    ///
    /// The default implementation clones `self` (requires `Clone`) and
    /// returns a boxed function that calls the cloned instance. Override this
    /// method if a more efficient conversion exists.
    ///
    /// # Returns
    ///
    /// A `BoxStatefulMutatingFunction<T, R>` that forwards to a clone of
    /// `self`.
    fn to_box(&self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Create a non-consuming `RcStatefulMutatingFunction<T, R>` that
    /// forwards to `self`.
    ///
    /// The default implementation clones `self` (requires `Clone`) and
    /// returns an `Rc`-backed function that forwards calls to the clone.
    /// Override to provide a more direct or efficient conversion if needed.
    ///
    /// # Returns
    ///
    /// An `RcStatefulMutatingFunction<T, R>` that forwards to a clone of
    /// `self`.
    fn to_rc(&self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_rc()
    }

    /// Create a non-consuming `ArcStatefulMutatingFunction<T, R>` that
    /// forwards to `self`.
    ///
    /// The default implementation clones `self` (requires
    /// `Clone + Send`) and returns an `Arc`-wrapped function that forwards
    /// calls to the clone. Override when a more efficient conversion is
    /// available.
    ///
    /// # Returns
    ///
    /// An `ArcStatefulMutatingFunction<T, R>` that forwards to a clone of
    /// `self`.
    fn to_arc(&self) -> ArcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Create a boxed `FnMut(&mut T) -> R` closure that forwards to `self`.
    ///
    /// The default implementation clones `self` (requires `Clone`) and
    /// returns a boxed closure that invokes the cloned instance. Override to
    /// provide a more efficient conversion when possible.
    ///
    /// # Returns
    ///
    /// A closure implementing `FnMut(&mut T) -> R` which forwards to the
    /// original function.
    fn to_fn(&self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn()
    }
}

// =======================================================================
// 2. Type Aliases
// =======================================================================

/// Type alias for Arc-wrapped stateful mutating function
type ArcStatefulMutatingFunctionFn<T, R> = Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>;

/// Type alias for Rc-wrapped stateful mutating function
type RcStatefulMutatingFunctionFn<T, R> = Rc<RefCell<dyn FnMut(&mut T) -> R>>;

// =======================================================================
// 3. BoxStatefulMutatingFunction - Single Ownership Implementation
// =======================================================================

/// BoxStatefulMutatingFunction struct
///
/// A stateful mutating function implementation based on
/// `Box<dyn FnMut(&mut T) -> R>` for single ownership scenarios. This is the
/// simplest and most efficient stateful mutating function type when sharing
/// is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Stateful**: Can modify captured environment (uses `FnMut`)
/// - **Builder Pattern**: Method chaining consumes `self` naturally
/// - **Factory Methods**: Convenient constructors for common patterns
///
/// # Use Cases
///
/// Choose `BoxStatefulMutatingFunction` when:
/// - The function needs to maintain internal state
/// - Building pipelines where ownership naturally flows
/// - No need to share the function across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Performance
///
/// `BoxStatefulMutatingFunction` has the best performance among the three
/// function types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulMutatingFunction,
///                       BoxStatefulMutatingFunction};
///
/// let mut counter = {
///     let mut count = 0;
///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x *= 2;
///         count
///     })
/// };
/// let mut value = 5;
/// assert_eq!(counter.apply(&mut value), 1);
/// assert_eq!(value, 10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulMutatingFunction<T, R> {
    function: Box<dyn FnMut(&mut T) -> R>,
}

impl<T, R> BoxStatefulMutatingFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxStatefulMutatingFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The stateful closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxStatefulMutatingFunction<T, R>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let mut counter = {
    ///     let mut count = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x += 1;
    ///         count
    ///     })
    /// };
    /// let mut value = 5;
    /// assert_eq!(counter.apply(&mut value), 1);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + 'static,
    {
        BoxStatefulMutatingFunction {
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
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let mut identity =
    ///     BoxStatefulMutatingFunction::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn identity() -> Self
    where
        T: Clone,
        R: From<T>,
    {
        BoxStatefulMutatingFunction::new(|t: &mut T| R::from(t.clone()))
    }

    /// Chains another stateful mutating function in sequence
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
    ///   ownership.** If you need to preserve the original function, clone it
    ///   first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &mut T| -> R2`
    ///   - A `BoxStatefulMutatingFunction<T, R2>`
    ///   - An `ArcStatefulMutatingFunction<T, R2>`
    ///   - An `RcStatefulMutatingFunction<T, R2>`
    ///   - Any type implementing `StatefulMutatingFunction<T, R2>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let first = {
    ///     let mut count1 = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count1 += 1;
    ///         *x *= 2;
    ///         count1
    ///     })
    /// };
    /// let second = {
    ///     let mut count2 = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count2 += 1;
    ///         *x += 10;
    ///         count2
    ///     })
    /// };
    ///
    /// let mut chained = first.and_then(second);
    /// let mut value = 5;
    /// assert_eq!(chained.apply(&mut value), 1);
    /// assert_eq!(value, 20); // (5 * 2) + 10
    /// ```
    pub fn and_then<F, R2>(self, next: F) -> BoxStatefulMutatingFunction<T, R2>
    where
        F: StatefulMutatingFunction<T, R2> + 'static,
        R2: 'static,
    {
        let mut first = self.function;
        let mut second = next.into_fn();
        BoxStatefulMutatingFunction::new(move |t| {
            let _ = (first)(t);
            (second)(t)
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
    /// Returns a new `BoxStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let func = {
    ///     let mut count = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x *= 2;
    ///         count
    ///     })
    /// };
    /// let mut mapped = func.map(|count| format!("Call #{}", count));
    ///
    /// let mut value = 5;
    /// assert_eq!(mapped.apply(&mut value), "Call #1");
    /// ```
    pub fn map<F, R2>(self, mapper: F) -> BoxStatefulMutatingFunction<T, R2>
    where
        F: Fn(R) -> R2 + 'static,
        R2: 'static,
    {
        let mut func = self.function;
        BoxStatefulMutatingFunction::new(move |t| {
            let result = (func)(t);
            mapper(result)
        })
    }
}

impl<T, R> StatefulMutatingFunction<T, R> for BoxStatefulMutatingFunction<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        self
    }

    fn into_rc(self) -> RcStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let mut self_fn = self.function;
        RcStatefulMutatingFunction::new(move |t| (self_fn)(t))
    }

    // do NOT override StatefulMutatingFunction::into_arc() because
    // BoxStatefulMutatingFunction is not Send and calling
    // BoxStatefulMutatingFunction::into_arc() will cause a compile error

    fn into_fn(mut self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |t| (self.function)(t)
    }

    // do NOT override StatefulMutatingFunction::to_xxx() because
    // BoxStatefulMutatingFunction is not Clone and calling
    // BoxStatefulMutatingFunction::to_xxx() will cause a compile error
}

// =======================================================================
// 4. RcStatefulMutatingFunction - Single-Threaded Shared Ownership
// =======================================================================

/// RcStatefulMutatingFunction struct
///
/// A stateful mutating function implementation based on
/// `Rc<RefCell<dyn FnMut(&mut T) -> R>>` for single-threaded shared
/// ownership scenarios. This type allows multiple references to the same
/// function without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Stateful**: Can modify captured environment (uses `FnMut`)
/// - **Chainable**: Method chaining via `&self` (non-consuming)
/// - **Performance**: More efficient than `ArcStatefulMutatingFunction` (no
///   locking)
///
/// # Use Cases
///
/// Choose `RcStatefulMutatingFunction` when:
/// - The function needs to be shared within a single thread for stateful
///   operations
/// - Thread safety is not required
/// - Performance is important (avoiding lock overhead)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulMutatingFunction,
///                       RcStatefulMutatingFunction};
///
/// let counter = {
///     let mut count = 0;
///     RcStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x *= 2;
///         count
///     })
/// };
/// let mut clone = counter.clone();
///
/// let mut value = 5;
/// assert_eq!(clone.apply(&mut value), 1);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcStatefulMutatingFunction<T, R> {
    function: RcStatefulMutatingFunctionFn<T, R>,
}

impl<T, R> RcStatefulMutatingFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcStatefulMutatingFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The stateful closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcStatefulMutatingFunction<T, R>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       RcStatefulMutatingFunction};
    ///
    /// let mut counter = {
    ///     let mut count = 0;
    ///     RcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x += 1;
    ///         count
    ///     })
    /// };
    /// let mut value = 5;
    /// assert_eq!(counter.apply(&mut value), 1);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + 'static,
    {
        RcStatefulMutatingFunction {
            function: Rc::new(RefCell::new(f)),
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
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       RcStatefulMutatingFunction};
    ///
    /// let mut identity =
    ///     RcStatefulMutatingFunction::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn identity() -> Self
    where
        T: Clone,
        R: From<T>,
    {
        RcStatefulMutatingFunction::new(|t: &mut T| R::from(t.clone()))
    }

    /// Chains another RcStatefulMutatingFunction in sequence
    ///
    /// Returns a new function that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original function.
    ///
    /// # Parameters
    ///
    /// * `next` - The function to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       RcStatefulMutatingFunction};
    ///
    /// let first = {
    ///     let mut count1 = 0;
    ///     RcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count1 += 1;
    ///         *x *= 2;
    ///         count1
    ///     })
    /// };
    /// let second = {
    ///     let mut count2 = 0;
    ///     RcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count2 += 1;
    ///         *x += 10;
    ///         count2
    ///     })
    /// };
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// assert_eq!(chained.apply(&mut value), 1);
    /// ```
    pub fn and_then<R2>(
        &self,
        next: &RcStatefulMutatingFunction<T, R2>,
    ) -> RcStatefulMutatingFunction<T, R2>
    where
        R2: 'static,
    {
        let first = self.function.clone();
        let second = next.function.clone();
        RcStatefulMutatingFunction::new(move |t: &mut T| {
            let _ = (first.borrow_mut())(t);
            (second.borrow_mut())(t)
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
    /// Returns a new `RcStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       RcStatefulMutatingFunction};
    ///
    /// let func = {
    ///     let mut count = 0;
    ///     RcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x *= 2;
    ///         count
    ///     })
    /// };
    /// let mut mapped = func.map(|count| format!("Call #{}", count));
    ///
    /// let mut value = 5;
    /// assert_eq!(mapped.apply(&mut value), "Call #1");
    /// ```
    pub fn map<F, R2>(&self, mapper: F) -> RcStatefulMutatingFunction<T, R2>
    where
        F: Fn(R) -> R2 + 'static,
        R2: 'static,
    {
        let func = self.function.clone();
        RcStatefulMutatingFunction::new(move |t| {
            let result = (func.borrow_mut())(t);
            mapper(result)
        })
    }
}

impl<T, R> StatefulMutatingFunction<T, R> for RcStatefulMutatingFunction<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        (self.function.borrow_mut())(input)
    }

    fn into_box(self) -> BoxStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        BoxStatefulMutatingFunction::new(move |t| (self_fn.borrow_mut())(t))
    }

    fn into_rc(self) -> RcStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        self
    }

    // do NOT override StatefulMutatingFunction::into_arc() because
    // RcStatefulMutatingFunction is not Send and calling
    // RcStatefulMutatingFunction::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        move |t| (self_fn.borrow_mut())(t)
    }

    fn to_box(&self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulMutatingFunction::new(move |t| (self_fn.borrow_mut())(t))
    }

    fn to_rc(&self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone()
    }

    // do NOT override StatefulMutatingFunction::to_arc() because
    // RcStatefulMutatingFunction is not Send and calling
    // RcStatefulMutatingFunction::to_arc() will cause a compile error

    fn to_fn(&self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t| (self_fn.borrow_mut())(t)
    }
}

impl<T, R> Clone for RcStatefulMutatingFunction<T, R> {
    /// Clones the RcStatefulMutatingFunction
    ///
    /// Creates a new RcStatefulMutatingFunction that shares the underlying
    /// function with the original instance.
    fn clone(&self) -> Self {
        RcStatefulMutatingFunction {
            function: self.function.clone(),
        }
    }
}

// =======================================================================
// 5. ArcStatefulMutatingFunction - Thread-Safe Shared Ownership
// =======================================================================

/// ArcStatefulMutatingFunction struct
///
/// A stateful mutating function implementation based on
/// `Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>` for thread-safe shared
/// ownership scenarios. This type allows the function to be safely shared
/// and used across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Stateful**: Can modify captured environment (uses `FnMut`)
/// - **Chainable**: Method chaining via `&self` (non-consuming)
///
/// # Use Cases
///
/// Choose `ArcStatefulMutatingFunction` when:
/// - The function needs to be shared across multiple threads for stateful
///   operations
/// - Concurrent task processing (e.g., thread pools)
/// - Thread safety is required (Send + Sync)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulMutatingFunction,
///                       ArcStatefulMutatingFunction};
///
/// let counter = {
///     let mut count = 0;
///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x *= 2;
///         count
///     })
/// };
/// let mut clone = counter.clone();
///
/// let mut value = 5;
/// assert_eq!(clone.apply(&mut value), 1);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcStatefulMutatingFunction<T, R> {
    function: ArcStatefulMutatingFunctionFn<T, R>,
}

impl<T, R> ArcStatefulMutatingFunction<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// Creates a new ArcStatefulMutatingFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The stateful closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcStatefulMutatingFunction<T, R>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       ArcStatefulMutatingFunction};
    ///
    /// let mut counter = {
    ///     let mut count = 0;
    ///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x += 1;
    ///         count
    ///     })
    /// };
    /// let mut value = 5;
    /// assert_eq!(counter.apply(&mut value), 1);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + Send + 'static,
    {
        ArcStatefulMutatingFunction {
            function: Arc::new(Mutex::new(f)),
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
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       ArcStatefulMutatingFunction};
    ///
    /// let mut identity =
    ///     ArcStatefulMutatingFunction::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn identity() -> Self
    where
        T: Clone,
        R: From<T>,
    {
        ArcStatefulMutatingFunction::new(|t: &mut T| R::from(t.clone()))
    }

    /// Chains another ArcStatefulMutatingFunction in sequence
    ///
    /// Returns a new function that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original function.
    ///
    /// # Parameters
    ///
    /// * `next` - The function to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       ArcStatefulMutatingFunction};
    ///
    /// let first = {
    ///     let mut count1 = 0;
    ///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count1 += 1;
    ///         *x *= 2;
    ///         count1
    ///     })
    /// };
    /// let second = {
    ///     let mut count2 = 0;
    ///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count2 += 1;
    ///         *x += 10;
    ///         count2
    ///     })
    /// };
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// assert_eq!(chained.apply(&mut value), 1);
    /// ```
    pub fn and_then<R2>(
        &self,
        next: &ArcStatefulMutatingFunction<T, R2>,
    ) -> ArcStatefulMutatingFunction<T, R2>
    where
        R2: Send + 'static,
    {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcStatefulMutatingFunction {
            function: Arc::new(Mutex::new(move |t: &mut T| {
                let _ = (first.lock().unwrap())(t);
                (second.lock().unwrap())(t)
            })),
        }
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
    /// Returns a new `ArcStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       ArcStatefulMutatingFunction};
    ///
    /// let func = {
    ///     let mut count = 0;
    ///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         *x *= 2;
    ///         count
    ///     })
    /// };
    /// let mut mapped = func.map(|count| format!("Call #{}", count));
    ///
    /// let mut value = 5;
    /// assert_eq!(mapped.apply(&mut value), "Call #1");
    /// ```
    pub fn map<F, R2>(&self, mapper: F) -> ArcStatefulMutatingFunction<T, R2>
    where
        F: Fn(R) -> R2 + Send + Sync + 'static,
        R2: Send + 'static,
    {
        let func = Arc::clone(&self.function);
        let mapper = Arc::new(mapper);
        ArcStatefulMutatingFunction {
            function: Arc::new(Mutex::new(move |t: &mut T| {
                let result = (func.lock().unwrap())(t);
                (mapper)(result)
            })),
        }
    }
}

impl<T, R> StatefulMutatingFunction<T, R> for ArcStatefulMutatingFunction<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        (self.function.lock().unwrap())(input)
    }

    fn into_box(self) -> BoxStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        BoxStatefulMutatingFunction::new(move |t| (self_fn.lock().unwrap())(t))
    }

    fn into_rc(self) -> RcStatefulMutatingFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        RcStatefulMutatingFunction::new(move |t| (self_fn.lock().unwrap())(t))
    }

    fn into_arc(self) -> ArcStatefulMutatingFunction<T, R>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        move |t| (self_fn.lock().unwrap())(t)
    }

    fn to_box(&self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulMutatingFunction::new(move |t| (self_fn.lock().unwrap())(t))
    }

    fn to_rc(&self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        RcStatefulMutatingFunction::new(move |t| (self_fn.lock().unwrap())(t))
    }

    fn to_arc(&self) -> ArcStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t| (self_fn.lock().unwrap())(t)
    }
}

impl<T, R> Clone for ArcStatefulMutatingFunction<T, R> {
    /// Clones the ArcStatefulMutatingFunction
    ///
    /// Creates a new ArcStatefulMutatingFunction that shares the underlying
    /// function with the original instance.
    fn clone(&self) -> Self {
        ArcStatefulMutatingFunction {
            function: self.function.clone(),
        }
    }
}

// =======================================================================
// 6. Implement StatefulMutatingFunction trait for closures
// =======================================================================

impl<T, R, F> StatefulMutatingFunction<T, R> for F
where
    F: FnMut(&mut T) -> R,
{
    fn apply(&mut self, input: &mut T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulMutatingFunction::new(self)
    }

    fn into_rc(self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        RcStatefulMutatingFunction::new(self)
    }

    fn into_arc(self) -> ArcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        ArcStatefulMutatingFunction::new(self)
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        self
    }

    fn to_box(&self) -> BoxStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cloned = self.clone();
        BoxStatefulMutatingFunction::new(cloned)
    }

    fn to_rc(&self) -> RcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cloned = self.clone();
        RcStatefulMutatingFunction::new(cloned)
    }

    fn to_arc(&self) -> ArcStatefulMutatingFunction<T, R>
    where
        Self: Sized + Clone + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let cloned = self.clone();
        ArcStatefulMutatingFunction::new(cloned)
    }

    fn to_fn(&self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone()
    }
}

// =======================================================================
// 7. Provide extension methods for closures
// =======================================================================

/// Extension trait providing stateful mutating function composition methods
/// for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnMut(&mut T) -> R`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxStatefulMutatingFunction**: Composition results are
///   `BoxStatefulMutatingFunction<T, R>` for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&mut T) -> R` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulMutatingFunction,
///                       FnMutStatefulMutatingFunctionOps};
///
/// let mut count1 = 0;
/// let mut count2 = 0;
/// let mut chained = (move |x: &mut i32| {
///     count1 += 1;
///     *x *= 2;
///     count1
/// })
/// .and_then(move |x: &mut i32| {
///     count2 += 1;
///     *x += 10;
///     count2
/// });
///
/// let mut value = 5;
/// assert_eq!(chained.apply(&mut value), 1);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnMutStatefulMutatingFunctionOps<T, R>: FnMut(&mut T) -> R + Sized {
    /// Chains another stateful mutating function in sequence
    ///
    /// Returns a new function that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxStatefulMutatingFunction<T, R2>`.
    ///
    /// # Parameters
    ///
    /// * `next` - The function to execute after the current operation.
    ///   **Note: This parameter is passed by value and will transfer
    ///   ownership.** If you need to preserve the original function, clone it
    ///   first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &mut T| -> R2`
    ///   - A `BoxStatefulMutatingFunction<T, R2>`
    ///   - An `ArcStatefulMutatingFunction<T, R2>`
    ///   - An `RcStatefulMutatingFunction<T, R2>`
    ///   - Any type implementing `StatefulMutatingFunction<T, R2>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       FnMutStatefulMutatingFunctionOps};
    ///
    /// let mut count1 = 0;
    /// let mut count2 = 0;
    /// let mut chained = (move |x: &mut i32| {
    ///     count1 += 1;
    ///     *x *= 2;
    ///     count1
    /// })
    /// .and_then(move |x: &mut i32| {
    ///     count2 += 1;
    ///     *x += 10;
    ///     count2
    /// });
    ///
    /// let mut value = 5;
    /// assert_eq!(chained.apply(&mut value), 1);
    /// ```
    fn and_then<F, R2>(self, next: F) -> BoxStatefulMutatingFunction<T, R2>
    where
        Self: 'static,
        F: StatefulMutatingFunction<T, R2> + 'static,
        T: 'static,
        R: 'static,
        R2: 'static,
    {
        let mut first = self;
        let mut second = next.into_fn();
        BoxStatefulMutatingFunction::new(move |t| {
            let _ = (first)(t);
            (second)(t)
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
    /// Returns a new `BoxStatefulMutatingFunction<T, R2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulMutatingFunction,
    ///                       FnMutStatefulMutatingFunctionOps};
    ///
    /// let mut count = 0;
    /// let mut mapped = (move |x: &mut i32| {
    ///     count += 1;
    ///     *x *= 2;
    ///     count
    /// })
    /// .map(|count| format!("Call #{}", count));
    ///
    /// let mut value = 5;
    /// assert_eq!(mapped.apply(&mut value), "Call #1");
    /// ```
    fn map<F, R2>(self, mapper: F) -> BoxStatefulMutatingFunction<T, R2>
    where
        Self: 'static,
        F: Fn(R) -> R2 + 'static,
        T: 'static,
        R: 'static,
        R2: 'static,
    {
        let mut func = self;
        BoxStatefulMutatingFunction::new(move |t| {
            let result = (func)(t);
            mapper(result)
        })
    }
}

/// Implements FnMutStatefulMutatingFunctionOps for all closure types
impl<T, R, F> FnMutStatefulMutatingFunctionOps<T, R> for F where F: FnMut(&mut T) -> R {}
