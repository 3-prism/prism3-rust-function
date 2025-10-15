/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # FunctionOnce Types
//!
//! Provides Rust implementations of consuming function traits similar to Rust's
//! `FnOnce` trait, but with value-oriented semantics for functional programming patterns.
//!
//! This module provides the `FunctionOnce<T, R>` trait and three implementations:
//!
//! - [`BoxFunctionOnce`]: Single ownership, one-time use
//! - [`ArcFunctionOnce`]: Thread-safe shared ownership, reusable (cloneable)
//! - [`RcFunctionOnce`]: Single-threaded shared ownership, reusable (cloneable)
//!
//! # Author
//!
//! Hu Haixing

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ============================================================================
// Core Trait
// ============================================================================

/// FunctionOnce trait - consuming transformation that takes ownership
///
/// Defines the behavior of a consuming function transformation: converting
/// a value of type `T` to a value of type `R` by taking ownership. This
/// trait is analogous to `FnOnce(T) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait FunctionOnce<T, R> {
    /// Applies the function to the input value, consuming both self and input
    ///
    /// # Parameters
    ///
    /// * `input` - The input value (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(self, input: T) -> R;

    /// Converts to BoxFunctionOnce
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxFunctionOnce<T, R>`
    fn into_box(self) -> BoxFunctionOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to RcFunctionOnce
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcFunctionOnce<T, R>`
    fn into_rc(self) -> RcFunctionOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to ArcFunctionOnce
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcFunctionOnce<T, R>`
    fn into_arc(self) -> ArcFunctionOnce<T, R>
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
    /// Returns a closure that implements `FnMut(T) -> R`
    fn into_fn(self) -> impl FnMut(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
}

// ============================================================================
// BoxFunctionOnce - Box<dyn FnOnce(T) -> R>
// ============================================================================

/// BoxFunctionOnce - consuming function wrapper based on `Box<dyn FnOnce>`
///
/// A function wrapper that provides single ownership with one-time use
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
pub struct BoxFunctionOnce<T, R> {
    f: Box<dyn FnOnce(T) -> R>,
}

impl<T, R> BoxFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxFunctionOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionOnce;
    ///
    /// let parse = BoxFunctionOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.apply("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxFunctionOnce { f: Box::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionOnce;
    ///
    /// let identity = BoxFunctionOnce::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> BoxFunctionOnce<T, T> {
        BoxFunctionOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionOnce;
    ///
    /// let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxFunctionOnce<T, S>
    where
        S: 'static,
        G: FnOnce(R) -> S + 'static,
    {
        BoxFunctionOnce::new(move |x| after((self.f)(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionOnce;
    ///
    /// let double = BoxFunctionOnce::new(|x: i32| x * 2);
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxFunctionOnce<S, R>
    where
        S: 'static,
        G: FnOnce(S) -> T + 'static,
    {
        BoxFunctionOnce::new(move |x| (self.f)(before(x)))
    }
}

impl<T, R> BoxFunctionOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionOnce;
    ///
    /// let constant = BoxFunctionOnce::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxFunctionOnce<T, R> {
        BoxFunctionOnce::new(move |_| value.clone())
    }
}

impl<T, R> FunctionOnce<T, R> for BoxFunctionOnce<T, R> {
    fn apply(self, input: T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Cannot directly convert Box<dyn FnOnce> to Rc<dyn Fn>
        // We need to wrap it in a way that makes it reusable
        let func = Rc::new(RefCell::new(Some(self)));
        RcFunctionOnce {
            f: Rc::new(move |x| {
                func.borrow_mut()
                    .take()
                    .expect("RcFunctionOnce can only be called once")
                    .apply(x)
            }),
        }
    }

    fn into_arc(self) -> ArcFunctionOnce<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "BoxFunctionOnce<T, R> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let mut func = Some(self);
        move |t: T| {
            func.take()
                .expect("BoxFunctionOnce can only be called once")
                .apply(t)
        }
    }
}

// ============================================================================
// ArcFunctionOnce - Arc<dyn Fn(T) -> R + Send + Sync> (reusable version)
// ============================================================================

/// ArcFunctionOnce - thread-safe reusable consuming function wrapper
///
/// Note: Despite the name, this is actually reusable because it's based on
/// `Fn(T) -> R` rather than `FnOnce(T) -> R`. This allows it to be cloned
/// and shared across threads while still consuming the input value.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(T) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (but consumes input each
///   time)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct ArcFunctionOnce<T, R> {
    f: Arc<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> ArcFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new ArcFunctionOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionOnce;
    ///
    /// let parse = ArcFunctionOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.apply("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        ArcFunctionOnce { f: Arc::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionOnce;
    ///
    /// let identity = ArcFunctionOnce::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> ArcFunctionOnce<T, T> {
        ArcFunctionOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable (but each apply still consumes its input).
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
    /// A new ArcFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionOnce;
    ///
    /// let double = ArcFunctionOnce::new(|x: i32| x * 2);
    /// let to_string = ArcFunctionOnce::new(|x: i32| x.to_string());
    /// let composed = double.and_then(&to_string);
    ///
    /// // Both original and composed can be used
    /// assert_eq!(double.apply(21), 42);
    /// assert_eq!(composed.apply(21), "42");
    /// ```
    pub fn and_then<S>(
        &self,
        after: &ArcFunctionOnce<R, S>,
    ) -> ArcFunctionOnce<T, S>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcFunctionOnce {
            f: Arc::new(move |x: T| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable (but each apply still consumes its input).
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
    /// A new ArcFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionOnce;
    ///
    /// let double = ArcFunctionOnce::new(|x: i32| x * 2);
    /// let add_one = ArcFunctionOnce::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S>(
        &self,
        before: &ArcFunctionOnce<S, T>,
    ) -> ArcFunctionOnce<S, R>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcFunctionOnce {
            f: Arc::new(move |x: S| self_clone(before_clone(x))),
        }
    }
}

impl<T, R> ArcFunctionOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionOnce;
    ///
    /// let constant = ArcFunctionOnce::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcFunctionOnce<T, R>
    where
        R: Send + Sync,
    {
        ArcFunctionOnce::new(move |_| value.clone())
    }
}

impl<T, R> FunctionOnce<T, R> for ArcFunctionOnce<T, R> {
    fn apply(self, input: T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce {
            f: Box::new(move |x| self.apply(x)),
        }
    }

    fn into_rc(self) -> RcFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        let arc = self.f;
        RcFunctionOnce {
            f: Rc::new(move |x| arc(x)),
        }
    }

    fn into_arc(self) -> ArcFunctionOnce<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| {
            let func = self.clone();
            func.apply(t)
        }
    }
}

impl<T, R> Clone for ArcFunctionOnce<T, R> {
    fn clone(&self) -> Self {
        ArcFunctionOnce {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcFunctionOnce - Rc<dyn Fn(T) -> R> (reusable version)
// ============================================================================

/// RcFunctionOnce - single-threaded reusable consuming function wrapper
///
/// Note: Despite the name, this is actually reusable because it's based on
/// `Fn(T) -> R` rather than `FnOnce(T) -> R`. This allows it to be cloned
/// while still consuming the input value.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(T) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (but consumes input each
///   time)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct RcFunctionOnce<T, R> {
    f: Rc<dyn Fn(T) -> R>,
}

impl<T, R> RcFunctionOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcFunctionOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionOnce;
    ///
    /// let parse = RcFunctionOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.apply("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        RcFunctionOnce { f: Rc::new(f) }
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionOnce;
    ///
    /// let identity = RcFunctionOnce::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> RcFunctionOnce<T, T> {
        RcFunctionOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new function that applies this function first, then applies
    /// the after function to the result. Uses &self, so original function
    /// remains usable (but each apply still consumes its input).
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
    /// A new RcFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionOnce;
    ///
    /// let double = RcFunctionOnce::new(|x: i32| x * 2);
    /// let to_string = RcFunctionOnce::new(|x: i32| x.to_string());
    /// let composed = double.and_then(&to_string);
    ///
    /// // Both original and composed can be used
    /// assert_eq!(double.apply(21), 42);
    /// assert_eq!(composed.apply(21), "42");
    /// ```
    pub fn and_then<S>(
        &self,
        after: &RcFunctionOnce<R, S>,
    ) -> RcFunctionOnce<T, S>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcFunctionOnce {
            f: Rc::new(move |x: T| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new function that applies the before function first, then
    /// applies this function to the result. Uses &self, so original function
    /// remains usable (but each apply still consumes its input).
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
    /// A new RcFunctionOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionOnce;
    ///
    /// let double = RcFunctionOnce::new(|x: i32| x * 2);
    /// let add_one = RcFunctionOnce::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<S>(
        &self,
        before: &RcFunctionOnce<S, T>,
    ) -> RcFunctionOnce<S, R>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcFunctionOnce {
            f: Rc::new(move |x: S| self_clone(before_clone(x))),
        }
    }
}

impl<T, R> RcFunctionOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionOnce;
    ///
    /// let constant = RcFunctionOnce::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> RcFunctionOnce<T, R> {
        RcFunctionOnce::new(move |_| value.clone())
    }
}

impl<T, R> FunctionOnce<T, R> for RcFunctionOnce<T, R> {
    fn apply(self, input: T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunctionOnce {
            f: Box::new(move |x| self.apply(x)),
        }
    }

    fn into_rc(self) -> RcFunctionOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcFunctionOnce<T, R>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        unreachable!(
            "RcFunctionOnce cannot be converted to ArcFunctionOnce because Rc \
             is not Send + Sync"
        )
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        move |t: T| {
            let func = self.clone();
            func.apply(t)
        }
    }
}

impl<T, R> Clone for RcFunctionOnce<T, R> {
    fn clone(&self) -> Self {
        RcFunctionOnce {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement FunctionOnce<T, R> for any type that implements FnOnce(T) -> R
///
/// This allows once-callable closures and function pointers to be used directly
/// with our FunctionOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::FunctionOnce;
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
/// # 作者
///
/// 胡海星
impl<F, T, R> FunctionOnce<T, R> for F
where
    F: FnOnce(T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply(self, input: T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxFunctionOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxFunctionOnce::new(self)
    }

    fn into_rc(self) -> RcFunctionOnce<T, R>
    where
        Self: Sized + 'static,
    {
        // Wrap in Option to allow taking the function
        let func = std::cell::RefCell::new(Some(self));
        RcFunctionOnce::new(move |t: T| {
            let f = func
                .borrow_mut()
                .take()
                .expect("FunctionOnce can only be called once");
            f(t)
        })
    }

    fn into_arc(self) -> ArcFunctionOnce<T, R>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        // Wrap in Mutex<Option> to allow taking the function in a thread-safe way
        let func = Mutex::new(Some(self));
        ArcFunctionOnce::new(move |t: T| {
            let f = func
                .lock()
                .unwrap()
                .take()
                .expect("FunctionOnce can only be called once");
            f(t)
        })
    }

    fn into_fn(self) -> impl FnMut(T) -> R
    where
        Self: Sized + 'static,
    {
        // Wrap FnOnce in an Option to make it callable once through FnMut
        let mut func = Some(self);
        move |input: T| -> R {
            let f = func.take().expect("FunctionOnce can only be called once");
            f(input)
        }
    }
}

