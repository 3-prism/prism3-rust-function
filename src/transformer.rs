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
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new BoxTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    /// let composed = double.and_then(to_string);
    /// assert_eq!(composed.transform(21), "42");
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
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new BoxTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
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
    /// When the predicate returns false, the input value is returned unchanged.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, can be:
    ///   - Closure: `|x: &T| -> bool`
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `BoxPredicate<T>`, `RcPredicate<T>`, `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure
    ///
    /// ```rust
    /// use prism3_function::{Transformer, BoxTransformer};
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0);
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional.transform(-5), -5); // Unchanged
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalTransformer<T, R>
    where
        P: Predicate<T> + 'static,
        R: From<T>,
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
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{Transformer, BoxTransformer};
///
/// let double = BoxTransformer::new(|x: i32| x * 2);
/// let conditional = double.when(|x: &i32| *x > 0);
///
/// assert_eq!(conditional.transform(5), 10); // Executed
/// assert_eq!(conditional.transform(-5), -5); // Not executed
/// ```
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

impl<T, R> Transformer<T, R> for BoxConditionalTransformer<T, R>
where
    T: 'static,
    R: From<T> + 'static,
{
    fn transform(&self, input: T) -> R {
        if self.predicate.test(&input) {
            self.transformer.transform(input)
        } else {
            R::from(input)
        }
    }

    fn into_box(self) -> BoxTransformer<T, R> {
        let pred = self.predicate;
        let transformer = self.transformer;
        BoxTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_rc(self) -> RcTransformer<T, R> {
        let pred = self.predicate.into_rc();
        let transformer = self.transformer.into_rc();
        RcTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        panic!(
            "Cannot convert BoxConditionalTransformer to ArcTransformer: \
             predicate and transformer may not be Send + Sync"
        )
    }

    fn into_fn(self) -> impl Fn(T) -> R {
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
    /// * `after` - The transformer to apply after self (consumed)
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let to_string = ArcTransformer::new(|x: i32| x.to_string());
    /// let composed = double.and_then(to_string);
    ///
    /// // Original double transformer still usable
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(21), "42");
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
    /// * `before` - The transformer to apply before self (consumed)
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTransformer, Transformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let add_one = ArcTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    ///
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
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
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, must be `Send + Sync`, can be:
    ///   - Closure: `|x: &T| -> bool` (requires `Send + Sync`)
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Transformer, ArcTransformer};
    ///
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional_clone.transform(-5), -5);
    /// ```
    pub fn when<P>(self, predicate: P) -> ArcConditionalTransformer<T, R>
    where
        P: Predicate<T> + Send + Sync + 'static,
        R: From<T> + Send + Sync,
    {
        ArcConditionalTransformer {
            transformer: self,
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, R> ArcTransformer<T, R>
where
    T: 'static,
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
/// let conditional = ArcTransformer::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0);
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

impl<T, R> Transformer<T, R> for ArcConditionalTransformer<T, R>
where
    T: Send + 'static,
    R: From<T> + 'static,
{
    fn transform(&self, input: T) -> R {
        if self.predicate.test(&input) {
            self.transformer.transform(input)
        } else {
            R::from(input)
        }
    }

    fn into_box(self) -> BoxTransformer<T, R> {
        let pred = self.predicate;
        let transformer = self.transformer;
        BoxTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_rc(self) -> RcTransformer<T, R> {
        let pred = self.predicate.to_rc();
        let transformer = self.transformer.into_rc();
        RcTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        let pred = self.predicate;
        let transformer = self.transformer;
        ArcTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_fn(self) -> impl Fn(T) -> R {
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

impl<T, R> ArcConditionalTransformer<T, R>
where
    T: Send + 'static,
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
    /// * `after` - The transformer to apply after self (consumed)
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let to_string = RcTransformer::new(|x: i32| x.to_string());
    /// let composed = double.and_then(to_string);
    ///
    /// // Original double transformer still usable
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(21), "42");
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
    /// * `before` - The transformer to apply before self (consumed)
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTransformer, Transformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let add_one = RcTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    ///
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
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
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check, can be:
    ///   - Closure: `|x: &T| -> bool`
    ///   - Function pointer: `fn(&T) -> bool`
    ///   - `RcPredicate<T>`, `BoxPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `RcConditionalTransformer<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Transformer, RcTransformer};
    ///
    /// let double = RcTransformer::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.transform(5), 10);
    /// assert_eq!(conditional_clone.transform(-5), -5);
    /// ```
    pub fn when<P>(self, predicate: P) -> RcConditionalTransformer<T, R>
    where
        P: Predicate<T> + 'static,
        R: From<T>,
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
/// let conditional = RcTransformer::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0);
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

impl<T, R> Transformer<T, R> for RcConditionalTransformer<T, R>
where
    T: 'static,
    R: From<T> + 'static,
{
    fn transform(&self, input: T) -> R {
        if self.predicate.test(&input) {
            self.transformer.transform(input)
        } else {
            R::from(input)
        }
    }

    fn into_box(self) -> BoxTransformer<T, R> {
        let pred = self.predicate;
        let transformer = self.transformer;
        BoxTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_rc(self) -> RcTransformer<T, R> {
        let pred = self.predicate;
        let transformer = self.transformer;
        RcTransformer::new(move |t| {
            if pred.test(&t) {
                transformer.transform(t)
            } else {
                R::from(t)
            }
        })
    }

    fn into_arc(self) -> ArcTransformer<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        panic!("Cannot convert RcConditionalTransformer to ArcTransformer: not Send + Sync")
    }

    fn into_fn(self) -> impl Fn(T) -> R {
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
