/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # TransformerMut Types
//!
//! Provides Rust implementations of mutable transformer traits that transform
//! values from type T to the same type T. This is a specialization of
//! `FunctionMut<T, T>` with additional convenience methods.
//!
//! This module provides the `TransformerMut<T>` trait and three
//! implementations:
//!
//! - [`BoxTransformerMut`]: Single ownership, not cloneable
//! - [`ArcTransformerMut`]: Thread-safe shared ownership, cloneable
//! - [`RcTransformerMut`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::function_mut::FunctionMut;

// ============================================================================
// Core Trait
// ============================================================================

/// TransformerMut trait - mutable self-transformation that borrows input
/// mutably
///
/// Defines the behavior of a mutable transformation: converting a mutably
/// borrowed value of type `&mut T` to a value of the same type `T`. This
/// trait extends `FunctionMut<T, T>` with specialized transformation
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
pub trait TransformerMut<T>: FunctionMut<T, T> {
    /// Applies the transformation to the mutable input value
    ///
    /// This is the core method that must be implemented.
    ///
    /// # Parameters
    ///
    /// * `input` - Mutable reference to the input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn transform(&mut self, input: &mut T) -> T;
}

// ============================================================================
// BoxTransformerMut - Box<dyn FnMut(&mut T) -> T>
// ============================================================================

/// BoxTransformerMut - mutable transformer wrapper based on `Box<dyn FnMut>`
///
/// A transformer wrapper that provides single ownership with mutable
/// transformation capability. Can modify the input value and maintain
/// internal mutable state.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(&mut T) -> T>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
///
/// # Author
///
/// Haixing Hu
pub struct BoxTransformerMut<T> {
    f: Box<dyn FnMut(&mut T) -> T>,
}

impl<T> BoxTransformerMut<T>
where
    T: 'static,
{
    /// Creates a new BoxTransformerMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerMut;
    ///
    /// let mut double_in_place = BoxTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.transform(&mut value), 42);
    /// assert_eq!(value, 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> T + 'static,
    {
        BoxTransformerMut { f: Box::new(f) }
    }

    /// Creates an identity transformer
    ///
    /// Returns a transformer that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerMut;
    ///
    /// let mut identity = BoxTransformerMut::<i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.transform(&mut value), 42);
    /// ```
    pub fn identity() -> BoxTransformerMut<T>
    where
        T: Clone,
    {
        BoxTransformerMut::new(|x: &mut T| x.clone())
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
    /// A new BoxTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerMut;
    ///
    /// let mut double = BoxTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = BoxTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.and_then(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1
    /// ```
    pub fn and_then(self, after: BoxTransformerMut<T>) -> BoxTransformerMut<T> {
        let mut self_f = self.f;
        let mut after_f = after.f;
        BoxTransformerMut::new(move |x: &mut T| {
            let mut result = self_f(x);
            after_f(&mut result)
        })
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
    /// A new BoxTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTransformerMut;
    ///
    /// let mut double = BoxTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = BoxTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(self, before: BoxTransformerMut<T>) -> BoxTransformerMut<T> {
        let mut self_f = self.f;
        let mut before_f = before.f;
        BoxTransformerMut::new(move |x: &mut T| {
            let mut intermediate = before_f(x);
            self_f(&mut intermediate)
        })
    }
}

impl<T> TransformerMut<T> for BoxTransformerMut<T>
where
    T: 'static,
{
    fn transform(&mut self, input: &mut T) -> T {
        (self.f)(input)
    }
}

impl<T> FunctionMut<T, T> for BoxTransformerMut<T>
where
    T: 'static,
{
    fn apply(&mut self, input: &mut T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_mut::BoxFunctionMut<T, T> {
        let mut f = self.f;
        crate::function_mut::BoxFunctionMut::new(move |x| f(x))
    }

    fn into_rc(self) -> crate::function_mut::RcFunctionMut<T, T> {
        crate::function_mut::RcFunctionMut::new(self.f)
    }

    fn into_arc(self) -> crate::function_mut::ArcFunctionMut<T, T>
    where
        Self: Send,
        T: Send,
    {
        unreachable!(
            "BoxTransformerMut<T> does not implement Send, so this method \
             can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> T {
        let mut transformer = self;
        move |t: &mut T| transformer.apply(t)
    }
}

// ============================================================================
// ArcTransformerMut - Arc<Mutex<dyn FnMut(&mut T) -> T + Send>>
// ============================================================================

/// ArcTransformerMut - thread-safe mutable transformer wrapper
///
/// A thread-safe, clonable transformer wrapper for mutable transformations.
/// Uses `Mutex` internally to ensure thread-safe mutable access.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(&mut T) -> T + Send>>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Thread-safe (uses `Mutex`)
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// 线程安全的可变转换器函数类型别名
type ArcTransformerMutFn<T> = Arc<Mutex<dyn FnMut(&mut T) -> T + Send>>;

/// # Author
///
/// Haixing Hu
pub struct ArcTransformerMut<T> {
    f: ArcTransformerMutFn<T>,
}

impl<T> ArcTransformerMut<T>
where
    T: 'static,
{
    /// Creates a new ArcTransformerMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerMut;
    ///
    /// let mut double_in_place = ArcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.transform(&mut value), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> T + Send + 'static,
    {
        ArcTransformerMut {
            f: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates an identity transformer
    ///
    /// Returns a transformer that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerMut;
    ///
    /// let mut identity = ArcTransformerMut::<i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.transform(&mut value), 42);
    /// ```
    pub fn identity() -> ArcTransformerMut<T>
    where
        T: Clone + Send,
    {
        ArcTransformerMut::new(|x: &mut T| x.clone())
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
    /// A new ArcTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerMut;
    ///
    /// let mut double = ArcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = ArcTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.and_then(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1
    /// ```
    pub fn and_then(self, after: ArcTransformerMut<T>) -> ArcTransformerMut<T>
    where
        T: Send,
    {
        let self_f = self.f;
        let after_f = after.f;
        ArcTransformerMut::new(move |x: &mut T| {
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
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new ArcTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTransformerMut;
    ///
    /// let mut double = ArcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = ArcTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(self, before: ArcTransformerMut<T>) -> ArcTransformerMut<T>
    where
        T: Send,
    {
        let self_f = self.f;
        let before_f = before.f;
        ArcTransformerMut::new(move |x: &mut T| {
            let mut intermediate = {
                let mut guard = before_f.lock().unwrap();
                guard(x)
            };
            let mut guard = self_f.lock().unwrap();
            guard(&mut intermediate)
        })
    }
}

impl<T> TransformerMut<T> for ArcTransformerMut<T>
where
    T: 'static,
{
    fn transform(&mut self, input: &mut T) -> T {
        let mut guard = self.f.lock().unwrap();
        guard(input)
    }
}

impl<T> FunctionMut<T, T> for ArcTransformerMut<T>
where
    T: 'static,
{
    fn apply(&mut self, input: &mut T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_mut::BoxFunctionMut<T, T> {
        crate::function_mut::BoxFunctionMut::new(move |x| {
            let mut guard = self.f.lock().unwrap();
            guard(x)
        })
    }

    fn into_rc(self) -> crate::function_mut::RcFunctionMut<T, T> {
        let arc = self.f;
        crate::function_mut::RcFunctionMut::new(move |x: &mut T| -> T {
            let mut guard = arc.lock().unwrap();
            guard(x)
        })
    }

    fn into_arc(self) -> crate::function_mut::ArcFunctionMut<T, T>
    where
        T: Send,
    {
        crate::function_mut::ArcFunctionMut::new(move |x| {
            let mut guard = self.f.lock().unwrap();
            guard(x)
        })
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> T {
        let mut transformer = self;
        move |t: &mut T| transformer.apply(t)
    }
}

impl<T> Clone for ArcTransformerMut<T> {
    fn clone(&self) -> Self {
        ArcTransformerMut {
            f: Arc::clone(&self.f),
        }
    }
}

// ============================================================================
// RcTransformerMut - Rc<RefCell<dyn FnMut(&mut T) -> T>>
// ============================================================================

/// RcTransformerMut - single-threaded mutable transformer wrapper
///
/// A single-threaded, clonable transformer wrapper for mutable
/// transformations. Uses `RefCell` internally for interior mutability.
///
/// # Features
///
/// - **Based on**: `Rc<RefCell<dyn FnMut(&mut T) -> T>>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (mutably borrows input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
///
/// 单线程共享的可变转换器函数类型别名
type RcTransformerMutFn<T> = Rc<RefCell<dyn FnMut(&mut T) -> T>>;

/// # Author
///
/// Haixing Hu
pub struct RcTransformerMut<T> {
    f: RcTransformerMutFn<T>,
}

impl<T> RcTransformerMut<T>
where
    T: 'static,
{
    /// Creates a new RcTransformerMut
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerMut;
    ///
    /// let mut double_in_place = RcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    ///
    /// let mut value = 21;
    /// assert_eq!(double_in_place.transform(&mut value), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) -> T + 'static,
    {
        RcTransformerMut {
            f: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates an identity transformer
    ///
    /// Returns a transformer that clones the input and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerMut;
    ///
    /// let mut identity = RcTransformerMut::<i32>::identity();
    /// let mut value = 42;
    /// assert_eq!(identity.transform(&mut value), 42);
    /// ```
    pub fn identity() -> RcTransformerMut<T>
    where
        T: Clone,
    {
        RcTransformerMut::new(|x: &mut T| x.clone())
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
    /// A new RcTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerMut;
    ///
    /// let mut double = RcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = RcTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.and_then(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1
    /// ```
    pub fn and_then(self, after: RcTransformerMut<T>) -> RcTransformerMut<T> {
        let self_f = self.f;
        let after_f = after.f;
        RcTransformerMut::new(move |x: &mut T| {
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
    /// Creates a new transformer that applies the before transformer first,
    /// then applies this transformer to the result. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `before` - The transformer to apply before self
    ///
    /// # Returns
    ///
    /// A new RcTransformerMut representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTransformerMut;
    ///
    /// let mut double = RcTransformerMut::new(|x: &mut i32| {
    ///     *x *= 2;
    ///     *x
    /// });
    /// let mut add_one = RcTransformerMut::new(|x: &mut i32| {
    ///     *x += 1;
    ///     *x
    /// });
    /// let mut composed = double.compose(add_one);
    ///
    /// let mut value = 5;
    /// assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2
    /// ```
    pub fn compose(self, before: RcTransformerMut<T>) -> RcTransformerMut<T> {
        let self_f = self.f;
        let before_f = before.f;
        RcTransformerMut::new(move |x: &mut T| {
            let mut intermediate = {
                let mut guard = before_f.borrow_mut();
                guard(x)
            };
            let mut guard = self_f.borrow_mut();
            guard(&mut intermediate)
        })
    }
}

impl<T> TransformerMut<T> for RcTransformerMut<T>
where
    T: 'static,
{
    fn transform(&mut self, input: &mut T) -> T {
        let mut guard = self.f.borrow_mut();
        guard(input)
    }
}

impl<T> FunctionMut<T, T> for RcTransformerMut<T>
where
    T: 'static,
{
    fn apply(&mut self, input: &mut T) -> T {
        self.transform(input)
    }

    fn into_box(self) -> crate::function_mut::BoxFunctionMut<T, T> {
        crate::function_mut::BoxFunctionMut::new(move |x| {
            let mut guard = self.f.borrow_mut();
            guard(x)
        })
    }

    fn into_rc(self) -> crate::function_mut::RcFunctionMut<T, T> {
        crate::function_mut::RcFunctionMut::new(move |x| {
            let mut guard = self.f.borrow_mut();
            guard(x)
        })
    }

    fn into_arc(self) -> crate::function_mut::ArcFunctionMut<T, T>
    where
        Self: Send,
        T: Send,
    {
        unreachable!(
            "RcTransformerMut cannot be converted to ArcFunctionMut because \
             Rc is not Send"
        )
    }

    fn into_fn(self) -> impl FnMut(&mut T) -> T {
        let mut transformer = self;
        move |t: &mut T| transformer.apply(t)
    }
}

impl<T> Clone for RcTransformerMut<T> {
    fn clone(&self) -> Self {
        RcTransformerMut {
            f: Rc::clone(&self.f),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnMut trait
// ============================================================================

/// Implement TransformerMut<T> for any type that implements FnMut(&mut T) ->
/// T
///
/// This allows mutable closures to be used directly with our TransformerMut
/// trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::TransformerMut;
///
/// let mut counter = 0;
/// let mut increment_and_double = |x: &mut i32| {
///     counter += 1;
///     *x *= 2;
///     *x
/// };
///
/// let mut value = 21;
/// assert_eq!(increment_and_double.transform(&mut value), 42);
/// assert_eq!(counter, 1);
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T> TransformerMut<T> for F
where
    F: FnMut(&mut T) -> T,
    T: 'static,
{
    fn transform(&mut self, input: &mut T) -> T {
        self(input)
    }
}
