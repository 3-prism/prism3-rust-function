/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiTransformer Types
//!
//! Provides Rust implementations of bi-transformer traits for type conversion
//! and value transformation with two inputs. BiTransformers consume two input
//! values (taking ownership) and produce an output value.
//!
//! This module provides the `BiTransformer<T, U, R>` trait and three
//! implementations:
//!
//! - [`BoxBiTransformer`]: Single ownership, not cloneable
//! - [`ArcBiTransformer`]: Thread-safe shared ownership, cloneable
//! - [`RcBiTransformer`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

use crate::bi_predicate::{ArcBiPredicate, BiPredicate, BoxBiPredicate, RcBiPredicate};

// ============================================================================
// Core Trait
// ============================================================================

/// BiTransformer trait - transforms two values to produce a result
///
/// Defines the behavior of a bi-transformation: converting two values of types
/// `T` and `U` to a value of type `R` by consuming the inputs. This is
/// analogous to `Fn(T, U) -> R` in Rust's standard library.
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
pub trait BiTransformer<T, U, R> {
    /// Transforms two input values to produce an output value
    ///
    /// # Parameters
    ///
    /// * `first` - The first input value to transform (consumed)
    /// * `second` - The second input value to transform (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(&self, first: T, second: U) -> R;

    /// Converts to BoxBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in a `Box` and creates a
    /// `BoxBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `BoxBiTransformer<T, U, R>`
    fn into_box(self) -> BoxBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(move |x, y| self.apply(x, y))
    }

    /// Converts to RcBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `Rc` and creates an
    /// `RcBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `RcBiTransformer<T, U, R>`
    fn into_rc(self) -> RcBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcBiTransformer::new(move |x, y| self.apply(x, y))
    }

    /// Converts to ArcBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `Arc` and creates an
    /// `ArcBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `ArcBiTransformer<T, U, R>`
    fn into_arc(self) -> ArcBiTransformer<T, U, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcBiTransformer::new(move |x, y| self.apply(x, y))
    }

    /// Converts bi-transformer to a closure
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a closure that captures `self`
    /// and calls its `apply` method. Types can override this method
    /// to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `Fn(T, U) -> R`
    fn into_fn(self) -> impl Fn(T, U) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t, u| self.apply(t, u)
    }

    /// Non-consuming conversion to `BoxBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_box`.
    fn to_box(&self) -> BoxBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Non-consuming conversion to `RcBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_rc`.
    fn to_rc(&self) -> RcBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_rc()
    }

    /// Non-consuming conversion to `ArcBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_arc`.
    fn to_arc(&self) -> ArcBiTransformer<T, U, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.clone().into_arc()
    }

    /// Non-consuming conversion to a boxed function using `&self`.
    ///
    /// Returns a `Box<dyn Fn(T, U) -> R>` that clones `self` and calls
    /// `apply` inside the boxed closure.
    fn to_fn(&self) -> impl Fn(T, U) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_fn()
    }
}

// ============================================================================
// BoxBiTransformer - Box<dyn Fn(T, U) -> R>
// ============================================================================

/// BoxBiTransformer - bi-transformer wrapper based on `Box<dyn Fn>`
///
/// A bi-transformer wrapper that provides single ownership with reusable
/// transformation. The bi-transformer consumes both inputs and can be called
/// multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn Fn(T, U) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxBiTransformer<T, U, R> {
    function: Box<dyn Fn(T, U) -> R>,
}

impl<T, U, R> BoxBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Creates a new BoxBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxBiTransformer, BiTransformer};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T, U) -> R + 'static,
    {
        BoxBiTransformer {
            function: Box::new(f),
        }
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Consumes self.
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
    /// A new `BoxBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, BoxBiTransformer, BoxTransformer};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    ///
    /// // double is moved here
    /// let composed = add.and_then(double);
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    /// // double.apply(10); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, BoxBiTransformer, BoxTransformer};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    ///
    /// // Clone to preserve original
    /// let composed = add.and_then(double.clone());
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    ///
    /// // Original still usable
    /// assert_eq!(double.apply(10), 20);
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxBiTransformer<T, U, S>
    where
        S: 'static,
        F: crate::transformer::Transformer<R, S> + 'static,
    {
        let self_fn = self.function;
        BoxBiTransformer::new(move |t: T, u: U| after.apply(self_fn(t, u)))
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
    /// Returns `BoxConditionalBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, BoxBiTransformer};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///     .or_else(multiply);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);  // add
    /// assert_eq!(conditional.apply(-5, 3), -15); // multiply
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, BoxBiTransformer, RcBiPredicate};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(BoxBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
    {
        BoxConditionalBiTransformer {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, U, R> BoxBiTransformer<T, U, R>
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
    /// use prism3_function::{BoxBiTransformer, BiTransformer};
    ///
    /// let constant = BoxBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> BoxBiTransformer<T, U, R> {
        BoxBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> BiTransformer<T, U, R> for BoxBiTransformer<T, U, R> {
    fn apply(&self, first: T, second: U) -> R {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcBiTransformer::new(move |t, u| (self.function)(t, u))
    }

    // do NOT override BoxBiTransformer::into_arc() because BoxBiTransformer is not Send + Sync
    // and calling BoxBiTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| (self.function)(t, u)
    }

    // do NOT override BoxBiTransformer::to_xxx() because BoxBiTransformer is not Clone
    // and calling BoxBiTransformer::to_xxx() will cause a compile error
}

// ============================================================================
// BoxConditionalBiTransformer - Box-based Conditional BiTransformer
// ============================================================================

/// BoxConditionalBiTransformer struct
///
/// A conditional bi-transformer that only executes when a bi-predicate is
/// satisfied. Uses `BoxBiTransformer` and `BoxBiPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxBiTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements BiTransformer**: Can be used anywhere a `BiTransformer` is expected
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{BiTransformer, BoxBiTransformer};
///
/// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(multiply);
///
/// assert_eq!(conditional.apply(5, 3), 8);  // when branch executed
/// assert_eq!(conditional.apply(-5, 3), -15); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalBiTransformer<T, U, R> {
    transformer: BoxBiTransformer<T, U, R>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U, R> BoxConditionalBiTransformer<T, U, R>
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
    ///   - `BoxBiTransformer<T, U, R>`, `RcBiTransformer<T, U, R>`, `ArcBiTransformer<T, U, R>`
    ///   - Any type implementing `BiTransformer<T, U, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, BoxBiTransformer};
    ///
    /// let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);   // Condition satisfied, execute add
    /// assert_eq!(conditional.apply(-5, 3), -15); // Condition not satisfied, execute multiply
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxBiTransformer<T, U, R>
    where
        F: BiTransformer<T, U, R> + 'static,
    {
        let pred = self.predicate;
        let then_trans = self.transformer;
        BoxBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_transformer.apply(t, u)
            }
        })
    }
}

// ============================================================================
// ArcBiTransformer - Arc<dyn Fn(T, U) -> R + Send + Sync>
// ============================================================================

/// ArcBiTransformer - thread-safe bi-transformer wrapper
///
/// A thread-safe, clonable bi-transformer wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(T, U) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct ArcBiTransformer<T, U, R> {
    function: Arc<dyn Fn(T, U) -> R + Send + Sync>,
}

impl<T, U, R> ArcBiTransformer<T, U, R>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    R: 'static,
{
    /// Creates a new ArcBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcBiTransformer, BiTransformer};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T, U) -> R + Send + Sync + 'static,
    {
        ArcBiTransformer {
            function: Arc::new(f),
        }
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Uses &self, so original
    /// bi-transformer remains usable.
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
    ///   `Clone`). Must be `Send + Sync`, can be:
    ///   - A closure: `|x: R| -> S` (must be `Send + Sync`)
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxTransformer<R, S>`
    ///   - An `RcTransformer<R, S>`
    ///   - An `ArcTransformer<R, S>` (will be moved)
    ///   - Any type implementing `Transformer<R, S> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, ArcBiTransformer, ArcTransformer};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    ///
    /// // double is moved here
    /// let composed = add.and_then(double);
    ///
    /// // Original add bi-transformer still usable (uses &self)
    /// assert_eq!(add.apply(20, 22), 42);
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    /// // double.apply(10); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, ArcBiTransformer, ArcTransformer};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = ArcTransformer::new(|x: i32| x * 2);
    ///
    /// // Clone to preserve original
    /// let composed = add.and_then(double.clone());
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    ///
    /// // Both originals still usable
    /// assert_eq!(add.apply(20, 22), 42);
    /// assert_eq!(double.apply(10), 20);
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> ArcBiTransformer<T, U, S>
    where
        S: Send + Sync + 'static,
        F: crate::transformer::Transformer<R, S> + Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.function);
        ArcBiTransformer {
            function: Arc::new(move |t: T, u: U| after.apply(self_clone(t, u))),
        }
    }

    /// Creates a conditional bi-transformer (thread-safe version)
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
    ///   Must be `Send + Sync`, can be:
    ///   - A closure: `|x: &T, y: &U| -> bool` (requires `Send + Sync`)
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, ArcBiTransformer};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///     .or_else(multiply);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional_clone.apply(-5, 3), -15);
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, ArcBiTransformer, ArcBiPredicate};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = ArcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(ArcBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        ArcConditionalBiTransformer {
            transformer: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, U, R> ArcBiTransformer<T, U, R>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    R: Clone + 'static,
{
    /// Creates a constant bi-transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcBiTransformer, BiTransformer};
    ///
    /// let constant = ArcBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> ArcBiTransformer<T, U, R>
    where
        R: Send + Sync,
    {
        ArcBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> BiTransformer<T, U, R> for ArcBiTransformer<T, U, R> {
    fn apply(&self, first: T, second: U) -> R {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(move |t, u| (self.function)(t, u))
    }

    fn into_rc(self) -> RcBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcBiTransformer::new(move |t, u| (self.function)(t, u))
    }

    fn into_arc(self) -> ArcBiTransformer<T, U, R>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl Fn(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| (self.function)(t, u)
    }

    fn to_box(&self) -> BoxBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiTransformer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        RcBiTransformer::new(move |t, u| self_fn(t, u))
    }

    fn to_arc(&self) -> ArcBiTransformer<T, U, R>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl Fn(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t: T, u: U| self_fn(t, u)
    }
}

impl<T, U, R> Clone for ArcBiTransformer<T, U, R> {
    fn clone(&self) -> Self {
        ArcBiTransformer {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// ArcConditionalBiTransformer - Arc-based Conditional BiTransformer
// ============================================================================

/// ArcConditionalBiTransformer struct
///
/// A thread-safe conditional bi-transformer that only executes when a
/// bi-predicate is satisfied. Uses `ArcBiTransformer` and `ArcBiPredicate` for
/// shared ownership across threads.
///
/// This type is typically created by calling `ArcBiTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiTransformer, ArcBiTransformer};
///
/// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(multiply);
///
/// let conditional_clone = conditional.clone();
///
/// assert_eq!(conditional.apply(5, 3), 8);
/// assert_eq!(conditional_clone.apply(-5, 3), -15);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalBiTransformer<T, U, R> {
    transformer: ArcBiTransformer<T, U, R>,
    predicate: ArcBiPredicate<T, U>,
}

impl<T, U, R> ArcConditionalBiTransformer<T, U, R>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    R: 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original bi-transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The bi-transformer for the else branch, can be:
    ///   - Closure: `|x: T, y: U| -> R` (must be `Send + Sync`)
    ///   - `ArcBiTransformer<T, U, R>`, `BoxBiTransformer<T, U, R>`
    ///   - Any type implementing `BiTransformer<T, U, R> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, ArcBiTransformer};
    ///
    /// let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional.apply(-5, 3), -15);
    /// ```
    pub fn or_else<F>(&self, else_transformer: F) -> ArcBiTransformer<T, U, R>
    where
        F: BiTransformer<T, U, R> + Send + Sync + 'static,
        R: Send + Sync,
    {
        let pred = self.predicate.clone();
        let then_trans = self.transformer.clone();
        ArcBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_transformer.apply(t, u)
            }
        })
    }
}

impl<T, U, R> Clone for ArcConditionalBiTransformer<T, U, R> {
    /// Clones the conditional bi-transformer
    ///
    /// Creates a new instance that shares the underlying bi-transformer and
    /// bi-predicate with the original instance.
    fn clone(&self) -> Self {
        ArcConditionalBiTransformer {
            transformer: self.transformer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcBiTransformer - Rc<dyn Fn(T, U) -> R>
// ============================================================================

/// RcBiTransformer - single-threaded bi-transformer wrapper
///
/// A single-threaded, clonable bi-transformer wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(T, U) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct RcBiTransformer<T, U, R> {
    function: Rc<dyn Fn(T, U) -> R>,
}

impl<T, U, R> RcBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Creates a new RcBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcBiTransformer, BiTransformer};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T, U) -> R + 'static,
    {
        RcBiTransformer {
            function: Rc::new(f),
        }
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Uses &self, so original
    /// bi-transformer remains usable.
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
    ///   - An `RcTransformer<R, S>` (will be moved)
    ///   - An `ArcTransformer<R, S>`
    ///   - Any type implementing `Transformer<R, S>`
    ///
    /// # Returns
    ///
    /// A new `RcBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, RcBiTransformer, RcTransformer};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = RcTransformer::new(|x: i32| x * 2);
    ///
    /// // double is moved here
    /// let composed = add.and_then(double);
    ///
    /// // Original add bi-transformer still usable (uses &self)
    /// assert_eq!(add.apply(20, 22), 42);
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    /// // double.apply(10); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, RcBiTransformer, RcTransformer};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = RcTransformer::new(|x: i32| x * 2);
    ///
    /// // Clone to preserve original
    /// let composed = add.and_then(double.clone());
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    ///
    /// // Both originals still usable
    /// assert_eq!(add.apply(20, 22), 42);
    /// assert_eq!(double.apply(10), 20);
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> RcBiTransformer<T, U, S>
    where
        S: 'static,
        F: crate::transformer::Transformer<R, S> + 'static,
    {
        let self_clone = Rc::clone(&self.function);
        RcBiTransformer {
            function: Rc::new(move |t: T, u: U| after.apply(self_clone(t, u))),
        }
    }

    /// Creates a conditional bi-transformer (single-threaded shared version)
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
    /// Returns `RcConditionalBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, RcBiTransformer};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = RcBiTransformer::new(|x: i32, y: i32| x * y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///     .or_else(multiply);
    ///
    /// let conditional_clone = conditional.clone();
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional_clone.apply(-5, 3), -15);
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, RcBiTransformer, RcBiPredicate};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(RcBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(&self, predicate: P) -> RcConditionalBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
    {
        RcConditionalBiTransformer {
            transformer: self.clone(),
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, U, R> RcBiTransformer<T, U, R>
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
    /// use prism3_function::{RcBiTransformer, BiTransformer};
    ///
    /// let constant = RcBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> RcBiTransformer<T, U, R> {
        RcBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> BiTransformer<T, U, R> for RcBiTransformer<T, U, R> {
    fn apply(&self, first: T, second: U) -> R {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(move |t, u| (self.function)(t, u))
    }

    fn into_rc(self) -> RcBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    // do NOT override RcBiTransformer::into_arc() because RcBiTransformer is not Send + Sync
    // and calling RcBiTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| (self.function)(t, u)
    }

    fn to_box(&self) -> BoxBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiTransformer::new(move |t, u| self_fn(t, u))
    }

    fn to_rc(&self) -> RcBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone()
    }

    // do NOT override RcBiTransformer::to_arc() because RcBiTransformer is not Send + Sync
    // and calling RcBiTransformer::to_arc() will cause a compile error

    fn to_fn(&self) -> impl Fn(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t: T, u: U| self_fn(t, u)
    }
}

impl<T, U, R> Clone for RcBiTransformer<T, U, R> {
    fn clone(&self) -> Self {
        RcBiTransformer {
            function: Rc::clone(&self.function),
        }
    }
}

// ============================================================================
// RcConditionalBiTransformer - Rc-based Conditional BiTransformer
// ============================================================================

/// RcConditionalBiTransformer struct
///
/// A single-threaded conditional bi-transformer that only executes when a
/// bi-predicate is satisfied. Uses `RcBiTransformer` and `RcBiPredicate` for
/// shared ownership within a single thread.
///
/// This type is typically created by calling `RcBiTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalBiTransformer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiTransformer, RcBiTransformer};
///
/// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = RcBiTransformer::new(|x: i32, y: i32| x * y);
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(multiply);
///
/// let conditional_clone = conditional.clone();
///
/// assert_eq!(conditional.apply(5, 3), 8);
/// assert_eq!(conditional_clone.apply(-5, 3), -15);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalBiTransformer<T, U, R> {
    transformer: RcBiTransformer<T, U, R>,
    predicate: RcBiPredicate<T, U>,
}

impl<T, U, R> RcConditionalBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original bi-transformer when the condition is satisfied,
    /// otherwise executes else_transformer.
    ///
    /// # Parameters
    ///
    /// * `else_transformer` - The bi-transformer for the else branch, can be:
    ///   - Closure: `|x: T, y: U| -> R`
    ///   - `RcBiTransformer<T, U, R>`, `BoxBiTransformer<T, U, R>`
    ///   - Any type implementing `BiTransformer<T, U, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, RcBiTransformer};
    ///
    /// let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional.apply(-5, 3), -15);
    /// ```
    pub fn or_else<F>(&self, else_transformer: F) -> RcBiTransformer<T, U, R>
    where
        F: BiTransformer<T, U, R> + 'static,
    {
        let pred = self.predicate.clone();
        let then_trans = self.transformer.clone();
        RcBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_transformer.apply(t, u)
            }
        })
    }
}

impl<T, U, R> Clone for RcConditionalBiTransformer<T, U, R> {
    /// Clones the conditional bi-transformer
    ///
    /// Creates a new instance that shares the underlying bi-transformer and
    /// bi-predicate with the original instance.
    fn clone(&self) -> Self {
        RcConditionalBiTransformer {
            transformer: self.transformer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// Blanket implementation for standard Fn trait
// ============================================================================

/// Implement BiTransformer<T, U, R> for any type that implements Fn(T, U) -> R
///
/// This allows closures and function pointers to be used directly with our
/// BiTransformer trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::BiTransformer;
///
/// fn add(x: i32, y: i32) -> i32 { x + y }
///
/// assert_eq!(add.apply(20, 22), 42);
///
/// let multiply = |x: i32, y: i32| x * y;
/// assert_eq!(multiply.apply(6, 7), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
impl<F, T, U, R> BiTransformer<T, U, R> for F
where
    F: Fn(T, U) -> R,
    T: 'static,
    U: 'static,
    R: 'static,
{
    fn apply(&self, first: T, second: U) -> R {
        self(first, second)
    }

    fn into_box(self) -> BoxBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
    {
        BoxBiTransformer::new(self)
    }

    fn into_rc(self) -> RcBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
    {
        RcBiTransformer::new(self)
    }

    fn into_arc(self) -> ArcBiTransformer<T, U, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcBiTransformer::new(self)
    }

    fn into_fn(self) -> impl Fn(T, U) -> R
    where
        Self: Sized + 'static,
    {
        move |t: T, u: U| self(t, u)
    }

    fn to_box(&self) -> BoxBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(self.clone())
    }

    fn to_rc(&self) -> RcBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcBiTransformer::new(self.clone())
    }

    fn to_arc(&self) -> ArcBiTransformer<T, U, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcBiTransformer::new(self.clone())
    }

    fn to_fn(&self) -> impl Fn(T, U) -> R
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone()
    }
}

// ============================================================================
// FnBiTransformerOps - Extension trait for Fn(T, U) -> R bi-transformers
// ============================================================================

/// Extension trait for closures implementing `Fn(T, U) -> R`
///
/// Provides composition methods (`and_then`, `when`) for bi-transformer
/// closures and function pointers without requiring explicit wrapping in
/// `BoxBiTransformer`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `Fn(T, U) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `BiTransformer<T, U, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then` and `when`. This extension trait provides those methods,
/// returning `BoxBiTransformer` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{BiTransformer, FnBiTransformerOps};
///
/// let add = |x: i32, y: i32| x + y;
/// let double = |x: i32| x * 2;
///
/// let composed = add.and_then(double);
/// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
/// ```
///
/// ## Conditional execution with when
///
/// ```rust
/// use prism3_function::{BiTransformer, FnBiTransformerOps};
///
/// let add = |x: i32, y: i32| x + y;
/// let multiply = |x: i32, y: i32| x * y;
///
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
///
/// assert_eq!(conditional.apply(5, 3), 8);   // add
/// assert_eq!(conditional.apply(-5, 3), -15); // multiply
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnBiTransformerOps<T, U, R>: Fn(T, U) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Consumes self and
    /// returns a `BoxBiTransformer`.
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
    /// A new `BoxBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, FnBiTransformerOps,
    ///     BoxTransformer};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // to_string is moved here
    /// let composed = add.and_then(to_string);
    /// assert_eq!(composed.apply(20, 22), "42");
    /// // to_string.apply(10); // Would not compile - moved
    /// ```
    ///
    /// ## Preserving original with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, FnBiTransformerOps,
    ///     BoxTransformer};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let to_string = BoxTransformer::new(|x: i32| x.to_string());
    ///
    /// // Clone to preserve original
    /// let composed = add.and_then(to_string.clone());
    /// assert_eq!(composed.apply(20, 22), "42");
    ///
    /// // Original still usable
    /// assert_eq!(to_string.apply(10), "10");
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxBiTransformer<T, U, S>
    where
        S: 'static,
        F: crate::transformer::Transformer<R, S> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(move |t: T, u: U| after.apply(self(t, u)))
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
    /// Returns `BoxConditionalBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, FnBiTransformerOps};
    ///
    /// let add = |x: i32, y: i32| x + y;
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0)
    ///     .or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional.apply(-5, 3), -15);
    /// ```
    ///
    /// ## Preserving bi-predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{BiTransformer, FnBiTransformerOps,
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
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformer::new(self).when(predicate)
    }
}

/// Blanket implementation of FnBiTransformerOps for all closures
///
/// Automatically implements `FnBiTransformerOps<T, U, R>` for any type that
/// implements `Fn(T, U) -> R`.
///
/// # Author
///
/// Hu Haixing
impl<T, U, R, F> FnBiTransformerOps<T, U, R> for F where F: Fn(T, U) -> R + 'static {}

// ============================================================================
// BinaryOperator Trait - Marker trait for BiTransformer<T, T, T>
// ============================================================================

/// BinaryOperator trait - marker trait for binary operators
///
/// A binary operator takes two values of type `T` and produces a value of the
/// same type `T`. This trait extends `BiTransformer<T, T, T>` to provide
/// semantic clarity for same-type binary operations. Equivalent to Java's
/// `BinaryOperator<T>` which extends `BiFunction<T, T, T>`.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement
/// `BiTransformer<T, T, T>`, so you don't need to implement it manually.
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
/// use prism3_function::{BinaryOperator, BiTransformer};
///
/// fn reduce<T, O>(values: Vec<T>, initial: T, op: O) -> T
/// where
///     O: BinaryOperator<T>,
///     T: Clone,
/// {
///     values.into_iter().fold(initial, |acc, val| op.apply(acc, val))
/// }
///
/// let sum = |a: i32, b: i32| a + b;
/// assert_eq!(reduce(vec![1, 2, 3, 4], 0, sum), 10);
/// ```
///
/// ## With concrete types
///
/// ```rust
/// use prism3_function::{BoxBinaryOperator, BinaryOperator, BiTransformer};
///
/// fn create_adder() -> BoxBinaryOperator<i32> {
///     BoxBinaryOperator::new(|x, y| x + y)
/// }
///
/// let op = create_adder();
/// assert_eq!(op.apply(20, 22), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait BinaryOperator<T>: BiTransformer<T, T, T> {}

/// Blanket implementation of BinaryOperator for all BiTransformer<T, T, T>
///
/// This automatically implements `BinaryOperator<T>` for any type that
/// implements `BiTransformer<T, T, T>`.
///
/// # Author
///
/// Hu Haixing
impl<F, T> BinaryOperator<T> for F
where
    F: BiTransformer<T, T, T>,
    T: 'static,
{
    // empty
}

// ============================================================================
// Type Aliases for BinaryOperator (BiTransformer<T, T, T>)
// ============================================================================

/// Type alias for `BoxBiTransformer<T, T, T>`
///
/// Represents a binary operator that takes two values of type `T` and produces
/// a value of the same type `T`, with single ownership semantics. Equivalent to
/// Java's `BinaryOperator<T>`.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxBinaryOperator, BiTransformer};
///
/// let add: BoxBinaryOperator<i32> = BoxBinaryOperator::new(|x, y| x + y);
/// assert_eq!(add.apply(20, 22), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type BoxBinaryOperator<T> = BoxBiTransformer<T, T, T>;

/// Type alias for `ArcBiTransformer<T, T, T>`
///
/// Represents a thread-safe binary operator that takes two values of type `T`
/// and produces a value of the same type `T`. Equivalent to Java's
/// `BinaryOperator<T>` with shared, thread-safe ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ArcBinaryOperator, BiTransformer};
///
/// let multiply: ArcBinaryOperator<i32> = ArcBinaryOperator::new(|x, y| x * y);
/// let multiply_clone = multiply.clone();
/// assert_eq!(multiply.apply(6, 7), 42);
/// assert_eq!(multiply_clone.apply(6, 7), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type ArcBinaryOperator<T> = ArcBiTransformer<T, T, T>;

/// Type alias for `RcBiTransformer<T, T, T>`
///
/// Represents a single-threaded binary operator that takes two values of type
/// `T` and produces a value of the same type `T`. Equivalent to Java's
/// `BinaryOperator<T>` with shared, single-threaded ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{RcBinaryOperator, BiTransformer};
///
/// let max: RcBinaryOperator<i32> = RcBinaryOperator::new(|x, y| if x > y { x } else { y });
/// let max_clone = max.clone();
/// assert_eq!(max.apply(30, 42), 42);
/// assert_eq!(max_clone.apply(30, 42), 42);
/// ```
///
/// # Author
///
/// Hu Haixing
pub type RcBinaryOperator<T> = RcBiTransformer<T, T, T>;
