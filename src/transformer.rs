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
//! Provides transformer types for transforming values from type T to the same
//! type T. This is a specialization of `Function<T, T>` with additional
//! convenience methods.
//!
//! This module provides a `Transformer<T>` trait as a unified interface, along
//! with four concrete implementations:
//!
//! - [`BoxTransformer`]: Based on `Box<dyn FnOnce>`, single ownership,
//!   one-time use
//! - [`BoxFnTransformer`]: Based on `Box<dyn Fn>`, reusable, single ownership
//! - [`ArcFnTransformer`]: Based on `Arc<dyn Fn>`, reusable, multi-threaded
//!   sharing
//! - [`RcFnTransformer`]: Based on `Rc<dyn Fn>`, reusable, single-threaded
//!   sharing
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Transformer Trait - Unified Interface
// ============================================================================

/// Transformer trait - unified transformation interface
///
/// Defines the core behavior of transformation: converting a value of type `T`
/// to another value of the same type `T`.
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Implementors
///
/// - All closure types `FnOnce(T) -> T`
/// - [`BoxTransformer<T>`]
/// - [`BoxFnTransformer<T>`]
/// - [`ArcFnTransformer<T>`]
/// - [`RcFnTransformer<T>`]
///
/// # Author
///
/// Hu Haixing
pub trait Transformer<T> {
    /// Applies the transformation to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(self, input: T) -> T;

    /// Converts transformer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the transformer and returns a closure that can be
    /// directly used with iterator methods like `map()`, `flat_map()`, etc.
    /// This provides a more ergonomic API when working with iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the transformer (takes ownership of `self`).
    /// After calling this method, the original transformer is no longer
    /// available. The returned closure captures the transformer by move.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(T) -> T`
    ///
    /// # Examples
    ///
    /// ## Simple Mapping
    ///
    /// ```rust
    /// use prism3_function::{BoxFnTransformer, Transformer};
    ///
    /// let transformer = BoxFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## With Complex Transformer
    ///
    /// ```rust
    /// use prism3_function::{BoxFnTransformer, Transformer};
    ///
    /// let add_one = BoxFnTransformer::new(|x: i32| x + 1);
    /// let double = BoxFnTransformer::new(|x: i32| x * 2);
    /// let transformer = add_one.then(double);
    ///
    /// let values = vec![1, 2, 3];
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![4, 6, 8]); // (1+1)*2=4, (2+1)*2=6, (3+1)*2=8
    /// ```
    ///
    /// ## With Other Iterator Methods
    ///
    /// ```rust
    /// use prism3_function::{ArcFnTransformer, Transformer};
    ///
    /// let transformer = ArcFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .filter(|x| *x > 4)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![6, 8, 10]);
    /// ```
    ///
    /// ## Ownership Behavior
    ///
    /// ```rust,compile_fail
    /// use prism3_function::{BoxTransformer, Transformer};
    ///
    /// let transformer = BoxTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// // ❌ Error: transformer was moved in the call to into_fn()
    /// let result2 = transformer.transform(5);
    /// ```
    fn into_fn(self) -> impl FnMut(T) -> T
    where
        Self: Sized + 'static,
        T: 'static;
}

// Implement Transformer trait for all FnOnce
impl<T, F> Transformer<T> for F
where
    F: FnOnce(T) -> T,
{
    fn transform(self, input: T) -> T {
        self(input)
    }

    fn into_fn(self) -> impl FnMut(T) -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        // FnOnce can only be called once, so we wrap it in Option
        // and panic if called more than once
        let mut func = Some(self);
        move |t: T| {
            func.take()
                .expect("FnOnce transformer can only be called once")(t)
        }
    }
}

// ============================================================================
// Closure Extension Trait - Provides composition methods for closures
// ============================================================================

/// Extension trait providing transformation composition methods for closures
///
/// Allows closures to use methods like `.and_then()` and `.compose()`,
/// returning a `BoxTransformer` after composition.
///
/// # Author
///
/// Hu Haixing
pub trait FnTransformerOps<T>: FnOnce(T) -> T + Sized {
    /// Chain composition - applies self first, then after
    ///
    /// Execution order: input -> self -> after -> output
    ///
    /// # Parameters
    ///
    /// * `after` - The transformation to apply after self
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformer<T>`, representing the composed transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FnTransformerOps;
    ///
    /// let add_one = |x: i32| x + 1;
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.then(double);
    /// ```
    fn then<G>(self, after: G) -> BoxTransformer<T>
    where
        Self: 'static,
        G: FnOnce(T) -> T + 'static,
        T: 'static,
    {
        BoxTransformer::new(move |x| after(self(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Execution order: input -> before -> self -> output
    ///
    /// # Parameters
    ///
    /// * `before` - The transformation to apply before self
    ///
    /// # Returns
    ///
    /// Returns `BoxTransformer<T>`, representing the composed transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FnTransformerOps;
    ///
    /// let double = |x: i32| x * 2;
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose_transformer(add_one);
    /// ```
    fn compose_transformer<G>(self, before: G) -> BoxTransformer<T>
    where
        Self: 'static,
        G: FnOnce(T) -> T + 'static,
        T: 'static,
    {
        BoxTransformer::new(move |x| self(before(x)))
    }
}

// Implement FnTransformerOps for all FnOnce
impl<T, F> FnTransformerOps<T> for F where F: FnOnce(T) -> T {}

// ============================================================================
// BoxTransformer - Single ownership, one-time use (FnOnce)
// ============================================================================

/// BoxTransformer - transformation wrapper based on `Box<dyn FnOnce>`
///
/// Used for single ownership scenarios where the transformation can only be
/// applied once.
///
/// # Features
///
/// - Based on `Box<dyn FnOnce(T) -> T>`
/// - Single ownership, cannot be cloned
/// - Can only be called once (transform consumes self)
/// - Suitable for one-time transformations and pipeline construction
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Examples
///
/// ```rust
/// use prism3_function::BoxTransformer;
///
/// let double = BoxTransformer::new(|x: i32| x * 2);
/// let result = double.transform(21);
/// assert_eq!(result, 42);
/// // double has been consumed and cannot be used again
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct BoxTransformer<T> {
    f: Box<dyn FnOnce(T) -> T>,
}

impl<T> BoxTransformer<T>
where
    T: 'static,
{
    /// Creates a new BoxTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Returns
    ///
    /// Returns a new BoxTransformer instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> T + 'static,
    {
        BoxTransformer { f: Box::new(f) }
    }

    /// Applies the transformation to the input value
    ///
    /// Consumes self and returns the result.
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let result = double.transform(21);
    /// assert_eq!(result, 42);
    /// ```
    pub fn transform(self, input: T) -> T {
        (self.f)(input)
    }

    /// Creates an identity transformation
    ///
    /// Returns a transformation that directly returns the input value.
    ///
    /// # Returns
    ///
    /// Returns an identity transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let identity = BoxTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> BoxTransformer<T> {
        BoxTransformer::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Execution order: input -> self -> after -> output
    ///
    /// # Parameters
    ///
    /// * `after` - The transformation to apply after self
    ///
    /// # Returns
    ///
    /// Returns the composed BoxTransformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.then(double);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
    /// ```
    pub fn then<G>(self, after: G) -> BoxTransformer<T>
    where
        G: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |x| after((self.f)(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Execution order: input -> before -> self -> output
    ///
    /// # Parameters
    ///
    /// * `before` - The transformation to apply before self
    ///
    /// # Returns
    ///
    /// Returns the composed BoxTransformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
    /// ```
    pub fn compose<G>(self, before: G) -> BoxTransformer<T>
    where
        G: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |x| (self.f)(before(x)))
    }

    /// Chains this transformer with another transformer
    ///
    /// Execution order: input -> self -> after -> output
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// Returns the composed BoxTransformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let add_one = BoxTransformer::new(|x: i32| x + 1);
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    /// let composed = add_one.chain(double);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
    /// ```
    pub fn chain(self, after: BoxTransformer<T>) -> BoxTransformer<T> {
        BoxTransformer::new(move |x| (after.f)((self.f)(x)))
    }
}

impl<T> BoxTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformation
    ///
    /// Returns a transformation that ignores the input value and always
    /// returns the same constant value.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// Returns a constant transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let constant = BoxTransformer::constant(42);
    /// assert_eq!(constant.transform(100), 42);
    /// ```
    pub fn constant(value: T) -> BoxTransformer<T> {
        BoxTransformer::new(move |_| value.clone())
    }

    /// Creates a conditional transformer
    ///
    /// Returns a transformer that applies the transformation only when the
    /// condition is met, otherwise returns the original value.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition predicate
    /// * `transform` - The transformation to apply when condition is true
    ///
    /// # Returns
    ///
    /// Returns a conditional transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let conditional = BoxTransformer::when(
    ///     |x: &i32| *x > 0,
    ///     |x: i32| x * 2
    /// );
    /// assert_eq!(conditional.transform(5), 10);
    /// ```
    pub fn when<P, F>(predicate: P, transform: F) -> BoxTransformer<T>
    where
        P: FnOnce(&T) -> bool + 'static,
        F: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |input| {
            if predicate(&input) {
                transform(input)
            } else {
                input
            }
        })
    }

    /// Creates a conditional transformer with else branch
    ///
    /// Returns a transformer that applies different transformations based on
    /// the condition.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition predicate
    /// * `if_true` - The transformation when condition is true
    /// * `if_false` - The transformation when condition is false
    ///
    /// # Returns
    ///
    /// Returns a conditional transformer with branching
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let transformer = BoxTransformer::if_else(
    ///     |x: &i32| *x > 0,
    ///     |x: i32| x * 2,
    ///     |x: i32| x.abs()
    /// );
    /// assert_eq!(transformer.transform(5), 10);
    /// ```
    pub fn if_else<P, F, G>(predicate: P, if_true: F, if_false: G) -> BoxTransformer<T>
    where
        P: FnOnce(&T) -> bool + 'static,
        F: FnOnce(T) -> T + 'static,
        G: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |input| {
            if predicate(&input) {
                if_true(input)
            } else {
                if_false(input)
            }
        })
    }

    /// Repeats the transformation multiple times
    ///
    /// Creates a new transformer that applies the given transformation
    /// repeatedly for the specified number of times.
    ///
    /// # Parameters
    ///
    /// * `f` - The transformation to repeat
    /// * `times` - The number of times to repeat
    ///
    /// # Returns
    ///
    /// Returns a new transformer that repeats the transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let add_one = |x: i32| x + 1;
    /// let add_three = BoxTransformer::repeat(add_one, 3);
    /// assert_eq!(add_three.transform(5), 8); // 5 + 1 + 1 + 1 = 8
    /// ```
    pub fn repeat<F>(f: F, times: usize) -> BoxTransformer<T>
    where
        F: Fn(T) -> T + Clone + 'static,
    {
        if times == 0 {
            return BoxTransformer::identity();
        }

        BoxTransformer::new(move |mut input| {
            for _ in 0..times {
                input = f.clone()(input);
            }
            input
        })
    }
}

// Option-related methods
impl<T> BoxTransformer<Option<T>>
where
    T: 'static,
{
    /// Creates a transformer that maps over Option values
    ///
    /// # Parameters
    ///
    /// * `f` - The function to transform the value inside Some
    ///
    /// # Returns
    ///
    /// Returns a transformer for Option values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = |x: i32| x * 2;
    /// let option_double = BoxTransformer::map_option(double);
    /// assert_eq!(option_double.transform(Some(21)), Some(42));
    ///
    /// // Create a new transformer for None case
    /// let double2 = |x: i32| x * 2;
    /// let option_double2 = BoxTransformer::map_option(double2);
    /// assert_eq!(option_double2.transform(None), None);
    /// ```
    pub fn map_option<F>(f: F) -> BoxTransformer<Option<T>>
    where
        F: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |opt: Option<T>| opt.map(f))
    }
}

// Result-related methods
impl<T, E> BoxTransformer<Result<T, E>>
where
    T: 'static,
    E: 'static,
{
    /// Creates a transformer that maps over Result success values
    ///
    /// # Parameters
    ///
    /// * `f` - The function to transform the value inside Ok
    ///
    /// # Returns
    ///
    /// Returns a transformer for Result values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = |x: i32| x * 2;
    /// let result_double = BoxTransformer::map_result(double);
    /// assert_eq!(result_double.transform(Ok::<i32, &str>(21)), Ok(42));
    ///
    /// // Create a new transformer for Err case
    /// let double2 = |x: i32| x * 2;
    /// let result_double2 = BoxTransformer::map_result(double2);
    /// assert_eq!(
    ///     result_double2.transform(Err::<i32, &str>("error")),
    ///     Err("error")
    /// );
    /// ```
    pub fn map_result<F>(f: F) -> BoxTransformer<Result<T, E>>
    where
        F: FnOnce(T) -> T + 'static,
    {
        BoxTransformer::new(move |result: Result<T, E>| result.map(f))
    }
}

// Implement Transformer trait for BoxTransformer
impl<T> Transformer<T> for BoxTransformer<T>
where
    T: 'static,
{
    fn transform(self, input: T) -> T {
        (self.f)(input)
    }

    fn into_fn(self) -> impl FnMut(T) -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        // BoxTransformer uses FnOnce internally, which can only be called once.
        // We need to use an Option to track whether it has been called.
        let mut f = Some(self.f);
        move |t: T| f.take().expect("BoxTransformer can only be called once")(t)
    }
}

// ============================================================================
// BoxFnTransformer - Single ownership, reusable (Fn)
// ============================================================================

/// BoxFnTransformer - transformation wrapper based on `Box<dyn Fn>`
///
/// Used for reusable scenarios, but still with single ownership (not
/// clonable).
///
/// # Features
///
/// - Based on `Box<dyn Fn(T) -> T>`
/// - Single ownership, cannot be cloned
/// - Can be called multiple times (transform uses &self)
/// - Composition methods consume self (because Box cannot be cloned)
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Examples
///
/// ```rust
/// use prism3_function::BoxFnTransformer;
///
/// let double = BoxFnTransformer::new(|x: i32| x * 2);
/// let r1 = double.transform(21);
/// let r2 = double.transform(42);
/// assert_eq!(r1, 42);
/// assert_eq!(r2, 84);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct BoxFnTransformer<T> {
    f: Box<dyn Fn(T) -> T>,
}

impl<T> BoxFnTransformer<T>
where
    T: 'static,
{
    /// Creates a new BoxFnTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Returns
    ///
    /// Returns a new BoxFnTransformer instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let double = BoxFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> T + 'static,
    {
        BoxFnTransformer { f: Box::new(f) }
    }

    /// Applies the transformation to the input value
    ///
    /// Uses &self, so can be called multiple times.
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let double = BoxFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(double.transform(42), 84);
    /// ```
    pub fn transform(&self, input: T) -> T {
        (self.f)(input)
    }

    /// Creates an identity transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let identity = BoxFnTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> BoxFnTransformer<T> {
        BoxFnTransformer::new(|x| x)
    }

    /// Chain composition (consumes self)
    ///
    /// Note: Although transform can be called multiple times, composition
    /// methods consume self because `Box<dyn Fn>` cannot be cloned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let add_one = BoxFnTransformer::new(|x: i32| x + 1);
    /// let double = BoxFnTransformer::new(|x: i32| x * 2);
    /// let composed = add_one.then(double);
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn then(self, after: BoxFnTransformer<T>) -> BoxFnTransformer<T> {
        let self_f = self.f;
        let after_f = after.f;
        BoxFnTransformer::new(move |x| after_f(self_f(x)))
    }

    /// Reverse composition (consumes self)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let double = BoxFnTransformer::new(|x: i32| x * 2);
    /// let add_one = BoxFnTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn compose(self, before: BoxFnTransformer<T>) -> BoxFnTransformer<T> {
        let self_f = self.f;
        let before_f = before.f;
        BoxFnTransformer::new(move |x| self_f(before_f(x)))
    }
}

impl<T> BoxFnTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let constant = BoxFnTransformer::constant(42);
    /// assert_eq!(constant.transform(100), 42);
    /// assert_eq!(constant.transform(200), 42);
    /// ```
    pub fn constant(value: T) -> BoxFnTransformer<T> {
        BoxFnTransformer::new(move |_| value.clone())
    }

    /// Converts transformer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// Consumes the transformer and returns a closure that can be used
    /// with iterator methods like `map()`. Unlike `BoxTransformer`,
    /// `BoxFnTransformer` uses `Fn` internally, so the returned closure
    /// can be called multiple times.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the transformer (takes ownership of `self`).
    /// After calling this method, the original transformer is no longer
    /// available.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(T) -> T` and can be called
    /// multiple times
    ///
    /// # Examples
    ///
    /// ## Basic Usage with Iterator
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let transformer = BoxFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## Multiple Calls to Returned Closure
    ///
    /// ```rust
    /// use prism3_function::BoxFnTransformer;
    ///
    /// let transformer = BoxFnTransformer::new(|x: i32| x * 2);
    /// let mut func = transformer.into_fn();
    ///
    /// // Can call the returned closure multiple times
    /// assert_eq!(func(1), 2);
    /// assert_eq!(func(2), 4);
    /// assert_eq!(func(3), 6);
    /// ```
    pub fn into_fn(self) -> impl FnMut(T) -> T {
        move |t: T| (self.f)(t)
    }
}

// ============================================================================
// ArcFnTransformer - Multi-threaded sharing, reusable (Arc + Fn)
// ============================================================================

/// ArcFnTransformer - transformation wrapper based on `Arc<dyn Fn>`
///
/// Used for multi-threaded sharing and reusable scenarios.
///
/// # Features
///
/// - Based on `Arc<dyn Fn(T) -> T + Send + Sync>`
/// - Can be cloned and used across threads
/// - Can be called multiple times (transform uses &self)
/// - Composition methods use &self, do not consume ownership
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Examples
///
/// ```rust
/// use prism3_function::ArcFnTransformer;
///
/// let double = ArcFnTransformer::new(|x: i32| x * 2);
/// let cloned = double.clone();
///
/// assert_eq!(double.transform(21), 42);
/// assert_eq!(cloned.transform(42), 84);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct ArcFnTransformer<T> {
    f: Arc<dyn Fn(T) -> T + Send + Sync>,
}

impl<T> ArcFnTransformer<T>
where
    T: 'static,
{
    /// Creates a new ArcFnTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Returns
    ///
    /// Returns a new ArcFnTransformer instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let double = ArcFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        ArcFnTransformer { f: Arc::new(f) }
    }

    /// Applies the transformation to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let double = ArcFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(double.transform(42), 84);
    /// ```
    pub fn transform(&self, input: T) -> T {
        (self.f)(input)
    }

    /// Creates an identity transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let identity = ArcFnTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> ArcFnTransformer<T>
    where
        T: Send + Sync,
    {
        ArcFnTransformer::new(|x| x)
    }

    /// Chain composition (uses &self, does not consume ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let add_one = ArcFnTransformer::new(|x: i32| x + 1);
    /// let double = ArcFnTransformer::new(|x: i32| x * 2);
    /// let composed = add_one.then(&double);
    ///
    /// // Original transformers are still usable
    /// assert_eq!(add_one.transform(5), 6);
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn then(&self, after: &ArcFnTransformer<T>) -> ArcFnTransformer<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcFnTransformer {
            f: Arc::new(move |x| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition (uses &self, does not consume ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let double = ArcFnTransformer::new(|x: i32| x * 2);
    /// let add_one = ArcFnTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn compose(&self, before: &ArcFnTransformer<T>) -> ArcFnTransformer<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcFnTransformer {
            f: Arc::new(move |x| self_clone(before_clone(x))),
        }
    }
}

impl<T> Clone for ArcFnTransformer<T> {
    fn clone(&self) -> Self {
        ArcFnTransformer {
            f: Arc::clone(&self.f),
        }
    }
}

impl<T> ArcFnTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let constant = ArcFnTransformer::constant(42);
    /// assert_eq!(constant.transform(100), 42);
    /// assert_eq!(constant.transform(200), 42);
    /// ```
    pub fn constant(value: T) -> ArcFnTransformer<T>
    where
        T: Send + Sync,
    {
        ArcFnTransformer::new(move |_| value.clone())
    }

    /// Converts transformer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// Consumes the transformer and returns a closure that can be used
    /// with iterator methods like `map()`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the transformer (takes ownership of `self`).
    /// After calling this method, the original transformer is no longer
    /// available.
    ///
    /// **Tip**: Since `ArcFnTransformer` is cloneable, you can call
    /// `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let transformer = ArcFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// // Clone before conversion to keep the original
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.clone().into_fn())
    ///     .collect();
    ///
    /// // Original is still available
    /// assert_eq!(transformer.transform(10), 20);
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(T) -> T`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let transformer = ArcFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## Cloning to Preserve Original
    ///
    /// ```rust
    /// use prism3_function::ArcFnTransformer;
    ///
    /// let transformer = ArcFnTransformer::new(|x: i32| x * 2);
    ///
    /// // Use clone() to keep the original available
    /// let values1 = vec![1, 2, 3];
    /// let result1: Vec<i32> = values1.into_iter()
    ///     .map(transformer.clone().into_fn())
    ///     .collect();
    ///
    /// // Original transformer is still usable
    /// let values2 = vec![4, 5];
    /// let result2: Vec<i32> = values2.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result1, vec![2, 4, 6]);
    /// assert_eq!(result2, vec![8, 10]);
    /// ```
    pub fn into_fn(self) -> impl FnMut(T) -> T
    where
        T: Send + Sync + 'static,
    {
        move |t: T| (self.f)(t)
    }
}

// ============================================================================
// RcFnTransformer - Single-threaded sharing, reusable (Rc + Fn)
// ============================================================================

/// RcFnTransformer - transformation wrapper based on `Rc<dyn Fn>`
///
/// Used for single-threaded sharing and reusable scenarios.
///
/// # Features
///
/// - Based on `Rc<dyn Fn(T) -> T>`
/// - Can be cloned, but cannot cross thread boundaries
/// - Can be called multiple times (transform uses &self)
/// - Composition methods use &self, do not consume ownership
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Examples
///
/// ```rust
/// use prism3_function::RcFnTransformer;
///
/// let double = RcFnTransformer::new(|x: i32| x * 2);
/// let cloned = double.clone();
///
/// assert_eq!(double.transform(21), 42);
/// assert_eq!(cloned.transform(42), 84);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct RcFnTransformer<T> {
    f: Rc<dyn Fn(T) -> T>,
}

impl<T> RcFnTransformer<T>
where
    T: 'static,
{
    /// Creates a new RcFnTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Returns
    ///
    /// Returns a new RcFnTransformer instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let double = RcFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> T + 'static,
    {
        RcFnTransformer { f: Rc::new(f) }
    }

    /// Applies the transformation to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let double = RcFnTransformer::new(|x: i32| x * 2);
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(double.transform(42), 84);
    /// ```
    pub fn transform(&self, input: T) -> T {
        (self.f)(input)
    }

    /// Creates an identity transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let identity = RcFnTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> RcFnTransformer<T> {
        RcFnTransformer::new(|x| x)
    }

    /// Chain composition (uses &self, does not consume ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let add_one = RcFnTransformer::new(|x: i32| x + 1);
    /// let double = RcFnTransformer::new(|x: i32| x * 2);
    /// let composed = add_one.then(&double);
    ///
    /// // Original transformers are still usable
    /// assert_eq!(add_one.transform(5), 6);
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn then(&self, after: &RcFnTransformer<T>) -> RcFnTransformer<T> {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcFnTransformer {
            f: Rc::new(move |x| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition (uses &self, does not consume ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let double = RcFnTransformer::new(|x: i32| x * 2);
    /// let add_one = RcFnTransformer::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(5), 12);
    /// ```
    pub fn compose(&self, before: &RcFnTransformer<T>) -> RcFnTransformer<T> {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcFnTransformer {
            f: Rc::new(move |x| self_clone(before_clone(x))),
        }
    }
}

impl<T> Clone for RcFnTransformer<T> {
    fn clone(&self) -> Self {
        RcFnTransformer {
            f: Rc::clone(&self.f),
        }
    }
}

impl<T> RcFnTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let constant = RcFnTransformer::constant(42);
    /// assert_eq!(constant.transform(100), 42);
    /// assert_eq!(constant.transform(200), 42);
    /// ```
    pub fn constant(value: T) -> RcFnTransformer<T> {
        RcFnTransformer::new(move |_| value.clone())
    }

    /// Converts transformer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original transformer becomes unavailable
    /// after calling this method.
    ///
    /// Consumes the transformer and returns a closure that can be used
    /// with iterator methods like `map()`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the transformer (takes ownership of `self`).
    /// After calling this method, the original transformer is no longer
    /// available.
    ///
    /// **Tip**: Since `RcFnTransformer` is cloneable, you can call
    /// `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let transformer = RcFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// // Clone before conversion to keep the original
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.clone().into_fn())
    ///     .collect();
    ///
    /// // Original is still available
    /// assert_eq!(transformer.transform(10), 20);
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(T) -> T`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let transformer = RcFnTransformer::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## Cloning to Preserve Original
    ///
    /// ```rust
    /// use prism3_function::RcFnTransformer;
    ///
    /// let transformer = RcFnTransformer::new(|x: i32| x * 2);
    ///
    /// // Use clone() to keep the original available
    /// let values1 = vec![1, 2, 3];
    /// let result1: Vec<i32> = values1.into_iter()
    ///     .map(transformer.clone().into_fn())
    ///     .collect();
    ///
    /// // Original transformer is still usable
    /// let values2 = vec![4, 5];
    /// let result2: Vec<i32> = values2.into_iter()
    ///     .map(transformer.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result1, vec![2, 4, 6]);
    /// assert_eq!(result2, vec![8, 10]);
    /// ```
    pub fn into_fn(self) -> impl FnMut(T) -> T {
        move |t: T| (self.f)(t)
    }
}
