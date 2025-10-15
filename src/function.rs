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
//! Provides Rust implementations of function traits similar to Rust's own
//! `Fn`, `FnMut`, and `FnOnce` traits, but with value-oriented semantics for
//! functional programming patterns.
//!
//! This module provides three core traits:
//!
//! - [`Function<T, R>`]: Immutable transformation, borrows input `&T`
//! - [`FunctionMut<T, R>`]: Mutable transformation, borrows input `&mut T`
//! - [`FunctionOnce<T, R>`]: Consuming transformation, takes ownership of `T`
//!
//! Each trait has three implementations based on different ownership models:
//!
//! - `Box*`: Single ownership, not cloneable
//! - `Arc*`: Thread-safe shared ownership, cloneable
//! - `Rc*`: Single-threaded shared ownership, cloneable
//!
//! # Implementation Matrix
//!
//! | Trait | Box | Arc | Rc |
//! |-------|-----|-----|----|
//! | `Function` | [`BoxFunction`] | [`ArcFunction`] | [`RcFunction`] |
//! | `FunctionMut` | [`BoxFunctionMut`] | [`ArcFunctionMut`] | [`RcFunctionMut`] |
//! | `FunctionOnce` | [`BoxFunctionOnce`] | [`ArcFunctionOnce`] | [`RcFunctionOnce`] |
//!
//! # Design Philosophy
//!
//! ## Function vs FunctionMut vs FunctionOnce
//!
//! - **Function**: For transformations that only need to read the input.
//!   Most efficient when you don't need to consume the input value.
//! - **FunctionMut**: For transformations that need to modify the input
//!   in place or maintain internal mutable state.
//! - **FunctionOnce**: For transformations that consume the input value,
//!   useful for ownership transfers and pipeline processing.
//!
//! ## Box vs Arc vs Rc
//!
//! - **Box**: Single ownership, not cloneable. Most efficient, use when
//!   you don't need to share the function.
//! - **Arc**: Thread-safe shared ownership. Use in multi-threaded contexts
//!   or when you need to share across threads.
//! - **Rc**: Single-threaded shared ownership. Slightly more efficient than
//!   Arc for single-threaded scenarios.
//!
//! # Examples
//!
//! ## Function - Immutable Transformation
//!
//! ```rust
//! use prism3_function::{BoxFunction, Function};
//!
//! let double = BoxFunction::new(|x: &i32| x * 2);
//!
//! let value = 21;
//! assert_eq!(double.apply(&value), 42);
//! // value is still usable
//! assert_eq!(value, 21);
//! ```
//!
//! ## FunctionMut - Mutable Transformation
//!
//! ```rust
//! use prism3_function::{BoxFunctionMut, FunctionMut};
//!
//! let mut double_in_place = BoxFunctionMut::new(|x: &mut i32| {
//!     *x *= 2;
//!     *x
//! });
//!
//! let mut value = 21;
//! assert_eq!(double_in_place.apply(&mut value), 42);
//! assert_eq!(value, 42); // value was modified
//! ```
//!
//! ## FunctionOnce - Consuming Transformation
//!
//! ```rust
//! use prism3_function::{BoxFunctionOnce, FunctionOnce};
//!
//! let parse = BoxFunctionOnce::new(|s: String| {
//!     s.parse::<i32>().unwrap_or(0)
//! });
//!
//! assert_eq!(parse.apply("42".to_string()), 42);
//! // String was consumed
//! ```
//!
//! ## Sharing with Arc
//!
//! ```rust
//! use prism3_function::{ArcFunction, Function};
//! use std::thread;
//!
//! let double = ArcFunction::new(|x: &i32| x * 2);
//! let double_clone = double.clone();
//!
//! let handle = thread::spawn(move || {
//!     double_clone.apply(&21)
//! });
//!
//! assert_eq!(handle.join().unwrap(), 42);
//! assert_eq!(double.apply(&21), 42); // Original still usable
//! ```
//!
//! # Author
//!
//! Hu Haixing

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ============================================================================
// Core Traits
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
}

/// FunctionMut trait - mutable transformation that borrows input mutably
///
/// Defines the behavior of a mutable function transformation: converting
/// a mutably borrowed value of type `&mut T` to a value of type `R`. This
/// trait is analogous to `FnMut(&mut T) -> R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (mutably borrowed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Hu Haixing
pub trait FunctionMut<T, R> {
    /// Applies the function to the mutable input value
    ///
    /// # Parameters
    ///
    /// * `input` - Mutable reference to the input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(&mut self, input: &mut T) -> R;
}

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
}

impl<T, R> Clone for RcFunction<T, R> {
    fn clone(&self) -> Self {
        RcFunction {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// BoxFunctionMut - Box<dyn FnMut(&mut T) -> R>
// ============================================================================

/// BoxFunctionMut - mutable function wrapper based on `Box<dyn FnMut>`
///
/// A function wrapper that provides single ownership with mutable
/// transformation capability. Can modify the input value and maintain
/// internal mutable state.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(&mut T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Hu Haixing
pub struct BoxFunctionMut<T, R> {
    f: Box<dyn FnMut(&mut T) -> R>,
}

impl<T, R> BoxFunctionMut<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxFunctionMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunctionMut;
    ///
    /// let mut double_in_place = BoxFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.apply(&mut value), 42);
    /// assert_eq!(value, 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + 'static,
    {
        BoxFunctionMut { f: Box::new(f) }
    }
}

impl<T, R> FunctionMut<T, R> for BoxFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        (self.f)(input)
    }
}

// ============================================================================
// ArcFunctionMut - Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>
// ============================================================================

/// ArcFunctionMut - thread-safe mutable function wrapper
///
/// A thread-safe, clonable function wrapper for mutable transformations.
/// Uses `Mutex` internally to ensure thread-safe mutable access.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Thread-safe (uses `Mutex`)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct ArcFunctionMut<T, R> {
    f: Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>,
}

impl<T, R> ArcFunctionMut<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new ArcFunctionMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFunctionMut;
    ///
    /// let mut double_in_place = ArcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.apply(&mut value), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + Send + 'static,
    {
        ArcFunctionMut {
            f: Arc::new(Mutex::new(f)),
        }
    }
}

impl<T, R> FunctionMut<T, R> for ArcFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        let mut guard = self.f.lock().unwrap();
        guard(input)
    }
}

impl<T, R> Clone for ArcFunctionMut<T, R> {
    fn clone(&self) -> Self {
        ArcFunctionMut {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcFunctionMut - Rc<RefCell<dyn FnMut(&mut T) -> R>>
// ============================================================================

/// RcFunctionMut - single-threaded mutable function wrapper
///
/// A single-threaded, clonable function wrapper for mutable
/// transformations. Uses `RefCell` internally for interior mutability.
///
/// # Features
///
/// - **Based on**: `Rc<RefCell<dyn FnMut(&mut T) -> R>>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// # Author
///
/// Hu Haixing
pub struct RcFunctionMut<T, R> {
    f: Rc<RefCell<dyn FnMut(&mut T) -> R>>,
}

impl<T, R> RcFunctionMut<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcFunctionMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFunctionMut;
    ///
    /// let mut double_in_place = RcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.apply(&mut value), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> R + 'static,
    {
        RcFunctionMut {
            f: Rc::new(RefCell::new(f)),
        }
    }
}

impl<T, R> FunctionMut<T, R> for RcFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        let mut guard = self.f.borrow_mut();
        guard(input)
    }
}

impl<T, R> Clone for RcFunctionMut<T, R> {
    fn clone(&self) -> Self {
        RcFunctionMut {
            f: Rc::clone(&self.f),
        }
    }
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
}

impl<T, R> Clone for RcFunctionOnce<T, R> {
    fn clone(&self) -> Self {
        RcFunctionOnce {
            f: Rc::clone(&self.f),
        }
    }
}
