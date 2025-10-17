/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # FunctionMut Types
//!
//! Provides Rust implementations of mutable function traits similar to Rust's
//! `FnMut` trait, but with value-oriented semantics for functional programming patterns.
//!
//! This module provides the `FunctionMut<T, R>` trait and three implementations:
//!
//! - [`BoxFunctionMut`]: Single ownership, not cloneable
//! - [`ArcFunctionMut`]: Thread-safe shared ownership, cloneable
//! - [`RcFunctionMut`]: Single-threaded shared ownership, cloneable
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

    /// Converts to BoxFunctionMut
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxFunctionMut<T, R>`
    fn into_box(self) -> BoxFunctionMut<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to RcFunctionMut
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcFunctionMut<T, R>`
    fn into_rc(self) -> RcFunctionMut<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to ArcFunctionMut
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcFunctionMut<T, R>`
    fn into_arc(self) -> ArcFunctionMut<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        R: Send + 'static;

    /// Converts function to a closure
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(&mut T) -> R`
    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
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

    /// Creates an identity function
    ///
    /// Returns a function that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionMut, FunctionMut};
    ///
    /// let mut identity = BoxFunctionMut::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn identity() -> BoxFunctionMut<T, T>
    where
        T: Clone,
    {
        BoxFunctionMut::new(|x: &mut T| x.clone())
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
    /// A new BoxFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionMut, FunctionMut};
    ///
    /// let mut double = BoxFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut to_string = BoxFunctionMut::new(|x: &mut i32| x.to_string());
    /// let mut composed = double.and_then(to_string);
    ///
    /// let mut value = 21;
    /// assert_eq!(composed.apply(&mut value), "42");
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn and_then<S>(self, after: BoxFunctionMut<R, S>) -> BoxFunctionMut<T, S>
    where
        S: 'static,
    {
        let mut self_f = self.f;
        let mut after_f = after.f;
        BoxFunctionMut::new(move |x: &mut T| {
            let mut result = self_f(x);
            after_f(&mut result)
        })
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
    /// A new BoxFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxFunctionMut, FunctionMut};
    ///
    /// let mut double = BoxFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = BoxFunctionMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.apply(&mut value), 12); // (5 + 1) * 2
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn compose<S>(self, before: BoxFunctionMut<S, T>) -> BoxFunctionMut<S, R>
    where
        S: 'static,
    {
        let mut self_f = self.f;
        let mut before_f = before.f;
        BoxFunctionMut::new(move |x: &mut S| {
            let mut intermediate = before_f(x);
            self_f(&mut intermediate)
        })
    }
}

impl<T, R> FunctionMut<T, R> for BoxFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        (self.f)(input)
    }

    fn into_box(self) -> BoxFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcFunctionMut {
            f: Rc::new(RefCell::new(self.f)),
        }
    }

    fn into_arc(self) -> ArcFunctionMut<T, R>
    where
        Self: Send,
        T: Send + 'static,
        R: Send + 'static,
    {
        unreachable!(
            "BoxFunctionMut<T, R> does not implement Send, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let mut func = self;
        move |t: &mut T| func.apply(t)
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
/// 线程安全的可变函数类型别名
type ArcFunctionMutFn<T, R> = Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>;

/// # Author
///
/// Hu Haixing
pub struct ArcFunctionMut<T, R> {
    f: ArcFunctionMutFn<T, R>,
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

    /// Creates an identity function
    ///
    /// Returns a function that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcFunctionMut, FunctionMut};
    ///
    /// let mut identity = ArcFunctionMut::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn identity() -> ArcFunctionMut<T, T>
    where
        T: Clone + Send,
    {
        ArcFunctionMut::new(|x: &mut T| x.clone())
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
    /// A new ArcFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcFunctionMut, FunctionMut};
    ///
    /// let mut double = ArcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut to_string = ArcFunctionMut::new(|x: &mut i32| x.to_string());
    /// let mut composed = double.and_then(to_string);
    ///
    /// let mut value = 21;
    /// assert_eq!(composed.apply(&mut value), "42");
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn and_then<S>(self, after: ArcFunctionMut<R, S>) -> ArcFunctionMut<T, S>
    where
        S: Send + 'static,
        R: Send,
    {
        let self_f = self.f;
        let after_f = after.f;
        ArcFunctionMut::new(move |x: &mut T| {
            let mut result = {
                let mut guard = self_f.lock().unwrap();
                guard(x)
            };
            let mut guard = after_f.lock().unwrap();
            guard(&mut result)
        })
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
    /// A new ArcFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcFunctionMut, FunctionMut};
    ///
    /// let mut double = ArcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = ArcFunctionMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.apply(&mut value), 12); // (5 + 1) * 2
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn compose<S>(self, before: ArcFunctionMut<S, T>) -> ArcFunctionMut<S, R>
    where
        S: Send + 'static,
        T: Send,
    {
        let self_f = self.f;
        let before_f = before.f;
        ArcFunctionMut::new(move |x: &mut S| {
            let mut intermediate = {
                let mut guard = before_f.lock().unwrap();
                guard(x)
            };
            let mut guard = self_f.lock().unwrap();
            guard(&mut intermediate)
        })
    }
}

impl<T, R> FunctionMut<T, R> for ArcFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        let mut guard = self.f.lock().unwrap();
        guard(input)
    }

    fn into_box(self) -> BoxFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunctionMut {
            f: Box::new(move |x| {
                let mut guard = self.f.lock().unwrap();
                guard(x)
            }),
        }
    }

    fn into_rc(self) -> RcFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Cannot directly convert ArcFunctionMut to RcFunctionMut
        // because of lifetime constraints. We need to create a new closure
        // that wraps the Arc<Mutex<...>>
        let arc = self.f;
        let wrapper = move |x: &mut T| -> R {
            let mut guard = arc.lock().unwrap();
            guard(x)
        };
        RcFunctionMut::new(wrapper)
    }

    fn into_arc(self) -> ArcFunctionMut<T, R>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let mut func = self;
        move |t: &mut T| func.apply(t)
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
/// 单线程共享的可变函数类型别名
type RcFunctionMutFn<T, R> = Rc<RefCell<dyn FnMut(&mut T) -> R>>;

/// # Author
///
/// Hu Haixing
pub struct RcFunctionMut<T, R> {
    f: RcFunctionMutFn<T, R>,
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

    /// Creates an identity function
    ///
    /// Returns a function that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcFunctionMut, FunctionMut};
    ///
    /// let mut identity = RcFunctionMut::<i32, i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.apply(&mut value), 42);
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn identity() -> RcFunctionMut<T, T>
    where
        T: Clone,
    {
        RcFunctionMut::new(|x: &mut T| x.clone())
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
    /// A new RcFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcFunctionMut, FunctionMut};
    ///
    /// let mut double = RcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut to_string = RcFunctionMut::new(|x: &mut i32| x.to_string());
    /// let mut composed = double.and_then(to_string);
    ///
    /// let mut value = 21;
    /// assert_eq!(composed.apply(&mut value), "42");
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn and_then<S>(self, after: RcFunctionMut<R, S>) -> RcFunctionMut<T, S>
    where
        S: 'static,
    {
        let self_f = self.f;
        let after_f = after.f;
        RcFunctionMut::new(move |x: &mut T| {
            let mut result = {
                let mut guard = self_f.borrow_mut();
                guard(x)
            };
            let mut guard = after_f.borrow_mut();
            guard(&mut result)
        })
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
    /// A new RcFunctionMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcFunctionMut, FunctionMut};
    ///
    /// let mut double = RcFunctionMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = RcFunctionMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.apply(&mut value), 12); // (5 + 1) * 2
    /// ```
    ///
    /// # 作者
    ///
    /// 胡海星
    pub fn compose<S>(self, before: RcFunctionMut<S, T>) -> RcFunctionMut<S, R>
    where
        S: 'static,
    {
        let self_f = self.f;
        let before_f = before.f;
        RcFunctionMut::new(move |x: &mut S| {
            let mut intermediate = {
                let mut guard = before_f.borrow_mut();
                guard(x)
            };
            let mut guard = self_f.borrow_mut();
            guard(&mut intermediate)
        })
    }
}

impl<T, R> FunctionMut<T, R> for RcFunctionMut<T, R> {
    fn apply(&mut self, input: &mut T) -> R {
        let mut guard = self.f.borrow_mut();
        guard(input)
    }

    fn into_box(self) -> BoxFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxFunctionMut {
            f: Box::new(move |x| {
                let mut guard = self.f.borrow_mut();
                guard(x)
            }),
        }
    }

    fn into_rc(self) -> RcFunctionMut<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcFunctionMut<T, R>
    where
        Self: Send,
        T: Send + 'static,
        R: Send + 'static,
    {
        unreachable!(
            "RcFunctionMut cannot be converted to ArcFunctionMut because Rc \
             is not Send"
        )
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        T: 'static,
        R: 'static,
    {
        let mut func = self;
        move |t: &mut T| func.apply(t)
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
// Blanket implementation for standard FnMut trait
// ============================================================================

/// Implement FunctionMut<T, R> for any type that implements FnMut(&mut T) -> R
///
/// This allows mutable closures to be used directly with our FunctionMut trait
/// without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::FunctionMut;
///
/// let mut counter = 0;
/// let mut increment_and_double = |x: &mut i32| {
///     counter += 1;
///     *x *= 2;
///     *x
/// };
///
/// let mut value = 21;
/// assert_eq!(increment_and_double.apply(&mut value), 42);
/// assert_eq!(counter, 1);
/// ```
///
/// # 作者
///
/// 胡海星
impl<F, T, R> FunctionMut<T, R> for F
where
    F: FnMut(&mut T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply(&mut self, input: &mut T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxFunctionMut<T, R>
    where
        Self: Sized + 'static,
    {
        BoxFunctionMut::new(self)
    }

    fn into_rc(self) -> RcFunctionMut<T, R>
    where
        Self: Sized + 'static,
    {
        RcFunctionMut::new(self)
    }

    fn into_arc(self) -> ArcFunctionMut<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        ArcFunctionMut::new(self)
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> R
    where
        Self: Sized + 'static,
    {
        self
    }
}
