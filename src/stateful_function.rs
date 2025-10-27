/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # StatefulFunction Types
//!
//! Provides Rust implementations of stateful function traits for stateful value
//! transformation. StatefulFunctions consume input values (taking ownership) and
//! produce output values while allowing internal state modification.
//!
//! This module provides the `StatefulFunction<T, R>` trait and three implementations:
//!
//! - [`BoxStatefulFunction`]: Single ownership, not cloneable
//! - [`ArcStatefulFunction`]: Thread-safe shared ownership, cloneable
//! - [`RcStatefulFunction`]: Single-threaded shared ownership, cloneable
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

// StatefulFunction does not have a "once" variant since it already uses FnMut
use crate::predicate::{
    ArcPredicate,
    BoxPredicate,
    Predicate,
    RcPredicate,
};

// ============================================================================
// Core Trait
// ============================================================================

/// StatefulFunction trait - transforms values from type T to type R with state
///
/// Defines the behavior of a stateful transformation: converting a value
/// of type `T` to a value of type `R` by consuming the input while
/// allowing modification of internal state. This is analogous to
/// `FnMut(&T) -> R` in Rust's standard library.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Haixing Hu
pub trait StatefulFunction<T, R> {
    /// Applies the mapping to the input value to produce an output value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value to transform (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(&mut self, input: &T) -> R;

    /// Converts to BoxStatefulFunction
    ///
    /// **⚠️ Consumes `self`**: The original stateful function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxStatefulFunction<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in a `BoxStatefulFunction` by creating
    /// a new closure that calls `self.apply()`. This provides a zero-cost
    /// abstraction for most use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, BoxStatefulFunction};
    ///
    /// struct CustomStatefulFunction {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let function = CustomStatefulFunction { multiplier: 0 };
    /// let mut boxed = function.into_box();
    /// assert_eq!(boxed.apply(10), 10);  // 10 * 1
    /// assert_eq!(boxed.apply(10), 20);  // 10 * 2
    /// ```
    fn into_box(self) -> BoxStatefulFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut function = self;
        BoxStatefulFunction::new(move |t| function.apply(t))
    }

    /// Converts to RcStatefulFunction
    ///
    /// **⚠️ Consumes `self`**: The original stateful function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcStatefulFunction<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation first converts to `BoxStatefulFunction` using
    /// `into_box()`, then wraps it in `RcStatefulFunction`. Specific implementations
    /// may override this for better efficiency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, RcStatefulFunction};
    ///
    /// struct CustomStatefulFunction {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let function = CustomStatefulFunction { multiplier: 0 };
    /// let mut rc_function = function.into_rc();
    /// assert_eq!(rc_function.apply(10), 10);  // 10 * 1
    /// assert_eq!(rc_function.apply(10), 20);  // 10 * 2
    /// ```
    fn into_rc(self) -> RcStatefulFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut function = self;
        RcStatefulFunction::new(move |t| function.apply(t))
    }

    /// Converts to ArcStatefulFunction
    ///
    /// **⚠️ Consumes `self`**: The original stateful function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcStatefulFunction<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `ArcStatefulFunction` by creating
    /// a new closure that calls `self.apply()`. Note that this requires `self`
    /// to implement `Send` due to Arc's thread-safety requirements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, ArcStatefulFunction};
    ///
    /// struct CustomStatefulFunction {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let function = CustomStatefulFunction { multiplier: 0 };
    /// let mut arc_function = function.into_arc();
    /// assert_eq!(arc_function.apply(10), 10);  // 10 * 1
    /// assert_eq!(arc_function.apply(10), 20);  // 10 * 2
    /// ```
    fn into_arc(self) -> ArcStatefulFunction<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        let mut function = self;
        ArcStatefulFunction::new(move |t| function.apply(t))
    }

    /// Converts to a closure implementing `FnMut(&T) -> R`
    ///
    /// **⚠️ Consumes `self`**: The original stateful function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns an implementation of `FnMut(&T) -> R`
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a new closure that calls `self.apply()`.
    /// Specific implementations may override this for better efficiency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, BoxStatefulFunction};
    ///
    /// let function = BoxStatefulFunction::new(|x: i32| x * 2);
    /// let mut closure = function.into_fn();
    /// assert_eq!(closure(10), 20);
    /// assert_eq!(closure(15), 30);
    /// ```
    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut function = self;
        move |t| function.apply(t)
    }

    /// Non-consuming conversion to `BoxStatefulFunction`.
    ///
    /// Default implementation requires `Self: Clone` and wraps a cloned
    /// instance in a `RefCell` so the returned stateful function can mutate state
    /// across calls.
    fn to_box(&self) -> BoxStatefulFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Non-consuming conversion to `RcStatefulFunction`.
    ///
    /// Default implementation clones `self` into an `Rc<RefCell<_>>` so the
    /// resulting stateful function can be shared within a single thread.
    fn to_rc(&self) -> RcStatefulFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_rc()
    }

    /// Non-consuming conversion to `ArcStatefulFunction` (thread-safe).
    ///
    /// Default implementation requires `Self: Clone + Send + Sync` and wraps
    /// the cloned instance in `Arc<Mutex<_>>` so it can be used across
    /// threads.
    fn to_arc(&self) -> ArcStatefulFunction<T, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Non-consuming conversion to a closure (`FnMut(&T) -> R`).
    ///
    /// Default implementation clones `self` into a `RefCell` and returns a
    /// closure that calls `apply` on the interior mutable value.
    fn to_fn(&self) -> impl FnMut(&T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn()
    }
}

// ============================================================================
// BoxStatefulFunction - Box<dyn FnMut(&T) -> R>
// ============================================================================

/// BoxStatefulFunction - stateful function wrapper based on `Box<dyn FnMut>`
///
/// A stateful function wrapper that provides single ownership with reusable stateful
/// transformation. The stateful function consumes the input and can be called
/// multiple times while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(&T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulFunction<T, R> {
    function: Box<dyn FnMut(&T) -> R>,
}

impl<T, R> BoxStatefulFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxStatefulFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = BoxStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(function.apply(10), 11);
    /// assert_eq!(function.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) -> R + 'static,
    {
        BoxStatefulFunction {
            function: Box::new(f),
        }
    }

    // BoxStatefulFunction is intentionally not given a `to_*` specialization here
    // because the boxed `FnMut` is not clonable and we cannot produce a
    // non-consuming adapter from `&self` without moving ownership or
    // requiring `Clone` on the inner function. Consumers should use the
    // blanket `StatefulFunction::to_*` defaults when their stateful function type implements
    // `Clone`.

    /// Creates an identity stateful function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulFunction, StatefulFunction};
    ///
    /// let mut identity = BoxStatefulFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> BoxStatefulFunction<T, T>
    where
        T: Clone,
    {
        BoxStatefulFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new stateful function that applies this stateful function first, then applies
    /// the after stateful function to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after stateful function
    /// * `F` - The type of the after stateful function (must implement StatefulFunction<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The stateful function to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original stateful function, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulFunction<R, S>`
    ///   - An `RcStatefulFunction<R, S>`
    ///   - An `ArcStatefulFunction<R, S>`
    ///   - Any type implementing `StatefulFunction<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulFunction, StatefulFunction};
    ///
    /// let mut counter1 = 0;
    /// let function1 = BoxStatefulFunction::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let function2 = BoxStatefulFunction::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = function1.and_then(function2);
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxStatefulFunction<T, S>
    where
        S: 'static,
        F: StatefulFunction<R, S> + 'static,
    {
        let mut self_function = self;
        let mut after_function = after;
        BoxStatefulFunction::new(move |x: &T| {
            let intermediate = self_function.apply(x);
            after_function.apply(&intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    /// * `F` - The type of the before function (must implement StatefulFunction<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If
    ///   you need to preserve the original function, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulFunction<S, T>`
    ///   - An `RcStatefulFunction<S, T>`
    ///   - An `ArcStatefulFunction<S, T>`
    ///   - Any type implementing `StatefulFunction<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let function = BoxStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = function.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(self, before: F) -> BoxStatefulFunction<S, R>
    where
        S: 'static,
        F: StatefulFunction<S, T> + 'static,
    {
        let mut self_fn = self.function;
        let mut before_fn = before;
        BoxStatefulFunction::new(move |x: &S| self_fn.apply(&before_fn.apply(x)))
    }

    /// Creates a conditional function
    ///
    /// Returns a function that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative function for
    /// when the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is
    ///   passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, BoxStatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = BoxStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(function.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(function.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalStatefulFunction<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalStatefulFunction {
            function: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxStatefulFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulFunction, StatefulFunction};
    ///
    /// let mut constant = BoxStatefulFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxStatefulFunction<T, R> {
        BoxStatefulFunction::new(move |_| value.clone())
    }
}

impl<T, R> StatefulFunction<T, R> for BoxStatefulFunction<T, R> {
    fn apply(&mut self, input: &T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        RcStatefulFunction::new(self_fn)
    }

    // do NOT override StatefulFunction::into_arc() because BoxStatefulFunction is not Send + Sync
    // and calling BoxStatefulFunction::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return the boxed function
        self.function
    }

    // do NOT override StatefulFunction::to_xxx() because BoxStatefulFunction is not Clone
    // and calling BoxStatefulFunction::to_xxx() will cause a compile error
}

// ============================================================================
// BoxConditionalStatefulFunction - Box-based Conditional StatefulFunction
// ============================================================================

/// BoxConditionalStatefulFunction struct
///
/// A conditional function that only executes when a predicate is satisfied.
/// Uses `BoxStatefulFunction` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxStatefulFunction::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else
///   logic
/// - **Implements StatefulFunction**: Can be used anywhere a `StatefulFunction` is expected
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulFunction, BoxStatefulFunction};
///
/// let mut high_count = 0;
/// let mut low_count = 0;
///
/// let mut function = BoxStatefulFunction::new(move |x: i32| {
///     high_count += 1;
///     x * 2
/// })
/// .when(|x: &i32| *x >= 10)
/// .or_else(move |x| {
///     low_count += 1;
///     x + 1
/// });
///
/// assert_eq!(function.apply(15), 30); // when branch executed
/// assert_eq!(function.apply(5), 6);   // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalStatefulFunction<T, R> {
    function: BoxStatefulFunction<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalStatefulFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch
    ///
    /// Executes the original function when the condition is satisfied,
    /// otherwise executes else_function.
    ///
    /// # Parameters
    ///
    /// * `else_function` - The function for the else branch, can be:
    ///   - Closure: `|x: &T| -> R`
    ///   - `BoxStatefulFunction<T, R>`, `RcStatefulFunction<T, R>`, `ArcStatefulFunction<T, R>`
    ///   - Any type implementing `StatefulFunction<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, BoxStatefulFunction};
    ///
    /// let mut function = BoxStatefulFunction::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(function.apply(5), 10);   // Condition satisfied
    /// assert_eq!(function.apply(-5), 5);   // Condition not satisfied
    /// ```
    pub fn or_else<F>(self, mut else_function: F) -> BoxStatefulFunction<T, R>
    where
        F: StatefulFunction<T, R> + 'static,
    {
        let pred = self.predicate;
        let mut then_function = self.function;
        BoxStatefulFunction::new(move |t| {
            if pred.test(t) {
                then_function.apply(t)
            } else {
                else_function.apply(t)
            }
        })
    }
}

// ============================================================================
// ArcStatefulFunction - Arc<Mutex<dyn FnMut(&T) -> R + Send>>
// ============================================================================

/// ArcStatefulFunction - thread-safe function wrapper
///
/// A thread-safe, clonable function wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads
/// while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(&T) -> R + Send>>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Thread-safe (`Send` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct ArcStatefulFunction<T, R> {
    function: ArcStatefulFn<T, R>,
}

type ArcStatefulFn<T, R> = Arc<Mutex<dyn FnMut(&T) -> R + Send>>;

impl<T, R> ArcStatefulFunction<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Creates a new ArcStatefulFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = ArcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(function.apply(10), 11);
    /// assert_eq!(function.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) -> R + Send + 'static,
    {
        ArcStatefulFunction {
            function: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulFunction, StatefulFunction};
    ///
    /// let mut identity = ArcStatefulFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> ArcStatefulFunction<T, T>
    where
        T: Clone,
    {
        ArcStatefulFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    /// * `F` - The type of the after function (must implement StatefulFunction<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulFunction<R, S>`
    ///   - An `RcStatefulFunction<R, S>`
    ///   - An `ArcStatefulFunction<R, S>` (will be cloned internally)
    ///   - Any type implementing `StatefulFunction<R, S> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter1 = 0;
    /// let function1 = ArcStatefulFunction::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let function2 = ArcStatefulFunction::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = function1.and_then(function2);
    ///
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> ArcStatefulFunction<T, S>
    where
        S: Send + 'static,
        F: StatefulFunction<R, S> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let after = Arc::new(Mutex::new(after));
        ArcStatefulFunction {
            function: Arc::new(Mutex::new(move |x: &T| {
                let intermediate = self_fn.lock().unwrap()(x);
                after.lock().unwrap().apply(&intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    /// * `F` - The type of the before function (must implement StatefulFunction<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulFunction<S, T>`
    ///   - An `RcStatefulFunction<S, T>`
    ///   - An `ArcStatefulFunction<S, T>` (will be cloned internally)
    ///   - Any type implementing `StatefulFunction<S, T> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let function = ArcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = function.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> ArcStatefulFunction<S, R>
    where
        S: Send + 'static,
        F: StatefulFunction<S, T> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let before = Arc::new(Mutex::new(before));
        ArcStatefulFunction {
            function: Arc::new(Mutex::new(move |x: &S| {
                let intermediate = before.lock().unwrap().apply(x);
                self_fn.lock().unwrap()(&intermediate)
            })),
        }
    }

    /// Creates a conditional function (thread-safe version)
    ///
    /// Returns a function that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative function.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Must be `Send`, can be:
    ///   - A closure: `|x: &T| -> bool` (requires `Send`)
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T> + Send`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, ArcStatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = ArcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(function.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(function.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalStatefulFunction<T, R>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        ArcConditionalStatefulFunction {
            function: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, R> ArcStatefulFunction<T, R>
where
    T: Send + Sync + 'static,
    R: Clone + Send + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulFunction, StatefulFunction};
    ///
    /// let mut constant = ArcStatefulFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcStatefulFunction<T, R> {
        ArcStatefulFunction::new(move |_| value.clone())
    }
}

impl<T, R> StatefulFunction<T, R> for ArcStatefulFunction<T, R> {
    fn apply(&mut self, input: &T) -> R {
        (self.function.lock().unwrap())(input)
    }

    fn into_box(self) -> BoxStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxStatefulFunction::new(move |x| self.function.lock().unwrap()(x))
    }

    fn into_rc(self) -> RcStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcStatefulFunction::new(move |x| self.function.lock().unwrap()(x))
    }

    fn into_arc(self) -> ArcStatefulFunction<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Efficient: use Arc cloning to create a closure
        move |input: &T| (self.function.lock().unwrap())(input)
    }

    fn to_box(&self) -> BoxStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulFunction::new(move |x| self_fn.lock().unwrap()(x))
    }

    fn to_rc(&self) -> RcStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        RcStatefulFunction::new(move |x| self_fn.lock().unwrap()(x))
    }

    fn to_arc(&self) -> ArcStatefulFunction<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |input: &T| self_fn.lock().unwrap()(input)
    }
}

impl<T, R> Clone for ArcStatefulFunction<T, R> {
    fn clone(&self) -> Self {
        ArcStatefulFunction {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// ArcConditionalStatefulFunction - Arc-based Conditional StatefulFunction
// ============================================================================

/// ArcConditionalStatefulFunction struct
///
/// A thread-safe conditional function that only executes when a predicate
/// is satisfied. Uses `ArcStatefulFunction` and `ArcPredicate` for shared
/// ownership across threads.
///
/// This type is typically created by calling `ArcStatefulFunction::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send`, safe for concurrent use
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else
///   logic
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulFunction, ArcStatefulFunction};
///
/// let mut function = ArcStatefulFunction::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut function_clone = function.clone();
///
/// assert_eq!(function.apply(5), 10);
/// assert_eq!(function_clone.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalStatefulFunction<T, R> {
    function: ArcStatefulFunction<T, R>,
    predicate: ArcPredicate<T>,
}

impl<T, R> ArcConditionalStatefulFunction<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original function when the condition is satisfied,
    /// otherwise executes else_function.
    ///
    /// # Parameters
    ///
    /// * `else_function` - The function for the else branch, can be:
    ///   - Closure: `|x: &T| -> R` (must be `Send`)
    ///   - `ArcStatefulFunction<T, R>`, `BoxStatefulFunction<T, R>`
    ///   - Any type implementing `StatefulFunction<T, R> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, ArcStatefulFunction};
    ///
    /// let mut function = ArcStatefulFunction::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(function.apply(5), 10);
    /// assert_eq!(function.apply(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_function: F) -> ArcStatefulFunction<T, R>
    where
        F: StatefulFunction<T, R> + Send + 'static,
    {
        let pred = self.predicate;
        let then_function = self.function;
        let else_function = Arc::new(Mutex::new(else_function));
        ArcStatefulFunction {
            function: Arc::new(Mutex::new(move |t: &T| {
                if pred.test(t) {
                    then_function.function.lock().unwrap()(t)
                } else {
                    else_function.lock().unwrap().apply(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for ArcConditionalStatefulFunction<T, R> {
    /// Clones the conditional function
    ///
    /// Creates a new instance that shares the underlying function and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcStatefulFunction - Rc<RefCell<dyn FnMut(&T) -> R>>
// ============================================================================

/// RcStatefulFunction - single-threaded function wrapper
///
/// A single-threaded, clonable function wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<RefCell<dyn FnMut(&T) -> R>>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct RcStatefulFunction<T, R> {
    function: RcStatefulFn<T, R>,
}

type RcStatefulFn<T, R> = Rc<RefCell<dyn FnMut(&T) -> R>>;

impl<T, R> RcStatefulFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcStatefulFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = RcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(function.apply(10), 11);
    /// assert_eq!(function.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) -> R + 'static,
    {
        RcStatefulFunction {
            function: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulFunction, StatefulFunction};
    ///
    /// let mut identity = RcStatefulFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> RcStatefulFunction<T, T>
    where
        T: Clone,
    {
        RcStatefulFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    /// * `F` - The type of the after function (must implement StatefulFunction<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulFunction<R, S>`
    ///   - An `RcStatefulFunction<R, S>` (will be cloned internally)
    ///   - An `ArcStatefulFunction<R, S>`
    ///   - Any type implementing `StatefulFunction<R, S>`
    ///
    /// # Returns
    ///
    /// A new RcStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter1 = 0;
    /// let function1 = RcStatefulFunction::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let function2 = RcStatefulFunction::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = function1.and_then(function2);
    ///
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> RcStatefulFunction<T, S>
    where
        S: 'static,
        F: StatefulFunction<R, S> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let after = Rc::new(RefCell::new(after));
        RcStatefulFunction {
            function: Rc::new(RefCell::new(move |x: &T| {
                let intermediate = self_fn.borrow_mut()(x);
                after.borrow_mut().apply(&intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    /// * `F` - The type of the before function (must implement StatefulFunction<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulFunction<S, T>`
    ///   - An `RcStatefulFunction<S, T>` (will be cloned internally)
    ///   - An `ArcStatefulFunction<S, T>`
    ///   - Any type implementing `StatefulFunction<S, T>`
    ///
    /// # Returns
    ///
    /// A new RcStatefulFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulFunction, StatefulFunction};
    ///
    /// let mut counter = 0;
    /// let function = RcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = function.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> RcStatefulFunction<S, R>
    where
        S: 'static,
        F: StatefulFunction<S, T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let before = Rc::new(RefCell::new(before));
        RcStatefulFunction {
            function: Rc::new(RefCell::new(move |x: &S| {
                let intermediate = before.borrow_mut().apply(x);
                self_fn.borrow_mut()(&intermediate)
            })),
        }
    }

    /// Creates a conditional function (single-threaded shared version)
    ///
    /// Returns a function that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative function.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `RcConditionalStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, RcStatefulFunction};
    ///
    /// let mut counter = 0;
    /// let mut function = RcStatefulFunction::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(function.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(function.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> RcConditionalStatefulFunction<T, R>
    where
        P: Predicate<T> + 'static,
    {
        RcConditionalStatefulFunction {
            function: self,
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, R> RcStatefulFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulFunction, StatefulFunction};
    ///
    /// let mut constant = RcStatefulFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> RcStatefulFunction<T, R> {
        RcStatefulFunction::new(move |_| value.clone())
    }
}

impl<T, R> StatefulFunction<T, R> for RcStatefulFunction<T, R> {
    fn apply(&mut self, input: &T) -> R {
        (self.function.borrow_mut())(input)
    }

    fn into_box(self) -> BoxStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxStatefulFunction {
            function: Box::new(move |x| self.function.borrow_mut()(x)),
        }
    }

    fn into_rc(self) -> RcStatefulFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    // do NOT override StatefulFunction::into_arc() because RcStatefulFunction is not Send + Sync
    // and calling RcStatefulFunction::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Efficient: use Rc cloning to create a closure
        move |input: &T| (self.function.borrow_mut())(input)
    }
}

impl<T, R> Clone for RcStatefulFunction<T, R> {
    fn clone(&self) -> Self {
        RcStatefulFunction {
            function: Rc::clone(&self.function),
        }
    }
}

// ============================================================================
// RcConditionalStatefulFunction - Rc-based Conditional StatefulFunction
// ============================================================================

/// RcConditionalStatefulFunction struct
///
/// A single-threaded conditional function that only executes when a
/// predicate is satisfied. Uses `RcStatefulFunction` and `RcPredicate` for shared
/// ownership within a single thread.
///
/// This type is typically created by calling `RcStatefulFunction::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalStatefulFunction`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulFunction, RcStatefulFunction};
///
/// let mut function = RcStatefulFunction::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut function_clone = function.clone();
///
/// assert_eq!(function.apply(5), 10);
/// assert_eq!(function_clone.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalStatefulFunction<T, R> {
    function: RcStatefulFunction<T, R>,
    predicate: RcPredicate<T>,
}

impl<T, R> RcConditionalStatefulFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original function when the condition is satisfied,
    /// otherwise executes else_function.
    ///
    /// # Parameters
    ///
    /// * `else_function` - The function for the else branch, can be:
    ///   - Closure: `|x: &T| -> R`
    ///   - `RcStatefulFunction<T, R>`, `BoxStatefulFunction<T, R>`
    ///   - Any type implementing `StatefulFunction<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, RcStatefulFunction};
    ///
    /// let mut function = RcStatefulFunction::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(function.apply(5), 10);
    /// assert_eq!(function.apply(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_function: F) -> RcStatefulFunction<T, R>
    where
        F: StatefulFunction<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_function = self.function;
        let else_function = Rc::new(RefCell::new(else_function));
        RcStatefulFunction {
            function: Rc::new(RefCell::new(move |t: &T| {
                if pred.test(t) {
                    then_function.function.borrow_mut()(t)
                } else {
                    else_function.borrow_mut().apply(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for RcConditionalStatefulFunction<T, R> {
    /// Clones the conditional function
    ///
    /// Creates a new instance that shares the underlying function and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnMut trait
// ============================================================================

/// Implement StatefulFunction<T, R> for any type that implements FnMut(&T) -> R
///
/// This allows closures to be used directly with our StatefulFunction trait
/// without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::StatefulFunction;
///
/// let mut counter = 0;
/// let mut function = |x: i32| {
///     counter += 1;
///     x + counter
/// };
///
/// assert_eq!(function.apply(10), 11);
/// assert_eq!(function.apply(10), 12);
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T, R> StatefulFunction<T, R> for F
where
    F: FnMut(&T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply(&mut self, input: &T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxStatefulFunction<T, R>
    where
        Self: Sized + 'static,
    {
        BoxStatefulFunction::new(self)
    }

    fn into_rc(self) -> RcStatefulFunction<T, R>
    where
        Self: Sized + 'static,
    {
        RcStatefulFunction::new(self)
    }

    fn into_arc(self) -> ArcStatefulFunction<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        ArcStatefulFunction::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself (the closure)
        self
    }

    /// Non-consuming conversion to `BoxStatefulFunction` for closures.
    ///
    /// We can create a `BoxStatefulFunction` by boxing the closure and returning a
    /// new `BoxStatefulFunction`. This does not require `Clone` because we consume
    /// the closure value passed by the caller when they call this
    /// method. For `&self`-style non-consuming `to_*` adapters, users can
    /// use the `StatefulFunction::to_*` defaults which clone the closure when
    /// possible.
    fn to_box(&self) -> BoxStatefulFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        // Clone the closure into a RefCell to allow interior mutability
        // across calls.
        let cell = RefCell::new(self.clone());
        BoxStatefulFunction::new(move |input: &T| cell.borrow_mut().apply(input))
    }

    fn to_rc(&self) -> RcStatefulFunction<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cell = Rc::new(RefCell::new(self.clone()));
        RcStatefulFunction::new(move |input: &T| cell.borrow_mut().apply(input))
    }

    fn to_arc(&self) -> ArcStatefulFunction<T, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        let cell = Arc::new(Mutex::new(self.clone()));
        ArcStatefulFunction::new(move |input: &T| cell.lock().unwrap().apply(input))
    }

    fn to_fn(&self) -> impl FnMut(&T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cell = RefCell::new(self.clone());
        move |input: &T| cell.borrow_mut().apply(input)
    }
}

// ============================================================================
// FnStatefulFunctionOps - Extension trait for closure functions
// ============================================================================

/// Extension trait for closures implementing `FnMut(&T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for
/// closures without requiring explicit wrapping in `BoxStatefulFunction`,
/// `RcStatefulFunction`, or `ArcStatefulFunction`.
///
/// This trait is automatically implemented for all closures that
/// implement `FnMut(&T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `StatefulFunction<T, R>` through blanket
/// implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides
/// those methods, returning `BoxStatefulFunction` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{StatefulFunction, FnStatefulFunctionOps};
///
/// let mut counter1 = 0;
/// let function1 = move |x: i32| {
///     counter1 += 1;
///     x + counter1
/// };
///
/// let mut counter2 = 0;
/// let function2 = move |x: i32| {
///     counter2 += 1;
///     x * counter2
/// };
///
/// let mut composed = function1.and_then(function2);
/// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{StatefulFunction, FnStatefulFunctionOps};
///
/// let mut counter = 0;
/// let function = move |x: i32| {
///     counter += 1;
///     x * counter
/// };
///
/// let mut composed = function.compose(|x: i32| x + 1);
/// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
/// ```
///
/// ## Conditional mapping with when
///
/// ```rust
/// use prism3_function::{StatefulFunction, FnStatefulFunctionOps};
///
/// let mut function = (|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// assert_eq!(function.apply(5), 10);
/// assert_eq!(function.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnStatefulFunctionOps<T, R>: FnMut(&T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Consumes self and returns a
    /// `BoxStatefulFunction`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    /// * `F` - The type of the after function (must implement StatefulFunction<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulFunction<R, S>`
    ///   - An `RcStatefulFunction<R, S>`
    ///   - An `ArcStatefulFunction<R, S>`
    ///   - Any type implementing `StatefulFunction<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulFunction<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, FnStatefulFunctionOps, BoxStatefulFunction};
    ///
    /// let mut counter1 = 0;
    /// let function1 = move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// };
    ///
    /// let mut counter2 = 0;
    /// let function2 = BoxStatefulFunction::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = function1.and_then(function2);
    /// assert_eq!(composed.apply(10), 11);
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxStatefulFunction<T, S>
    where
        S: 'static,
        F: StatefulFunction<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulFunction::new(self).and_then(after)
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Consumes self and returns a
    /// `BoxStatefulFunction`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    /// * `F` - The type of the before function (must implement StatefulFunction<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulFunction<S, T>`
    ///   - An `RcStatefulFunction<S, T>`
    ///   - An `ArcStatefulFunction<S, T>`
    ///   - Any type implementing `StatefulFunction<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulFunction<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, FnStatefulFunctionOps, BoxStatefulFunction};
    ///
    /// let mut counter = 0;
    /// let function = move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// };
    ///
    /// let before = BoxStatefulFunction::new(|x: i32| x + 1);
    ///
    /// let mut composed = function.compose(before);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// ```
    fn compose<S, F>(self, before: F) -> BoxStatefulFunction<S, R>
    where
        S: 'static,
        F: StatefulFunction<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulFunction::new(self).compose(before)
    }

    /// Creates a conditional function
    ///
    /// Returns a function that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative function for
    /// when the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalStatefulFunction<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulFunction, FnStatefulFunctionOps};
    ///
    /// let mut function = (|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(function.apply(5), 10);
    /// assert_eq!(function.apply(-5), 5);
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalStatefulFunction<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulFunction::new(self).when(predicate)
    }
}

/// Blanket implementation of FnStatefulFunctionOps for all closures
///
/// Automatically implements `FnStatefulFunctionOps<T, R>` for any type that
/// implements `FnMut(&T) -> R`.
///
/// # Author
///
/// Haixing Hu
impl<T, R, F> FnStatefulFunctionOps<T, R> for F where F: FnMut(&T) -> R + 'static {}
