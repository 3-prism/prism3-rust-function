/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Mapper Types
//!
//! Provides Rust implementations of mapper traits for stateful value
//! transformation. Mappers consume input values (taking ownership) and
//! produce output values while allowing internal state modification.
//!
//! This module provides the `Mapper<T, R>` trait and three implementations:
//!
//! - [`BoxMapper`]: Single ownership, not cloneable
//! - [`ArcMapper`]: Thread-safe shared ownership, cloneable
//! - [`RcMapper`]: Single-threaded shared ownership, cloneable
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

// ============================================================================
// Core Trait
// ============================================================================

/// Mapper trait - transforms values from type T to type R with state
///
/// Defines the behavior of a stateful transformation: converting a value
/// of type `T` to a value of type `R` by consuming the input while
/// allowing modification of internal state. This is analogous to
/// `FnMut(T) -> R` in Rust's standard library.
///
/// # Type Parameters
///
/// * `T` - The type of the input value (consumed)
/// * `R` - The type of the output value
///
/// # Author
///
/// Haixing Hu
pub trait Mapper<T, R> {
    /// Maps the input value to produce an output value
    ///
    /// # Parameters
    ///
    /// * `input` - The input value to transform (consumed)
    ///
    /// # Returns
    ///
    /// The transformed output value
    fn map(&mut self, input: T) -> R;

    /// Converts to BoxMapper
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `BoxMapper<T, R>`
    fn into_box(self) -> BoxMapper<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to RcMapper
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `RcMapper<T, R>`
    fn into_rc(self) -> RcMapper<T, R>
    where
        Self: Sized + 'static,
        T: 'static,
        R: 'static;

    /// Converts to ArcMapper
    ///
    /// **⚠️ Consumes `self`**: The original mapper becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns `ArcMapper<T, R>`
    fn into_arc(self) -> ArcMapper<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static;
}

// ============================================================================
// BoxMapper - Box<dyn FnMut(T) -> R>
// ============================================================================

/// BoxMapper - mapper wrapper based on `Box<dyn FnMut>`
///
/// A mapper wrapper that provides single ownership with reusable stateful
/// transformation. The mapper consumes the input and can be called
/// multiple times while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(T) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct BoxMapper<T, R> {
    function: Box<dyn FnMut(T) -> R>,
}

impl<T, R> BoxMapper<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new BoxMapper
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = BoxMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.map(10), 11);
    /// assert_eq!(mapper.map(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + 'static,
    {
        BoxMapper {
            function: Box::new(f),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapper, Mapper};
    ///
    /// let mut identity = BoxMapper::<i32, i32>::identity();
    /// assert_eq!(identity.map(42), 42);
    /// ```
    pub fn identity() -> BoxMapper<T, T> {
        BoxMapper::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement Mapper<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original mapper, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxMapper<R, S>`
    ///   - An `RcMapper<R, S>`
    ///   - An `ArcMapper<R, S>`
    ///   - Any type implementing `Mapper<R, S>`
    ///
    /// # Returns
    ///
    /// A new BoxMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapper, Mapper};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = BoxMapper::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = BoxMapper::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    /// assert_eq!(composed.map(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(mut self, mut after: F) -> BoxMapper<T, S>
    where
        S: 'static,
        F: Mapper<R, S> + 'static,
    {
        BoxMapper::new(move |x: T| {
            let intermediate = self.map(x);
            after.map(intermediate)
        })
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement Mapper<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If
    ///   you need to preserve the original mapper, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxMapper<S, T>`
    ///   - An `RcMapper<S, T>`
    ///   - An `ArcMapper<S, T>`
    ///   - Any type implementing `Mapper<S, T>`
    ///
    /// # Returns
    ///
    /// A new BoxMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mapper = BoxMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(mut self, mut before: F) -> BoxMapper<S, R>
    where
        S: 'static,
        F: Mapper<S, T> + 'static,
    {
        BoxMapper::new(move |x: S| {
            let intermediate = before.map(x);
            self.map(intermediate)
        })
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper for
    /// when the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is
    ///   passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, BoxMapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = BoxMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.map(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.map(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalMapper<T, R>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalMapper {
            mapper: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, R> BoxMapper<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxMapper, Mapper};
    ///
    /// let mut constant = BoxMapper::constant("hello");
    /// assert_eq!(constant.map(123), "hello");
    /// ```
    pub fn constant(value: R) -> BoxMapper<T, R> {
        BoxMapper::new(move |_| value.clone())
    }
}

impl<T, R> Mapper<T, R> for BoxMapper<T, R> {
    fn map(&mut self, input: T) -> R {
        (self.function)(input)
    }

    fn into_box(self) -> BoxMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcMapper {
            function: Rc::new(RefCell::new(self.function)),
        }
    }

    fn into_arc(self) -> ArcMapper<T, R>
    where
        Self: Send,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        unreachable!(
            "BoxMapper<T, R> does not implement Send, so this method can \
             never be called"
        )
    }
}

// ============================================================================
// BoxConditionalMapper - Box-based Conditional Mapper
// ============================================================================

/// BoxConditionalMapper struct
///
/// A conditional mapper that only executes when a predicate is satisfied.
/// Uses `BoxMapper` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxMapper::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else
///   logic
/// - **Implements Mapper**: Can be used anywhere a `Mapper` is expected
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mapper, BoxMapper};
///
/// let mut high_count = 0;
/// let mut low_count = 0;
///
/// let mut mapper = BoxMapper::new(move |x: i32| {
///     high_count += 1;
///     x * 2
/// })
/// .when(|x: &i32| *x >= 10)
/// .or_else(move |x| {
///     low_count += 1;
///     x + 1
/// });
///
/// assert_eq!(mapper.map(15), 30); // when branch executed
/// assert_eq!(mapper.map(5), 6);   // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalMapper<T, R> {
    mapper: BoxMapper<T, R>,
    predicate: BoxPredicate<T>,
}

impl<T, R> BoxConditionalMapper<T, R>
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
    ///   - `BoxMapper<T, R>`, `RcMapper<T, R>`, `ArcMapper<T, R>`
    ///   - Any type implementing `Mapper<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, BoxMapper};
    ///
    /// let mut mapper = BoxMapper::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.map(5), 10);   // Condition satisfied
    /// assert_eq!(mapper.map(-5), 5);   // Condition not satisfied
    /// ```
    pub fn or_else<F>(self, mut else_mapper: F) -> BoxMapper<T, R>
    where
        F: Mapper<T, R> + 'static,
    {
        let pred = self.predicate;
        let mut then_mapper = self.mapper;
        BoxMapper::new(move |t| {
            if pred.test(&t) {
                then_mapper.map(t)
            } else {
                else_mapper.map(t)
            }
        })
    }
}

// ============================================================================
// ArcMapper - Arc<Mutex<dyn FnMut(T) -> R + Send>>
// ============================================================================

/// ArcMapper - thread-safe mapper wrapper
///
/// A thread-safe, clonable mapper wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads
/// while maintaining internal state.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(T) -> R + Send>>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Thread-safe (`Send` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct ArcMapper<T, R> {
    function: Arc<Mutex<dyn FnMut(T) -> R + Send>>,
}

impl<T, R> ArcMapper<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Creates a new ArcMapper
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap (must be Send)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = ArcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.map(10), 11);
    /// assert_eq!(mapper.map(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + Send + 'static,
    {
        ArcMapper {
            function: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcMapper, Mapper};
    ///
    /// let mut identity = ArcMapper::<i32, i32>::identity();
    /// assert_eq!(identity.map(42), 42);
    /// ```
    pub fn identity() -> ArcMapper<T, T> {
        ArcMapper::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement Mapper<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxMapper<R, S>`
    ///   - An `RcMapper<R, S>`
    ///   - An `ArcMapper<R, S>` (will be cloned internally)
    ///   - Any type implementing `Mapper<R, S> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcMapper, Mapper};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = ArcMapper::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = ArcMapper::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    ///
    /// assert_eq!(composed.map(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> ArcMapper<T, S>
    where
        S: Send + 'static,
        F: Mapper<R, S> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let after = Arc::new(Mutex::new(after));
        ArcMapper {
            function: Arc::new(Mutex::new(move |x: T| {
                let intermediate = self_fn.lock().unwrap()(x);
                after.lock().unwrap().map(intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement Mapper<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxMapper<S, T>`
    ///   - An `RcMapper<S, T>`
    ///   - An `ArcMapper<S, T>` (will be cloned internally)
    ///   - Any type implementing `Mapper<S, T> + Send`
    ///
    /// # Returns
    ///
    /// A new ArcMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mapper = ArcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> ArcMapper<S, R>
    where
        S: Send + 'static,
        F: Mapper<S, T> + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let before = Arc::new(Mutex::new(before));
        ArcMapper {
            function: Arc::new(Mutex::new(move |x: S| {
                let intermediate = before.lock().unwrap().map(x);
                self_fn.lock().unwrap()(intermediate)
            })),
        }
    }

    /// Creates a conditional mapper (thread-safe version)
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Must be `Send`, can be:
    ///   - A closure: `|x: &T| -> bool` (requires `Send`)
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T> + Send`
    ///
    /// # Returns
    ///
    /// Returns `ArcConditionalMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, ArcMapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = ArcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.map(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.map(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> ArcConditionalMapper<T, R>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        ArcConditionalMapper {
            mapper: self,
            predicate: predicate.into_arc(),
        }
    }
}

impl<T, R> ArcMapper<T, R>
where
    T: Send + Sync + 'static,
    R: Clone + Send + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcMapper, Mapper};
    ///
    /// let mut constant = ArcMapper::constant("hello");
    /// assert_eq!(constant.map(123), "hello");
    /// ```
    pub fn constant(value: R) -> ArcMapper<T, R> {
        ArcMapper::new(move |_| value.clone())
    }
}

impl<T, R> Mapper<T, R> for ArcMapper<T, R> {
    fn map(&mut self, input: T) -> R {
        (self.function.lock().unwrap())(input)
    }

    fn into_box(self) -> BoxMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxMapper {
            function: Box::new(move |x| self.function.lock().unwrap()(x)),
        }
    }

    fn into_rc(self) -> RcMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        RcMapper {
            function: Rc::new(RefCell::new(Box::new(move |x| {
                self.function.lock().unwrap()(x)
            }))),
        }
    }

    fn into_arc(self) -> ArcMapper<T, R>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        // Zero-cost: directly return itself
        self
    }
}

impl<T, R> Clone for ArcMapper<T, R> {
    fn clone(&self) -> Self {
        ArcMapper {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// ArcConditionalMapper - Arc-based Conditional Mapper
// ============================================================================

/// ArcConditionalMapper struct
///
/// A thread-safe conditional mapper that only executes when a predicate
/// is satisfied. Uses `ArcMapper` and `ArcPredicate` for shared
/// ownership across threads.
///
/// This type is typically created by calling `ArcMapper::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send`, safe for concurrent use
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else
///   logic
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mapper, ArcMapper};
///
/// let mut mapper = ArcMapper::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut mapper_clone = mapper.clone();
///
/// assert_eq!(mapper.map(5), 10);
/// assert_eq!(mapper_clone.map(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConditionalMapper<T, R> {
    mapper: ArcMapper<T, R>,
    predicate: ArcPredicate<T>,
}

impl<T, R> ArcConditionalMapper<T, R>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    /// Adds an else branch (thread-safe version)
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R` (must be `Send`)
    ///   - `ArcMapper<T, R>`, `BoxMapper<T, R>`
    ///   - Any type implementing `Mapper<T, R> + Send`
    ///
    /// # Returns
    ///
    /// Returns the composed `ArcMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, ArcMapper};
    ///
    /// let mut mapper = ArcMapper::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.map(5), 10);
    /// assert_eq!(mapper.map(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_mapper: F) -> ArcMapper<T, R>
    where
        F: Mapper<T, R> + Send + 'static,
    {
        let pred = self.predicate;
        let then_mapper = self.mapper;
        let else_mapper = Arc::new(Mutex::new(else_mapper));
        ArcMapper {
            function: Arc::new(Mutex::new(move |t| {
                if pred.test(&t) {
                    then_mapper.function.lock().unwrap()(t)
                } else {
                    else_mapper.lock().unwrap().map(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for ArcConditionalMapper<T, R> {
    /// Clones the conditional mapper
    ///
    /// Creates a new instance that shares the underlying mapper and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// RcMapper - Rc<RefCell<dyn FnMut(T) -> R>>
// ============================================================================

/// RcMapper - single-threaded mapper wrapper
///
/// A single-threaded, clonable mapper wrapper optimized for scenarios
/// that require sharing without thread-safety overhead.
///
/// # Features
///
/// - **Based on**: `Rc<RefCell<dyn FnMut(T) -> R>>`
/// - **Ownership**: Shared ownership via reference counting (non-atomic)
/// - **Reusability**: Can be called multiple times (each call consumes
///   its input)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync`)
/// - **Clonable**: Cheap cloning via `Rc::clone`
/// - **Statefulness**: Can modify internal state between calls
///
/// # Author
///
/// Haixing Hu
pub struct RcMapper<T, R> {
    function: Rc<RefCell<dyn FnMut(T) -> R>>,
}

impl<T, R> RcMapper<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Creates a new RcMapper
    ///
    /// # Parameters
    ///
    /// * `f` - The closure or function to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = RcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x + counter
    /// });
    /// assert_eq!(mapper.map(10), 11);
    /// assert_eq!(mapper.map(10), 12);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(T) -> R + 'static,
    {
        RcMapper {
            function: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates an identity mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcMapper, Mapper};
    ///
    /// let mut identity = RcMapper::<i32, i32>::identity();
    /// assert_eq!(identity.map(42), 42);
    /// ```
    pub fn identity() -> RcMapper<T, T> {
        RcMapper::new(|x| x)
    }

    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement Mapper<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxMapper<R, S>`
    ///   - An `RcMapper<R, S>` (will be cloned internally)
    ///   - An `ArcMapper<R, S>`
    ///   - Any type implementing `Mapper<R, S>`
    ///
    /// # Returns
    ///
    /// A new RcMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcMapper, Mapper};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = RcMapper::new(move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// });
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = RcMapper::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    ///
    /// assert_eq!(composed.map(10), 11);  // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 24);  // (10 + 2) * 2
    /// ```
    pub fn and_then<S, F>(&self, after: F) -> RcMapper<T, S>
    where
        S: 'static,
        F: Mapper<R, S> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let after = Rc::new(RefCell::new(after));
        RcMapper {
            function: Rc::new(RefCell::new(move |x: T| {
                let intermediate = self_fn.borrow_mut()(x);
                after.borrow_mut().map(intermediate)
            })),
        }
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Uses &self, so original mapper
    /// remains usable.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement Mapper<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxMapper<S, T>`
    ///   - An `RcMapper<S, T>` (will be cloned internally)
    ///   - An `ArcMapper<S, T>`
    ///   - Any type implementing `Mapper<S, T>`
    ///
    /// # Returns
    ///
    /// A new RcMapper representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcMapper, Mapper};
    ///
    /// let mut counter = 0;
    /// let mapper = RcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// });
    ///
    /// let mut composed = mapper.compose(|x: i32| x + 1);
    /// assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    /// assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    /// ```
    pub fn compose<S, F>(&self, before: F) -> RcMapper<S, R>
    where
        S: 'static,
        F: Mapper<S, T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let before = Rc::new(RefCell::new(before));
        RcMapper {
            function: Rc::new(RefCell::new(move |x: S| {
                let intermediate = before.borrow_mut().map(x);
                self_fn.borrow_mut()(intermediate)
            })),
        }
    }

    /// Creates a conditional mapper (single-threaded shared version)
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `RcConditionalMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, RcMapper};
    ///
    /// let mut counter = 0;
    /// let mut mapper = RcMapper::new(move |x: i32| {
    ///     counter += 1;
    ///     x * 2
    /// })
    /// .when(|x: &i32| *x > 10)
    /// .or_else(|x| x + 1);
    ///
    /// assert_eq!(mapper.map(15), 30);  // 15 > 10, apply * 2
    /// assert_eq!(mapper.map(5), 6);    // 5 <= 10, apply + 1
    /// ```
    pub fn when<P>(self, predicate: P) -> RcConditionalMapper<T, R>
    where
        P: Predicate<T> + 'static,
    {
        RcConditionalMapper {
            mapper: self,
            predicate: predicate.into_rc(),
        }
    }
}

impl<T, R> RcMapper<T, R>
where
    T: 'static,
    R: Clone + 'static,
{
    /// Creates a constant mapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcMapper, Mapper};
    ///
    /// let mut constant = RcMapper::constant("hello");
    /// assert_eq!(constant.map(123), "hello");
    /// ```
    pub fn constant(value: R) -> RcMapper<T, R> {
        RcMapper::new(move |_| value.clone())
    }
}

impl<T, R> Mapper<T, R> for RcMapper<T, R> {
    fn map(&mut self, input: T) -> R {
        (self.function.borrow_mut())(input)
    }

    fn into_box(self) -> BoxMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        BoxMapper {
            function: Box::new(move |x| self.function.borrow_mut()(x)),
        }
    }

    fn into_rc(self) -> RcMapper<T, R>
    where
        T: 'static,
        R: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcMapper<T, R>
    where
        Self: Send,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        unreachable!(
            "RcMapper cannot be converted to ArcMapper because Rc is not \
             Send"
        )
    }
}

impl<T, R> Clone for RcMapper<T, R> {
    fn clone(&self) -> Self {
        RcMapper {
            function: Rc::clone(&self.function),
        }
    }
}

// ============================================================================
// RcConditionalMapper - Rc-based Conditional Mapper
// ============================================================================

/// RcConditionalMapper struct
///
/// A single-threaded conditional mapper that only executes when a
/// predicate is satisfied. Uses `RcMapper` and `RcPredicate` for shared
/// ownership within a single thread.
///
/// This type is typically created by calling `RcMapper::when()` and is
/// designed to work with the `or_else()` method to create if-then-else
/// logic.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Conditional Execution**: Only maps when predicate returns `true`
/// - **No Lock Overhead**: More efficient than `ArcConditionalMapper`
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mapper, RcMapper};
///
/// let mut mapper = RcMapper::new(|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// let mut mapper_clone = mapper.clone();
///
/// assert_eq!(mapper.map(5), 10);
/// assert_eq!(mapper_clone.map(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConditionalMapper<T, R> {
    mapper: RcMapper<T, R>,
    predicate: RcPredicate<T>,
}

impl<T, R> RcConditionalMapper<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Adds an else branch (single-threaded shared version)
    ///
    /// Executes the original mapper when the condition is satisfied,
    /// otherwise executes else_mapper.
    ///
    /// # Parameters
    ///
    /// * `else_mapper` - The mapper for the else branch, can be:
    ///   - Closure: `|x: T| -> R`
    ///   - `RcMapper<T, R>`, `BoxMapper<T, R>`
    ///   - Any type implementing `Mapper<T, R>`
    ///
    /// # Returns
    ///
    /// Returns the composed `RcMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, RcMapper};
    ///
    /// let mut mapper = RcMapper::new(|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.map(5), 10);
    /// assert_eq!(mapper.map(-5), 5);
    /// ```
    pub fn or_else<F>(self, else_mapper: F) -> RcMapper<T, R>
    where
        F: Mapper<T, R> + 'static,
    {
        let pred = self.predicate;
        let then_mapper = self.mapper;
        let else_mapper = Rc::new(RefCell::new(else_mapper));
        RcMapper {
            function: Rc::new(RefCell::new(move |t| {
                if pred.test(&t) {
                    then_mapper.function.borrow_mut()(t)
                } else {
                    else_mapper.borrow_mut().map(t)
                }
            })),
        }
    }
}

impl<T, R> Clone for RcConditionalMapper<T, R> {
    /// Clones the conditional mapper
    ///
    /// Creates a new instance that shares the underlying mapper and
    /// predicate with the original instance.
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

// ============================================================================
// Blanket implementation for standard FnMut trait
// ============================================================================

/// Implement Mapper<T, R> for any type that implements FnMut(T) -> R
///
/// This allows closures to be used directly with our Mapper trait
/// without wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::Mapper;
///
/// let mut counter = 0;
/// let mut mapper = |x: i32| {
///     counter += 1;
///     x + counter
/// };
///
/// assert_eq!(mapper.map(10), 11);
/// assert_eq!(mapper.map(10), 12);
/// ```
///
/// # Author
///
/// Haixing Hu
impl<F, T, R> Mapper<T, R> for F
where
    F: FnMut(T) -> R,
    T: 'static,
    R: 'static,
{
    fn map(&mut self, input: T) -> R {
        self(input)
    }

    fn into_box(self) -> BoxMapper<T, R>
    where
        Self: Sized + 'static,
    {
        BoxMapper::new(self)
    }

    fn into_rc(self) -> RcMapper<T, R>
    where
        Self: Sized + 'static,
    {
        RcMapper::new(self)
    }

    fn into_arc(self) -> ArcMapper<T, R>
    where
        Self: Sized + Send + 'static,
        T: Send + Sync + 'static,
        R: Send + 'static,
    {
        ArcMapper::new(self)
    }
}

// ============================================================================
// FnMapperOps - Extension trait for closure mappers
// ============================================================================

/// Extension trait for closures implementing `FnMut(T) -> R`
///
/// Provides composition methods (`and_then`, `compose`, `when`) for
/// closures without requiring explicit wrapping in `BoxMapper`,
/// `RcMapper`, or `ArcMapper`.
///
/// This trait is automatically implemented for all closures that
/// implement `FnMut(T) -> R`.
///
/// # Design Rationale
///
/// While closures automatically implement `Mapper<T, R>` through blanket
/// implementation, they don't have access to instance methods like
/// `and_then`, `compose`, and `when`. This extension trait provides
/// those methods, returning `BoxMapper` for maximum flexibility.
///
/// # Examples
///
/// ## Chain composition with and_then
///
/// ```rust
/// use prism3_function::{Mapper, FnMapperOps};
///
/// let mut counter1 = 0;
/// let mapper1 = move |x: i32| {
///     counter1 += 1;
///     x + counter1
/// };
///
/// let mut counter2 = 0;
/// let mapper2 = move |x: i32| {
///     counter2 += 1;
///     x * counter2
/// };
///
/// let mut composed = mapper1.and_then(mapper2);
/// assert_eq!(composed.map(10), 11);  // (10 + 1) * 1
/// ```
///
/// ## Reverse composition with compose
///
/// ```rust
/// use prism3_function::{Mapper, FnMapperOps};
///
/// let mut counter = 0;
/// let mapper = move |x: i32| {
///     counter += 1;
///     x * counter
/// };
///
/// let mut composed = mapper.compose(|x: i32| x + 1);
/// assert_eq!(composed.map(10), 11); // (10 + 1) * 1
/// ```
///
/// ## Conditional mapping with when
///
/// ```rust
/// use prism3_function::{Mapper, FnMapperOps};
///
/// let mut mapper = (|x: i32| x * 2)
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: i32| -x);
///
/// assert_eq!(mapper.map(5), 10);
/// assert_eq!(mapper.map(-5), 5);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnMapperOps<T, R>: FnMut(T) -> R + Sized + 'static {
    /// Chain composition - applies self first, then after
    ///
    /// Creates a new mapper that applies this mapper first, then applies
    /// the after mapper to the result. Consumes self and returns a
    /// `BoxMapper`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The output type of the after mapper
    /// * `F` - The type of the after mapper (must implement Mapper<R, S>)
    ///
    /// # Parameters
    ///
    /// * `after` - The mapper to apply after self. Can be:
    ///   - A closure: `|x: R| -> S`
    ///   - A `BoxMapper<R, S>`
    ///   - An `RcMapper<R, S>`
    ///   - An `ArcMapper<R, S>`
    ///   - Any type implementing `Mapper<R, S>`
    ///
    /// # Returns
    ///
    /// A new `BoxMapper<T, S>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, FnMapperOps, BoxMapper};
    ///
    /// let mut counter1 = 0;
    /// let mapper1 = move |x: i32| {
    ///     counter1 += 1;
    ///     x + counter1
    /// };
    ///
    /// let mut counter2 = 0;
    /// let mapper2 = BoxMapper::new(move |x: i32| {
    ///     counter2 += 1;
    ///     x * counter2
    /// });
    ///
    /// let mut composed = mapper1.and_then(mapper2);
    /// assert_eq!(composed.map(10), 11);
    /// ```
    fn and_then<S, F>(self, after: F) -> BoxMapper<T, S>
    where
        S: 'static,
        F: Mapper<R, S> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapper::new(self).and_then(after)
    }

    /// Reverse composition - applies before first, then self
    ///
    /// Creates a new mapper that applies the before mapper first, then
    /// applies this mapper to the result. Consumes self and returns a
    /// `BoxMapper`.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The input type of the before mapper
    /// * `F` - The type of the before mapper (must implement Mapper<S, T>)
    ///
    /// # Parameters
    ///
    /// * `before` - The mapper to apply before self. Can be:
    ///   - A closure: `|x: S| -> T`
    ///   - A `BoxMapper<S, T>`
    ///   - An `RcMapper<S, T>`
    ///   - An `ArcMapper<S, T>`
    ///   - Any type implementing `Mapper<S, T>`
    ///
    /// # Returns
    ///
    /// A new `BoxMapper<S, R>` representing the composition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, FnMapperOps, BoxMapper};
    ///
    /// let mut counter = 0;
    /// let mapper = move |x: i32| {
    ///     counter += 1;
    ///     x * counter
    /// };
    ///
    /// let before = BoxMapper::new(|x: i32| x + 1);
    ///
    /// let mut composed = mapper.compose(before);
    /// assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    /// ```
    fn compose<S, F>(self, before: F) -> BoxMapper<S, R>
    where
        S: 'static,
        F: Mapper<S, T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapper::new(self).compose(before)
    }

    /// Creates a conditional mapper
    ///
    /// Returns a mapper that only executes when a predicate is satisfied.
    /// You must call `or_else()` to provide an alternative mapper for
    /// when the condition is not satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalMapper<T, R>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mapper, FnMapperOps};
    ///
    /// let mut mapper = (|x: i32| x * 2)
    ///     .when(|x: &i32| *x > 0)
    ///     .or_else(|x: i32| -x);
    ///
    /// assert_eq!(mapper.map(5), 10);
    /// assert_eq!(mapper.map(-5), 5);
    /// ```
    fn when<P>(self, predicate: P) -> BoxConditionalMapper<T, R>
    where
        P: Predicate<T> + 'static,
        T: 'static,
        R: 'static,
    {
        BoxMapper::new(self).when(predicate)
    }
}

/// Blanket implementation of FnMapperOps for all closures
///
/// Automatically implements `FnMapperOps<T, R>` for any type that
/// implements `FnMut(T) -> R`.
///
/// # Author
///
/// Haixing Hu
impl<T, R, F> FnMapperOps<T, R> for F where F: FnMut(T) -> R + 'static {}
