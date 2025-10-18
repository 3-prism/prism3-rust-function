/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # TransformerOnce Types
//!
//! Provides Rust implementations of consuming transformer traits similar to
//! Rust's `FnOnce` trait, but with value-oriented semantics for functional
//! programming patterns.
//!
//! This module provides the `TransformerOnce<T, R>` trait and one-time use
//! implementations:
//!
//! - [`BoxTransformerOnce`]: Single ownership, one-time use
//!
//! # Author
//!
//! Hu Haixing

use crate::predicate::{BoxPredicate, Predicate};

// ============================================================================
// Core Trait
// ============================================================================

/// TransformerOnce trait - consuming transformation that takes ownership
///
/// Defines the behavior of a consuming transformer: converting a value of
/// type `T` to a value of type `R` by taking ownership of both self and the
/// input. This trait is analogous to `FnOnce(T) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait TransformerOnce<T, R> {
    /// Transforms the input value, consuming both self and input
    ///
    /// # Parameters
    ///
    /// * `input` - The input value (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(self, input: T) -> R;

    /// Converts to BoxTransformerOnce
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformerOnce<T, R>`
    fn into_box(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts transformer to a closure
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
}

// ============================================================================
// BoxTransformerOnce - Box<dyn FnOnce(T) -> R>
// ============================================================================

/// BoxTransformerOnce - consuming transformer wrapper based on
/// `Box<dyn FnOnce>`
///
/// A transformer wrapper that provides single ownership with one-time use
/// semantics. Consumes both self and the input value.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (consumes self and input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxTransformerOnce<T, R> {
    function: Box<dyn FnOnce(T) -> R>,
}

impl<T, R> BoxTransformerOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxTransformerOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformerOnce, TransformerOnce};
    ///
    /// let parse = BoxTransformerOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.transform("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxTransformerOnce {
            function: Box::new(f),
        }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformerOnce, TransformerOnce};
    ///
    /// let identity = BoxTransformerOnce::<i32, i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> BoxTransformerOnce<T, T> {
        BoxTransformerOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `G` - The type of the after transformer (must implement
    ///   TransformerOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new BoxTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformerOnce, TransformerOnce};
    ///
    /// let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxTransformerOnce<T, S>
    where
        S: 'static,
        G: TransformerOnce<R, S> + 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = (self.function)(x);
            after.transform(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `G` - The type of the before transformer (must implement
    ///   TransformerOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new BoxTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformerOnce, TransformerOnce};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxTransformerOnce<S, R>
    where
        S: 'static,
        G: TransformerOnce<S, T> + 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = before.transform(x);
            (self.function)(intermediate)
        })
    }

    /// Creates a conditional transformer
    ///
    /// Returns a transformer that only executes when a predicate is satisfied.
    /// When the predicate returns false, the input value is returned unchanged.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, can be:
    ///   - Closure: `|x: &T| -> bool`
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `BoxPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, BoxTransformerOnce};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), -5); // Unchanged
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalTransformerOnce<T, R>
    where
        P: Predicate<T> + 'static,
        R: From<T>,
    {
        BoxConditionalTransformerOnce {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxTransformerOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformerOnce, TransformerOnce};
    ///
    /// let constant = BoxTransformerOnce::constant("hello");
    /// assert_eq!(constant.transform(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxTransformerOnce<T, R> {
        BoxTransformerOnce::new(move |_| value.clone())
    }
}

impl<T, R> TransformerOnce<T, R> for BoxTransformerOnce<T, R> {
    fn transform(self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxTransformerOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| self.transform(t)
    }
}

// ============================================================================
// BoxConditionalTransformerOnce - Box-based Conditional Transformer
// ============================================================================

/// BoxConditionalTransformerOnce struct
///
/// A conditional consuming transformer that only executes when a predicate is
/// satisfied. Uses `BoxTransformerOnce` and `BoxPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxTransformerOnce::when()` and
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
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{TransformerOnce, BoxTransformerOnce};
///
/// let double = BoxTransformerOnce::new(|x: i32| x * 2);
/// let conditional = double.when(|x: &i32| *x > 0);
///
/// assert_eq!(conditional.transform(5), 10); // Executed
/// assert_eq!(conditional.transform(-5), -5); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{TransformerOnce, BoxTransformerOnce};
///
/// let double = BoxTransformerOnce::new(|x: i32| x * 2);
/// let negate = BoxTransformerOnce::new(|x: i32| -x);
/// let conditional = double.when(|x: &i32| *x > 0).or_else(negate);
///
/// assert_eq!(conditional.transform(5), 10); // when branch executed
/// assert_eq!(conditional.transform(-5), 5); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalTransformerOnce<T, R> {
    transformer: BoxTransformerOnce<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> TransformerOnce<T, R> for BoxConditionalTransformerOnce<T, R>
where
    T: 'static,
    R: From<T> + 'static,
{
    fn transform(self, input: T) -> R {
        if self.predicate.test(&input) {
            self.transformer.transform(input)
        } else {
            R::from(input)
        }
    }

    fn into_box(self) -> BoxTransformerOnce<T, R> {
        let pred = self.predicate;
        let transformer = self.transformer;
        BoxTransformerOnce::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_fn(self) -> impl FnOnce(T) -> R {
        let pred = self.predicate;
        let transformer = self.transformer;
        move |t: T| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        }
    }
}

impl<T, R> BoxConditionalTransformerOnce<T, R>
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
    ///   - `BoxTransformerOnce<T, R>`
    ///   - Any type implementing `TransformerOnce<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, BoxTransformerOnce};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10); // Condition satisfied, execute double
    /// assert_eq!(conditional.transform(-5), 5); // Condition not satisfied, execute negate
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxTransformerOnce<T, R>
    where
        F: TransformerOnce<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxTransformerOnce::new(move |t| {
            if pred.test(&t) {
                then_trans.transform(t)
            } else {
                else_transformer.transform(t)
            }
        })
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement TransformerOnce<T, R> for any type that implements
/// FnOnce(T) -> R
///
/// This allows once-callable closures and function pointers to be used
/// directly with our TransformerOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::TransformerOnce;
///
/// fn parse(s: String) -> i32 {
///     s.parse().unwrap_or(0)
/// }
///
/// assert_eq!(parse.transform("42".to_string()), 42);
///
/// let owned_value = String::from("hello");
/// let consume = |s: String| {
///     format!("{} world", s)
/// };
/// assert_eq!(consume.transform(owned_value), "hello world");
/// ```
///
/// # Author
///
/// Hu Haixing
impl<F, T, R> TransformerOnce<T, R> for F
where
    F: FnOnce(T) -> R,
    T: 'static,
    R: 'static,
{
    fn transform(self, input: T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxTransformerOnce::new(self)
    }

    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
    {
        move |input: T| -> R { self(input) }
    }
}
