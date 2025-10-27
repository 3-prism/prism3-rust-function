/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # StatefulTransformer Types
//!
//! Provides Rust implementations of stateful transformer traits for stateful value
//! transformation. StatefulTransformers consume input values (taking ownership) and
//! produce output values while allowing internal state modification. This is
//! analogous to `FnMut(T) -> R` in Rust's standard library.
//!
//! This module provides the `StatefulTransformer<T, R>` trait and three implementations:
//!
//! - [`BoxStatefulTransformer`]: Single ownership, not cloneable
//! - [`ArcStatefulTransformer`]: Thread-safe shared ownership, cloneable
//! - [`RcStatefulTransformer`]: Single-threaded shared ownership, cloneable
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

use crate::predicate::{
    ArcPredicate,
    BoxPredicate,
    Predicate,
    RcPredicate,
};
use crate::transformer_once::{
    BoxTransformerOnce,
    TransformerOnce,
};

// ============================================================================
// Core Trait
// ============================================================================

/// StatefulTransformer trait - transforms values from type T to type R with state
///
/// Defines the behavior of a stateful transformation: converting a value
/// of type `T` to a value of type `R` by consuming the input while
/// allowing modification of internal state. This is analogous to
/// `FnMut(T) -> R` in Rust's standard library.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Haixing Hu
pub trait StatefulTransformer<T, R> {
    /// Applies the transformation to the input value to produce an output value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value to transform (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(&mut self, input: T) -> R;

    /// Converts to BoxStatefulTransformer
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxStatefulTransformer<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in a `BoxStatefulTransformer` by creating
    /// a new closure that calls `self.apply()`. This provides a zero-cost
    /// abstraction for most use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, BoxStatefulTransformer};
    ///
    /// struct CustomMapper {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulTransformer<i32, i32> for CustomMapper {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let mapper = CustomMapper { multiplier: 0 };
    /// let mut boxed = mapper.into_box();
    /// assert_eq!(boxed.apply(10), 10);  // 10 * 1
    /// assert_eq!(boxed.apply(10), 20);  // 10 * 2
    /// ```
    fn into_box(self) -> BoxStatefulTransformer<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut mapper = self;
        BoxStatefulTransformer::new(move |t| mapper.apply(t))
    }

    /// Converts to RcStatefulTransformer
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcStatefulTransformer<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation first converts to `BoxStatefulTransformer` using
    /// `into_box()`, then wraps it in `RcStatefulTransformer`. Specific implementations
    /// may override this for better efficiency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, RcStatefulTransformer};
    ///
    /// struct CustomMapper {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulTransformer<i32, i32> for CustomMapper {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let mapper = CustomMapper { multiplier: 0 };
    /// let mut rc_mapper = mapper.into_rc();
    /// assert_eq!(rc_mapper.apply(10), 10);  // 10 * 1
    /// assert_eq!(rc_mapper.apply(10), 20);  // 10 * 2
    /// ```
    fn into_rc(self) -> RcStatefulTransformer<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut mapper = self;
        RcStatefulTransformer::new(move |t| mapper.apply(t))
    }

    /// Converts to ArcStatefulTransformer
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcStatefulTransformer<T, R>`
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `ArcStatefulTransformer` by creating
    /// a new closure that calls `self.apply()`. Note that this requires `self`
    /// to implement `Send` due to Arc's thread-safety requirements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, ArcStatefulTransformer};
    ///
    /// struct CustomMapper {
    ///     multiplier: i32,
    /// }
    ///
    /// impl StatefulTransformer<i32, i32> for CustomMapper {
    ///     fn apply(&mut self, input: i32) -> i32 {
    ///         self.multiplier += 1;
    ///         input * self.multiplier
    ///     }
    /// }
    ///
    /// let mapper = CustomMapper { multiplier: 0 };
    /// let mut arc_mapper = mapper.into_arc();
    /// assert_eq!(arc_mapper.apply(10), 10);  // 10 * 1
    /// assert_eq!(arc_mapper.apply(10), 20);  // 10 * 2
    /// ```
    fn into_arc(self) -> ArcStatefulTransformer<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        let mut mapper = self;
        ArcStatefulTransformer::new(move |t| mapper.apply(t))
    }

    /// Converts to a closure implementing `FnMut(T) -> R`
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns an implementation of `FnMut(T) -> R`
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a new closure that calls `self.apply()`.
    /// Specific implementations may override this for better efficiency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, BoxStatefulTransformer};
    ///
    /// let mapper = BoxStatefulTransformer::new(|x: i32| x * 2);
    /// let mut closure = mapper.into_fn();
    /// assert_eq!(closure(10), 20);
    /// assert_eq!(closure(15), 30);
    /// ```
    fn into_fn(self) -> impl FnMut(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        let mut mapper = self;
        move |t| mapper.apply(t)
    }

    /// Non-consuming conversion to `BoxStatefulTransformer`.
    ///
    /// Default implementation requires `Self: Clone` and wraps a cloned
    /// instance in a `RefCell` so the returned mapper can mutate state
    /// across calls.
    fn to_box(&self) -> BoxStatefulTransformer<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Non-consuming conversion to `RcStatefulTransformer`.
    ///
    /// Default implementation clones `self` into an `Rc<RefCell<_>>` so the
    /// resulting mapper can be shared within a single thread.
    fn to_rc(&self) -> RcStatefulTransformer<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_rc()
    }

    /// Non-consuming conversion to `ArcStatefulTransformer` (thread-safe).
    ///
    /// Default implementation requires `Self: Clone + Send + Sync` and wraps
    /// the cloned instance in `Arc<Mutex<_>>` so it can be used across
    /// threads.
    fn to_arc(&self) -> ArcStatefulTransformer<T, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Non-consuming conversion to a closure (`FnMut(T) -> R`).
    ///
    /// Default implementation clones `self` into a `RefCell` and returns a
    /// closure that calls `apply` on the interior mutable value.
    fn to_fn(&self) -> impl FnMut(T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn()
    }
}

// ============================================================================
// BoxStatefulTransformer - Box<dyn FnMut(T) -> R>
// ============================================================================

/// BoxStatefulTransformer - mapper wrapper based on `Box<dyn FnMut>`
///
/// A mapper wrapper that provides single ownership with reusable stateful
/// transformation. The mapper consumes the input and can be called
/// multiple times while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulTransformer<T, R> {
    function: Box<dyn FnMut(T) -> R>,
}

impl<T, R> BoxStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxStatefulTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.apply(10), 11);
    /// assert_eq!(mapper.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + 'static,
    {
        BoxStatefulTransformer {
            function: Box::new(f),
        }
    }

    // BoxStatefulTransformer is intentionally not given a `to_*` specialization here
    // because the boxed `FnMut` is not clonable and we cannot produce a
    // non-consuming adapter from `&self` without moving ownership or
    // requiring `Clone` on the inner function. Consumers should use the
    // blanket `StatefulTransformer::to_*` defaults when their mapper type implements
    // `Clone`.

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulTransformer, StatefulTransformer};
    ///
    /// let mut identity = BoxStatefulTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> BoxStatefulTransformer<T, T> {
        BoxStatefulTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement StatefulTransformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original mapper, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulTransformer<R, S>`
    ///   - An `RcStatefulTransformer<R, S>`
    ///   - An `ArcStatefulTransformer<R, S>`
    ///   - Any type implementing `StatefulTransformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxStatefulTransformer<T, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
    {
        let mut self_mapper = self;
        let mut after_mapper = after;
        BoxStatefulTransformer::new(move |x: T| {
            let intermediate = self_mapper.apply(x);
            after_mapper.apply(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement StatefulTransformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If
    ///   you need to preserve the original mapper, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulTransformer<S, T>`
    ///   - An `RcStatefulTransformer<S, T>`
    ///   - An `ArcStatefulTransformer<S, T>`
    ///   - Any type implementing `StatefulTransformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mapper = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(self, before: F) -> BoxStatefulTransformer<S, R>
    where
        S: 'static,
        F: StatefulTransformer<S, T> + 'static,
    {
        let mut self_mapper = self;
        let mut before_mapper = before;
        BoxStatefulTransformer::new(move |x: S| {
            let intermediate = before_mapper.apply(x);
            self_mapper.apply(intermediate)
        })
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper for
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
    /// Returns `BoxConditionalStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, BoxStatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalStatefulTransformer<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalStatefulTransformer {
            mapper: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxStatefulTransformer<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulTransformer, StatefulTransformer};
    ///
    /// let mut constant = BoxStatefulTransformer::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxStatefulTransformer<T, R> {
        BoxStatefulTransformer::new(move |_| value.clone())
    }
}

impl<T, R> StatefulTransformer<T, R> for BoxStatefulTransformer<T, R> {
    fn apply(&mut self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function;
        RcStatefulTransformer::new(self_fn)
    }

    // do NOT override StatefulTransformer::into_arc() because BoxStatefulTransformer is not Send + Sync
    // and calling BoxStatefulTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return the boxed function
        self.function
    }

    // do NOT override StatefulTransformer::to_xxx() because BoxStatefulTransformer is not Clone
    // and calling BoxStatefulTransformer::to_xxx() will cause a compile error
}

impl<T, R> TransformerOnce<T, R> for BoxStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    fn apply_once(mut self, input: T) -> R {
        StatefulTransformer::apply(&mut self, input)
    }

    fn into_box_once(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxTransformerOnce::new(self.function)
    }

    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
    {
        let mut f = self.function;
        move |input: T| f(input)
    }

    // NOTE: `BoxStatefulTransformer` is not `Clone`, so it cannot offer
    // `to_box_once` or `to_fn_once` implementations. Invoking the default
    // trait methods will not compile because the required `Clone`
    // bound is not satisfied.
}

// ============================================================================
// BoxConditionalStatefulTransformer - Box-based Conditional StatefulTransformer
// ============================================================================

/// BoxConditionalStatefulTransformer struct
///
/// A conditional mapper that only executes when a predicate is satisfied.
/// Uses `BoxStatefulTransformer` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxStatefulTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else
///   logic
/// - **Implements StatefulTransformer**: Can be used anywhere a `StatefulTransformer` is expected
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulTransformer, BoxStatefulTransformer};
///
/// let mut high_count = 0;
/// let mut low_count = 0;
///
/// let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
///     high_count += 1;
///     x * 2
/// })
/// .when(|x: &i32| *x >= 10)
/// .or_else(move |x| {
///     low_count += 1;
///     x + 1
/// });
///
/// assert_eq!(mapper.apply(15), 30); // when branch executed
/// assert_eq!(mapper.apply(5), 6);   // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalStatefulTransformer<T, R> {
    mapper: BoxStatefulTransformer<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `BoxStatefulTransformer<T, R>`, `RcStatefulTransformer<T, R>`, `ArcStatefulTransformer<T, R>`
    ///   - Any type implementing `StatefulTransformer<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, BoxStatefulTransformer};
    ///
    /// let mut mapper = BoxStatefulTransformer::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.apply(5), 10);   // Condition satisfied
    /// assert_eq!(mapper.apply(-5), 5);   // Condition not satisfied
    /// ```
    pub fn or_else<F>(self, mut else_mapper: F) -> BoxStatefulTransformer<T, R>
    where
        F: StatefulTransformer<T, R> + 'static,
    {
        let pred = self.predicate;
        let mut then_mapper = self.mapper;
        BoxStatefulTransformer::new(move |t| {
            if pred.test(&t) {
                then_mapper.apply(t)
            } else {
                else_mapper.apply(t)
            }
        })
    }
}

// ============================================================================
// ArcStatefulTransformer - Arc<Mutex<dyn FnMut(T) -> R + Send>>
// ============================================================================

/// ArcStatefulTransformer - thread-safe mapper wrapper
///
/// A thread-safe, clonable mapper wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads
/// while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(T) -> R + Send>>`
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
pub struct ArcStatefulTransformer<T, R> {
    function: Arc<Mutex<dyn FnMut(T) -> R + Send>>,
}

impl<T, R> ArcStatefulTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Creates a new ArcStatefulTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = ArcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.apply(10), 11);
    /// assert_eq!(mapper.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + Send + 'static,
    {
        ArcStatefulTransformer {
            function: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut identity = ArcStatefulTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> ArcStatefulTransformer<T, T> {
        ArcStatefulTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement StatefulTransformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulTransformer<R, S>`
    ///   - An `RcStatefulTransformer<R, S>`
    ///   - An `ArcStatefulTransformer<R, S>` (will be cloned internally)
    ///   - Any type implementing `StatefulTransformer<R, S> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = ArcStatefulTransformer::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = ArcStatefulTransformer::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    ///
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> ArcStatefulTransformer<T, S>
    where
        S: Send + 'static,
        F: StatefulTransformer<R, S> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let after = Arc::new(Mutex::new(after));
        ArcStatefulTransformer {
            function: Arc::new(Mutex::new(move |x: T| {
                let intermediate = self_fn.lock().unwrap()(x);
                after.lock().unwrap().apply(intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement StatefulTransformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulTransformer<S, T>`
    ///   - An `RcStatefulTransformer<S, T>`
    ///   - An `ArcStatefulTransformer<S, T>` (will be cloned internally)
    ///   - Any type implementing `StatefulTransformer<S, T> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mapper = ArcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> ArcStatefulTransformer<S, R>
    where
        S: Send + 'static,
        F: StatefulTransformer<S, T> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let before = Arc::new(Mutex::new(before));
        ArcStatefulTransformer {
            function: Arc::new(Mutex::new(move |x: S| {
                let intermediate = before.lock().unwrap().apply(x);
                self_fn.lock().unwrap()(intermediate)
            })),
        }
    }

    /// Creates a conditional mapper (thread-safe version)
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper.
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
    /// Returns `ArcConditionalStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, ArcStatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = ArcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalStatefulTransformer<T, R>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        ArcConditionalStatefulTransformer {
            mapper: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, R> ArcStatefulTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: Clone + Send + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut constant = ArcStatefulTransformer::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcStatefulTransformer<T, R> {
        ArcStatefulTransformer::new(move |_| value.clone())
    }
}

impl<T, R> StatefulTransformer<T, R> for ArcStatefulTransformer<T, R> {
    fn apply(&mut self, input: T) -> R {
        (self.function.lock().unwrap())(input)
    }

    fn into_box(self) -> BoxStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxStatefulTransformer::new(move |x| self.function.lock().unwrap()(x))
    }

    fn into_rc(self) -> RcStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcStatefulTransformer::new(move |x| self.function.lock().unwrap()(x))
    }

    fn into_arc(self) -> ArcStatefulTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Efficient: use Arc cloning to create a closure
        move |input: T| (self.function.lock().unwrap())(input)
    }

    fn to_box(&self) -> BoxStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxStatefulTransformer::new(move |x| self_fn.lock().unwrap()(x))
    }

    fn to_rc(&self) -> RcStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        RcStatefulTransformer::new(move |x| self_fn.lock().unwrap()(x))
    }

    fn to_arc(&self) -> ArcStatefulTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |input: T| self_fn.lock().unwrap()(input)
    }
}

impl<T, R> Clone for ArcStatefulTransformer<T, R> {
    fn clone(&self) -> Self {
        ArcStatefulTransformer {
            function: Arc::clone(&self.function),
        }
    }
}

impl<T, R> TransformerOnce<T, R> for ArcStatefulTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    fn apply_once(mut self, input: T) -> R {
        StatefulTransformer::apply(&mut self, input)
    }

    fn into_box_once(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformerOnce::new(move |input| self.function.lock().unwrap()(input))
    }

    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |input: T| self.function.lock().unwrap()(input)
    }

    fn to_box_once(&self) -> BoxTransformerOnce<T, R>
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxTransformerOnce::new(move |input| self_fn.lock().unwrap()(input))
    }

    fn to_fn_once(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |input: T| self_fn.lock().unwrap()(input)
    }
}

// ============================================================================
// ArcConditionalStatefulTransformer - Arc-based Conditional StatefulTransformer
// ============================================================================

/// ArcConditionalStatefulTransformer struct
///
/// A thread-safe conditional mapper that only executes when a predicate
/// is satisfied. Uses `ArcStatefulTransformer` and `ArcPredicate` for shared
/// ownership across threads.
///
/// This type is typically created by calling `ArcStatefulTransformer::when()` and is
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
/// use prism3_function::{StatefulTransformer, ArcStatefulTransformer};
///
/// let mut mapper = ArcStatefulTransformer::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut mapper_clone = mapper.clone();
///
/// assert_eq!(mapper.apply(5), 10);
/// assert_eq!(mapper_clone.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalStatefulTransformer<T, R> {
    mapper: ArcStatefulTransformer<T, R>,
    predicate: ArcPredicate<T>,
}

impl<T, R> ArcConditionalStatefulTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R` (must be `Send`)
    ///   - `ArcStatefulTransformer<T, R>`, `BoxStatefulTransformer<T, R>`
    ///   - Any type implementing `StatefulTransformer<T, R> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, ArcStatefulTransformer};
    ///
    /// let mut mapper = ArcStatefulTransformer::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.apply(5), 10);
    /// assert_eq!(mapper.apply(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_mapper: F) -> ArcStatefulTransformer<T, R>
    where
        F: StatefulTransformer<T, R> + Send + 'static,
    {
        let pred = self.predicate;
        let then_mapper = self.mapper;
        let else_mapper = Arc::new(Mutex::new(else_mapper));
        ArcStatefulTransformer {
            function: Arc::new(Mutex::new(move |t| {
                if pred.test(&t) {
                    then_mapper.function.lock().unwrap()(t)
                } else {
                    else_mapper.lock().unwrap().apply(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for ArcConditionalStatefulTransformer<T, R> {
    /// Clones the conditional mapper
    ///
    /// Creates a new instance that shares the underlying mapper and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcStatefulTransformer - Rc<RefCell<dyn FnMut(T) -> R>>
// ============================================================================

/// RcStatefulTransformer - single-threaded mapper wrapper
///
/// A single-threaded, clonable mapper wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<RefCell<dyn FnMut(T) -> R>>`
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
pub struct RcStatefulTransformer<T, R> {
    function: Rc<RefCell<dyn FnMut(T) -> R>>,
}

impl<T, R> RcStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcStatefulTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = RcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.apply(10), 11);
    /// assert_eq!(mapper.apply(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + 'static,
    {
        RcStatefulTransformer {
            function: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut identity = RcStatefulTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> RcStatefulTransformer<T, T> {
        RcStatefulTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement StatefulTransformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulTransformer<R, S>`
    ///   - An `RcStatefulTransformer<R, S>` (will be cloned internally)
    ///   - An `ArcStatefulTransformer<R, S>`
    ///   - Any type implementing `StatefulTransformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new RcStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = RcStatefulTransformer::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = RcStatefulTransformer::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    ///
    /// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> RcStatefulTransformer<T, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let after = Rc::new(RefCell::new(after));
        RcStatefulTransformer {
            function: Rc::new(RefCell::new(move |x: T| {
                let intermediate = self_fn.borrow_mut()(x);
                after.borrow_mut().apply(intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement StatefulTransformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulTransformer<S, T>`
    ///   - An `RcStatefulTransformer<S, T>` (will be cloned internally)
    ///   - An `ArcStatefulTransformer<S, T>`
    ///   - Any type implementing `StatefulTransformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new RcStatefulTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mapper = RcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> RcStatefulTransformer<S, R>
    where
        S: 'static,
        F: StatefulTransformer<S, T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let before = Rc::new(RefCell::new(before));
        RcStatefulTransformer {
            function: Rc::new(RefCell::new(move |x: S| {
                let intermediate = before.borrow_mut().apply(x);
                self_fn.borrow_mut()(intermediate)
            })),
        }
    }

    /// Creates a conditional mapper (single-threaded shared version)
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper.
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
    /// Returns `RcConditionalStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, RcStatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mut mapper = RcStatefulTransformer::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.apply(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.apply(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> RcConditionalStatefulTransformer<T, R>
    where
        P: Predicate<T> + 'static,
    {
        RcConditionalStatefulTransformer {
            mapper: self,
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, R> RcStatefulTransformer<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulTransformer, StatefulTransformer};
    ///
    /// let mut constant = RcStatefulTransformer::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> RcStatefulTransformer<T, R> {
        RcStatefulTransformer::new(move |_| value.clone())
    }
}

impl<T, R> StatefulTransformer<T, R> for RcStatefulTransformer<T, R> {
    fn apply(&mut self, input: T) -> R {
        (self.function.borrow_mut())(input)
    }

    fn into_box(self) -> BoxStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxStatefulTransformer {
            function: Box::new(move |x| self.function.borrow_mut()(x)),
        }
    }

    fn into_rc(self) -> RcStatefulTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    // do NOT override StatefulTransformer::into_arc() because RcStatefulTransformer is not Send + Sync
    // and calling RcStatefulTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Efficient: use Rc cloning to create a closure
        move |input: T| (self.function.borrow_mut())(input)
    }
}

impl<T, R> Clone for RcStatefulTransformer<T, R> {
    fn clone(&self) -> Self {
        RcStatefulTransformer {
            function: Rc::clone(&self.function),
        }
    }
}

impl<T, R> TransformerOnce<T, R> for RcStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    fn apply_once(mut self, input: T) -> R {
        StatefulTransformer::apply(&mut self, input)
    }

    fn into_box_once(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxTransformerOnce::new(move |input| self.function.borrow_mut()(input))
    }

    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
    {
        move |input: T| self.function.borrow_mut()(input)
    }

    fn to_box_once(&self) -> BoxTransformerOnce<T, R>
    where
        Self: Clone + 'static,
    {
        let self_fn = self.function.clone();
        BoxTransformerOnce::new(move |input| self_fn.borrow_mut()(input))
    }

    fn to_fn_once(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + 'static,
    {
        let self_fn = self.function.clone();
        move |input: T| self_fn.borrow_mut()(input)
    }
}

// ============================================================================
// RcConditionalStatefulTransformer - Rc-based Conditional StatefulTransformer
// ============================================================================

/// RcConditionalStatefulTransformer struct
///
/// A single-threaded conditional mapper that only executes when a
/// predicate is satisfied. Uses `RcStatefulTransformer` and `RcPredicate` for shared
/// ownership within a single thread.
///
/// This type is typically created by calling `RcStatefulTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalStatefulTransformer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulTransformer, RcStatefulTransformer};
///
/// let mut mapper = RcStatefulTransformer::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut mapper_clone = mapper.clone();
///
/// assert_eq!(mapper.apply(5), 10);
/// assert_eq!(mapper_clone.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalStatefulTransformer<T, R> {
    mapper: RcStatefulTransformer<T, R>,
    predicate: RcPredicate<T>,
}

impl<T, R> RcConditionalStatefulTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `RcStatefulTransformer<T, R>`, `BoxStatefulTransformer<T, R>`
    ///   - Any type implementing `StatefulTransformer<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, RcStatefulTransformer};
    ///
    /// let mut mapper = RcStatefulTransformer::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.apply(5), 10);
    /// assert_eq!(mapper.apply(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_mapper: F) -> RcStatefulTransformer<T, R>
    where
        F: StatefulTransformer<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_mapper = self.mapper;
        let else_mapper = Rc::new(RefCell::new(else_mapper));
        RcStatefulTransformer {
            function: Rc::new(RefCell::new(move |t| {
                if pred.test(&t) {
                    then_mapper.function.borrow_mut()(t)
                } else {
                    else_mapper.borrow_mut().apply(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for RcConditionalStatefulTransformer<T, R> {
    /// Clones the conditional mapper
    ///
    /// Creates a new instance that shares the underlying mapper and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnMut trait
// ============================================================================

/// Implement StatefulTransformer<T, R> for any type that implements FnMut(T) -> R
///
/// This allows closures to be used directly with our StatefulTransformer trait
/// without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::StatefulTransformer;
///
/// let mut counter = 0;
/// let mut mapper = |x: i32| {
///     counter += 1;
///     x + counter
/// };
///
/// assert_eq!(mapper.apply(10), 11);
/// assert_eq!(mapper.apply(10), 12);
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T, R> StatefulTransformer<T, R> for F
where
    F: FnMut(T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply(&mut self, input: T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxStatefulTransformer<T, R>
    where
        Self: Sized + 'static,
    {
        BoxStatefulTransformer::new(self)
    }

    fn into_rc(self) -> RcStatefulTransformer<T, R>
    where
        Self: Sized + 'static,
    {
        RcStatefulTransformer::new(self)
    }

    fn into_arc(self) -> ArcStatefulTransformer<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        ArcStatefulTransformer::new(self)
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself (the closure)
        self
    }

    /// Non-consuming conversion to `BoxStatefulTransformer` for closures.
    ///
    /// We can create a `BoxStatefulTransformer` by boxing the closure and returning a
    /// new `BoxStatefulTransformer`. This does not require `Clone` because we consume
    /// the closure value passed by the caller when they call this
    /// method. For `&self`-style non-consuming `to_*` adapters, users can
    /// use the `StatefulTransformer::to_*` defaults which clone the closure when
    /// possible.
    fn to_box(&self) -> BoxStatefulTransformer<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        // Clone the closure into a RefCell to allow interior mutability
        // across calls.
        let cell = RefCell::new(self.clone());
        BoxStatefulTransformer::new(move |input: T| cell.borrow_mut().apply(input))
    }

    fn to_rc(&self) -> RcStatefulTransformer<T, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cell = Rc::new(RefCell::new(self.clone()));
        RcStatefulTransformer::new(move |input: T| cell.borrow_mut().apply(input))
    }

    fn to_arc(&self) -> ArcStatefulTransformer<T, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        let cell = Arc::new(Mutex::new(self.clone()));
        ArcStatefulTransformer::new(move |input: T| cell.lock().unwrap().apply(input))
    }

    fn to_fn(&self) -> impl FnMut(T) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        R: 'static,
    {
        let cell = RefCell::new(self.clone());
        move |input: T| cell.borrow_mut().apply(input)
    }
}

// ============================================================================
// FnStatefulTransformerOps - Extension trait for closure mappers
// ============================================================================

/// Extension trait for closures implementing `FnMut(T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for
/// closures without requiring explicit wrapping in `BoxStatefulTransformer`,
/// `RcStatefulTransformer`, or `ArcStatefulTransformer`.
///
/// This trait is automatically implemented for all closures that
/// implement `FnMut(T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `StatefulTransformer<T, R>` through blanket
/// implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides
/// those methods, returning `BoxStatefulTransformer` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps};
///
/// let mut counter1 = 0;
/// let mapper1 = move |x: i32| {
///     counter1 += 1;
///     x + counter1
/// };
///
/// let mut counter2 = 0;
/// let mapper2 = move |x: i32| {
///     counter2 += 1;
///     x * counter2
/// };
///
/// let mut composed = mapper1.and_then(mapper2);
/// assert_eq!(composed.apply(10), 11);  // (10 + 1) * 1
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps};
///
/// let mut counter = 0;
/// let mapper = move |x: i32| {
///     counter += 1;
///     x * counter
/// };
///
/// let mut composed = mapper.compose(|x: i32| x + 1);
/// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
/// ```
///
/// ## Conditional mapping with when
///
/// ```rust
/// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps};
///
/// let mut mapper = (|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// assert_eq!(mapper.apply(5), 10);
/// assert_eq!(mapper.apply(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnStatefulTransformerOps<T, R>: FnMut(T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Consumes self and returns a
    /// `BoxStatefulTransformer`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement StatefulTransformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxStatefulTransformer<R, S>`
    ///   - An `RcStatefulTransformer<R, S>`
    ///   - An `ArcStatefulTransformer<R, S>`
    ///   - Any type implementing `StatefulTransformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulTransformer<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps, BoxStatefulTransformer};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// };
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = BoxStatefulTransformer::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    /// assert_eq!(composed.apply(10), 11);
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxStatefulTransformer<T, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulTransformer::new(self).and_then(after)
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Consumes self and returns a
    /// `BoxStatefulTransformer`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement StatefulTransformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxStatefulTransformer<S, T>`
    ///   - An `RcStatefulTransformer<S, T>`
    ///   - An `ArcStatefulTransformer<S, T>`
    ///   - Any type implementing `StatefulTransformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulTransformer<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps, BoxStatefulTransformer};
    ///
    /// let mut counter = 0;
    /// let mapper = move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// };
    ///
    /// let before = BoxStatefulTransformer::new(|x: i32| x + 1);
    ///
    /// let mut composed = mapper.compose(before);
    /// assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    /// ```
    fn compose<S, F>(self, before: F) -> BoxStatefulTransformer<S, R>
    where
        S: 'static,
        F: StatefulTransformer<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulTransformer::new(self).compose(before)
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper for
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
    /// Returns `BoxConditionalStatefulTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{StatefulTransformer, FnStatefulTransformerOps};
    ///
    /// let mut mapper = (|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.apply(5), 10);
    /// assert_eq!(mapper.apply(-5), 5);
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalStatefulTransformer<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxStatefulTransformer::new(self).when(predicate)
    }
}

/// Blanket implementation of FnStatefulTransformerOps for all closures
///
/// Automatically implements `FnStatefulTransformerOps<T, R>` for any type that
/// implements `FnMut(T) -> R`.
///
/// # Author
///
/// Haixing Hu
impl<T, R, F> FnStatefulTransformerOps<T, R> for F where F: FnMut(T) -> R + 'static {}
