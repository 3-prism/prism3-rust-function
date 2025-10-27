/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiTransformerOnce Types
//!
//! Provides Rust implementations of consuming bi-transformer traits similar to
//! Rust's `FnOnce` trait, but with value-oriented semantics for functional
//! programming patterns with two inputs.
//!
//! This module provides the `BiTransformerOnce<T, U, R>` trait and one-time use
//! implementations:
//!
//! - [`BoxBiTransformerOnce`]: Single ownership, one-time use
//!
//! # Author
//!
//! Hu Haixing

use crate::bi_predicate::{
    BiPredicate,
    BoxBiPredicate,
};

// ============================================================================
// Core Trait
// ============================================================================

/// BiTransformerOnce trait - consuming bi-transformation that takes ownership
///
/// Defines the behavior of a consuming bi-transformer: converting two values of
/// types `T` and `U` to a value of type `R` by taking ownership of self and
/// both inputs. This trait is analogous to `FnOnce(T, U) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the first input value (consumed)
/// * `U` - The type of the second input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait BiTransformerOnce<T, U, R> {
    /// Transforms two input values, consuming self and both inputs
    ///
    /// # Parameters
    ///
    /// * `first` - The first input value (consumed)
    /// * `second` - The second input value (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply_once(self, first: T, second: U) -> R;

    /// Converts to BoxBiTransformerOnce
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxBiTransformerOnce<T, U, R>`
    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(move |t: T, u: U| self.apply_once(t, u))
    }

    /// Converts bi-transformer to a closure
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T, U) -> R`
    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| self.apply_once(t, u)
    }

    /// Converts bi-transformer to a boxed function pointer
    ///
    /// **📌 Borrows `&self`**: The original bi-transformer remains usable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a boxed function pointer that implements `FnOnce(T, U) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiTransformerOnce;
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let func = add.to_fn_once();
    /// assert_eq!(func(20, 22), 42);
    /// ```
    fn to_box_once(&self) -> BoxBiTransformerOnce<T, U, R>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_box_once()
    }

    /// Converts bi-transformer to a closure
    ///
    /// **📌 Borrows `&self`**: The original bi-transformer remains usable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T, U) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiTransformerOnce;
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let func = add.to_fn_once();
    /// assert_eq!(func(20, 22), 42);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(T, U) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_fn_once()
    }
}

// ============================================================================
// BoxBiTransformerOnce - Box<dyn FnOnce(T, U) -> R>
// ============================================================================

/// BoxBiTransformerOnce - consuming bi-transformer wrapper based on
/// `Box<dyn FnOnce>`
///
/// A bi-transformer wrapper that provides single ownership with one-time use
/// semantics. Consumes self and both input values.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(T, U) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (consumes self and inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxBiTransformerOnce<T, U, R> {
    function: Box<dyn FnOnce(T, U) -> R>,
}

impl<T, U, R> BoxBiTransformerOnce<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Creates a new BoxBiTransformerOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxBiTransformerOnce, BiTransformerOnce};
    ///
    /// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply_once(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T, U) -> R + 'static,
    {
        BoxBiTransformerOnce {
            function: Box::new(f),
        }
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Consumes self and
    /// returns a new `BoxBiTransformerOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement TransformerOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxBiTransformerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformerOnce<R, S>`
    ///   - Any type implementing `TransformerOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiTransformerOnce<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};
    ///
    /// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let double = |x: i32| x * 2;
    ///
    /// // Both add and double are moved and consumed
    /// let composed = add.and_then(double);
    /// assert_eq!(composed.apply_once(3, 5), 16); // (3 + 5) * 2
    /// // add.apply_once(1, 2); // Would not compile - moved
    /// // double(10); // Would not compile - moved
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxBiTransformerOnce<T, U, S>
    where
        S: 'static,
        F: crate::transformer_once::TransformerOnce<R, S> + 'static,
    {
        let self_fn = self.function;
        BoxBiTransformerOnce::new(move |t: T, u: U| after.apply_once(self_fn(t, u)))
    }

    /// Creates a conditional bi-transformer
    ///
    /// Returns a bi-transformer that only executes when a bi-predicate is
    /// satisfied. You must call `or_else()` to provide an alternative
    /// bi-transformer.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`).
    ///   Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalBiTransformerOnce<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};
    ///
    /// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///     .or_else(multiply);
    /// assert_eq!(conditional.apply_once(5, 3), 8);
    ///
    /// let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let multiply2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    /// let conditional2 = add2.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///     .or_else(multiply2);
    /// assert_eq!(conditional2.apply_once(-5, 3), -15);
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce, RcBiPredicate};
    ///
    /// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(BoxBiTransformerOnce::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply_once(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalBiTransformerOnce<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
    {
        BoxConditionalBiTransformerOnce {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, U, R> BoxBiTransformerOnce<T, U, R>
where
    T: 'static,
    U: 'static,
    R: Clone + 'static,
{
    /// Creates a constant bi-transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxBiTransformerOnce, BiTransformerOnce};
    ///
    /// let constant = BoxBiTransformerOnce::constant("hello");
    /// assert_eq!(constant.apply_once(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> BoxBiTransformerOnce<T, U, R> {
        BoxBiTransformerOnce::new(move |_, _| value.clone())
    }
}

impl<T, U, R> BiTransformerOnce<T, U, R> for BoxBiTransformerOnce<T, U, R> {
    fn apply_once(self, first: T, second: U) -> R {
        (self.function)(first, second)
    }

    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| self.apply_once(t, u)
    }

    //  do NOT override BoxBiTransformerOnce::to_xxxx() because BoxBiTransformerOnce is not Clone
    //  and calling BoxBiTransformerOnce::to_xxxx() will cause a compile error
}

// ============================================================================
// BoxConditionalBiTransformerOnce - Box-based Conditional BiTransformer
// ============================================================================

/// BoxConditionalBiTransformerOnce struct
///
/// A conditional consuming bi-transformer that only executes when a bi-predicate
/// is satisfied. Uses `BoxBiTransformerOnce` and `BoxBiPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxBiTransformerOnce::when()` and
/// is designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **One-time Use**: Can only be called once
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};
///
/// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
/// let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
/// assert_eq!(conditional.apply_once(5, 3), 8); // when branch executed
///
/// let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
/// let multiply2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
/// let conditional2 = add2.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply2);
/// assert_eq!(conditional2.apply_once(-5, 3), -15); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalBiTransformerOnce<T, U, R> {
    transformer: BoxBiTransformerOnce<T, U, R>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U, R> BoxConditionalBiTransformerOnce<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Adds an else branch
    ///
    /// Executes the original bi-transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The bi-transformer for the else branch, can be:
    ///   - Closure: `|x: T, y: U| -> R`
    ///   - `BoxBiTransformerOnce<T, U, R>`
    ///   - Any type implementing `BiTransformerOnce<T, U, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiTransformerOnce<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};
    ///
    /// let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(|x: i32, y: i32| x * y);
    /// assert_eq!(conditional.apply_once(5, 3), 8); // Condition satisfied, execute add
    ///
    /// let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    /// let conditional2 = add2.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(|x: i32, y: i32| x * y);
    /// assert_eq!(conditional2.apply_once(-5, 3), -15); // Condition not satisfied, execute multiply
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxBiTransformerOnce<T, U, R>
    where
        F: BiTransformerOnce<T, U, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxBiTransformerOnce::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply_once(t, u)
            } else {
                else_transformer.apply_once(t, u)
            }
        })
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement BiTransformerOnce<T, U, R> for any type that implements
/// FnOnce(T, U) -> R
///
/// This allows once-callable closures and function pointers to be used
/// directly with our BiTransformerOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::BiTransformerOnce;
///
/// fn add(x: i32, y: i32) -> i32 {
///     x + y
/// }
///
/// assert_eq!(add.apply_once(20, 22), 42);
///
/// let owned_x = String::from("hello");
/// let owned_y = String::from("world");
/// let concat = |x: String, y: String| {
///     format!("{} {}", x, y)
/// };
/// assert_eq!(concat.apply_once(owned_x, owned_y), "hello world");
/// ```
///
/// # Author
///
/// Hu Haixing
impl<F, T, U, R> BiTransformerOnce<T, U, R> for F
where
    F: FnOnce(T, U) -> R,
    T: 'static,
    U: 'static,
    R: 'static,
{
    fn apply_once(self, first: T, second: U) -> R {
        self(first, second)
    }

    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        Self: Sized + 'static,
    {
        BoxBiTransformerOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        Self: Sized + 'static,
    {
        move |first: T, second: U| -> R { self(first, second) }
    }

    fn to_box_once(&self) -> BoxBiTransformerOnce<T, U, R>
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(self.clone())
    }

    fn to_fn_once(&self) -> impl FnOnce(T, U) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone()
    }
}

// ============================================================================
// FnBiTransformerOnceOps - Extension trait for FnOnce(T, U) -> R bi-transformers
// ============================================================================

/// Extension trait for closures implementing `FnOnce(T, U) -> R`
///
/// Provides composition methods (`and_then`, `when`) for one-time use
/// bi-transformer closures and function pointers without requiring explicit
/// wrapping in `BoxBiTransformerOnce`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnOnce(T, U) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `BiTransformerOnce<T, U, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then` and `when`. This extension trait provides those methods,
/// returning `BoxBiTransformerOnce` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps};
///
/// let add = |x: i32, y: i32| x + y;
/// let double = |x: i32| x * 2;
///
/// let composed = add.and_then(double);
/// assert_eq!(composed.apply_once(3, 5), 16); // (3 + 5) * 2
/// ```
///
/// ## Conditional execution with when
///
/// ```rust
/// use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps};
///
/// let add = |x: i32, y: i32| x + y;
/// let multiply = |x: i32, y: i32| x * y;
///
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
/// assert_eq!(conditional.apply_once(5, 3), 8); // add
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnBiTransformerOnceOps<T, U, R>: FnOnce(T, U) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Consumes self and
    /// returns a `BoxBiTransformerOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement TransformerOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` bi-transformer, the parameter will be consumed. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformerOnce<R, S>`
    ///   - Any type implementing `TransformerOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiTransformerOnce<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps,
    ///     BoxTransformerOnce};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved and consumed
    /// let composed = add.and_then(to_string);
    /// assert_eq!(composed.apply_once(20, 22), "42");
    /// // to_string.apply_once(10); // Would not compile - moved
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxBiTransformerOnce<T, U, S>
    where
        S: 'static,
        F: crate::transformer_once::TransformerOnce<R, S> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(move |t: T, u: U| after.apply_once(self(t, u)))
    }

    /// Creates a conditional bi-transformer
    ///
    /// Returns a bi-transformer that only executes when a bi-predicate is
    /// satisfied. You must call `or_else()` to provide an alternative
    /// bi-transformer for when the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`).
    ///   Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalBiTransformerOnce<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let multiply = |x: i32, y: i32| x * y;
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0)
    ///     .or_else(multiply);
    ///
    /// assert_eq!(conditional.apply_once(5, 3), 8);
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps,
    ///     RcBiPredicate};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply_once(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalBiTransformerOnce<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(self).when(predicate)
    }
}

/// Blanket implementation of FnBiTransformerOnceOps for all closures
///
/// Automatically implements `FnBiTransformerOnceOps<T, U, R>` for any type that
/// implements `FnOnce(T, U) -> R`.
///
/// # Author
///
/// Hu Haixing
impl<T, U, R, F> FnBiTransformerOnceOps<T, U, R> for F where F: FnOnce(T, U) -> R + 'static {}

// ============================================================================
// BinaryOperatorOnce Trait - Marker trait for BiTransformerOnce<T, T, T>
// ============================================================================

/// BinaryOperatorOnce trait - marker trait for one-time use binary operators
///
/// A one-time use binary operator takes two values of type `T` and produces a
/// value of the same type `T`, consuming self in the process. This trait
/// extends `BiTransformerOnce<T, T, T>` to provide semantic clarity for
/// same-type binary operations with consuming semantics. Equivalent to Java's
/// `BinaryOperator<T>` but with FnOnce semantics.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement
/// `BiTransformerOnce<T, T, T>`, so you don't need to implement it manually.
///
/// # Type Parameters
///
/// * `T` - The type of both input values and the output value
///
/// # Examples
///
/// ## Using in generic constraints
///
/// ```rust
/// use prism3_function::{BinaryOperatorOnce, BiTransformerOnce};
///
/// fn combine<T, O>(a: T, b: T, op: O) -> T
/// where
///     O: BinaryOperatorOnce<T>,
/// {
///     op.apply_once(a, b)
/// }
///
/// let multiply = |x: i32, y: i32| x * y;
/// assert_eq!(combine(6, 7, multiply), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait BinaryOperatorOnce<T>: BiTransformerOnce<T, T, T> {}

/// Blanket implementation of BinaryOperatorOnce for all BiTransformerOnce<T, T, T>
///
/// This automatically implements `BinaryOperatorOnce<T>` for any type that
/// implements `BiTransformerOnce<T, T, T>`.
///
/// # Author
///
/// Hu Haixing
impl<F, T> BinaryOperatorOnce<T> for F
where
    F: BiTransformerOnce<T, T, T>,
    T: 'static,
{
    // empty
}

// ============================================================================
// Type Aliases for BinaryOperatorOnce (BiTransformerOnce<T, T, T>)
// ============================================================================

/// Type alias for `BoxBiTransformerOnce<T, T, T>`
///
/// Represents a one-time use binary operator that takes two values of type `T`
/// and produces a value of the same type `T`. Equivalent to Java's
/// `BinaryOperator<T>` with consuming semantics (FnOnce).
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxBinaryOperatorOnce, BiTransformerOnce};
///
/// let add: BoxBinaryOperatorOnce<i32> = BoxBinaryOperatorOnce::new(|x, y| x + y);
/// assert_eq!(add.apply_once(20, 22), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type BoxBinaryOperatorOnce<T> = BoxBiTransformerOnce<T, T, T>;
