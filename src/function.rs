/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Function Types
//!
//! Provides Rust implementations of immutable function traits similar to Rust's
//! `Fn` trait, but with value-oriented semantics for functional programming patterns.
//!
//! This module provides the `Function<T, R>` trait and three implementations:
//!
//! - [`BoxFunction`]: Single ownership, not cloneable
//! - [`ArcFunction`]: Thread-safe shared ownership, cloneable
//! - [`RcFunction`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Core Trait
// ============================================================================

/// Function trait - immutable transformation that borrows input
///
/// Defines the behavior of an immutable function transformation: converting
/// a borrowed value of type `&T` to a value of type `R`. This trait is
/// analogous to `Fn(&T) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (borrowed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait Function<T, R> {
    /// Applies the function to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - Borrowed reference to the input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(&self, input: &T) -> R;

    /// Converts to BoxFunction
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxFunction<T, R>`
    fn into_box(self) -> BoxFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to RcFunction
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcFunction<T, R>`
    fn into_rc(self) -> RcFunction<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to ArcFunction
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcFunction<T, R>`
    fn into_arc(self) -> ArcFunction<T, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static;

    /// Converts function to a closure
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(&T) -> R`
    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
}

// ============================================================================
// BoxFunction - Box<dyn Fn(&T) -> R>
// ============================================================================

/// BoxFunction - immutable function wrapper based on `Box<dyn Fn>`
///
/// A function wrapper that provides single ownership with reusable
/// immutable transformation. The function borrows the input and can be
/// called multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn Fn(&T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxFunction<T, R> {
    f: Box<dyn Fn(&T) -> R>,
}

impl<T, R> BoxFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = BoxFunction::new(|x: &i32| x * 2);
    /// assert_eq!(double.apply(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> R + 'static,
    {
        BoxFunction { f: Box::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let identity = BoxFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(&42), 42);
    /// ```
    pub fn identity() -> BoxFunction<T, T>
    where
        T: Clone,
    {
        BoxFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self
    ///
    /// # Returns
    ///
    /// A new BoxFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = BoxFunction::new(|x: &i32| x * 2);
    /// let to_string = BoxFunction::new(|x: &i32| x.to_string());
    /// let composed = double.and_then(to_string);
    /// assert_eq!(composed.apply(&21), "42");
    /// ```
    pub fn and_then<S>(self, after: BoxFunction<R, S>) -> BoxFunction<T, S>
    where
        S: 'static,
    {
        let self_f = self.f;
        let after_f = after.f;
        BoxFunction::new(move |x: &T| after_f(&self_f(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self
    ///
    /// # Returns
    ///
    /// A new BoxFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = BoxFunction::new(|x: &i32| x * 2);
    /// let add_one = BoxFunction::new(|x: &i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S>(self, before: BoxFunction<S, T>) -> BoxFunction<S, R>
    where
        S: 'static,
    {
        let self_f = self.f;
        let before_f = before.f;
        BoxFunction::new(move |x: &S| self_f(&before_f(x)))
    }
}

impl<T, R> BoxFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let constant = BoxFunction::constant("hello");
    /// assert_eq!(constant.apply(&123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxFunction<T, R> {
        BoxFunction::new(move |_| value.clone())
    }
}

impl<T, R> Function<T, R> for BoxFunction<T, R> {
    fn apply(&self, input: &T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcFunction { f: Rc::from(self.f) }
    }

    fn into_arc(self) -> ArcFunction<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "BoxFunction<T, R> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: &T| self.apply(t)
    }
}

// ============================================================================
// ArcFunction - Arc<dyn Fn(&T) -> R + Send + Sync>
// ============================================================================

/// ArcFunction - thread-safe immutable function wrapper
///
/// A thread-safe, clonable function wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(&T) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct ArcFunction<T, R> {
    f: Arc<dyn Fn(&T) -> R + Send + Sync>,
}

impl<T, R> ArcFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new ArcFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunction;
    ///
    /// let double = ArcFunction::new(|x: &i32| x * 2);
    /// assert_eq!(double.apply(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> R + Send + Sync + 'static,
    {
        ArcFunction { f: Arc::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunction;
    ///
    /// let identity = ArcFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(&42), 42);
    /// ```
    pub fn identity() -> ArcFunction<T, T>
    where
        T: Clone + Send + Sync,
    {
        ArcFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self
    ///
    /// # Returns
    ///
    /// A new ArcFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunction;
    ///
    /// let double = ArcFunction::new(|x: &i32| x * 2);
    /// let to_string = ArcFunction::new(|x: &i32| x.to_string());
    /// let composed = double.and_then(&to_string);
    ///
    /// // Original functions still usable
    /// assert_eq!(double.apply(&21), 42);
    /// assert_eq!(composed.apply(&21), "42");
    /// ```
    pub fn and_then<S>(&self, after: &ArcFunction<R, S>) -> ArcFunction<T, S>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcFunction {
            f: Arc::new(move |x: &T| after_clone(&self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self
    ///
    /// # Returns
    ///
    /// A new ArcFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunction;
    ///
    /// let double = ArcFunction::new(|x: &i32| x * 2);
    /// let add_one = ArcFunction::new(|x: &i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S>(&self, before: &ArcFunction<S, T>) -> ArcFunction<S, R>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcFunction {
            f: Arc::new(move |x: &S| self_clone(&before_clone(x))),
        }
    }
}

impl<T, R> ArcFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunction;
    ///
    /// let constant = ArcFunction::constant("hello");
    /// assert_eq!(constant.apply(&123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcFunction<T, R>
    where
        R: Send + Sync,
    {
        ArcFunction::new(move |_| value.clone())
    }
}

impl<T, R> Function<T, R> for ArcFunction<T, R> {
    fn apply(&self, input: &T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunction {
            f: Box::new(move |x| self.apply(x)),
        }
    }

    fn into_rc(self) -> RcFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcFunction {
            f: Rc::new(move |x| self.apply(x)),
        }
    }

    fn into_arc(self) -> ArcFunction<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: &T| self.apply(t)
    }
}

impl<T, R> Clone for ArcFunction<T, R> {
    fn clone(&self) -> Self {
        ArcFunction {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcFunction - Rc<dyn Fn(&T) -> R>
// ============================================================================

/// RcFunction - single-threaded immutable function wrapper
///
/// A single-threaded, clonable function wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(&T) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct RcFunction<T, R> {
    f: Rc<dyn Fn(&T) -> R>,
}

impl<T, R> RcFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunction;
    ///
    /// let double = RcFunction::new(|x: &i32| x * 2);
    /// assert_eq!(double.apply(&21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> R + 'static,
    {
        RcFunction { f: Rc::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunction;
    ///
    /// let identity = RcFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(&42), 42);
    /// ```
    pub fn identity() -> RcFunction<T, T>
    where
        T: Clone,
    {
        RcFunction::new(|x: &T| x.clone())
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after function
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self
    ///
    /// # Returns
    ///
    /// A new RcFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunction;
    ///
    /// let double = RcFunction::new(|x: &i32| x * 2);
    /// let to_string = RcFunction::new(|x: &i32| x.to_string());
    /// let composed = double.and_then(&to_string);
    ///
    /// // Original functions still usable
    /// assert_eq!(double.apply(&21), 42);
    /// assert_eq!(composed.apply(&21), "42");
    /// ```
    pub fn and_then<S>(&self, after: &RcFunction<R, S>) -> RcFunction<T, S>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcFunction {
            f: Rc::new(move |x: &T| after_clone(&self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before function
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self
    ///
    /// # Returns
    ///
    /// A new RcFunction representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunction;
    ///
    /// let double = RcFunction::new(|x: &i32| x * 2);
    /// let add_one = RcFunction::new(|x: &i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S>(&self, before: &RcFunction<S, T>) -> RcFunction<S, R>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcFunction {
            f: Rc::new(move |x: &S| self_clone(&before_clone(x))),
        }
    }
}

impl<T, R> RcFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunction;
    ///
    /// let constant = RcFunction::constant("hello");
    /// assert_eq!(constant.apply(&123), "hello");
    /// ```
    pub fn constant(value: R) -> RcFunction<T, R> {
        RcFunction::new(move |_| value.clone())
    }
}

impl<T, R> Function<T, R> for RcFunction<T, R> {
    fn apply(&self, input: &T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunction {
            f: Box::new(move |x| self.apply(x)),
        }
    }

    fn into_rc(self) -> RcFunction<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcFunction<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "RcFunction cannot be converted to ArcFunction because Rc \
             is not Send + Sync"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: &T| self.apply(t)
    }
}

impl<T, R> Clone for RcFunction<T, R> {
    fn clone(&self) -> Self {
        RcFunction {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// Blanket implementation for standard Fn trait
// ============================================================================

/// Implement Function<T, R> for any type that implements Fn(&T) -> R
///
/// This allows closures and function pointers to be used directly with our
/// Function trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::Function;
///
/// fn double(x: &i32) -> i32 { x * 2 }
///
/// assert_eq!(double.apply(&21), 42);
///
/// let triple = |x: &i32| x * 3;
/// assert_eq!(triple.apply(&14), 42);
/// ```
///
/// # 作者
///
/// 胡海星
impl<F, T, R> Function<T, R> for F
where
    F: Fn(&T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply(&self, input: &T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxFunction<T, R>
    where
        Self: Sized + 'static,
    {
        BoxFunction::new(self)
    }

    fn into_rc(self) -> RcFunction<T, R>
    where
        Self: Sized + 'static,
    {
        RcFunction::new(self)
    }

    fn into_arc(self) -> ArcFunction<T, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        ArcFunction::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T) -> R
    where
        Self: Sized + 'static,
    {
        move |t: &T| self(t)
    }
}
