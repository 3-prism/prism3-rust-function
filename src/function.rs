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
//! Provides a Rust implementation similar to Java's `Function<T, R>`
//! interface, used to transform values from one type to another.
//!
//! This module provides a [`Function<T, R>`] trait as a unified interface,
//! along with four concrete implementations to cover different use cases:
//!
//! - [`BoxFunction`]: Based on `Box<dyn FnOnce>`, single ownership, one-time
//!   use
//! - [`BoxFnFunction`]: Based on `Box<dyn Fn>`, reusable, single ownership
//! - [`ArcFnFunction`]: Based on `Arc<dyn Fn>`, reusable, multi-threaded
//!   sharing
//! - [`RcFnFunction`]: Based on `Rc<dyn Fn>`, reusable, single-threaded
//!   sharing
//!
//! # Design Philosophy
//!
//! Unlike predicates (which test values), functions **transform** values and
//! typically **consume** the input (`T` rather than `&T`). This makes them
//! naturally suited for one-time use (`FnOnce`), though reusable versions
//! (`Fn`) are also provided for scenarios requiring multiple invocations.
//!
//! # Comparison of Implementations
//!
//! | Type            | Ownership | Reusable | Clonable | Thread-Safe |
//! |-----------------|-----------|----------|----------|-------------|
//! | `BoxFunction`   | Single    | No       | No       | No          |
//! | `BoxFnFunction` | Single    | Yes      | No       | No          |
//! | `ArcFnFunction` | Shared    | Yes      | Yes      | Yes         |
//! | `RcFnFunction`  | Shared    | Yes      | Yes      | No          |
//!
//! # When to Use Which
//!
//! - **[`BoxFunction`]**: One-time transformations, data pipelines, builder
//!   patterns
//! - **[`BoxFnFunction`]**: Local repeated use without needing to clone
//! - **[`ArcFnFunction`]**: Multi-threaded scenarios, shared configuration,
//!   concurrent processing
//! - **[`RcFnFunction`]**: Single-threaded callback systems, event handlers
//!
//! # Examples
//!
//! ## One-Time Transformation with BoxFunction
//!
//! ```rust
//! use prism3_function::BoxFunction;
//!
//! let parse_and_double = BoxFunction::new(|s: String| s.parse::<i32>().ok())
//!     .and_then(|opt| opt.unwrap_or(0))
//!     .and_then(|x| x * 2);
//!
//! assert_eq!(parse_and_double.apply("21".to_string()), 42);
//! ```
//!
//! ## Reusable with BoxFnFunction
//!
//! ```rust
//! use prism3_function::BoxFnFunction;
//!
//! let double = BoxFnFunction::new(|x: i32| x * 2);
//!
//! // Can be called multiple times
//! assert_eq!(double.apply(21), 42);
//! assert_eq!(double.apply(42), 84);
//! ```
//!
//! ## Thread-Safe Sharing with ArcFnFunction
//!
//! ```rust
//! use prism3_function::ArcFnFunction;
//! use std::thread;
//!
//! let double = ArcFnFunction::new(|x: i32| x * 2);
//! let cloned = double.clone();
//!
//! let handle = thread::spawn(move || cloned.apply(21));
//! assert_eq!(handle.join().unwrap(), 42);
//!
//! // Original still usable
//! assert_eq!(double.apply(42), 84);
//! ```
//!
//! ## Function Composition
//!
//! All function types support composition via `and_then` (chain) and
//! `compose` (reverse):
//!
//! ```rust
//! use prism3_function::BoxFunction;
//!
//! let add_one = BoxFunction::new(|x: i32| x + 1);
//! let double = |x: i32| x * 2;
//! let to_string = |x: i32| x.to_string();
//!
//! let pipeline = add_one.and_then(double).and_then(to_string);
//! assert_eq!(pipeline.apply(5), "12"); // (5 + 1) * 2 = "12"
//! ```
//!
//! ## Working with Option and Result
//!
//! Specialized constructors are provided for common patterns:
//!
//! ```rust
//! use prism3_function::BoxFunction;
//!
//! // Map over Option
//! let double_opt = BoxFunction::map_option(|x: i32| x * 2);
//! assert_eq!(double_opt.apply(Some(21)), Some(42));
//! assert_eq!(double_opt.apply(None), None);
//!
//! // Convert Result to Option
//! let to_option = BoxFunction::result_to_option();
//! assert_eq!(to_option.apply(Ok::<i32, &str>(42)), Some(42));
//! assert_eq!(to_option.apply(Err::<i32, &str>("error")), None);
//! ```
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Function Trait - Unified Interface
// ============================================================================

/// Function trait - unified function transformation interface
///
/// Defines the core behavior of function transformation: converting a value of
/// type `T` to a value of type `R`.
///
/// # Type Parameters
///
/// * `T` - The type of the input value
/// * `R` - The type of the output value
///
/// # Implementors
///
/// - All closure types `FnOnce(T) -> R`
/// - [`BoxFunction<T, R>`]
/// - [`BoxFnFunction<T, R>`]
/// - [`ArcFnFunction<T, R>`]
/// - [`RcFnFunction<T, R>`]
///
/// # Author
///
/// Hu Haixing
pub trait Function<T, R> {
    /// Applies the function to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn apply(self, input: T) -> R;

    /// Converts function to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the function and returns a closure that can be
    /// directly used with iterator methods like `map()`, `filter_map()`,
    /// etc. This provides a more ergonomic API when working with iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the function (takes ownership of `self`).
    /// After calling this method, the original function is no longer
    /// available. The returned closure captures the function by move.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnOnce(T) -> R`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::{BoxFunction, Function};
    ///
    /// let double = BoxFunction::new(|x: i32| x * 2);
    /// let closure = double.into_fn();
    ///
    /// assert_eq!(closure(21), 42);
    /// ```
    ///
    /// ## With Complex Function
    ///
    /// ```rust
    /// use prism3_function::{BoxFunction, Function};
    ///
    /// let parse_and_double = BoxFunction::new(|s: String| {
    ///     s.parse::<i32>().unwrap_or(0) * 2
    /// });
    ///
    /// let closure = parse_and_double.into_fn();
    /// assert_eq!(closure("21".to_string()), 42);
    /// ```
    ///
    /// ## Returning Option
    ///
    /// ```rust
    /// use prism3_function::{BoxFunction, Function};
    ///
    /// let parse = BoxFunction::new(|s: String| s.parse::<i32>().ok());
    /// let closure = parse.into_fn();
    ///
    /// // Note: FnOnce can only be called once
    /// assert_eq!(closure("42".to_string()), Some(42));
    /// ```
    ///
    /// ## Ownership Behavior
    ///
    /// ```rust,compile_fail
    /// use prism3_function::{BoxFunction, Function};
    ///
    /// let double = BoxFunction::new(|x: i32| x * 2);
    /// let closure = double.into_fn();
    ///
    /// // ❌ Error: double was moved in the call to into_fn()
    /// let x = double.apply(5);
    /// ```
    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;
}

// Implement Function trait for all FnOnce
impl<T, R, F> Function<T, R> for F
where
    F: FnOnce(T) -> R,
{
    fn apply(self, input: T) -> R {
        self(input)
    }

    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |t: T| self(t)
    }
}

// ============================================================================
// Closure Extension Trait - Provides composition methods for closures
// ============================================================================

/// Extension trait providing function composition methods for closures
///
/// Allows closures to use methods like `.and_then()` and `.compose()`,
/// returning a `BoxFunction` after composition.
///
/// # Author
///
/// Hu Haixing
pub trait FnFunctionOps<T, R>: FnOnce(T) -> R + Sized {
    /// Chain composition - applies self first, then after
    ///
    /// Execution order: input -> self -> after -> output
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the second function
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self
    ///
    /// # Returns
    ///
    /// Returns `BoxFunction<T, S>`, representing the composed function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FnFunctionOps;
    ///
    /// let add_one = |x: i32| x + 1;
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.and_then(double);
    /// ```
    fn and_then<S, G>(self, after: G) -> BoxFunction<T, S>
    where
        Self: 'static,
        G: FnOnce(R) -> S + 'static,
        T: 'static,
        S: 'static,
    {
        BoxFunction::new(move |x| after(self(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Execution order: input -> before -> self -> output
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the first function
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self
    ///
    /// # Returns
    ///
    /// Returns `BoxFunction<S, R>`, representing the composed function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::FnFunctionOps;
    ///
    /// let double = |x: i32| x * 2;
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose(add_one);
    /// ```
    fn compose<S, G>(self, before: G) -> BoxFunction<S, R>
    where
        Self: 'static,
        G: FnOnce(S) -> T + 'static,
        S: 'static,
        R: 'static,
    {
        BoxFunction::new(move |x| self(before(x)))
    }
}

// Implement FnFunctionOps for all FnOnce
impl<T, R, F> FnFunctionOps<T, R> for F where F: FnOnce(T) -> R {}

// ============================================================================
// BoxFunction - Single ownership, one-time use (FnOnce)
// ============================================================================

/// BoxFunction - function wrapper based on `Box<dyn FnOnce>`
///
/// A function wrapper that provides single ownership with one-time use
/// semantics. This is the most basic and lightweight function wrapper,
/// suitable for value transformations that consume the input and only need to
/// be executed once.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (`apply` consumes `self`)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
/// - **Performance**: Zero-cost abstraction, single boxing overhead
///
/// # Use Cases
///
/// - **One-time transformations**: Parse-validate-transform pipelines
/// - **Data pipelines**: Sequential data transformations where each step
///   executes once
/// - **Builder patterns**: Method chaining that culminates in a single build
///   operation
/// - **Memory efficiency**: When you don't need to reuse the function and
///   want to avoid reference counting overhead
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed by the function)
/// * `R` - The type of the output value
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::BoxFunction;
///
/// let double = BoxFunction::new(|x: i32| x * 2);
/// let result = double.apply(21);
/// assert_eq!(result, 42);
/// // double has been consumed and cannot be used again
/// ```
///
/// ## Pipeline Construction
///
/// ```rust
/// use prism3_function::BoxFunction;
///
/// let pipeline = BoxFunction::new(|s: String| s.trim().to_string())
///     .and_then(|s| s.parse::<i32>().ok())
///     .and_then(|opt| opt.unwrap_or(0))
///     .and_then(|x| x * 2)
///     .and_then(|x| format!("Result: {}", x));
///
/// assert_eq!(pipeline.apply("  21  ".to_string()), "Result: 42");
/// ```
///
/// ## Capturing and Consuming Variables
///
/// ```rust
/// use prism3_function::BoxFunction;
///
/// let prefix = String::from("Hello, ");
/// // Closure captures and consumes prefix
/// let greeter = BoxFunction::new(move |name: String| prefix + &name);
///
/// assert_eq!(greeter.apply("World".to_string()), "Hello, World");
/// // Both greeter and prefix are consumed
/// ```
///
/// # Comparison with Other Types
///
/// - **vs [`BoxFnFunction`]**: Use `BoxFunction` if you only need to call
///   the function once; use `BoxFnFunction` if you need to call it multiple
///   times.
/// - **vs [`ArcFnFunction`]**: Use `BoxFunction` for single-threaded,
///   one-time use; use `ArcFnFunction` for multi-threaded or repeated use.
/// - **vs [`RcFnFunction`]**: Use `BoxFunction` for simpler, non-shared
///   scenarios; use `RcFnFunction` for single-threaded sharing.
///
/// # Author
///
/// Hu Haixing
pub struct BoxFunction<T, R> {
    f: Box<dyn FnOnce(T) -> R>,
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
    /// # Returns
    ///
    /// Returns a new BoxFunction instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = BoxFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxFunction { f: Box::new(f) }
    }

    /// Applies the function to the input value
    ///
    /// Consumes self and returns the result.
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let to_string = BoxFunction::new(|x: i32| x.to_string());
    /// let result = to_string.apply(42);
    /// assert_eq!(result, "42");
    /// ```
    pub fn apply(self, input: T) -> R {
        (self.f)(input)
    }

    /// Creates an identity function
    ///
    /// Returns a function that directly returns the input value.
    ///
    /// # Returns
    ///
    /// Returns an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let identity = BoxFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> BoxFunction<T, T> {
        BoxFunction::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Execution order: input -> self -> after -> output
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the second function
    ///
    /// # Parameters
    ///
    /// * `after` - The function to apply after self
    ///
    /// # Returns
    ///
    /// Returns the composed BoxFunction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let add_one = BoxFunction::new(|x: i32| x + 1);
    /// let double = |x: i32| x * 2;
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2 = 12
    /// ```
    pub fn and_then<S, G>(self, after: G) -> BoxFunction<T, S>
    where
        S: 'static,
        G: FnOnce(R) -> S + 'static,
    {
        BoxFunction::new(move |x| after((self.f)(x)))
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Execution order: input -> before -> self -> output
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the first function
    ///
    /// # Parameters
    ///
    /// * `before` - The function to apply before self
    ///
    /// # Returns
    ///
    /// Returns the composed BoxFunction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = BoxFunction::new(|x: i32| x * 2);
    /// let add_one = |x: i32| x + 1;
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2 = 12
    /// ```
    pub fn compose<S, G>(self, before: G) -> BoxFunction<S, R>
    where
        S: 'static,
        G: FnOnce(S) -> T + 'static,
    {
        BoxFunction::new(move |x| (self.f)(before(x)))
    }
}

impl<T, R> BoxFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// Returns a function that ignores the input value and always returns the
    /// same constant value.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// Returns a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let constant = BoxFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxFunction<T, R> {
        BoxFunction::new(move |_| value.clone())
    }
}

// Option-related methods
impl<T, R> BoxFunction<Option<T>, Option<R>>
where
    T: 'static,
    R: 'static,
{
    /// Creates an Option mapping function
    ///
    /// Converts a normal function into a function that operates on Option.
    ///
    /// # Parameters
    ///
    /// * `f` - The function to apply to Some values
    ///
    /// # Returns
    ///
    /// Returns an Option mapping function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = |x: i32| x * 2;
    /// let option_double = BoxFunction::map_option(double);
    /// assert_eq!(option_double.apply(Some(21)), Some(42));
    /// assert_eq!(option_double.apply(None), None);
    /// ```
    pub fn map_option<F>(f: F) -> BoxFunction<Option<T>, Option<R>>
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxFunction::new(move |opt: Option<T>| opt.map(f))
    }
}

// Result-related methods
impl<T, E, R> BoxFunction<Result<T, E>, Result<R, E>>
where
    T: 'static,
    E: 'static,
    R: 'static,
{
    /// Creates a Result mapping function
    ///
    /// Converts a normal function into a function that operates on Result.
    ///
    /// # Parameters
    ///
    /// * `f` - The function to apply to Ok values
    ///
    /// # Returns
    ///
    /// Returns a Result mapping function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let double = |x: i32| x * 2;
    /// let result_double = BoxFunction::map_result(double);
    /// assert_eq!(result_double.apply(Ok::<i32, &str>(21)), Ok(42));
    /// assert_eq!(result_double.apply(Err::<i32, &str>("error")), Err("error"));
    /// ```
    pub fn map_result<F>(f: F) -> BoxFunction<Result<T, E>, Result<R, E>>
    where
        F: FnOnce(T) -> R + 'static,
    {
        BoxFunction::new(move |result: Result<T, E>| result.map(f))
    }
}

// Type conversion methods
impl<T, E> BoxFunction<Result<T, E>, Option<T>>
where
    T: 'static,
    E: 'static,
{
    /// Creates a Result to Option conversion function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let to_option = BoxFunction::<Result<i32, &str>, Option<i32>>::result_to_option();
    /// assert_eq!(to_option.apply(Ok(42)), Some(42));
    /// assert_eq!(to_option.apply(Err("error")), None);
    /// ```
    pub fn result_to_option() -> BoxFunction<Result<T, E>, Option<T>> {
        BoxFunction::new(|result: Result<T, E>| result.ok())
    }
}

impl<T, E> BoxFunction<Option<T>, Result<T, E>>
where
    T: 'static,
    E: Clone + 'static,
{
    /// Creates an Option to Result conversion function
    ///
    /// # Parameters
    ///
    /// * `error` - The error value to use when Option is None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let to_result = BoxFunction::option_to_result("value is missing");
    /// assert_eq!(to_result.apply(Some(42)), Ok(42));
    /// assert_eq!(to_result.apply(None), Err("value is missing"));
    /// ```
    pub fn option_to_result(error: E) -> BoxFunction<Option<T>, Result<T, E>> {
        BoxFunction::new(move |opt: Option<T>| opt.ok_or(error.clone()))
    }
}

impl<T, E> BoxFunction<Result<T, E>, T>
where
    T: 'static,
    E: 'static,
{
    /// Creates an unwrap function with error handling
    ///
    /// # Parameters
    ///
    /// * `f` - The function to convert error values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let use_default = |_err: &str| 0;
    /// let unwrap = BoxFunction::unwrap_or_else(use_default);
    /// assert_eq!(unwrap.apply(Ok(42)), 42);
    /// assert_eq!(unwrap.apply(Err("error")), 0);
    /// ```
    pub fn unwrap_or_else<F>(f: F) -> BoxFunction<Result<T, E>, T>
    where
        F: FnOnce(E) -> T + 'static,
    {
        BoxFunction::new(move |result: Result<T, E>| result.unwrap_or_else(f))
    }
}

impl<T, E, R> BoxFunction<Result<T, E>, R>
where
    T: 'static,
    E: 'static,
    R: 'static,
{
    /// Creates a Result matching function
    ///
    /// # Parameters
    ///
    /// * `on_ok` - The function to handle Ok values
    /// * `on_err` - The function to handle Err values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let handle_result = BoxFunction::match_result(
    ///     |x: i32| x.to_string(),
    ///     |e: &str| format!("Error: {}", e)
    /// );
    /// assert_eq!(handle_result.apply(Ok(42)), "42");
    /// assert_eq!(handle_result.apply(Err("failed")), "Error: failed");
    /// ```
    pub fn match_result<F, G>(on_ok: F, on_err: G) -> BoxFunction<Result<T, E>, R>
    where
        F: FnOnce(T) -> R + 'static,
        G: FnOnce(E) -> R + 'static,
    {
        BoxFunction::new(move |result: Result<T, E>| match result {
            Ok(value) => on_ok(value),
            Err(error) => on_err(error),
        })
    }
}

impl<T, E> BoxFunction<Result<Result<T, E>, E>, Result<T, E>>
where
    T: 'static,
    E: 'static,
{
    /// Creates a nested Result flattening function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let flatten = BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
    /// assert_eq!(flatten.apply(Ok(Ok(42))), Ok(42));
    /// assert_eq!(flatten.apply(Ok(Err("inner"))), Err("inner"));
    /// assert_eq!(flatten.apply(Err("outer")), Err("outer"));
    /// ```
    pub fn flatten_result() -> BoxFunction<Result<Result<T, E>, E>, Result<T, E>> {
        BoxFunction::new(|result: Result<Result<T, E>, E>| result.and_then(|inner| inner))
    }
}

impl<T> BoxFunction<Option<Option<T>>, Option<T>>
where
    T: 'static,
{
    /// Creates a nested Option flattening function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFunction;
    ///
    /// let flatten = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
    /// assert_eq!(flatten.apply(Some(Some(42))), Some(42));
    /// assert_eq!(flatten.apply(Some(None)), None);
    /// assert_eq!(flatten.apply(None), None);
    /// ```
    pub fn flatten_option() -> BoxFunction<Option<Option<T>>, Option<T>> {
        BoxFunction::new(|opt: Option<Option<T>>| opt.and_then(|inner| inner))
    }
}

// Implement Function trait for BoxFunction
impl<T, R> Function<T, R> for BoxFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    fn apply(self, input: T) -> R {
        self.apply(input)
    }

    fn into_fn(self) -> impl FnOnce(T) -> R
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static,
    {
        move |t: T| self.apply(t)
    }
}

// ============================================================================
// BoxFnFunction - Single ownership, reusable (Fn)
// ============================================================================

/// BoxFnFunction - function wrapper based on `Box<dyn Fn>`
///
/// A function wrapper that can be called multiple times with single
/// ownership. This provides reusability while maintaining the efficiency of
/// single ownership (no reference counting overhead).
///
/// # Features
///
/// - **Based on**: `Box<dyn Fn(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (`apply` uses `&self`)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
/// - **Composition**: Consumes `self` during composition (Box cannot be
///   cloned)
///
/// # Use Cases
///
/// - **Local repeated use**: When you need to call the function multiple
///   times within a single scope
/// - **Non-shared operations**: Transform operations that don't need to be
///   shared across threads or cloned
/// - **Efficiency over sharing**: When you want reusability without the
///   overhead of `Arc` or `Rc`
/// - **Stateless transformations**: Pure functions that can be safely called
///   multiple times
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed on each call)
/// * `R` - The type of the output value
///
/// # Important Notes
///
/// While `apply` can be called multiple times, the composition methods
/// (`and_then`, `compose`) still consume `self` because `Box<dyn Fn>` cannot
/// be cloned. If you need both reusability and composition without consuming
/// ownership, consider using [`ArcFnFunction`] or [`RcFnFunction`].
///
/// For non-`Copy` input types, each `apply` call requires a new input value
/// since the function consumes it.
///
/// # Examples
///
/// ## Basic Reusable Function
///
/// ```rust
/// use prism3_function::BoxFnFunction;
///
/// let double = BoxFnFunction::new(|x: i32| x * 2);
///
/// // Can be called multiple times
/// let r1 = double.apply(21);
/// let r2 = double.apply(42);
/// assert_eq!(r1, 42);
/// assert_eq!(r2, 84);
/// ```
///
/// ## With Non-Copy Types
///
/// ```rust
/// use prism3_function::BoxFnFunction;
///
/// let length = BoxFnFunction::new(|s: String| s.len());
///
/// // Each call consumes a String, but the function remains usable
/// assert_eq!(length.apply("hello".to_string()), 5);
/// assert_eq!(length.apply("world".to_string()), 5);
/// ```
///
/// ## Composition (Consumes Self)
///
/// ```rust
/// use prism3_function::BoxFnFunction;
///
/// let add_one = BoxFnFunction::new(|x: i32| x + 1);
/// let double = BoxFnFunction::new(|x: i32| x * 2);
///
/// // Composition consumes both functions
/// let composed = add_one.and_then(double);
/// assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
/// ```
///
/// # Comparison with Other Types
///
/// - **vs [`BoxFunction`]**: Use `BoxFnFunction` when you need multiple
///   calls; use `BoxFunction` for one-time use.
/// - **vs [`ArcFnFunction`]**: Use `BoxFnFunction` for single-threaded
///   scenarios without cloning needs; use `ArcFnFunction` for thread-safe
///   sharing.
/// - **vs [`RcFnFunction`]**: Use `BoxFnFunction` when you don't need to
///   clone; use `RcFnFunction` when you need single-threaded cloning and
///   composition without consuming ownership.
///
/// # Author
///
/// Hu Haixing
pub struct BoxFnFunction<T, R> {
    f: Box<dyn Fn(T) -> R>,
}

impl<T, R> BoxFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxFnFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        BoxFnFunction { f: Box::new(f) }
    }

    /// Applies the function to the input value
    ///
    /// Uses &self, so it can be called multiple times.
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// assert_eq!(double.apply(42), 84);
    /// ```
    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let identity = BoxFnFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> BoxFnFunction<T, T> {
        BoxFnFunction::new(|x| x)
    }

    /// Chain composition (consumes self)
    ///
    /// Note: Although apply can be called repeatedly, composition methods
    /// consume self because `Box<dyn Fn>` cannot be cloned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let add_one = BoxFnFunction::new(|x: i32| x + 1);
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// let composed = add_one.and_then(double);
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn and_then<S>(self, after: BoxFnFunction<R, S>) -> BoxFnFunction<T, S>
    where
        S: 'static,
    {
        let self_f = self.f;
        let after_f = after.f;
        BoxFnFunction::new(move |x| after_f(self_f(x)))
    }

    /// Reverse composition (consumes self)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// let add_one = BoxFnFunction::new(|x: i32| x + 1);
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn compose<S>(self, before: BoxFnFunction<S, T>) -> BoxFnFunction<S, R>
    where
        S: 'static,
    {
        let self_f = self.f;
        let before_f = before.f;
        BoxFnFunction::new(move |x| self_f(before_f(x)))
    }

    /// Converts function to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the function and returns a closure that can be
    /// directly used with iterator methods like `map()`, `flat_map()`, etc.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `Fn(T) -> R`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// let closure = double.into_fn();
    ///
    /// // Can be called multiple times
    /// assert_eq!(closure(5), 10);
    /// assert_eq!(closure(10), 20);
    /// // Note: double is consumed and no longer available
    /// ```
    ///
    /// ## With Iterator
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let double = BoxFnFunction::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let closure = double.into_fn();
    /// let doubled: Vec<i32> = values.iter()
    ///     .map(|&x| closure(x))
    ///     .collect();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    /// ```
    pub fn into_fn(self) -> impl Fn(T) -> R {
        move |t: T| (self.f)(t)
    }
}

impl<T, R> BoxFnFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxFnFunction;
    ///
    /// let constant = BoxFnFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// assert_eq!(constant.apply(456), "hello");
    /// ```
    pub fn constant(value: R) -> BoxFnFunction<T, R> {
        BoxFnFunction::new(move |_| value.clone())
    }
}

// ============================================================================
// ArcFnFunction - Multi-threaded sharing, reusable (Arc + Fn)
// ============================================================================

/// ArcFnFunction - function wrapper based on `Arc<dyn Fn>`
///
/// A thread-safe, clonable function wrapper suitable for multi-threaded
/// scenarios. This is the most flexible function type, allowing both
/// reusability and sharing across threads without consuming ownership during
/// composition.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(T) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (`apply` uses `&self`)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone` (increments reference
///   count)
/// - **Composition**: Uses `&self`, preserving original functions after
///   composition
///
/// # Use Cases
///
/// - **Multi-threaded processing**: Share transformation logic across threads
/// - **Concurrent pipelines**: Parallel data processing with shared
///   transformers
/// - **Shared configuration**: Store transformation functions in shared
///   configuration
/// - **Event systems**: Multi-threaded event handlers and callbacks
/// - **Long-lived functions**: Functions that need to outlive their creators
/// - **Functional composition**: Build complex function networks without
///   consuming components
///
/// # Type Parameters
///
/// * `T` - The type of the input value (must be `Send` if used across
///   threads)
/// * `R` - The type of the output value (must be `Send` if used across
///   threads)
///
/// # Performance Considerations
///
/// - **Memory overhead**: Uses `Arc` for reference counting (8-16 bytes
///   overhead)
/// - **Atomic operations**: Clone and drop involve atomic operations
/// - **Trade-off**: Slightly slower than `Box` but enables sharing and
///   composition
///
/// # Examples
///
/// ## Basic Thread-Safe Usage
///
/// ```rust
/// use prism3_function::ArcFnFunction;
/// use std::thread;
///
/// let double = ArcFnFunction::new(|x: i32| x * 2);
/// let cloned = double.clone();
///
/// let handle = thread::spawn(move || cloned.apply(21));
/// assert_eq!(handle.join().unwrap(), 42);
///
/// // Original still usable
/// assert_eq!(double.apply(42), 84);
/// ```
///
/// ## Composition Without Consuming
///
/// ```rust
/// use prism3_function::ArcFnFunction;
///
/// let add_one = ArcFnFunction::new(|x: i32| x + 1);
/// let double = ArcFnFunction::new(|x: i32| x * 2);
///
/// // Composition uses &self, original functions remain usable
/// let composed = add_one.and_then(&double);
///
/// assert_eq!(add_one.apply(5), 6);      // Still usable
/// assert_eq!(double.apply(5), 10);      // Still usable
/// assert_eq!(composed.apply(5), 12);    // (5 + 1) * 2
/// ```
///
/// ## Shared Configuration
///
/// ```rust
/// use prism3_function::ArcFnFunction;
/// use std::sync::Arc;
///
/// struct Config {
///     transformer: ArcFnFunction<i32, String>,
/// }
///
/// let config = Arc::new(Config {
///     transformer: ArcFnFunction::new(|x| format!("Value: {}", x * 2)),
/// });
///
/// // Multiple threads can share the config
/// let result = config.transformer.apply(21);
/// assert_eq!(result, "Value: 42");
/// ```
///
/// ## Multiple Thread Processing
///
/// ```rust
/// use prism3_function::ArcFnFunction;
/// use std::thread;
///
/// let processor = ArcFnFunction::new(|x: i32| x * x);
///
/// let handles: Vec<_> = (0..4)
///     .map(|i| {
///         let proc = processor.clone();
///         thread::spawn(move || proc.apply(i))
///     })
///     .collect();
///
/// let results: Vec<_> = handles.into_iter()
///     .map(|h| h.join().unwrap())
///     .collect();
///
/// assert_eq!(results, vec![0, 1, 4, 9]);
/// ```
///
/// # Comparison with Other Types
///
/// - **vs [`BoxFunction`]**: Use `ArcFnFunction` for reusable, shareable
///   functions; use `BoxFunction` for one-time use.
/// - **vs [`BoxFnFunction`]**: Use `ArcFnFunction` for thread-safe sharing
///   or composition without consuming; use `BoxFnFunction` for local reuse.
/// - **vs [`RcFnFunction`]**: Use `ArcFnFunction` for multi-threaded
///   scenarios; use `RcFnFunction` for single-threaded scenarios (slightly
///   faster).
///
/// # Author
///
/// Hu Haixing
pub struct ArcFnFunction<T, R> {
    f: Arc<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> ArcFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new ArcFnFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send + Sync)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        ArcFnFunction { f: Arc::new(f) }
    }

    /// Applies the function to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// assert_eq!(double.apply(42), 84);
    /// ```
    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let identity = ArcFnFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> ArcFnFunction<T, T>
    where
        T: Send + Sync,
    {
        ArcFnFunction::new(|x| x)
    }

    /// Chain composition (uses &self, not consuming ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let add_one = ArcFnFunction::new(|x: i32| x + 1);
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let composed = add_one.and_then(&double);
    ///
    /// // Original function can still be used
    /// assert_eq!(add_one.apply(5), 6);
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn and_then<S>(&self, after: &ArcFnFunction<R, S>) -> ArcFnFunction<T, S>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let after_clone = Arc::clone(&after.f);
        ArcFnFunction {
            f: Arc::new(move |x| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition (uses &self, not consuming ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let add_one = ArcFnFunction::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn compose<S>(&self, before: &ArcFnFunction<S, T>) -> ArcFnFunction<S, R>
    where
        S: Send + Sync + 'static,
    {
        let self_clone = Arc::clone(&self.f);
        let before_clone = Arc::clone(&before.f);
        ArcFnFunction {
            f: Arc::new(move |x| self_clone(before_clone(x))),
        }
    }

    /// Converts function to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the function and returns a closure that can be
    /// directly used with iterator methods. Since this uses Arc internally,
    /// the returned closure is cloneable and can be used multiple times.
    ///
    /// **Tip**: Since `ArcFnFunction` is `Clone`, you can call `.clone()`
    /// first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let closure = double.clone().into_fn();  // Clone first
    ///
    /// // Original still available
    /// assert_eq!(double.apply(5), 10);
    /// assert_eq!(closure(5), 10);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `Fn(T) -> R + Clone + Send + Sync`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let closure = double.into_fn();
    ///
    /// // Can be called multiple times
    /// assert_eq!(closure(5), 10);
    /// assert_eq!(closure(10), 20);
    /// // Note: double is consumed and no longer available
    /// ```
    ///
    /// ## With Iterator
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let closure = double.into_fn();
    /// let doubled: Vec<i32> = values.iter()
    ///     .map(|&x| closure(x))
    ///     .collect();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## Clone Before Conversion
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    ///
    /// // Clone before conversion to keep the original
    /// let closure = double.clone().into_fn();
    ///
    /// // Both are still usable
    /// assert_eq!(double.apply(5), 10);
    /// assert_eq!(closure(5), 10);
    /// ```
    ///
    /// ## Closure is Cloneable
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let double = ArcFnFunction::new(|x: i32| x * 2);
    /// let closure1 = double.into_fn();
    /// let closure2 = closure1.clone();
    ///
    /// // Both closures work
    /// assert_eq!(closure1(5), 10);
    /// assert_eq!(closure2(10), 20);
    /// ```
    pub fn into_fn(self) -> impl Fn(T) -> R + Clone + Send + Sync {
        let f = self.f;
        move |t: T| f(t)
    }
}

impl<T, R> Clone for ArcFnFunction<T, R> {
    fn clone(&self) -> Self {
        ArcFnFunction {
            f: Arc::clone(&self.f),
        }
    }
}

impl<T, R> ArcFnFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcFnFunction;
    ///
    /// let constant = ArcFnFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// assert_eq!(constant.apply(456), "hello");
    /// ```
    pub fn constant(value: R) -> ArcFnFunction<T, R>
    where
        R: Send + Sync,
    {
        ArcFnFunction::new(move |_| value.clone())
    }
}

// ============================================================================
// RcFnFunction - Single-threaded sharing, reusable (Rc + Fn)
// ============================================================================

/// RcFnFunction - function wrapper based on `Rc<dyn Fn>`
///
/// A single-threaded, clonable function wrapper optimized for scenarios that
/// require sharing and composition without thread-safety overhead. This is
/// the single-threaded equivalent of [`ArcFnFunction`], offering similar
/// flexibility with better performance in non-concurrent contexts.
///
/// # Features
///
/// - **Based on**: `Rc<dyn Fn(T) -> R>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (`apply` uses `&self`)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone` (increments reference
///   count)
/// - **Composition**: Uses `&self`, preserving original functions after
///   composition
///
/// # Use Cases
///
/// - **Single-threaded callbacks**: Event handlers and callback systems
/// - **UI event handlers**: GUI applications with complex event processing
/// - **Functional composition**: Building complex function networks without
///   consuming components
/// - **Shared state transformations**: When multiple components need the same
///   transformation logic
/// - **Performance-critical single-threaded code**: Where `Arc`'s atomic
///   operations are unnecessary overhead
/// - **Local function registries**: Storing and sharing functions within a
///   single thread
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed on each call)
/// * `R` - The type of the output value
///
/// # Performance Considerations
///
/// - **Memory overhead**: Uses `Rc` for reference counting (4-8 bytes
///   overhead)
/// - **Non-atomic operations**: Faster than `Arc` (no atomic operations)
/// - **Single-threaded only**: Cannot be sent across threads
/// - **Trade-off**: Slightly slower than `Box` but enables sharing and
///   composition
///
/// # Examples
///
/// ## Basic Clonable Usage
///
/// ```rust
/// use prism3_function::RcFnFunction;
///
/// let double = RcFnFunction::new(|x: i32| x * 2);
/// let cloned = double.clone();
///
/// assert_eq!(double.apply(21), 42);
/// assert_eq!(cloned.apply(42), 84);
/// ```
///
/// ## Composition Without Consuming
///
/// ```rust
/// use prism3_function::RcFnFunction;
///
/// let add_one = RcFnFunction::new(|x: i32| x + 1);
/// let double = RcFnFunction::new(|x: i32| x * 2);
///
/// // Composition uses &self, original functions remain usable
/// let composed = add_one.and_then(&double);
///
/// assert_eq!(add_one.apply(5), 6);      // Still usable
/// assert_eq!(double.apply(5), 10);      // Still usable
/// assert_eq!(composed.apply(5), 12);    // (5 + 1) * 2
/// ```
///
/// ## Callback Registry
///
/// ```rust
/// use prism3_function::RcFnFunction;
/// use std::collections::HashMap;
///
/// struct EventSystem {
///     handlers: HashMap<String, RcFnFunction<i32, String>>,
/// }
///
/// impl EventSystem {
///     fn new() -> Self {
///         EventSystem {
///             handlers: HashMap::new(),
///         }
///     }
///
///     fn register(&mut self, event: &str, handler: RcFnFunction<i32, String>) {
///         self.handlers.insert(event.to_string(), handler);
///     }
///
///     fn trigger(&self, event: &str, value: i32) -> Option<String> {
///         self.handlers.get(event).map(|h| h.apply(value))
///     }
/// }
///
/// let mut system = EventSystem::new();
/// let handler = RcFnFunction::new(|x| format!("Value: {}", x * 2));
///
/// system.register("double", handler.clone());
///
/// // Handler can still be used independently
/// assert_eq!(handler.apply(21), "Value: 42");
/// assert_eq!(system.trigger("double", 21), Some("Value: 42".to_string()));
/// ```
///
/// ## Complex Function Network
///
/// ```rust
/// use prism3_function::RcFnFunction;
///
/// // Build reusable transformation components
/// let parse = RcFnFunction::new(|s: String| s.parse::<i32>().unwrap_or(0));
/// let double = RcFnFunction::new(|x: i32| x * 2);
/// let format = RcFnFunction::new(|x: i32| format!("Result: {}", x));
///
/// // Compose into different pipelines without consuming components
/// let pipeline1 = parse.and_then(&double).and_then(&format);
/// let pipeline2 = parse.and_then(&format);
///
/// assert_eq!(pipeline1.apply("21".to_string()), "Result: 42");
/// assert_eq!(pipeline2.apply("21".to_string()), "Result: 21");
///
/// // Original functions still usable
/// assert_eq!(parse.apply("42".to_string()), 42);
/// assert_eq!(double.apply(10), 20);
/// ```
///
/// # Comparison with Other Types
///
/// - **vs [`BoxFunction`]**: Use `RcFnFunction` for reusable, shareable
///   functions; use `BoxFunction` for one-time use.
/// - **vs [`BoxFnFunction`]**: Use `RcFnFunction` for cloning and
///   composition without consuming; use `BoxFnFunction` for local reuse
///   without cloning.
/// - **vs [`ArcFnFunction`]**: Use `RcFnFunction` for single-threaded
///   scenarios (faster); use `ArcFnFunction` for multi-threaded scenarios.
///
/// # Author
///
/// Hu Haixing
pub struct RcFnFunction<T, R> {
    f: Rc<dyn Fn(T) -> R>,
}

impl<T, R> RcFnFunction<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcFnFunction
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        RcFnFunction { f: Rc::new(f) }
    }

    /// Applies the function to the input value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value
    ///
    /// # Returns
    ///
    /// The transformed output value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// assert_eq!(double.apply(21), 42);
    /// assert_eq!(double.apply(42), 84);
    /// ```
    pub fn apply(&self, input: T) -> R {
        (self.f)(input)
    }

    /// Creates an identity function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let identity = RcFnFunction::<i32, i32>::identity();
    /// assert_eq!(identity.apply(42), 42);
    /// ```
    pub fn identity() -> RcFnFunction<T, T> {
        RcFnFunction::new(|x| x)
    }

    /// Chain composition (uses &self, not consuming ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let add_one = RcFnFunction::new(|x: i32| x + 1);
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let composed = add_one.and_then(&double);
    ///
    /// // Original function can still be used
    /// assert_eq!(add_one.apply(5), 6);
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn and_then<S>(&self, after: &RcFnFunction<R, S>) -> RcFnFunction<T, S>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let after_clone = Rc::clone(&after.f);
        RcFnFunction {
            f: Rc::new(move |x| after_clone(self_clone(x))),
        }
    }

    /// Reverse composition (uses &self, not consuming ownership)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let add_one = RcFnFunction::new(|x: i32| x + 1);
    /// let composed = double.compose(&add_one);
    ///
    /// assert_eq!(composed.apply(5), 12);
    /// ```
    pub fn compose<S>(&self, before: &RcFnFunction<S, T>) -> RcFnFunction<S, R>
    where
        S: 'static,
    {
        let self_clone = Rc::clone(&self.f);
        let before_clone = Rc::clone(&before.f);
        RcFnFunction {
            f: Rc::new(move |x| self_clone(before_clone(x))),
        }
    }

    /// Converts function to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original function becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the function and returns a closure that can be
    /// directly used with iterator methods. Since this uses Rc internally,
    /// the returned closure is cloneable and can be used multiple times.
    ///
    /// **Tip**: Since `RcFnFunction` is `Clone`, you can call `.clone()`
    /// first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let closure = double.clone().into_fn();  // Clone first
    ///
    /// // Original still available
    /// assert_eq!(double.apply(5), 10);
    /// assert_eq!(closure(5), 10);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `Fn(T) -> R + Clone`
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let closure = double.into_fn();
    ///
    /// // Can be called multiple times
    /// assert_eq!(closure(5), 10);
    /// assert_eq!(closure(10), 20);
    /// // Note: double is consumed and no longer available
    /// ```
    ///
    /// ## With Iterator
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let values = vec![1, 2, 3, 4, 5];
    ///
    /// let closure = double.into_fn();
    /// let doubled: Vec<i32> = values.iter()
    ///     .map(|&x| closure(x))
    ///     .collect();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## Clone Before Conversion
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    ///
    /// // Clone before conversion to keep the original
    /// let closure = double.clone().into_fn();
    ///
    /// // Both are still usable
    /// assert_eq!(double.apply(5), 10);
    /// assert_eq!(closure(5), 10);
    /// ```
    ///
    /// ## Closure is Cloneable
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let double = RcFnFunction::new(|x: i32| x * 2);
    /// let closure1 = double.into_fn();
    /// let closure2 = closure1.clone();
    ///
    /// // Both closures work
    /// assert_eq!(closure1(5), 10);
    /// assert_eq!(closure2(10), 20);
    /// ```
    pub fn into_fn(self) -> impl Fn(T) -> R + Clone {
        let f = self.f;
        move |t: T| f(t)
    }
}

impl<T, R> Clone for RcFnFunction<T, R> {
    fn clone(&self) -> Self {
        RcFnFunction {
            f: Rc::clone(&self.f),
        }
    }
}

impl<T, R> RcFnFunction<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcFnFunction;
    ///
    /// let constant = RcFnFunction::constant("hello");
    /// assert_eq!(constant.apply(123), "hello");
    /// assert_eq!(constant.apply(456), "hello");
    /// ```
    pub fn constant(value: R) -> RcFnFunction<T, R> {
        RcFnFunction::new(move |_| value.clone())
    }
}
