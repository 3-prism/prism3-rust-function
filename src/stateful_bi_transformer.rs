/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # StatefulBiTransformer Types
//!
//! Provides Rust implementations of stateful bi-transformer traits for type
//! conversion and value transformation with two inputs. StatefulBiTransformers
//! consume two input values (taking ownership) and produce an output value.
//!
//! This module provides the `StatefulBiTransformer<T, U, R>` trait and three
//! implementations:
//!
//! - [`BoxStatefulBiTransformer`]: Single ownership, not cloneable
//! - [`ArcStatefulBiTransformer`]: Thread-safe shared ownership, cloneable
//! - [`RcStatefulBiTransformer`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::bi_predicate::{
    ArcBiPredicate,
    BiPredicate,
    BoxBiPredicate,
    RcBiPredicate,
};
use crate::bi_transformer_once::{
    BiTransformerOnce,
    BoxBiTransformerOnce,
};
use crate::stateful_transformer::StatefulTransformer;

// ============================================================================
// Core Trait
// ============================================================================

/// StatefulBiTransformer trait - transforms two values to produce a result
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
/// Haixing Hu
pub trait StatefulBiTransformer<T, U, R> {
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
    fn apply(&mut self, first: T, second: U) -> R;

    /// Converts to BoxStatefulBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in a `Box` and creates a
    /// `BoxStatefulBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `BoxStatefulBiTransformer<T, U, R>`
    fn into_box(self) -> BoxStatefulBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let mut trans = self;
        BoxStatefulBiTransformer::new(move |x, y| trans.apply(x, y))
    }

    /// Converts to RcStatefulBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `Rc` and creates an
    /// `RcStatefulBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `RcStatefulBiTransformer<T, U, R>`
    fn into_rc(self) -> RcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let mut trans = self;
        RcStatefulBiTransformer::new(move |x, y| trans.apply(x, y))
    }

    /// Converts to ArcStatefulBiTransformer
    ///
    /// **⚠️ Consumes `self`**: The original bi-transformer becomes unavailable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps `self` in an `Arc` and creates an
    /// `ArcStatefulBiTransformer`. Types can override this method to provide more
    /// efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `ArcStatefulBiTransformer<T, U, R>`
    fn into_arc(self) -> ArcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + 'static,
    {
        let mut trans = self;
        ArcStatefulBiTransformer::new(move |x, y| trans.apply(x, y))
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
    /// Returns a closure that implements `FnMut(T, U) -> R`
    fn into_fn(self) -> impl FnMut(T, U) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let mut trans = self;
        move |t, u| trans.apply(t, u)
    }

    /// Non-consuming conversion to `BoxStatefulBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_box`.
    fn to_box(&self) -> BoxStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_box()
    }

    /// Non-consuming conversion to `RcStatefulBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_rc`.
    fn to_rc(&self) -> RcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone().into_rc()
    }

    /// Non-consuming conversion to `ArcStatefulBiTransformer` using `&self`.
    ///
    /// Default implementation clones `self` and delegates to `into_arc`.
    fn to_arc(&self) -> ArcStatefulBiTransformer<T, U, R>
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
    /// Returns a `Box<dyn FnMut(T, U) -> R>` that clones `self` and calls
    /// `apply` inside the boxed closure.
    fn to_fn(&self) -> impl FnMut(T, U) -> R
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
// BoxStatefulBiTransformer - Box<dyn FnMut(T, U) -> R>
// ============================================================================

/// BoxStatefulBiTransformer - bi-transformer wrapper based on `Box<dyn Fn>`
///
/// A bi-transformer wrapper that provides single ownership with reusable
/// transformation. The bi-transformer consumes both inputs and can be called
/// multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(T, U) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulBiTransformer<T, U, R> {
    function: Box<dyn FnMut(T, U) -> R>,
}

impl<T, U, R> BoxStatefulBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Creates a new BoxStatefulBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T, U) -> R + 'static,
    {
        BoxStatefulBiTransformer {
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
    /// A new `BoxStatefulBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer, BoxTransformer};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
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
    /// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer, BoxTransformer};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let double = BoxTransformer::new(|x: i32| x * 2);
    ///
    /// // Clone to preserve original
    /// let composed = add.and_then(double.clone());
    /// assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    ///
    /// // Original still usable
    /// assert_eq!(double.apply(10), 20);
    /// ```
    pub fn and_then<S, F>(self, after: F) -> BoxStatefulBiTransformer<T, U, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
    {
        let mut self_fn = self.function;
        let mut after_mut = after;
        BoxStatefulBiTransformer::new(move |t: T, u: U| after_mut.apply(self_fn(t, u)))
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
    /// Returns `BoxConditionalStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = BoxStatefulBiTransformer::new(|x: i32, y: i32| x * y);
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
    /// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer, RcBiPredicate};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(BoxStatefulBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalStatefulBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
    {
        BoxConditionalStatefulBiTransformer {
            transformer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, U, R> BoxStatefulBiTransformer<T, U, R>
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
    /// use prism3_function::{BoxStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let constant = BoxStatefulBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> BoxStatefulBiTransformer<T, U, R> {
        BoxStatefulBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> StatefulBiTransformer<T, U, R> for BoxStatefulBiTransformer<T, U, R> {
    fn apply(&mut self, first: T, second: U) -> R {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcStatefulBiTransformer::new(self.function)
    }

    // do NOT override BoxStatefulBiTransformer::into_arc() because BoxStatefulBiTransformer is not Send + Sync
    // and calling BoxStatefulBiTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
       self.function
    }

    // do NOT override BoxStatefulBiTransformer::to_xxx() because BoxStatefulBiTransformer is not Clone
    // and calling BoxStatefulBiTransformer::to_xxx() will cause a compile error
}

// ============================================================================
// BoxStatefulBiTransformer BiTransformerOnce Implementation
// ============================================================================

impl<T, U, R> BiTransformerOnce<T, U, R> for BoxStatefulBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
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
    fn apply_once(self, first: T, second: U) -> R {
        let mut self_fn = self.function;
        self_fn(first, second)
    }

    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(self.function)
    }

    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let mut self_fn = self.function;
        move |t: T, u: U| self_fn(t, u)
    }

    // do NOT override BoxBiTransformer::to_xxxx() because BoxBiTransformer is not Clone
    // and calling BoxBiTransformer::to_xxxx() will cause a compile error
}

// ============================================================================
// BoxConditionalStatefulBiTransformer - Box-based Conditional StatefulBiTransformer
// ============================================================================

/// BoxConditionalStatefulBiTransformer struct
///
/// A conditional bi-transformer that only executes when a bi-predicate is
/// satisfied. Uses `BoxStatefulBiTransformer` and `BoxBiPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxStatefulBiTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements StatefulBiTransformer**: Can be used anywhere a `StatefulBiTransformer` is expected
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer};
///
/// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = BoxStatefulBiTransformer::new(|x: i32, y: i32| x * y);
/// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(multiply);
///
/// assert_eq!(conditional.apply(5, 3), 8);  // when branch executed
/// assert_eq!(conditional.apply(-5, 3), -15); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalStatefulBiTransformer<T, U, R> {
    transformer: BoxStatefulBiTransformer<T, U, R>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U, R> BoxConditionalStatefulBiTransformer<T, U, R>
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
    ///   - `BoxStatefulBiTransformer<T, U, R>`, `RcStatefulBiTransformer<T, U, R>`, `ArcStatefulBiTransformer<T, U, R>`
    ///   - Any type implementing `StatefulBiTransformer<T, U, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, BoxStatefulBiTransformer};
    ///
    /// let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);   // Condition satisfied, execute add
    /// assert_eq!(conditional.apply(-5, 3), -15); // Condition not satisfied, execute multiply
    /// ```
    pub fn or_else<F>(self, else_transformer: F) -> BoxStatefulBiTransformer<T, U, R>
    where
        F: StatefulBiTransformer<T, U, R> + 'static,
    {
        let pred = self.predicate;
        let mut then_trans = self.transformer;
        let mut else_trans = else_transformer;
        BoxStatefulBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_trans.apply(t, u)
            }
        })
    }
}

// ============================================================================
// ArcStatefulBiTransformer - Arc<dyn FnMut(T, U) -> R + Send + Sync>
// ============================================================================

/// ArcStatefulBiTransformer - thread-safe bi-transformer wrapper
///
/// A thread-safe, clonable bi-transformer wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn FnMut(T, U) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct ArcStatefulBiTransformer<T, U, R> {
    function: Arc<Mutex<dyn FnMut(T, U) -> R + Send>>,
}

impl<T, U, R> ArcStatefulBiTransformer<T, U, R>
where
    T: Send + 'static,
    U: Send + 'static,
    R: 'static,
{
    /// Creates a new ArcStatefulBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T, U) -> R + Send + 'static,
    {
        ArcStatefulBiTransformer {
            function: Arc::new(Mutex::new(f)),
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
    /// A new `ArcStatefulBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer, ArcTransformer};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
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
    /// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer, ArcTransformer};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
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
    pub fn and_then<S, F>(&self, after: F) -> ArcStatefulBiTransformer<T, U, S>
    where
        S: Send + 'static,
        R: Send + Sync + 'static,
        F: StatefulTransformer<R, S> + Send + 'static,
    {
        let self_fn = self.function.clone();
        let mut after_arc = after.into_arc();
        ArcStatefulBiTransformer::new(move |t: T, u: U| {
            let mut func = self_fn.lock().unwrap();
            let intermediate = func(t, u);
            after_arc.apply(intermediate)
        })
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
    /// Returns `ArcConditionalStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = ArcStatefulBiTransformer::new(|x: i32, y: i32| x * y);
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
    /// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer, ArcBiPredicate};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = ArcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(ArcStatefulBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(&self, predicate: P) -> ArcConditionalStatefulBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        ArcConditionalStatefulBiTransformer {
            transformer: self.clone(),
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, U, R> ArcStatefulBiTransformer<T, U, R>
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
    /// use prism3_function::{ArcStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let constant = ArcStatefulBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> ArcStatefulBiTransformer<T, U, R>
    where
        R: Send + Sync,
    {
        ArcStatefulBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> StatefulBiTransformer<T, U, R> for ArcStatefulBiTransformer<T, U, R> {
    fn apply(&mut self, first: T, second: U) -> R {
        let mut func = self.function.lock().unwrap();
        func(first, second)
    }

    fn into_box(self) -> BoxStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxStatefulBiTransformer::new(move |t, u| {
            let mut func = self.function.lock().unwrap();
            func(t, u)
        })
    }

    fn into_rc(self) -> RcStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcStatefulBiTransformer::new(move |t, u| {
            let mut func = self.function.lock().unwrap();
            func(t, u)
        })
    }

    fn into_arc(self) -> ArcStatefulBiTransformer<T, U, R>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| {
            let mut func = self.function.lock().unwrap();
            func(t, u)
        }
    }

    fn to_arc(&self) -> ArcStatefulBiTransformer<T, U, R>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.clone()
    }
}

impl<T, U, R> Clone for ArcStatefulBiTransformer<T, U, R> {
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
        }
    }
}


// ============================================================================
// ArcStatefulBiTransformer BiTransformerOnce Implementation
// ============================================================================

impl<T, U, R> BiTransformerOnce<T, U, R> for ArcStatefulBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
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
    fn apply_once(self, first: T, second: U) -> R {
        let mut func = self.function.lock().unwrap();
        func(first, second)
    }

    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(move |t: T, u: U| {
            let mut func = self.function.lock().unwrap();
            func(t, u)
        })
    }

    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| {
            let mut func = self.function.lock().unwrap();
            func(t, u)
        }
    }

    fn to_box_once(&self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiTransformerOnce::new(move |t: T, u: U| {
            let mut func = self_fn.lock().unwrap();
            func(t, u)
        })
    }

    fn to_fn_once(&self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t: T, u: U| {
            let mut func = self_fn.lock().unwrap();
            func(t, u)
        }
    }
}


// ============================================================================
// ArcConditionalStatefulBiTransformer - Arc-based Conditional StatefulBiTransformer
// ============================================================================

/// ArcConditionalStatefulBiTransformer struct
///
/// A thread-safe conditional bi-transformer that only executes when a
/// bi-predicate is satisfied. Uses `ArcStatefulBiTransformer` and `ArcBiPredicate` for
/// shared ownership across threads.
///
/// This type is typically created by calling `ArcStatefulBiTransformer::when()` and is
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
/// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer};
///
/// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = ArcStatefulBiTransformer::new(|x: i32, y: i32| x * y);
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
pub struct ArcConditionalStatefulBiTransformer<T, U, R> {
    transformer: ArcStatefulBiTransformer<T, U, R>,
    predicate: ArcBiPredicate<T, U>,
}

impl<T, U, R> ArcConditionalStatefulBiTransformer<T, U, R>
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
    ///   - `ArcStatefulBiTransformer<T, U, R>`, `BoxStatefulBiTransformer<T, U, R>`
    ///   - Any type implementing `StatefulBiTransformer<T, U, R> + Send + Sync`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, ArcStatefulBiTransformer};
    ///
    /// let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional.apply(-5, 3), -15);
    /// ```
    pub fn or_else<F>(&self, else_transformer: F) -> ArcStatefulBiTransformer<T, U, R>
    where
        F: StatefulBiTransformer<T, U, R> + Send + 'static,
        R: Send + Sync,
    {
        let pred = self.predicate.clone();
        let mut then_trans = self.transformer.clone();
        let mut else_trans = else_transformer;
        ArcStatefulBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_trans.apply(t, u)
            }
        })
    }
}

impl<T, U, R> Clone for ArcConditionalStatefulBiTransformer<T, U, R> {
    /// Clones the conditional bi-transformer
    ///
    /// Creates a new instance that shares the underlying bi-transformer and
    /// bi-predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            transformer: self.transformer.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcStatefulBiTransformer - Rc<dyn FnMut(T, U) -> R>
// ============================================================================

/// RcStatefulBiTransformer - single-threaded bi-transformer wrapper
///
/// A single-threaded, clonable bi-transformer wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<dyn FnMut(T, U) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct RcStatefulBiTransformer<T, U, R> {
    function: Rc<RefCell<dyn FnMut(T, U) -> R>>,
}

impl<T, U, R> RcStatefulBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
    /// Creates a new RcStatefulBiTransformer
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// assert_eq!(add.apply(20, 22), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T, U) -> R + 'static,
    {
        RcStatefulBiTransformer {
            function: Rc::new(RefCell::new(f)),
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
    /// A new `RcStatefulBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer, RcTransformer};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
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
    /// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer, RcTransformer};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
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
    pub fn and_then<S, F>(&self, after: F) -> RcStatefulBiTransformer<T, U, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
    {
        let self_fn = self.function.clone();
        let mut after_mut = after;
        RcStatefulBiTransformer::new(move |t: T, u: U| {
            let intermediate = self_fn.borrow_mut()(t, u);
            after_mut.apply(intermediate)
        })
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
    /// Returns `RcConditionalStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let multiply = RcStatefulBiTransformer::new(|x: i32, y: i32| x * y);
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
    /// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer, RcBiPredicate};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let both_positive = RcBiPredicate::new(|x: &i32, y: &i32|
    ///     *x > 0 && *y > 0);
    ///
    /// // Clone to preserve original bi-predicate
    /// let conditional = add.when(both_positive.clone())
    ///     .or_else(RcStatefulBiTransformer::new(|x, y| x * y));
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    ///
    /// // Original bi-predicate still usable
    /// assert!(both_positive.test(&5, &3));
    /// ```
    pub fn when<P>(&self, predicate: P) -> RcConditionalStatefulBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
    {
        RcConditionalStatefulBiTransformer {
            transformer: self.clone(),
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, U, R> RcStatefulBiTransformer<T, U, R>
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
    /// use prism3_function::{RcStatefulBiTransformer, StatefulBiTransformer};
    ///
    /// let constant = RcStatefulBiTransformer::constant("hello");
    /// assert_eq!(constant.apply(123, 456), "hello");
    /// ```
    pub fn constant(value: R) -> RcStatefulBiTransformer<T, U, R> {
        RcStatefulBiTransformer::new(move |_, _| value.clone())
    }
}

impl<T, U, R> StatefulBiTransformer<T, U, R> for RcStatefulBiTransformer<T, U, R> {
    fn apply(&mut self, first: T, second: U) -> R {
        let mut self_fn = self.function.borrow_mut();
        self_fn(first, second)
    }

    fn into_box(self) -> BoxStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxStatefulBiTransformer::new(move |t, u| {
            let mut self_fn = self.function.borrow_mut();
            self_fn(t, u)
        })
    }

    fn into_rc(self) -> RcStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    // do NOT override RcStatefulBiTransformer::into_arc() because RcStatefulBiTransformer is not Send + Sync
    // and calling RcStatefulBiTransformer::into_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| {
            let mut self_fn = self.function.borrow_mut();
            self_fn(t, u)
        }
    }

    fn to_rc(&self) -> RcStatefulBiTransformer<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        self.clone()
    }
}

impl<T, U, R> Clone for RcStatefulBiTransformer<T, U, R> {
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
        }
    }
}

// ============================================================================
// RcStatefulBiTransformer StatefulBiTransformerOnce Implementation
// ============================================================================

impl<T, U, R> BiTransformerOnce<T, U, R> for RcStatefulBiTransformer<T, U, R>
where
    T: 'static,
    U: 'static,
    R: 'static,
{
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
    fn apply_once(self, first: T, second: U) -> R {
        let mut func = self.function.borrow_mut();
        func(first, second)
    }

    fn into_box_once(self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxBiTransformerOnce::new(move |t: T, u: U| {
            let mut func = self.function.borrow_mut();
            func(t, u)
        })
    }

    fn into_fn_once(self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        move |t: T, u: U| {
            let mut func = self.function.borrow_mut();
            func(t, u)
        }
    }

    fn to_box_once(&self) -> BoxBiTransformerOnce<T, U, R>
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        BoxBiTransformerOnce::new(move |t: T, u: U| {
            let mut func = self_fn.borrow_mut();
            func(t, u)
        })
    }

    fn to_fn_once(&self) -> impl FnOnce(T, U) -> R
    where
        T: 'static,
        U: 'static,
        R: 'static,
    {
        let self_fn = self.function.clone();
        move |t: T, u: U| {
            let mut func = self_fn.borrow_mut();
            func(t, u)
        }
    }
}

// ============================================================================
// RcConditionalStatefulBiTransformer - Rc-based Conditional StatefulBiTransformer
// ============================================================================

/// RcConditionalStatefulBiTransformer struct
///
/// A single-threaded conditional bi-transformer that only executes when a
/// bi-predicate is satisfied. Uses `RcStatefulBiTransformer` and `RcBiPredicate` for
/// shared ownership within a single thread.
///
/// This type is typically created by calling `RcStatefulBiTransformer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only transforms when bi-predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalStatefulBiTransformer`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer};
///
/// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
/// let multiply = RcStatefulBiTransformer::new(|x: i32, y: i32| x * y);
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
pub struct RcConditionalStatefulBiTransformer<T, U, R> {
    transformer: RcStatefulBiTransformer<T, U, R>,
    predicate: RcBiPredicate<T, U>,
}

impl<T, U, R> RcConditionalStatefulBiTransformer<T, U, R>
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
    ///   - `RcStatefulBiTransformer<T, U, R>`, `BoxStatefulBiTransformer<T, U, R>`
    ///   - Any type implementing `StatefulBiTransformer<T, U, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, RcStatefulBiTransformer};
    ///
    /// let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    /// let conditional = add.when(|x: &i32, y: &i32| *x > 0).or_else(|x: i32, y: i32| x * y);
    ///
    /// assert_eq!(conditional.apply(5, 3), 8);
    /// assert_eq!(conditional.apply(-5, 3), -15);
    /// ```
    pub fn or_else<F>(&self, else_transformer: F) -> RcStatefulBiTransformer<T, U, R>
    where
        F: StatefulBiTransformer<T, U, R> + 'static,
    {
        let pred = self.predicate.clone();
        let mut then_trans = self.transformer.clone();
        let mut else_trans = else_transformer;
        RcStatefulBiTransformer::new(move |t, u| {
            if pred.test(&t, &u) {
                then_trans.apply(t, u)
            } else {
                else_trans.apply(t, u)
            }
        })
    }
}

impl<T, U, R> Clone for RcConditionalStatefulBiTransformer<T, U, R> {
    /// Clones the conditional bi-transformer
    ///
    /// Creates a new instance that shares the underlying bi-transformer and
    /// bi-predicate with the original instance.
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

/// Implement StatefulBiTransformer<T, U, R> for any type that implements FnMut(T, U) -> R
///
/// This allows closures and function pointers to be used directly with our
/// StatefulBiTransformer trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::StatefulBiTransformer;
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
/// Haixing Hu
impl<F, T, U, R> StatefulBiTransformer<T, U, R> for F
where
    F: FnMut(T, U) -> R,
    T: 'static,
    U: 'static,
    R: 'static,
{
    fn apply(&mut self, first: T, second: U) -> R {
        self(first, second)
    }

    fn into_box(self) -> BoxStatefulBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
    {
        BoxStatefulBiTransformer::new(self)
    }

    fn into_rc(self) -> RcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + 'static,
    {
        RcStatefulBiTransformer::new(self)
    }

    fn into_arc(self) -> ArcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
        R: Send + 'static,
    {
        ArcStatefulBiTransformer::new(self)
    }

    fn into_fn(self) -> impl FnMut(T, U) -> R
    where
        Self: Sized + 'static,
    {
        self
    }

    fn to_box(&self) -> BoxStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxStatefulBiTransformer::new(self.clone())
    }

    fn to_rc(&self) -> RcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        RcStatefulBiTransformer::new(self.clone())
    }

    fn to_arc(&self) -> ArcStatefulBiTransformer<T, U, R>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcStatefulBiTransformer::new(self.clone())
    }

    fn to_fn(&self) -> impl FnMut(T, U) -> R
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
// FnStatefulBiTransformerOps - Extension trait for FnMut(T, U) -> R bi-transformers
// ============================================================================

/// Extension trait for closures implementing `FnMut(T, U) -> R`
///
/// Provides composition methods (`and_then`, `when`) for bi-transformer
/// closures and function pointers without requiring explicit wrapping in
/// `BoxStatefulBiTransformer`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnMut(T, U) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `StatefulBiTransformer<T, U, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then` and `when`. This extension trait provides those methods,
/// returning `BoxStatefulBiTransformer` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps};
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
/// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps};
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
/// Haixing Hu
pub trait FnStatefulBiTransformerOps<T, U, R>: FnMut(T, U) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new bi-transformer that applies this bi-transformer first,
    /// then applies the after transformer to the result. Consumes self and
    /// returns a `BoxStatefulBiTransformer`.
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
    /// A new `BoxStatefulBiTransformer<T, U, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ## Direct value passing (ownership transfer)
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps,
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
    /// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps,
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
    fn and_then<S, F>(self, after: F) -> BoxStatefulBiTransformer<T, U, S>
    where
        S: 'static,
        F: StatefulTransformer<R, S> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxStatefulBiTransformer::new(self).and_then(after)
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
    /// Returns `BoxConditionalStatefulBiTransformer<T, U, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps};
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
    /// use prism3_function::{StatefulBiTransformer, FnStatefulBiTransformerOps,
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
    fn when<P>(self, predicate: P) -> BoxConditionalStatefulBiTransformer<T, U, R>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
        R: 'static,
    {
        BoxStatefulBiTransformer::new(self).when(predicate)
    }
}

/// Blanket implementation of FnStatefulBiTransformerOps for all closures
///
/// Automatically implements `FnStatefulBiTransformerOps<T, U, R>` for any type that
/// implements `FnMut(T, U) -> R`.
///
/// # Author
///
/// Haixing Hu
impl<T, U, R, F> FnStatefulBiTransformerOps<T, U, R> for F where F: FnMut(T, U) -> R + 'static {}

// ============================================================================
// BinaryOperator Trait - Marker trait for StatefulBiTransformer<T, T, T>
// ============================================================================

/// BinaryOperator trait - marker trait for binary operators
///
/// A binary operator takes two values of type `T` and produces a value of the
/// same type `T`. This trait extends `StatefulBiTransformer<T, T, T>` to provide
/// semantic clarity for same-type binary operations. Equivalent to Java's
/// `BinaryOperator<T>` which extends `BiFunction<T, T, T>`.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all types that implement
/// `StatefulBiTransformer<T, T, T>`, so you don't need to implement it manually.
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
/// use prism3_function::{BinaryOperator, StatefulBiTransformer};
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
/// use prism3_function::{BoxBinaryOperator, BinaryOperator, StatefulBiTransformer};
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
/// Haixing Hu
pub trait BinaryOperator<T>: StatefulBiTransformer<T, T, T> {}

/// Blanket implementation of BinaryOperator for all StatefulBiTransformer<T, T, T>
///
/// This automatically implements `BinaryOperator<T>` for any type that
/// implements `StatefulBiTransformer<T, T, T>`.
///
/// # Author
///
/// Haixing Hu
impl<F, T> BinaryOperator<T> for F
where
    F: StatefulBiTransformer<T, T, T>,
    T: 'static,
{
    // empty
}

// ============================================================================
// Type Aliases for BinaryOperator (StatefulBiTransformer<T, T, T>)
// ============================================================================

/// Type alias for `BoxStatefulBiTransformer<T, T, T>`
///
/// Represents a binary operator that takes two values of type `T` and produces
/// a value of the same type `T`, with single ownership semantics. Equivalent to
/// Java's `BinaryOperator<T>`.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxBinaryOperator, StatefulBiTransformer};
///
/// let add: BoxBinaryOperator<i32> = BoxBinaryOperator::new(|x, y| x + y);
/// assert_eq!(add.apply(20, 22), 42);
/// ```
///
/// # Author
///
/// Haixing Hu
pub type BoxBinaryOperator<T> = BoxStatefulBiTransformer<T, T, T>;

/// Type alias for `ArcStatefulBiTransformer<T, T, T>`
///
/// Represents a thread-safe binary operator that takes two values of type `T`
/// and produces a value of the same type `T`. Equivalent to Java's
/// `BinaryOperator<T>` with shared, thread-safe ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ArcBinaryOperator, StatefulBiTransformer};
///
/// let multiply: ArcBinaryOperator<i32> = ArcBinaryOperator::new(|x, y| x * y);
/// let multiply_clone = multiply.clone();
/// assert_eq!(multiply.apply(6, 7), 42);
/// assert_eq!(multiply_clone.apply(6, 7), 42);
/// ```
///
/// # Author
///
/// Haixing Hu
pub type ArcBinaryOperator<T> = ArcStatefulBiTransformer<T, T, T>;

/// Type alias for `RcStatefulBiTransformer<T, T, T>`
///
/// Represents a single-threaded binary operator that takes two values of type
/// `T` and produces a value of the same type `T`. Equivalent to Java's
/// `BinaryOperator<T>` with shared, single-threaded ownership.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{RcBinaryOperator, StatefulBiTransformer};
///
/// let max: RcBinaryOperator<i32> = RcBinaryOperator::new(|x, y| if x > y { x } else { y });
/// let max_clone = max.clone();
/// assert_eq!(max.apply(30, 42), 42);
/// assert_eq!(max_clone.apply(30, 42), 42);
/// ```
///
/// # Author
///
/// Haixing Hu
pub type RcBinaryOperator<T> = RcStatefulBiTransformer<T, T, T>;
