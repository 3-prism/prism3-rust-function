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
    fn apply(self, input: T) -> R;

    /// Converts to BoxTransformerOnce
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::TransformerOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let boxed = double.into_box();
    /// assert_eq!(boxed.apply(21), 42);
    /// ```
    fn into_box(self) -> BoxTransformerOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformerOnce::new(move |input: T| self.apply(input))
    }

    /// Converts transformer to a closure
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::TransformerOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let func = double.into_fn();
    /// assert_eq!(func(21), 42);
    /// ```
    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |input: T| self.apply(input)
    }

    /// Converts to BoxTransformerOnce without consuming self
    ///
    /// **📌 Borrows `&self`**: The original transformer remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a new `BoxTransformerOnce` that
    /// captures a clone. Types implementing `Clone` can override this method
    /// to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::TransformerOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let boxed = double.to_box();
    /// assert_eq!(boxed.apply(21), 42);
    /// ```
    fn to_box(&self) -> BoxTransformerOnce<T, R>
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Converts transformer to a closure without consuming self
    ///
    /// **📌 Borrows `&self`**: The original transformer remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a closure that captures a
    /// clone of `self` and calls its `transform` method. Types can
    /// override this method to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::TransformerOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let func = double.to_fn();
    /// assert_eq!(func(21), 42);
    /// ```
    fn to_fn(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn()
    }
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
    /// assert_eq!(parse.apply("42".to_string()), 42);
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
    /// assert_eq!(identity.apply(42), 42);
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
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxTransformerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformerOnce<R, S>`
    ///   - Any type implementing `TransformerOnce<R, S>`
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
    ///
    /// // Both add_one and double are moved and consumed
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// // add_one.apply(3); // Would not compile - moved
    /// // double.apply(4);  // Would not compile - moved
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxTransformerOnce<T, S>
    where
        S: 'static,
        G: TransformerOnce<R, S> + 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = (self.function)(x);
            after.apply(intermediate)
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
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxTransformerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformerOnce<S, T>`
    ///   - Any type implementing `TransformerOnce<S, T>`
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
    ///
    /// // Both double and add_one are moved and consumed
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// // double.apply(3); // Would not compile - moved
    /// // add_one.apply(4); // Would not compile - moved
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxTransformerOnce<S, R>
    where
        S: 'static,
        G: TransformerOnce<S, T> + 'static,
    {
        BoxTransformerOnce::new(move |x| {
            let intermediate = before.apply(x);
            (self.function)(intermediate)
        })
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
    /// Returns `BoxConditionalTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, BoxTransformerOnce};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let identity = BoxTransformerOnce::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    /// assert_eq!(conditional.apply(5), 10);
    ///
    /// let double2 = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let identity2 = BoxTransformerOnce::<i32, i32>::identity();
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(identity2);
    /// assert_eq!(conditional2.apply(-5), -5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, BoxTransformerOnce, RcPredicate};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(BoxTransformerOnce::identity());
    ///
    /// assert_eq!(conditional.apply(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalTransformerOnce<T, R>
    where
        P: Predicate<T> + 'static,
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
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxTransformerOnce<T, R> {
        BoxTransformerOnce::new(move |_| value.clone())
    }
}

impl<T, R> TransformerOnce<T, R> for BoxTransformerOnce<T, R> {
    fn apply(self, input: T) -> R {
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
        // Zero-cost: directly return the inner function
        self.function
    }

    // do NOT override BoxTransformer::to_box() and BoxTransformer::to_fn()
    // because BoxTransformer is not Clone and calling BoxTransformer::to_box()
    // or BoxTransformer::to_fn() will cause a compile error
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
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{TransformerOnce, BoxTransformerOnce};
///
/// let double = BoxTransformerOnce::new(|x: i32| x * 2);
/// let negate = BoxTransformerOnce::new(|x: i32| -x);
/// let conditional = double.when(|x: &i32| *x > 0).or_else(negate);
/// assert_eq!(conditional.apply(5), 10); // when branch executed
///
/// let double2 = BoxTransformerOnce::new(|x: i32| x * 2);
/// let negate2 = BoxTransformerOnce::new(|x: i32| -x);
/// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(negate2);
/// assert_eq!(conditional2.apply(-5), 5); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalTransformerOnce<T, R> {
    transformer: BoxTransformerOnce<T, R>,
    predicate: BoxPredicate<T>,
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
    /// assert_eq!(conditional.apply(5), 10); // Condition satisfied, execute double
    ///
    /// let double2 = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    /// assert_eq!(conditional2.apply(-5), 5); // Condition not satisfied, execute negate
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxTransformerOnce<T, R>
    where
        F: TransformerOnce<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxTransformerOnce::new(move |t| {
            if pred.test(&t) {
                then_trans.apply(t)
            } else {
                else_transformer.apply(t)
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
/// assert_eq!(parse.apply("42".to_string()), 42);
///
/// let owned_value = String::from("hello");
/// let consume = |s: String| {
///     format!("{} world", s)
/// };
/// assert_eq!(consume.apply(owned_value), "hello world");
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
    fn apply(self, input: T) -> R {
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
        // Zero-cost: directly return self since F is already FnOnce(T) -> R
        self
    }

    fn to_box(&self) -> BoxTransformerOnce<T, R>
    where
        Self: Clone + Sized + 'static,
    {
        self.clone().into_box()
    }

    fn to_fn(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + Sized + 'static,
    {
        self.clone()
    }
}

// ============================================================================
// FnTransformerOnceOps - Extension trait for FnOnce transformers
// ============================================================================

/// Extension trait for closures implementing `FnOnce(T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for one-time
/// use closures and function pointers without requiring explicit wrapping in
/// `BoxTransformerOnce`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnOnce(T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `TransformerOnce<T, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides those
/// methods, returning `BoxTransformerOnce` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{TransformerOnce, FnTransformerOnceOps};
///
/// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
/// let double = |x: i32| x * 2;
///
/// let composed = parse.and_then(double);
/// assert_eq!(composed.apply("21".to_string()), 42);
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{TransformerOnce, FnTransformerOnceOps};
///
/// let double = |x: i32| x * 2;
/// let to_string = |x: i32| x.to_string();
///
/// let composed = to_string.compose(double);
/// assert_eq!(composed.apply(21), "42");
/// ```
///
/// ## Conditional transformation with when
///
/// ```rust
/// use prism3_function::{TransformerOnce, FnTransformerOnceOps};
///
/// let double = |x: i32| x * 2;
/// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
///
/// assert_eq!(conditional.apply(5), 10);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnTransformerOnceOps<T, R>: FnOnce(T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Consumes self and returns
    /// a `BoxTransformerOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `G` - The type of the after transformer (must implement
    ///   TransformerOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` transformer, the parameter will be consumed. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformerOnce<R, S>`
    ///   - Any type implementing `TransformerOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxTransformerOnce<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, FnTransformerOnceOps,
    ///     BoxTransformerOnce};
    ///
    /// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    ///
    /// // double is moved and consumed
    /// let composed = parse.and_then(double);
    /// assert_eq!(composed.apply("21".to_string()), 42);
    /// // double.apply(5); // Would not compile - moved
    /// ```
    fn and_then<S, G>(self, after: G) -> BoxTransformerOnce<T, S>
    where
        S: 'static,
        G: TransformerOnce<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformerOnce::new(move |x: T| {
            let intermediate = self(x);
            after.apply(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self and returns
    /// a `BoxTransformerOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `G` - The type of the before transformer (must implement
    ///   TransformerOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` transformer, the parameter will be consumed. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformerOnce<S, T>`
    ///   - Any type implementing `TransformerOnce<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxTransformerOnce<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, FnTransformerOnceOps,
    ///     BoxTransformerOnce};
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let to_string = |x: i32| x.to_string();
    ///
    /// // double is moved and consumed
    /// let composed = to_string.compose(double);
    /// assert_eq!(composed.apply(21), "42");
    /// // double.apply(5); // Would not compile - moved
    /// ```
    fn compose<S, G>(self, before: G) -> BoxTransformerOnce<S, R>
    where
        S: 'static,
        G: TransformerOnce<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformerOnce::new(move |x: S| {
            let intermediate = before.apply(x);
            self(intermediate)
        })
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
    /// Returns `BoxConditionalTransformerOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, FnTransformerOnceOps};
    ///
    /// let double = |x: i32| x * 2;
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply(5), 10);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{TransformerOnce, FnTransformerOnceOps,
    ///     RcPredicate};
    ///
    /// let double = |x: i32| x * 2;
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalTransformerOnce<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformerOnce::new(self).when(predicate)
    }
}

/// Blanket implementation of FnTransformerOnceOps for all FnOnce closures
///
/// Automatically implements `FnTransformerOnceOps<T, R>` for any type that
/// implements `FnOnce(T) -> R`.
///
/// # Author
///
/// Hu Haixing
impl<T, R, F> FnTransformerOnceOps<T, R> for F where F: FnOnce(T) -> R + 'static {}

// ============================================================================
// UnaryOperatorOnce Trait - Marker trait for TransformerOnce<T, T>
// ============================================================================

/// UnaryOperatorOnce trait - marker trait for one-time use unary operators
///
/// A one-time use unary operator transforms a value of type `T` to another
/// value of the same type `T`, consuming self in the process. This trait
/// extends `TransformerOnce<T, T>` to provide semantic clarity for same-type
/// transformations with consuming semantics. Equivalent to Java's
/// `UnaryOperator<T>` but with FnOnce semantics.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement
/// `TransformerOnce<T, T>`, so you don't need to implement it manually.
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Examples
///
/// ## Using in generic constraints
///
/// ```rust
/// use prism3_function::{UnaryOperatorOnce, TransformerOnce};
///
/// fn apply_once<T, O>(value: T, op: O) -> T
/// where
///     O: UnaryOperatorOnce<T>,
/// {
///     op.apply(value)
/// }
///
/// let double = |x: i32| x * 2;
/// assert_eq!(apply_once(21, double), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait UnaryOperatorOnce<T>: TransformerOnce<T, T> {}

/// Blanket implementation of UnaryOperatorOnce for all TransformerOnce<T, T>
///
/// This automatically implements `UnaryOperatorOnce<T>` for any type that
/// implements `TransformerOnce<T, T>`.
///
/// # Author
///
/// Hu Haixing
impl<F, T> UnaryOperatorOnce<T> for F
where
    F: TransformerOnce<T, T>,
    T: 'static,
{
    // empty
}

// ============================================================================
// Type Aliases for UnaryOperatorOnce (TransformerOnce<T, T>)
// ============================================================================

/// Type alias for `BoxTransformerOnce<T, T>`
///
/// Represents a one-time use unary operator that transforms a value of type `T`
/// to another value of the same type `T`. Equivalent to Java's `UnaryOperator<T>`
/// with consuming semantics (FnOnce).
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxUnaryOperatorOnce, TransformerOnce};
///
/// let increment: BoxUnaryOperatorOnce<i32> = BoxUnaryOperatorOnce::new(|x| x + 1);
/// assert_eq!(increment.apply(41), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type BoxUnaryOperatorOnce<T> = BoxTransformerOnce<T, T>;
