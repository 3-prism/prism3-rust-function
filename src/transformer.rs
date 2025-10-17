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
//! Provides Rust implementations of immutable transformer traits that
//! transform values from type T to the same type T. This is a specialization
//! of `Function<T, T>` with additional convenience methods.
//!
//! This module provides the `Transformer<T>` trait and three implementations:
//!
//! - [`BoxTransformer`]: Single ownership, not cloneable
//! - [`ArcTransformer`]: Thread-safe shared ownership, cloneable
//! - [`RcTransformer`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

use crate::function::Function;

// ============================================================================
// Core Trait
// ============================================================================

/// Transformer trait - immutable self-transformation that borrows input
///
/// Defines the behavior of an immutable transformation: converting a borrowed
/// value of type `&T` to a value of the same type `T`. This trait extends
/// `Function<T, T>` with specialized transformation semantics.
///
/// The core method is `transform()`, which provides the transformation logic.
/// Implementations should have `apply()` call `transform()` to maintain
/// consistency.
///
/// # Type Parameters
///
/// * `T` - The type of both input and output values
///
/// # Author
///
/// Haixing Hu
pub trait Transformer<T>: Function<T, T> {
    /// Applies the transformation to the input value
    ///
    /// This is the core method that must be implemented.
    ///
    /// # Parameters
    ///
    /// * `input` - Borrowed reference to the input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(&self, input: &T) -> T;
}

// ============================================================================
// BoxTransformer - Box<dyn Fn(&T) -> T>
// ============================================================================

/// BoxTransformer - immutable transformer wrapper based on `Box<dyn Fn>`
///
/// A transformer wrapper that provides single ownership with reusable
/// immutable transformation. The transformer borrows the input and can be
/// called multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn Fn(&T) -> T>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Haixing Hu
pub struct BoxTransformer<T> {
    f: Box<dyn Fn(&T) -> T>,
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
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: &i32| x * 2);
    /// assert_eq!(double.transform(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> T + 'static,
    {
        BoxTransformer { f: Box::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let identity = BoxTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(&42), 42);
    /// ```
    pub fn identity() -> BoxTransformer<T>
    where
        T: Clone,
    {
        BoxTransformer::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Consumes self.
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
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: &i32| x * 2);
    /// let add_one = BoxTransformer::new(|x: &i32| x + 1);
    /// let composed = double.and_then(add_one);
    /// assert_eq!(composed.transform(&5), 11); // 5 * 2 + 1
    /// ```
    pub fn and_then(self, after: BoxTransformer<T>) -> BoxTransformer<T> {
        let self_f = self.f;
        let after_f = after.f;
        BoxTransformer::new(move |x: &T| after_f(&self_f(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self.
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
    /// use prism3_function::BoxTransformer;
    ///
    /// let double = BoxTransformer::new(|x: &i32| x * 2);
    /// let add_one = BoxTransformer::new(|x: &i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(self, before: BoxTransformer<T>) -> BoxTransformer<T> {
        let self_f = self.f;
        let before_f = before.f;
        BoxTransformer::new(move |x: &T| self_f(&before_f(x)))
    }
}

impl<T> BoxTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformer;
    ///
    /// let constant = BoxTransformer::constant(42);
    /// assert_eq!(constant.transform(&123), 42);
    /// ```
    pub fn constant(value: T) -> BoxTransformer<T> {
        BoxTransformer::new(move |_| value.clone())
    }
}

impl<T> Transformer<T> for BoxTransformer<T>
where
    T: 'static,
{
    fn transform(&self, input: &T) -> T {
        (self.f)(input)
    }
}

impl<T: 'static> Function<T, T> for BoxTransformer<T> {
    fn apply(&self, input: &T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function::BoxFunction<T, T> {
        crate::function::BoxFunction::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function::RcFunction<T, T> {
        crate::function::RcFunction::new(move |x| self.apply(x))
    }

    fn into_arc(self) -> crate::function::ArcFunction<T, T>
    where
        Self: Send + Sync,
        T: Send + Sync,
    {
        unreachable!(
            "BoxTransformer<T> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> T {
        move |t: &T| self.apply(t)
    }
}

// ============================================================================
// ArcTransformer - Arc<dyn Fn(&T) -> T + Send + Sync>
// ============================================================================

/// ArcTransformer - thread-safe immutable transformer wrapper
///
/// A thread-safe, clonable transformer wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(&T) -> T + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct ArcTransformer<T> {
    f: Arc<dyn Fn(&T) -> T + Send + Sync>,
}

impl<T> ArcTransformer<T>
where
    T: 'static,
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
    /// use prism3_function::ArcTransformer;
    ///
    /// let double = ArcTransformer::new(|x: &i32| x * 2);
    /// assert_eq!(double.transform(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> T + Send + Sync + 'static,
    {
        ArcTransformer { f: Arc::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformer;
    ///
    /// let identity = ArcTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(&42), 42);
    /// ```
    pub fn identity() -> ArcTransformer<T>
    where
        T: Clone + Send + Sync,
    {
        ArcTransformer::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformer;
    ///
    /// let double = ArcTransformer::new(|x: &i32| x * 2);
    /// let add_one = ArcTransformer::new(|x: &i32| x + 1);
    /// let composed = double.and_then(&add_one);
    ///
    /// // Original transformers still usable
    /// assert_eq!(double.transform(&21), 42);
    /// assert_eq!(composed.transform(&5), 11);
    /// ```
    pub fn and_then(&self, after: &ArcTransformer<T>) -> ArcTransformer<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcTransformer {
            f: Arc::new(move |x: &T| after_clone(&self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new ArcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformer;
    ///
    /// let double = ArcTransformer::new(|x: &i32| x * 2);
    /// let add_one = ArcTransformer::new(|x: &i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(&self, before: &ArcTransformer<T>) -> ArcTransformer<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcTransformer {
            f: Arc::new(move |x: &T| self_clone(&before_clone(x))),
        }
    }
}

impl<T> ArcTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformer;
    ///
    /// let constant = ArcTransformer::constant(42);
    /// assert_eq!(constant.transform(&123), 42);
    /// ```
    pub fn constant(value: T) -> ArcTransformer<T>
    where
        T: Send + Sync,
    {
        ArcTransformer::new(move |_| value.clone())
    }
}

impl<T> Transformer<T> for ArcTransformer<T>
where
    T: 'static,
{
    fn transform(&self, input: &T) -> T {
        (self.f)(input)
    }
}

impl<T> Function<T, T> for ArcTransformer<T>
where
    T: 'static,
{
    fn apply(&self, input: &T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function::BoxFunction<T, T> {
        crate::function::BoxFunction::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function::RcFunction<T, T> {
        crate::function::RcFunction::new(move |x| self.apply(x))
    }

    fn into_arc(self) -> crate::function::ArcFunction<T, T>
    where
        T: Send + Sync,
    {
        crate::function::ArcFunction::new(move |x| self.apply(x))
    }

    fn into_fn(self) -> impl FnMut(&T) -> T {
        move |t: &T| self.apply(t)
    }
}

impl<T> Clone for ArcTransformer<T> {
    fn clone(&self) -> Self {
        ArcTransformer {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcTransformer - Rc<dyn Fn(&T) -> T>
// ============================================================================

/// RcTransformer - single-threaded immutable transformer wrapper
///
/// A single-threaded, clonable transformer wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(&T) -> T>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct RcTransformer<T> {
    f: Rc<dyn Fn(&T) -> T>,
}

impl<T> RcTransformer<T>
where
    T: 'static,
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
    /// use prism3_function::RcTransformer;
    ///
    /// let double = RcTransformer::new(|x: &i32| x * 2);
    /// assert_eq!(double.transform(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> T + 'static,
    {
        RcTransformer { f: Rc::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformer;
    ///
    /// let identity = RcTransformer::<i32>::identity();
    /// assert_eq!(identity.transform(&42), 42);
    /// ```
    pub fn identity() -> RcTransformer<T>
    where
        T: Clone,
    {
        RcTransformer::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformer;
    ///
    /// let double = RcTransformer::new(|x: &i32| x * 2);
    /// let add_one = RcTransformer::new(|x: &i32| x + 1);
    /// let composed = double.and_then(&add_one);
    ///
    /// // Original transformers still usable
    /// assert_eq!(double.transform(&21), 42);
    /// assert_eq!(composed.transform(&5), 11);
    /// ```
    pub fn and_then(&self, after: &RcTransformer<T>) -> RcTransformer<T> {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcTransformer {
            f: Rc::new(move |x: &T| after_clone(&self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable.
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new RcTransformer representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformer;
    ///
    /// let double = RcTransformer::new(|x: &i32| x * 2);
    /// let add_one = RcTransformer::new(|x: &i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(&self, before: &RcTransformer<T>) -> RcTransformer<T> {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcTransformer {
            f: Rc::new(move |x: &T| self_clone(&before_clone(x))),
        }
    }
}

impl<T> RcTransformer<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformer;
    ///
    /// let constant = RcTransformer::constant(42);
    /// assert_eq!(constant.transform(&123), 42);
    /// ```
    pub fn constant(value: T) -> RcTransformer<T> {
        RcTransformer::new(move |_| value.clone())
    }
}

impl<T> Transformer<T> for RcTransformer<T>
where
    T: 'static,
{
    fn transform(&self, input: &T) -> T {
        (self.f)(input)
    }
}

impl<T> Function<T, T> for RcTransformer<T>
where
    T: 'static,
{
    fn apply(&self, input: &T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function::BoxFunction<T, T> {
        crate::function::BoxFunction::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function::RcFunction<T, T> {
        crate::function::RcFunction::new(move |x| self.apply(x))
    }

    fn into_arc(self) -> crate::function::ArcFunction<T, T>
    where
        Self: Send + Sync,
        T: Send + Sync,
    {
        unreachable!(
            "RcTransformer cannot be converted to ArcFunction because Rc is \
             not Send + Sync"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> T {
        move |t: &T| self.apply(t)
    }
}

impl<T> Clone for RcTransformer<T> {
    fn clone(&self) -> Self {
        RcTransformer {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// Blanket implementation for standard Fn trait
// ============================================================================

/// Implement Transformer<T> for any type that implements Fn(&T) -> T
///
/// This allows closures and function pointers to be used directly with our
/// Transformer trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::Transformer;
///
/// fn double(x: &i32) -> i32 { x * 2 }
///
/// assert_eq!(double.transform(&21), 42);
///
/// let triple = |x: &i32| x * 3;
/// assert_eq!(triple.transform(&14), 42);
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T> Transformer<T> for F
where
    F: Fn(&T) -> T,
    T: 'static,
{
    fn transform(&self, input: &T) -> T {
        self(input)
    }
}
