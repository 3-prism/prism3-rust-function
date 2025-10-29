/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # FunctionOnce Types
//!
//! Provides Rust implementations of consuming function traits similar to
//! Rust's `FnOnce` trait, for computing output from input references.
//!
//! This module provides the `FunctionOnce<T, R>` trait and one-time use
//! implementations:
//!
//! - [`BoxFunctionOnce`]: Single ownership, one-time use
//!
//! # Author
//!
//! Haixing Hu

use crate::predicates::predicate::{
    BoxPredicate,
    Predicate,
};

// ============================================================================
// Core Trait
// ============================================================================

/// FunctionOnce trait - consuming function that takes ownership
///
/// Defines the behavior of a consuming function: computing a value of
/// type `R` from a reference to type `T` by taking ownership of self.
/// This trait is analogous to `FnOnce(&T) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (borrowed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Haixing Hu
pub trait FunctionOnce<T, R> {
    /// Applies the function to the input reference, consuming self
    ///
    /// # Parameters
    ///
    /// * `input` - Reference to the input value
    ///
    /// # Returns
    ///
    /// The computed output value
    fn apply_once(self, input: &T) -> R;

    /// Converts to BoxFunctionOnce
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxFunctionOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FunctionOnce;
    ///
    /// let double = |x: &i32| x * 2;
    /// let boxed = double.into_box_once();
    /// assert_eq!(boxed.apply_once(&21), 42);
    /// ```
    fn into_box_once(self) -> BoxFunctionOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce::new(move |input: &T| self.apply_once(input))
    }

    /// Converts function to a closure
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(&T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FunctionOnce;
    ///
    /// let double = |x: &i32| x * 2;
    /// let func = double.into_fn_once();
    /// assert_eq!(func(&21), 42);
    /// ```
    fn into_fn_once(self) -> impl FnOnce(&T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |input: &T| self.apply_once(input)
    }

    /// Converts to BoxFunctionOnce without consuming self
    ///
    /// **📌 Borrows `&self`**: The original function remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a new `BoxFunctionOnce` that
    /// captures a clone. Types implementing `Clone` can override this method
    /// to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `BoxFunctionOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FunctionOnce;
    ///
    /// let double = |x: &i32| x * 2;
    /// let boxed = double.to_box_once();
    /// assert_eq!(boxed.apply_once(&21), 42);
    /// ```
    fn to_box_once(&self) -> BoxFunctionOnce<T, R>
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box_once()
    }

    /// Converts function to a closure without consuming self
    ///
    /// **📌 Borrows `&self`**: The original function remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a closure that captures a
    /// clone of `self` and calls its `apply_once` method. Types can
    /// override this method to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(&T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FunctionOnce;
    ///
    /// let double = |x: &i32| x * 2;
    /// let func = double.to_fn_once();
    /// assert_eq!(func(&21), 42);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(&T) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn_once()
    }
}

// ============================================================================
// BoxFunctionOnce - Box<dyn FnOnce(&T) -> R>
// ============================================================================

/// BoxFunctionOnce - consuming transformer wrapper based on
/// `Box<dyn FnOnce>`
///
/// A transformer wrapper that provides single ownership with one-time use
/// semantics. Consumes both self and the input value.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(&T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (consumes self and input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Haixing Hu
pub struct BoxFunctionOnce<T, R> {
    function: Box<dyn FnOnce(&T) -> R>,
}

impl<T, R> BoxFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxFunctionOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionOnce, FunctionOnce};
    ///
    /// let parse = BoxFunctionOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.apply_once("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&T) -> R + 'static,
    {
        BoxFunctionOnce {
            function: Box::new(f),
        }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionOnce, FunctionOnce};
    ///
    /// let identity = BoxFunctionOnce::<i32, i32>::identity();
    /// assert_eq!(identity.apply_once(42), 42);
    /// ```
    pub fn identity() -> BoxFunctionOnce<T, T>
    where
        T: Clone,
    {
        BoxFunctionOnce::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `G` - The type of the after transformer (must implement
    ///   FunctionOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxFunctionOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxFunctionOnce<R, S>`
    ///   - Any type implementing `FunctionOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionOnce, FunctionOnce};
    ///
    /// let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    ///
    /// // Both add_one and double are moved and consumed
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    /// // add_one.apply_once(3); // Would not compile - moved
    /// // double.apply_once(4);  // Would not compile - moved
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxFunctionOnce<T, S>
    where
        S: 'static,
        G: FunctionOnce<R, S> + 'static,
    {
        BoxFunctionOnce::new(move |x| {
            let intermediate = (self.function)(x);
            after.apply_once(&intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `G` - The type of the before transformer (must implement
    ///   FunctionOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxFunctionOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxFunctionOnce<S, T>`
    ///   - Any type implementing `FunctionOnce<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionOnce, FunctionOnce};
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
    ///
    /// // Both double and add_one are moved and consumed
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    /// // double.apply_once(3); // Would not compile - moved
    /// // add_one.apply_once(4); // Would not compile - moved
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxFunctionOnce<S, R>
    where
        S: 'static,
        G: FunctionOnce<S, T> + 'static,
    {
        let self_fn = self.function;
        BoxFunctionOnce::new(move |x: &S| self_fn(&before.apply_once(x)))
    }

    /// Creates a conditional transformer
    ///
    /// Returns a transformer that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative transformer.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalFunctionOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, BoxFunctionOnce};
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let identity = BoxFunctionOnce::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// let double2 = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let identity2 = BoxFunctionOnce::<i32, i32>::identity();
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(identity2);
    /// assert_eq!(conditional2.apply_once(-5), -5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, BoxFunctionOnce, RcPredicate};
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(BoxFunctionOnce::identity());
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalFunctionOnce<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalFunctionOnce {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxFunctionOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionOnce, FunctionOnce};
    ///
    /// let constant = BoxFunctionOnce::constant("hello");
    /// assert_eq!(constant.apply_once(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxFunctionOnce<T, R> {
        BoxFunctionOnce::new(move |_| value.clone())
    }
}

impl<T, R> FunctionOnce<T, R> for BoxFunctionOnce<T, R> {
    fn apply_once(self, input: &T) -> R {
        (self.function)(input)
    }

    fn into_box_once(self) -> BoxFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn_once(self) -> impl FnOnce(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return the inner function
        self.function
    }

    // do NOT override BoxFunction::to_box_once() and BoxFunction::to_fn_once()
    // because BoxFunction is not Clone and calling BoxFunction::to_box_once()
    // or BoxFunction::to_fn_once() will cause a compile error
}

// ============================================================================
// BoxConditionalFunctionOnce - Box-based Conditional Function
// ============================================================================

/// BoxConditionalFunctionOnce struct
///
/// A conditional consuming transformer that only executes when a predicate is
/// satisfied. Uses `BoxFunctionOnce` and `BoxPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxFunctionOnce::when()` and
/// is designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **One-time Use**: Can only be called once
/// - **Conditional Execution**: Only transforms when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{FunctionOnce, BoxFunctionOnce};
///
/// let double = BoxFunctionOnce::new(|x: i32| x * 2);
/// let negate = BoxFunctionOnce::new(|x: i32| -x);
/// let conditional = double.when(|x: &i32| *x > 0).or_else(negate);
/// assert_eq!(conditional.apply_once(5), 10); // when branch executed
///
/// let double2 = BoxFunctionOnce::new(|x: i32| x * 2);
/// let negate2 = BoxFunctionOnce::new(|x: i32| -x);
/// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(negate2);
/// assert_eq!(conditional2.apply_once(-5), 5); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalFunctionOnce<T, R> {
    transformer: BoxFunctionOnce<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch
    ///
    /// Executes the original transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The transformer for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `BoxFunctionOnce<T, R>`
    ///   - Any type implementing `FunctionOnce<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxFunctionOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, BoxFunctionOnce};
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    /// assert_eq!(conditional.apply_once(5), 10); // Condition satisfied, execute double
    ///
    /// let double2 = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    /// assert_eq!(conditional2.apply_once(-5), 5); // Condition not satisfied, execute negate
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxFunctionOnce<T, R>
    where
        F: FunctionOnce<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxFunctionOnce::new(move |t| {
            if pred.test(t) {
                then_trans.apply_once(t)
            } else {
                else_transformer.apply_once(t)
            }
        })
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement FunctionOnce<T, R> for any type that implements
/// FnOnce(&T) -> R
///
/// This allows once-callable closures and function pointers to be used
/// directly with our FunctionOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::FunctionOnce;
///
/// fn parse(s: String) -> i32 {
///     s.parse().unwrap_or(0)
/// }
///
/// assert_eq!(parse.apply_once("42".to_string()), 42);
///
/// let owned_value = String::from("hello");
/// let consume = |s: String| {
///     format!("{} world", s)
/// };
/// assert_eq!(consume.apply_once(owned_value), "hello world");
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T, R> FunctionOnce<T, R> for F
where
    F: FnOnce(&T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply_once(self, input: &T) -> R {
        self(input)
    }

    fn into_box_once(self) -> BoxFunctionOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxFunctionOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(&T) -> R
    where
        Self: Sized + 'static,
    {
        // Zero-cost: directly return self since F is already FnOnce(&T) -> R
        self
    }

    fn to_box_once(&self) -> BoxFunctionOnce<T, R>
    where
        Self: Clone + Sized + 'static,
    {
        self.clone().into_box_once()
    }

    fn to_fn_once(&self) -> impl FnOnce(&T) -> R
    where
        Self: Clone + Sized + 'static,
    {
        self.clone()
    }
}

// ============================================================================
// FnFunctionOnceOps - Extension trait for FnOnce transformers
// ============================================================================

/// Extension trait for closures implementing `FnOnce(&T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for one-time
/// use closures and function pointers without requiring explicit wrapping in
/// `BoxFunctionOnce`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnOnce(&T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `FunctionOnce<T, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides those
/// methods, returning `BoxFunctionOnce` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{FunctionOnce, FnFunctionOnceOps};
///
/// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
/// let double = |x: i32| x * 2;
///
/// let composed = parse.and_then(double);
/// assert_eq!(composed.apply_once("21".to_string()), 42);
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{FunctionOnce, FnFunctionOnceOps};
///
/// let double = |x: i32| x * 2;
/// let to_string = |x: i32| x.to_string();
///
/// let composed = to_string.compose(double);
/// assert_eq!(composed.apply_once(21), "42");
/// ```
///
/// ## Conditional transformation with when
///
/// ```rust
/// use prism3_function::{FunctionOnce, FnFunctionOnceOps};
///
/// let double = |x: i32| x * 2;
/// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
///
/// assert_eq!(conditional.apply_once(5), 10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnFunctionOnceOps<T, R>: FnOnce(&T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Consumes self and returns
    /// a `BoxFunctionOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `G` - The type of the after transformer (must implement
    ///   FunctionOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` transformer, the parameter will be consumed. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxFunctionOnce<R, S>`
    ///   - Any type implementing `FunctionOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxFunctionOnce<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, FnFunctionOnceOps,
    ///     BoxFunctionOnce};
    ///
    /// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    ///
    /// // double is moved and consumed
    /// let composed = parse.and_then(double);
    /// assert_eq!(composed.apply_once("21".to_string()), 42);
    /// // double.apply_once(5); // Would not compile - moved
    /// ```
    fn and_then<S, G>(self, after: G) -> BoxFunctionOnce<T, S>
    where
        S: 'static,
        G: FunctionOnce<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce::new(move |x: &T| after.apply_once(&self(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self and returns
    /// a `BoxFunctionOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `G` - The type of the before transformer (must implement
    ///   FunctionOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` transformer, the parameter will be consumed. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxFunctionOnce<S, T>`
    ///   - Any type implementing `FunctionOnce<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxFunctionOnce<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, FnFunctionOnceOps,
    ///     BoxFunctionOnce};
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let to_string = |x: i32| x.to_string();
    ///
    /// // double is moved and consumed
    /// let composed = to_string.compose(double);
    /// assert_eq!(composed.apply_once(21), "42");
    /// // double.apply_once(5); // Would not compile - moved
    /// ```
    fn compose<S, G>(self, before: G) -> BoxFunctionOnce<S, R>
    where
        S: 'static,
        G: FunctionOnce<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce::new(move |x: &S| self(&before.apply_once(x)))
    }

    /// Creates a conditional transformer
    ///
    /// Returns a transformer that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative transformer for when
    /// the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalFunctionOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, FnFunctionOnceOps};
    ///
    /// let double = |x: i32| x * 2;
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{FunctionOnce, FnFunctionOnceOps,
    ///     RcPredicate};
    ///
    /// let double = |x: i32| x * 2;
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalFunctionOnce<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce::new(self).when(predicate)
    }
}

/// Blanket implementation of FnFunctionOnceOps for all FnOnce closures
///
/// Automatically implements `FnFunctionOnceOps<T, R>` for any type that
/// implements `FnOnce(&T) -> R`.
///
/// # Author
///
/// Haixing Hu
impl<T, R, F> FnFunctionOnceOps<T, R> for F where F: FnOnce(&T) -> R + 'static {}
