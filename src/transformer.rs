/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Transformer Types
//!
//! Provides Rust implementations of transformer traits for type conversion
//! and value transformation. Transformers consume input values (taking
//! ownership) and produce output values.
//!
//! This module provides the `Transformer<T, R>` trait and three
//! implementations:
//!
//! - [`BoxTransformer`]: Single ownership, not cloneable
//! - [`ArcTransformer`]: Thread-safe shared ownership, cloneable
//! - [`RcTransformer`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

use crate::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

// ============================================================================
// Core Trait
// ============================================================================

/// Transformer trait - transforms values from type T to type R
///
/// Defines the behavior of a transformation: converting a value of type `T`
/// to a value of type `R` by consuming the input. This is analogous to
/// `Fn(T) -> R` in Rust's standard library.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait Transformer<T, R> {
    /// Transforms the input value to produce an output value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value to transform (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(&self, input: T) -> R;

    /// Converts to BoxTransformer
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformer<T, R>`
    fn into_box(self) -> BoxTransformer<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to RcTransformer
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcTransformer<T, R>`
    fn into_rc(self) -> RcTransformer<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to ArcTransformer
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcTransformer<T, R>`
    fn into_arc(self) -> ArcTransformer<T, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static;

    /// Converts transformer to a closure
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `Fn(T) -> R`
    fn into_fn(self) -> impl Fn(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
}

// ============================================================================
// BoxTransformer - Box<dyn Fn(T) -> R>
// ============================================================================

/// BoxTransformer - transformer wrapper based on `Box<dyn Fn>`
///
/// A transformer wrapper that provides single ownership with reusable
/// transformation. The transformer consumes the input and can be called
/// multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn Fn(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes its
///   input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxTransformer<T, R> {
    function: Box<dyn Fn(T) -> R>,
}

impl<T, R> BoxTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        BoxTransformer {
            function: Box::new(f),
        }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let identity = BoxTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> BoxTransformer<T, T> {
        BoxTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement
    ///   Transformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformer<R, S>`
    ///   - An `RcTransformer<R, S>`
    ///   - An `ArcTransformer<R, S>`
    ///   - Any type implementing `Transformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved here
    /// let composed = double.and_then(to_string);
    /// assert_eq!(composed.transform(21), "42");
    /// // to_string.transform(5); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // Clone to preserve original
    /// let composed = double.and_then(to_string.clone());
    /// assert_eq!(composed.transform(21), "42");
    ///
    /// // Original still usable
    /// assert_eq!(to_string.transform(5), "5");
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxTransformer<T, S>
    where
        S: 'static,
        F: Transformer<R, S> + 'static,
    {
        let self_fn = self.function;
        BoxTransformer::new(move |x: T| after.transform(self_fn(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `F` - The type of the before transformer (must implement
    ///   Transformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformer<S, T>`
    ///   - An `RcTransformer<S, T>`
    ///   - An `ArcTransformer<S, T>`
    ///   - Any type implementing `Transformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    ///
    /// // add_one is moved here
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// // add_one.transform(3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    ///
    /// // Clone to preserve original
    /// let composed = double.compose(add_one.clone());
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    ///
    /// // Original still usable
    /// assert_eq!(add_one.transform(3), 4);
    /// ```
    pub fn compose<S, F>(self, before: F) -> BoxTransformer<S, R>
    where
        S: 'static,
        F: Transformer<S, T> + 'static,
    {
        let self_fn = self.function;
        BoxTransformer::new(move |x: S| self_fn(before.transform(x)))
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
    /// Returns `BoxConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{Transformer, BoxTransformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let identity = BoxTransformer::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), -5); // identity
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, BoxTransformer, BoxPredicate};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(BoxTransformer::identity());
    ///
    /// assert_eq!(conditional.transform(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalTransformer<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalTransformer {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxTransformer<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let constant = BoxTransformer::constant("hello");
    /// assert_eq!(constant.transform(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxTransformer<T, R> {
        BoxTransformer::new(move |_| value.clone())
    }
}

impl<T, R> Transformer<T, R> for BoxTransformer<T, R> {
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcTransformer {
            function: Rc::from(self.function),
        }
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "BoxTransformer<T, R> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl Fn(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| self.transform(t)
    }
}

// ============================================================================
// BoxConditionalTransformer - Box-based Conditional Transformer
// ============================================================================

/// BoxConditionalTransformer struct
///
/// A conditional transformer that only executes when a predicate is satisfied.
/// Uses `BoxTransformer` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only transforms when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Transformer**: Can be used anywhere a `Transformer` is expected
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{Transformer, BoxTransformer};
///
/// let double = BoxTransformer::new(|x: i32| x * 2);
/// let negate = BoxTransformer::new(|x: i32| -x);
/// let conditional = double.when(|x: &i32| *x > 0).or_else(negate);
///
/// assert_eq!(conditional.transform(5), 10); // when branch executed
/// assert_eq!(conditional.transform(-5), 5); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalTransformer<T, R> {
    transformer: BoxTransformer<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalTransformer<T, R>
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
    ///   - `BoxTransformer<T, R>`, `RcTransformer<T, R>`, `ArcTransformer<T, R>`
    ///   - Any type implementing `Transformer<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Transformer, BoxTransformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10); // Condition satisfied, execute double
    /// assert_eq!(conditional.transform(-5), 5); // Condition not satisfied, execute negate
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxTransformer<T, R>
    where
        F: Transformer<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxTransformer::new(move |t| {
            if pred.test(&t) {
                then_trans.transform(t)
            } else {
                else_transformer.transform(t)
            }
        })
    }
}

// ============================================================================
// ArcTransformer - Arc<dyn Fn(T) -> R + Send + Sync>
// ============================================================================

/// ArcTransformer - thread-safe transformer wrapper
///
/// A thread-safe, clonable transformer wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(T) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes its
///   input)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct ArcTransformer<T, R> {
    function: Arc<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> ArcTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: 'static,
{
    /// Creates a new ArcTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        ArcTransformer {
            function: Arc::new(f),
        }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let identity = ArcTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> ArcTransformer<T, T> {
        ArcTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement
    ///   Transformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformer<R, S>`
    ///   - An `RcTransformer<R, S>`
    ///   - An `ArcTransformer<R, S>` (will be moved)
    ///   - Any type implementing `Transformer<R, S> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let to_string = ArcTransformer::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved here
    /// let composed = double.and_then(to_string);
    ///
    /// // Original double transformer still usable (uses &self)
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(21), "42");
    /// // to_string.transform(5); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let to_string = ArcTransformer::new(|x: i32| x.to_string());
    ///
    /// // Clone to preserve original
    /// let composed = double.and_then(to_string.clone());
    /// assert_eq!(composed.transform(21), "42");
    ///
    /// // Both originals still usable
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(to_string.transform(5), "5");
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> ArcTransformer<T, S>
    where
        S: Send + Sync + 'static,
        F: Transformer<R, S> + Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.function);
        ArcTransformer {
            function: Arc::new(move |x: T| after.transform(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `F` - The type of the before transformer (must implement
    ///   Transformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformer<S, T>`
    ///   - An `RcTransformer<S, T>`
    ///   - An `ArcTransformer<S, T>` (will be moved)
    ///   - Any type implementing `Transformer<S, T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let add_one = ArcTransformer::new(|x: i32| x + 1);
    ///
    /// // add_one is moved here
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// // add_one.transform(3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let add_one = ArcTransformer::new(|x: i32| x + 1);
    ///
    /// // Clone to preserve original
    /// let composed = double.compose(add_one.clone());
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    ///
    /// // Both originals still usable
    /// assert_eq!(double.transform(10), 20);
    /// assert_eq!(add_one.transform(3), 4);
    /// ```
    pub fn compose<S, F>(&self, before: F) -> ArcTransformer<S, R>
    where
        S: Send + Sync + 'static,
        F: Transformer<S, T> + Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.function);
        ArcTransformer {
            function: Arc::new(move |x: S| self_clone(before.transform(x))),
        }
    }

    /// Creates a conditional transformer (thread-safe version)
    ///
    /// Returns a transformer that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative transformer.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Must be
    ///   `Send + Sync`, can be:
    ///   - A closure: `|x: &T| -> bool` (requires `Send + Sync`)
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{Transformer, ArcTransformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let identity = ArcTransformer::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional_clone.transform(-5), -5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, ArcTransformer, ArcPredicate};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(ArcTransformer::identity());
    ///
    /// assert_eq!(conditional.transform(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> ArcConditionalTransformer<T, R>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        ArcConditionalTransformer {
            transformer: self,
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, R> ArcTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let constant = ArcTransformer::constant("hello");
    /// assert_eq!(constant.transform(123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcTransformer<T, R>
    where
        R: Send + Sync,
    {
        ArcTransformer::new(move |_| value.clone())
    }
}

impl<T, R> Transformer<T, R> for ArcTransformer<T, R> {
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxTransformer {
            function: Box::new(move |x| self.transform(x)),
        }
    }

    fn into_rc(self) -> RcTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcTransformer {
            function: Rc::new(move |x| self.transform(x)),
        }
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl Fn(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| self.transform(t)
    }
}

impl<T, R> Clone for ArcTransformer<T, R> {
    fn clone(&self) -> Self {
        ArcTransformer {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// ArcConditionalTransformer - Arc-based Conditional Transformer
// ============================================================================

/// ArcConditionalTransformer struct
///
/// A thread-safe conditional transformer that only executes when a predicate is
/// satisfied. Uses `ArcTransformer` and `ArcPredicate` for shared ownership
/// across threads.
///
/// This type is typically created by calling `ArcTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Conditional Execution**: Only transforms when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Transformer, ArcTransformer};
///
/// let double = ArcTransformer::new(|x: i32| x * 2);
/// let identity = ArcTransformer::<i32, i32>::identity();
/// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
///
/// let conditional_clone = conditional.clone();
///
/// assert_eq!(conditional.transform(5), 10);
/// assert_eq!(conditional_clone.transform(-5), -5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalTransformer<T, R> {
    transformer: ArcTransformer<T, R>,
    predicate: ArcPredicate<T>,
}

impl<T, R> ArcConditionalTransformer<T, R>
where
    T: Send + Sync + 'static,
    R: 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The transformer for the else branch, can be:
    ///   - Closure: `|x: T| -> R` (must be `Send + Sync`)
    ///   - `ArcTransformer<T, R>`, `BoxTransformer<T, R>`
    ///   - Any type implementing `Transformer<T, R> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Transformer, ArcTransformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> ArcTransformer<T, R>
    where
        F: Transformer<T, R> + Send + Sync + 'static,
        R: Send + Sync,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        ArcTransformer::new(move |t| {
            if pred.test(&t) {
                then_trans.transform(t)
            } else {
                else_transformer.transform(t)
            }
        })
    }
}

impl<T, R> Clone for ArcConditionalTransformer<T, R> {
    /// Clones the conditional transformer
    ///
    /// Creates a new instance that shares the underlying transformer and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            transformer: self.transformer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcTransformer - Rc<dyn Fn(T) -> R>
// ============================================================================

/// RcTransformer - single-threaded transformer wrapper
///
/// A single-threaded, clonable transformer wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(T) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (each call consumes its
///   input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct RcTransformer<T, R> {
    function: Rc<dyn Fn(T) -> R>,
}

impl<T, R> RcTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        RcTransformer {
            function: Rc::new(f),
        }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let identity = RcTransformer::<i32, i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> RcTransformer<T, T> {
        RcTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement
    ///   Transformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformer<R, S>`
    ///   - An `RcTransformer<R, S>` (will be moved)
    ///   - An `ArcTransformer<R, S>`
    ///   - Any type implementing `Transformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let to_string = RcTransformer::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved here
    /// let composed = double.and_then(to_string);
    ///
    /// // Original double transformer still usable (uses &self)
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(21), "42");
    /// // to_string.transform(5); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let to_string = RcTransformer::new(|x: i32| x.to_string());
    ///
    /// // Clone to preserve original
    /// let composed = double.and_then(to_string.clone());
    /// assert_eq!(composed.transform(21), "42");
    ///
    /// // Both originals still usable
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(to_string.transform(5), "5");
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> RcTransformer<T, S>
    where
        S: 'static,
        F: Transformer<R, S> + 'static,
    {
        let self_clone = Rc::clone(&self.function);
        RcTransformer {
            function: Rc::new(move |x: T| after.transform(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `F` - The type of the before transformer (must implement
    ///   Transformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformer<S, T>`
    ///   - An `RcTransformer<S, T>` (will be moved)
    ///   - An `ArcTransformer<S, T>`
    ///   - Any type implementing `Transformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let add_one = RcTransformer::new(|x: i32| x + 1);
    ///
    /// // add_one is moved here
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// // add_one.transform(3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let add_one = RcTransformer::new(|x: i32| x + 1);
    ///
    /// // Clone to preserve original
    /// let composed = double.compose(add_one.clone());
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    ///
    /// // Both originals still usable
    /// assert_eq!(double.transform(10), 20);
    /// assert_eq!(add_one.transform(3), 4);
    /// ```
    pub fn compose<S, F>(&self, before: F) -> RcTransformer<S, R>
    where
        S: 'static,
        F: Transformer<S, T> + 'static,
    {
        let self_clone = Rc::clone(&self.function);
        RcTransformer {
            function: Rc::new(move |x: S| self_clone(before.transform(x))),
        }
    }

    /// Creates a conditional transformer (single-threaded shared version)
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
    /// Returns `RcConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{Transformer, RcTransformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let identity = RcTransformer::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional_clone.transform(-5), -5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, RcTransformer, RcPredicate};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(RcTransformer::identity());
    ///
    /// assert_eq!(conditional.transform(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> RcConditionalTransformer<T, R>
    where
        P: Predicate<T> + 'static,
    {
        RcConditionalTransformer {
            transformer: self,
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, R> RcTransformer<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let constant = RcTransformer::constant("hello");
    /// assert_eq!(constant.transform(123), "hello");
    /// ```
    pub fn constant(value: R) -> RcTransformer<T, R> {
        RcTransformer::new(move |_| value.clone())
    }
}

impl<T, R> Transformer<T, R> for RcTransformer<T, R> {
    fn transform(&self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxTransformer {
            function: Box::new(move |x| self.transform(x)),
        }
    }

    fn into_rc(self) -> RcTransformer<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "RcTransformer cannot be converted to ArcTransformer because Rc \
             is not Send + Sync"
        )
    }

    fn into_fn(self) -> impl Fn(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| self.transform(t)
    }
}

impl<T, R> Clone for RcTransformer<T, R> {
    fn clone(&self) -> Self {
        RcTransformer {
            function: Rc::clone(&self.function),
        }
    }
}

// ============================================================================
// RcConditionalTransformer - Rc-based Conditional Transformer
// ============================================================================

/// RcConditionalTransformer struct
///
/// A single-threaded conditional transformer that only executes when a
/// predicate is satisfied. Uses `RcTransformer` and `RcPredicate` for shared
/// ownership within a single thread.
///
/// This type is typically created by calling `RcTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only transforms when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalTransformer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Transformer, RcTransformer};
///
/// let double = RcTransformer::new(|x: i32| x * 2);
/// let identity = RcTransformer::<i32, i32>::identity();
/// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
///
/// let conditional_clone = conditional.clone();
///
/// assert_eq!(conditional.transform(5), 10);
/// assert_eq!(conditional_clone.transform(-5), -5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalTransformer<T, R> {
    transformer: RcTransformer<T, R>,
    predicate: RcPredicate<T>,
}

impl<T, R> RcConditionalTransformer<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The transformer for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `RcTransformer<T, R>`, `BoxTransformer<T, R>`
    ///   - Any type implementing `Transformer<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{Transformer, RcTransformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> RcTransformer<T, R>
    where
        F: Transformer<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        RcTransformer::new(move |t| {
            if pred.test(&t) {
                then_trans.transform(t)
            } else {
                else_transformer.transform(t)
            }
        })
    }
}

impl<T, R> Clone for RcConditionalTransformer<T, R> {
    /// Clones the conditional transformer
    ///
    /// Creates a new instance that shares the underlying transformer and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            transformer: self.transformer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// Blanket implementation for standard Fn trait
// ============================================================================

/// Implement Transformer<T, R> for any type that implements Fn(T) -> R
///
/// This allows closures and function pointers to be used directly with our
/// Transformer trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::Transformer;
///
/// fn double(x: i32) -> i32 { x * 2 }
///
/// assert_eq!(double.transform(21), 42);
///
/// let triple = |x: i32| x * 3;
/// assert_eq!(triple.transform(14), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
impl<F, T, R> Transformer<T, R> for F
where
    F: Fn(T) -> R,
    T: 'static,
    R: 'static,
{
    fn transform(&self, input: T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxTransformer<T, R>
    where
        Self: Sized + 'static,
    {
        BoxTransformer::new(self)
    }

    fn into_rc(self) -> RcTransformer<T, R>
    where
        Self: Sized + 'static,
    {
        RcTransformer::new(self)
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcTransformer::new(self)
    }

    fn into_fn(self) -> impl Fn(T) -> R
    where
        Self: Sized + 'static,
    {
        move |t: T| self(t)
    }
}

// ============================================================================
// FnTransformerOps - Extension trait for closure transformers
// ============================================================================

/// Extension trait for closures implementing `Fn(T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for closures
/// and function pointers without requiring explicit wrapping in
/// `BoxTransformer`, `RcTransformer`, or `ArcTransformer`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `Fn(T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `Transformer<T, R>` through blanket
/// implementation, they don't have access to instance methods like `and_then`,
/// `compose`, and `when`. This extension trait provides those methods,
/// returning `BoxTransformer` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{Transformer, FnTransformerOps};
///
/// let double = |x: i32| x * 2;
/// let to_string = |x: i32| x.to_string();
///
/// let composed = double.and_then(to_string);
/// assert_eq!(composed.transform(21), "42");
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{Transformer, FnTransformerOps};
///
/// let double = |x: i32| x * 2;
/// let add_one = |x: i32| x + 1;
///
/// let composed = double.compose(add_one);
/// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
/// ```
///
/// ## Conditional transformation with when
///
/// ```rust
/// use prism3_function::{Transformer, FnTransformerOps};
///
/// let double = |x: i32| x * 2;
/// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
///
/// assert_eq!(conditional.transform(5), 10);
/// assert_eq!(conditional.transform(-5), 5);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnTransformerOps<T, R>: Fn(T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Consumes self and returns
    /// a `BoxTransformer`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after transformer
    /// * `F` - The type of the after transformer (must implement Transformer<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformer<R, S>`
    ///   - An `RcTransformer<R, S>`
    ///   - An `ArcTransformer<R, S>`
    ///   - Any type implementing `Transformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxTransformer<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps, BoxTransformer};
    ///
    /// let double = |x: i32| x * 2;
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved here
    /// let composed = double.and_then(to_string);
    /// assert_eq!(composed.transform(21), "42");
    /// // to_string.transform(5); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps, BoxTransformer};
    ///
    /// let double = |x: i32| x * 2;
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // Clone to preserve original
    /// let composed = double.and_then(to_string.clone());
    /// assert_eq!(composed.transform(21), "42");
    ///
    /// // Original still usable
    /// assert_eq!(to_string.transform(5), "5");
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxTransformer<T, S>
    where
        S: 'static,
        F: Transformer<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformer::new(move |x: T| after.transform(self(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self and returns
    /// a `BoxTransformer`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before transformer
    /// * `F` - The type of the before transformer (must implement Transformer<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original transformer, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxTransformer<S, T>`
    ///   - An `RcTransformer<S, T>`
    ///   - An `ArcTransformer<S, T>`
    ///   - Any type implementing `Transformer<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxTransformer<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps, BoxTransformer};
    ///
    /// let double = |x: i32| x * 2;
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    ///
    /// // add_one is moved here
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// // add_one.transform(3); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps, BoxTransformer};
    ///
    /// let double = |x: i32| x * 2;
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    ///
    /// // Clone to preserve original
    /// let composed = double.compose(add_one.clone());
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    ///
    /// // Original still usable
    /// assert_eq!(add_one.transform(3), 4);
    /// ```
    fn compose<S, F>(self, before: F) -> BoxTransformer<S, R>
    where
        S: 'static,
        F: Transformer<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformer::new(move |x: S| self(before.transform(x)))
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
    /// Returns `BoxConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps};
    ///
    /// let double = |x: i32| x * 2;
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), 5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{Transformer, FnTransformerOps, BoxPredicate};
    ///
    /// let double = |x: i32| x * 2;
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalTransformer<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxTransformer::new(self).when(predicate)
    }
}

/// Blanket implementation of FnTransformerOps for all closures
///
/// Automatically implements `FnTransformerOps<T, R>` for any type that
/// implements `Fn(T) -> R`.
///
/// # Author
///
/// Hu Haixing
impl<T, R, F> FnTransformerOps<T, R> for F where F: Fn(T) -> R + 'static {}

// ============================================================================
// UnaryOperator Trait - Marker trait for Transformer<T, T>
// ============================================================================

/// UnaryOperator trait - marker trait for unary operators
///
/// A unary operator transforms a value of type `T` to another value of the
/// same type `T`. This trait extends `Transformer<T, T>` to provide semantic
/// clarity for same-type transformations. Equivalent to Java's `UnaryOperator<T>`
/// which extends `Function<T, T>`.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement
/// `Transformer<T, T>`, so you don't need to implement it manually.
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
/// use prism3_function::{UnaryOperator, Transformer};
///
/// fn apply_twice<T, O>(value: T, op: O) -> T
/// where
///     O: UnaryOperator<T>,
///     T: Clone,
/// {
///     let result = op.transform(value.clone());
///     op.transform(result)
/// }
///
/// let increment = |x: i32| x + 1;
/// assert_eq!(apply_twice(5, increment), 7); // (5 + 1) + 1
/// ```
///
/// ## With concrete types
///
/// ```rust
/// use prism3_function::{BoxUnaryOperator, UnaryOperator, Transformer};
///
/// fn create_incrementer() -> BoxUnaryOperator<i32> {
///     BoxUnaryOperator::new(|x| x + 1)
/// }
///
/// let op = create_incrementer();
/// assert_eq!(op.transform(41), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait UnaryOperator<T>: Transformer<T, T> {}

/// Blanket implementation of UnaryOperator for all Transformer<T, T>
///
/// This automatically implements `UnaryOperator<T>` for any type that
/// implements `Transformer<T, T>`.
///
/// # Author
///
/// Hu Haixing
impl<F, T> UnaryOperator<T> for F
where
    F: Transformer<T, T>,
    T: 'static,
{
    // empty
}

// ============================================================================
// Type Aliases for UnaryOperator (Transformer<T, T>)
// ============================================================================

/// Type alias for `BoxTransformer<T, T>`
///
/// Represents a unary operator that transforms a value of type `T` to another
/// value of the same type `T`, with single ownership semantics. Equivalent to
/// Java's `UnaryOperator<T>`.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxUnaryOperator, Transformer};
///
/// let increment: BoxUnaryOperator<i32> = BoxUnaryOperator::new(|x| x + 1);
/// assert_eq!(increment.transform(41), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type BoxUnaryOperator<T> = BoxTransformer<T, T>;

/// Type alias for `ArcTransformer<T, T>`
///
/// Represents a thread-safe unary operator that transforms a value of type `T`
/// to another value of the same type `T`. Equivalent to Java's `UnaryOperator<T>`
/// with shared, thread-safe ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ArcUnaryOperator, Transformer};
///
/// let double: ArcUnaryOperator<i32> = ArcUnaryOperator::new(|x| x * 2);
/// let double_clone = double.clone();
/// assert_eq!(double.transform(21), 42);
/// assert_eq!(double_clone.transform(21), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type ArcUnaryOperator<T> = ArcTransformer<T, T>;

/// Type alias for `RcTransformer<T, T>`
///
/// Represents a single-threaded unary operator that transforms a value of type
/// `T` to another value of the same type `T`. Equivalent to Java's
/// `UnaryOperator<T>` with shared, single-threaded ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{RcUnaryOperator, Transformer};
///
/// let negate: RcUnaryOperator<i32> = RcUnaryOperator::new(|x: i32| -x);
/// let negate_clone = negate.clone();
/// assert_eq!(negate.transform(42), -42);
/// assert_eq!(negate_clone.transform(42), -42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type RcUnaryOperator<T> = RcTransformer<T, T>;
