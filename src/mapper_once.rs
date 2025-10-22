/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MapperOnce Types
//!
//! Provides Rust implementations of consuming mapper traits similar to
//! Rust's `FnOnce` trait, but with value-oriented semantics for functional
//! programming patterns with state consumption.
//!
//! This module provides the `MapperOnce<T, R>` trait and one-time use
//! implementation:
//!
//! - [`BoxMapperOnce`]: Single ownership, one-time use
//!
//! # Author
//!
//! 胡海星

use crate::predicate::{BoxPredicate, Predicate};

// ============================================================================
// Core Trait
// ============================================================================

/// MapperOnce trait - consuming mapper that takes ownership
///
/// Defines the behavior of a consuming mapper: converting a value of
/// type `T` to a value of type `R` by taking ownership of both self and the
/// input. This trait is analogous to `FnOnce(T) -> R`.
///
/// Unlike `Mapper` (which is like `FnMut`), `MapperOnce` consumes itself on
/// the first call, making it suitable for one-time transformations that may
/// own or consume resources.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// 胡海星
pub trait MapperOnce<T, R> {
    /// Applies the mapping to the input value, consuming both self and input
    ///
    /// # Parameters
    ///
    /// * `input` - The input value (consumed)
    ///
    /// # Returns
    ///
    /// The mapped output value
    fn apply_once(self, input: T) -> R;

    /// Converts to BoxMapperOnce
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxMapperOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::MapperOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let boxed = double.into_box_once();
    /// assert_eq!(boxed.apply_once(21), 42);
    /// ```
    fn into_box_once(self) -> BoxMapperOnce<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapperOnce::new(move |input: T| self.apply_once(input))
    }

    /// Converts mapper to a closure
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::MapperOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let func = double.into_fn_once();
    /// assert_eq!(func(21), 42);
    /// ```
    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |input: T| self.apply_once(input)
    }

    /// Converts to BoxMapperOnce without consuming self
    ///
    /// **📌 Borrows `&self`**: The original mapper remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a new `BoxMapperOnce` that
    /// captures a clone. Types implementing `Clone` can override this method
    /// to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns `BoxMapperOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::MapperOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let boxed = double.to_box_once();
    /// assert_eq!(boxed.apply_once(21), 42);
    /// ```
    fn to_box_once(&self) -> BoxMapperOnce<T, R>
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_box_once()
    }

    /// Converts mapper to a closure without consuming self
    ///
    /// **📌 Borrows `&self`**: The original mapper remains usable
    /// after calling this method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation creates a closure that captures a
    /// clone of `self` and calls its `apply` method. Types can
    /// override this method to provide more efficient conversions.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::MapperOnce;
    ///
    /// let double = |x: i32| x * 2;
    /// let func = double.to_fn_once();
    /// assert_eq!(func(21), 42);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + 'static,
        T: 'static,
        R: 'static,
    {
        self.clone().into_fn_once()
    }
}

// ============================================================================
// BoxMapperOnce - Box<dyn FnOnce(T) -> R>
// ============================================================================

/// BoxMapperOnce - consuming mapper wrapper based on `Box<dyn FnOnce>`
///
/// A mapper wrapper that provides single ownership with one-time use
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
/// 胡海星
pub struct BoxMapperOnce<T, R> {
    function: Box<dyn FnOnce(T) -> R>,
}

impl<T, R> BoxMapperOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxMapperOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapperOnce, MapperOnce};
    ///
    /// let parse = BoxMapperOnce::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0)
    /// });
    ///
    /// assert_eq!(parse.apply_once("42".to_string()), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxMapperOnce {
            function: Box::new(f),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapperOnce, MapperOnce};
    ///
    /// let identity = BoxMapperOnce::<i32, i32>::identity();
    /// assert_eq!(identity.apply_once(42), 42);
    /// ```
    pub fn identity() -> BoxMapperOnce<T, T> {
        BoxMapperOnce::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `G` - The type of the after mapper (must implement MapperOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxMapperOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxMapperOnce<R, S>`
    ///   - Any type implementing `MapperOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxMapperOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapperOnce, MapperOnce};
    ///
    /// let add_one = BoxMapperOnce::new(|x: i32| x + 1);
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    ///
    /// // Both add_one and double are moved and consumed
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    /// // add_one.apply_once(3); // Would not compile - moved
    /// // double.apply_once(4);  // Would not compile - moved
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxMapperOnce<T, S>
    where
        S: 'static,
        G: MapperOnce<R, S> + 'static,
    {
        BoxMapperOnce::new(move |x| {
            let intermediate = (self.function)(x);
            after.apply_once(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `G` - The type of the before mapper (must implement MapperOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxMapperOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxMapperOnce<S, T>`
    ///   - Any type implementing `MapperOnce<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxMapperOnce representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapperOnce, MapperOnce};
    ///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let add_one = BoxMapperOnce::new(|x: i32| x + 1);
    ///
    /// // Both double and add_one are moved and consumed
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    /// // double.apply_once(3); // Would not compile - moved
    /// // add_one.apply_once(4); // Would not compile - moved
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxMapperOnce<S, R>
    where
        S: 'static,
        G: MapperOnce<S, T> + 'static,
    {
        BoxMapperOnce::new(move |x| {
            let intermediate = before.apply_once(x);
            (self.function)(intermediate)
        })
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalMapperOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, BoxMapperOnce};
    ///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let identity = BoxMapperOnce::<i32, i32>::identity();
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(identity);
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// let double2 = BoxMapperOnce::new(|x: i32| x * 2);
    /// let identity2 = BoxMapperOnce::<i32, i32>::identity();
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(identity2);
    /// assert_eq!(conditional2.apply_once(-5), -5);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, BoxMapperOnce, RcPredicate, Predicate};
    ///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(BoxMapperOnce::<i32, i32>::identity());
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalMapperOnce<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalMapperOnce {
            mapper: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxMapperOnce<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapperOnce, MapperOnce};
    ///
    /// let constant = BoxMapperOnce::constant("hello");
    /// assert_eq!(constant.apply_once(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxMapperOnce<T, R> {
        BoxMapperOnce::new(move |_| value.clone())
    }
}

impl<T, R> MapperOnce<T, R> for BoxMapperOnce<T, R> {
    fn apply_once(self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box_once(self) -> BoxMapperOnce<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return the inner function
        self.function
    }

    // do NOT override BoxMapperOnce::to_box_once() and BoxMapperOnce::to_fn_once()
    // because BoxMapperOnce is not Clone and calling BoxMapperOnce::to_box_once()
    // or BoxMapperOnce::to_fn_once() will cause a compile error
}

// ============================================================================
// BoxConditionalMapperOnce - Box-based Conditional Mapper
// ============================================================================

/// BoxConditionalMapperOnce struct
///
/// A conditional consuming mapper that only executes when a predicate is
/// satisfied. Uses `BoxMapperOnce` and `BoxPredicate` for single
/// ownership semantics.
///
/// This type is typically created by calling `BoxMapperOnce::when()` and
/// is designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **One-time Use**: Can only be called once
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
///
/// # Examples
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{MapperOnce, BoxMapperOnce};
///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let negate = BoxMapperOnce::new(|x: i32| -x);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(negate);
    /// assert_eq!(conditional.apply_once(5), 10); // when branch executed
    ///
    /// let double2 = BoxMapperOnce::new(|x: i32| x * 2);
    /// let negate2 = BoxMapperOnce::new(|x: i32| -x);
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(negate2);
    /// assert_eq!(conditional2.apply_once(-5), 5); // or_else branch executed
    /// ```
///
/// # Author
///
/// 胡海星
pub struct BoxConditionalMapperOnce<T, R> {
    mapper: BoxMapperOnce<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalMapperOnce<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `BoxMapperOnce<T, R>`
    ///   - Any type implementing `MapperOnce<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMapperOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, BoxMapperOnce};
    ///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    /// assert_eq!(conditional.apply_once(5), 10); // Condition satisfied, execute double
    ///
    /// let double2 = BoxMapperOnce::new(|x: i32| x * 2);
    /// let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    /// assert_eq!(conditional2.apply_once(-5), 5); // Condition not satisfied, execute negate
    /// ```
    pub fn or_else<F>(self, else_mapper: F) -> BoxMapperOnce<T, R>
    where
        F: MapperOnce<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_mapper = self.mapper;
        BoxMapperOnce::new(move |t| {
            if pred.test(&t) {
                then_mapper.apply_once(t)
            } else {
                else_mapper.apply_once(t)
            }
        })
    }
}

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

/// Implement MapperOnce<T, R> for any type that implements FnOnce(T) -> R
///
/// This allows once-callable closures and function pointers to be used
/// directly with our MapperOnce trait without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::MapperOnce;
///
/// fn parse(s: String) -> i32 {
///     s.parse().unwrap_or(0)
/// }
///
/// assert_eq!(parse.apply_once("42".to_string()), 42);
///
/// let owned_value = String::from("hello");
/// let consume = |s: String| {
///     format!("{} world", s)
/// };
/// assert_eq!(consume.apply_once(owned_value), "hello world");
/// ```
///
/// # Author
///
/// 胡海星
impl<F, T, R> MapperOnce<T, R> for F
where
    F: FnOnce(T) -> R,
    T: 'static,
    R: 'static,
{
    fn apply_once(self, input: T) -> R {
        self(input)
    }

    fn into_box_once(self) -> BoxMapperOnce<T, R>
    where
        Self: Sized + 'static,
    {
        BoxMapperOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
    {
        // Zero-cost: directly return self since F is already FnOnce(T) -> R
        self
    }

    fn to_box_once(&self) -> BoxMapperOnce<T, R>
    where
        Self: Clone + Sized + 'static,
    {
        self.clone().into_box_once()
    }

    fn to_fn_once(&self) -> impl FnOnce(T) -> R
    where
        Self: Clone + Sized + 'static,
    {
        self.clone()
    }
}

// ============================================================================
// FnMapperOnceOps - Extension trait for FnOnce mappers
// ============================================================================

/// Extension trait for closures implementing `FnOnce(T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for one-time
/// use closures and function pointers without requiring explicit wrapping in
/// `BoxMapperOnce`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnOnce(T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `MapperOnce<T, R>` through
/// blanket implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides those
/// methods, returning `BoxMapperOnce` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{MapperOnce, FnMapperOnceOps};
///
/// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
/// let double = |x: i32| x * 2;
///
/// let composed = parse.and_then(double);
/// assert_eq!(composed.apply_once("21".to_string()), 42);
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{MapperOnce, FnMapperOnceOps};
///
/// let double = |x: i32| x * 2;
/// let to_string = |x: i32| x.to_string();
///
/// let composed = to_string.compose(double);
/// assert_eq!(composed.apply_once(21), "42");
/// ```
///
/// ## Conditional mapping with when
///
/// ```rust
/// use prism3_function::{MapperOnce, FnMapperOnceOps};
///
/// let double = |x: i32| x * 2;
/// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
///
/// assert_eq!(conditional.apply_once(5), 10);
/// ```
///
/// # Author
///
/// 胡海星
pub trait FnMapperOnceOps<T, R>: FnOnce(T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then
    /// applies the after mapper to the result. Consumes self and returns
    /// a `BoxMapperOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `G` - The type of the after mapper (must implement MapperOnce<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` mapper, the parameter will be consumed. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A function pointer: `fn(R) -> S`
    ///   - A `BoxMapperOnce<R, S>`
    ///   - Any type implementing `MapperOnce<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxMapperOnce<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, FnMapperOnceOps, BoxMapperOnce};
    ///
    /// let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    ///
    /// // double is moved and consumed
    /// let composed = parse.and_then(double);
    /// assert_eq!(composed.apply_once("21".to_string()), 42);
    /// // double.apply_once(5); // Would not compile - moved
    /// ```
    fn and_then<S, G>(self, after: G) -> BoxMapperOnce<T, S>
    where
        S: 'static,
        G: MapperOnce<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapperOnce::new(move |x: T| {
            let intermediate = self(x);
            after.apply_once(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first,
    /// then applies this mapper to the result. Consumes self and returns
    /// a `BoxMapperOnce`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `G` - The type of the before mapper (must implement MapperOnce<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since this is a
    ///   `FnOnce` mapper, the parameter will be consumed. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A function pointer: `fn(S) -> T`
    ///   - A `BoxMapperOnce<S, T>`
    ///   - Any type implementing `MapperOnce<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxMapperOnce<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, FnMapperOnceOps, BoxMapperOnce};
    ///
    /// let double = BoxMapperOnce::new(|x: i32| x * 2);
    /// let to_string = |x: i32| x.to_string();
    ///
    /// // double is moved and consumed
    /// let composed = to_string.compose(double);
    /// assert_eq!(composed.apply_once(21), "42");
    /// // double.apply_once(5); // Would not compile - moved
    /// ```
    fn compose<S, G>(self, before: G) -> BoxMapperOnce<S, R>
    where
        S: 'static,
        G: MapperOnce<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapperOnce::new(move |x: S| {
            let intermediate = before.apply_once(x);
            self(intermediate)
        })
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper for when
    /// the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalMapperOnce<T, R>`
    ///
    /// # Examples
    ///
    /// ## Basic usage with or_else
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, FnMapperOnceOps};
    ///
    /// let double = |x: i32| x * 2;
    /// let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{MapperOnce, FnMapperOnceOps, RcPredicate, Predicate};
    ///
    /// let double = |x: i32| x * 2;
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone to preserve original predicate
    /// let conditional = double.when(is_positive.clone())
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(conditional.apply_once(5), 10);
    ///
    /// // Original predicate still usable
    /// assert!(is_positive.test(&3));
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalMapperOnce<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapperOnce::new(self).when(predicate)
    }
}

/// Blanket implementation of FnMapperOnceOps for all FnOnce closures
///
/// Automatically implements `FnMapperOnceOps<T, R>` for any type that
/// implements `FnOnce(T) -> R`.
///
/// # Author
///
/// 胡海星
impl<T, R, F> FnMapperOnceOps<T, R> for F where F: FnOnce(T) -> R + 'static {}
