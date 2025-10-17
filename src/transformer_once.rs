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
//! Provides Rust implementations of consuming transformer traits that
//! transform values from type T to the same type T. This is a specialization
//! of `FunctionOnce<T, T>` with additional convenience methods.
//!
//! This module provides the `TransformerOnce<T>` trait and three
//! implementations:
//!
//! - [`BoxTransformerOnce`]: Single ownership, one-time use
//! - [`ArcTransformerOnce`]: Thread-safe shared ownership, reusable
//!   (cloneable)
//! - [`RcTransformerOnce`]: Single-threaded shared ownership, reusable
//!   (cloneable)
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

use crate::function_once::FunctionOnce;

// ============================================================================
// Core Trait
// ============================================================================

/// TransformerOnce trait - consuming self-transformation that takes ownership
///
/// Defines the behavior of a consuming transformation: converting a value of
/// type `T` to another value of the same type `T` by taking ownership. This
/// trait extends `FunctionOnce<T, T>` with specialized transformation
/// semantics.
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
pub trait TransformerOnce<T>: FunctionOnce<T, T> {
    /// Applies the transformation to the input value, consuming both self
    /// and input
    ///
    /// This is the core method that must be implemented.
    ///
    /// # Parameters
    ///
    /// * `input` - The input value (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(self, input: T) -> T
    where
        Self: Sized;
}

// ============================================================================
// BoxTransformerOnce - Box<dyn FnOnce(T) -> T>
// ============================================================================

/// BoxTransformerOnce - consuming transformer wrapper based on
/// `Box<dyn FnOnce>`
///
/// A transformer wrapper that provides single ownership with one-time use
/// semantics. Consumes both self and the input value.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(T) -> T>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (consumes self and input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Haixing Hu
pub struct BoxTransformerOnce<T> {
    f: Box<dyn FnOnce(T) -> T>,
}

impl<T> BoxTransformerOnce<T>
where
    T: 'static,
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
    /// use prism3_function::BoxTransformerOnce;
    ///
    /// let parse = BoxTransformerOnce::new(|s: String| {
    ///     s.trim().to_uppercase()
    /// });
    ///
    /// assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> T + 'static,
    {
        BoxTransformerOnce { f: Box::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerOnce;
    ///
    /// let identity = BoxTransformerOnce::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> BoxTransformerOnce<T> {
        BoxTransformerOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerOnce;
    ///
    /// let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn and_then<G>(self, after: G) -> BoxTransformerOnce<T>
    where
        G: FnOnce(T) -> T + 'static,
    {
        BoxTransformerOnce::new(move |x| after((self.f)(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerOnce;
    ///
    /// let double = BoxTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose<G>(self, before: G) -> BoxTransformerOnce<T>
    where
        G: FnOnce(T) -> T + 'static,
    {
        BoxTransformerOnce::new(move |x| (self.f)(before(x)))
    }
}

impl<T> BoxTransformerOnce<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerOnce;
    ///
    /// let constant = BoxTransformerOnce::constant(42);
    /// assert_eq!(constant.transform(123), 42);
    /// ```
    pub fn constant(value: T) -> BoxTransformerOnce<T> {
        BoxTransformerOnce::new(move |_| value.clone())
    }
}

impl<T> TransformerOnce<T> for BoxTransformerOnce<T>
where
    T: 'static,
{
    fn transform(self, input: T) -> T {
        (self.f)(input)
    }
}

impl<T> FunctionOnce<T, T> for BoxTransformerOnce<T>
where
    T: 'static,
{
    fn apply(self, input: T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_once::BoxFunctionOnce<T, T>
    where
        T: 'static,
    {
        crate::function_once::BoxFunctionOnce::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function_once::RcFunctionOnce<T, T>
    where
        T: 'static,
    {
        // Cannot directly convert Box<dyn FnOnce> to Rc<dyn Fn>
        // We need to wrap it in a way that makes it reusable
        let func = std::rc::Rc::new(std::cell::RefCell::new(Some(self)));
        crate::function_once::RcFunctionOnce::new(move |x| {
            func.borrow_mut()
                .take()
                .expect("RcFunctionOnce can only be called once")
                .apply(x)
        })
    }

    fn into_arc(self) -> crate::function_once::ArcFunctionOnce<T, T>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
    {
        unreachable!(
            "BoxTransformerOnce<T> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(T) -> T
    where
        T: 'static,
    {
        let mut func = Some(self);
        move |t: T| {
            func.take()
                .expect("BoxTransformerOnce can only be called once")
                .apply(t)
        }
    }
}

// ============================================================================
// ArcTransformerOnce - Arc<dyn Fn(T) -> T + Send + Sync> (reusable version)
// ============================================================================

/// ArcTransformerOnce - thread-safe reusable consuming transformer wrapper
///
/// Note: Despite the name, this is actually reusable because it's based on
/// `Fn(T) -> T` rather than `FnOnce(T) -> T`. This allows it to be cloned
/// and shared across threads while still consuming the input value.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(T) -> T + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (but consumes input each
///   time)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct ArcTransformerOnce<T> {
    f: Arc<dyn Fn(T) -> T + Send + Sync>,
}

impl<T> ArcTransformerOnce<T>
where
    T: 'static,
{
    /// Creates a new ArcTransformerOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerOnce;
    ///
    /// let parse = ArcTransformerOnce::new(|s: String| {
    ///     s.trim().to_uppercase()
    /// });
    ///
    /// assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        ArcTransformerOnce { f: Arc::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerOnce;
    ///
    /// let identity = ArcTransformerOnce::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> ArcTransformerOnce<T> {
        ArcTransformerOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable (but each apply still consumes its input).
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new ArcTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerOnce;
    ///
    /// let double = ArcTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    /// let composed = double.and_then(&add_one);
    ///
    /// // Both original and composed can be used
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(5), 11);
    /// ```
    pub fn and_then(&self, after: &ArcTransformerOnce<T>) -> ArcTransformerOnce<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcTransformerOnce {
            f: Arc::new(move |x: T| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable (but each apply still consumes its input).
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new ArcTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerOnce;
    ///
    /// let double = ArcTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(&self, before: &ArcTransformerOnce<T>) -> ArcTransformerOnce<T>
    where
        T: Send + Sync,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcTransformerOnce {
            f: Arc::new(move |x: T| self_clone(before_clone(x))),
        }
    }
}

impl<T> ArcTransformerOnce<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerOnce;
    ///
    /// let constant = ArcTransformerOnce::constant(42);
    /// assert_eq!(constant.transform(123), 42);
    /// ```
    pub fn constant(value: T) -> ArcTransformerOnce<T>
    where
        T: Send + Sync,
    {
        ArcTransformerOnce::new(move |_| value.clone())
    }
}

impl<T> TransformerOnce<T> for ArcTransformerOnce<T>
where
    T: 'static,
{
    fn transform(self, input: T) -> T {
        (self.f)(input)
    }
}

impl<T> FunctionOnce<T, T> for ArcTransformerOnce<T>
where
    T: 'static,
{
    fn apply(self, input: T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_once::BoxFunctionOnce<T, T>
    where
        T: 'static,
    {
        crate::function_once::BoxFunctionOnce::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function_once::RcFunctionOnce<T, T>
    where
        T: 'static,
    {
        let arc = self.f;
        crate::function_once::RcFunctionOnce::new(move |x| arc(x))
    }

    fn into_arc(self) -> crate::function_once::ArcFunctionOnce<T, T>
    where
        T: Send + Sync + 'static,
    {
        let f = self.f;
        crate::function_once::ArcFunctionOnce::new(move |x| f(x))
    }

    fn into_fn(self) -> impl FnMut(T) -> T
    where
        T: 'static,
    {
        let f = self.f;
        move |t: T| f(t)
    }
}

impl<T> Clone for ArcTransformerOnce<T> {
    fn clone(&self) -> Self {
        ArcTransformerOnce {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcTransformerOnce - Rc<dyn Fn(T) -> T> (reusable version)
// ============================================================================

/// RcTransformerOnce - single-threaded reusable consuming transformer wrapper
///
/// Note: Despite the name, this is actually reusable because it's based on
/// `Fn(T) -> T` rather than `FnOnce(T) -> T`. This allows it to be cloned
/// while still consuming the input value.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(T) -> T>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (but consumes input each
///   time)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Haixing Hu
pub struct RcTransformerOnce<T> {
    f: Rc<dyn Fn(T) -> T>,
}

impl<T> RcTransformerOnce<T>
where
    T: 'static,
{
    /// Creates a new RcTransformerOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerOnce;
    ///
    /// let parse = RcTransformerOnce::new(|s: String| {
    ///     s.trim().to_uppercase()
    /// });
    ///
    /// assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> T + 'static,
    {
        RcTransformerOnce { f: Rc::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerOnce;
    ///
    /// let identity = RcTransformerOnce::<i32>::identity();
    /// assert_eq!(identity.transform(42), 42);
    /// ```
    pub fn identity() -> RcTransformerOnce<T> {
        RcTransformerOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new transformer that applies this transformer first, then
    /// applies the after transformer to the result. Uses &self, so original
    /// transformer remains usable (but each apply still consumes its input).
    ///
    /// # Parameters
    ///
    /// * `after` - The transformer to apply after self
    ///
    /// # Returns
    ///
    /// A new RcTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerOnce;
    ///
    /// let double = RcTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    /// let composed = double.and_then(&add_one);
    ///
    /// // Both original and composed can be used
    /// assert_eq!(double.transform(21), 42);
    /// assert_eq!(composed.transform(5), 11);
    /// ```
    pub fn and_then(&self, after: &RcTransformerOnce<T>) -> RcTransformerOnce<T> {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcTransformerOnce {
            f: Rc::new(move |x: T| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Uses &self, so original
    /// transformer remains usable (but each apply still consumes its input).
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new RcTransformerOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerOnce;
    ///
    /// let double = RcTransformerOnce::new(|x: i32| x * 2);
    /// let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(&self, before: &RcTransformerOnce<T>) -> RcTransformerOnce<T> {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcTransformerOnce {
            f: Rc::new(move |x: T| self_clone(before_clone(x))),
        }
    }
}

impl<T> RcTransformerOnce<T>
where
    T: Clone + 'static,
{
    /// Creates a constant transformer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerOnce;
    ///
    /// let constant = RcTransformerOnce::constant(42);
    /// assert_eq!(constant.transform(123), 42);
    /// ```
    pub fn constant(value: T) -> RcTransformerOnce<T> {
        RcTransformerOnce::new(move |_| value.clone())
    }
}

impl<T> TransformerOnce<T> for RcTransformerOnce<T>
where
    T: 'static,
{
    fn transform(self, input: T) -> T {
        (self.f)(input)
    }
}

impl<T> FunctionOnce<T, T> for RcTransformerOnce<T>
where
    T: 'static,
{
    fn apply(self, input: T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_once::BoxFunctionOnce<T, T>
    where
        T: 'static,
    {
        crate::function_once::BoxFunctionOnce::new(move |x| self.apply(x))
    }

    fn into_rc(self) -> crate::function_once::RcFunctionOnce<T, T>
    where
        T: 'static,
    {
        let f = self.f;
        crate::function_once::RcFunctionOnce::new(move |x| f(x))
    }

    fn into_arc(self) -> crate::function_once::ArcFunctionOnce<T, T>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
    {
        unreachable!(
            "RcTransformerOnce cannot be converted to ArcFunctionOnce \
             because Rc is not Send + Sync"
        )
    }

    fn into_fn(self) -> impl FnMut(T) -> T
    where
        T: 'static,
    {
        let f = self.f;
        move |t: T| f(t)
    }
}

impl<T> Clone for RcTransformerOnce<T> {
    fn clone(&self) -> Self {
        RcTransformerOnce {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement TransformerOnce<T> for any type that implements FnOnce(T) -> T
///
/// This allows once-callable closures and function pointers to be used
/// directly with our TransformerOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::TransformerOnce;
///
/// fn parse(s: String) -> String {
///     s.trim().to_uppercase()
/// }
///
/// assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
///
/// let owned_value = String::from("hello");
/// let consume = |s: String| {
///     format!("{} world", s)
/// };
/// assert_eq!(consume.transform(owned_value), "hello world");
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T> TransformerOnce<T> for F
where
    F: FnOnce(T) -> T,
    T: 'static,
{
    fn transform(self, input: T) -> T {
        self(input)
    }
}
