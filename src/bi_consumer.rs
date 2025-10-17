/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiConsumer Types
//!
//! Provides Java-like `BiConsumer` interface implementations for performing
//! operations that accept two input parameters and return no result.
//!
//! This module provides a unified `BiConsumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxBiConsumer<T, U>`**: Box-based single ownership implementation for
//!   one-time use scenarios and builder patterns
//! - **`ArcBiConsumer<T, U>`**: Arc<Mutex<>>-based thread-safe shared ownership
//!   implementation for multi-threaded scenarios
//! - **`RcBiConsumer<T, U>`**: Rc<RefCell<>>-based single-threaded shared
//!   ownership implementation with no lock overhead
//!
//! # Design Philosophy
//!
//! Unlike `BiPredicate` which uses `Fn(&T, &U) -> bool`, `BiConsumer` requires
//! `FnMut(&mut T, &mut U)` for mutability. This introduces interior mutability
//! challenges when sharing across contexts:
//!
//! - **Single Ownership**: `BoxBiConsumer` uses `Box<dyn FnMut>` with no
//!   sharing overhead
//! - **Thread-Safe Sharing**: `ArcBiConsumer` uses `Arc<Mutex<dyn FnMut>>`
//!   for safe concurrent access
//! - **Single-Threaded Sharing**: `RcBiConsumer` uses `Rc<RefCell<dyn FnMut>>`
//!   for efficient single-threaded reuse
//!
//! # Comparison Table
//!
//! | Feature          | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
//! |------------------|---------------|---------------|--------------|
//! | Ownership        | Single        | Shared        | Shared       |
//! | Cloneable        | ❌            | ✅            | ✅           |
//! | Thread-Safe      | ❌            | ✅            | ❌           |
//! | Interior Mut.    | N/A           | Mutex         | RefCell      |
//! | `and_then` API   | `self`        | `&self`       | `&self`      |
//! | Lock Overhead    | None          | Yes           | None         |
//!
//! # Use Cases
//!
//! ## BoxBiConsumer
//!
//! - One-time operations that don't require sharing
//! - Builder patterns where ownership naturally flows
//! - Simple scenarios with no reuse requirements
//!
//! ## ArcBiConsumer
//!
//! - Multi-threaded shared operations
//! - Concurrent task processing (e.g., thread pools)
//! - Situations requiring the same consumer across threads
//!
//! ## RcBiConsumer
//!
//! - Single-threaded operations with multiple uses
//! - Event handling in single-threaded UI frameworks
//! - Performance-critical single-threaded scenarios
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxBiConsumer, ArcBiConsumer, RcBiConsumer, BiConsumer};
//!
//! // BoxBiConsumer: Single ownership, consumes self
//! let mut consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| {
//!     *x += *y;
//!     *y = 0;
//! });
//! let mut a = 5;
//! let mut b = 3;
//! consumer.accept(&mut a, &mut b);
//! assert_eq!(a, 8);
//! assert_eq!(b, 0);
//!
//! // ArcBiConsumer: Shared ownership, cloneable, thread-safe
//! let shared = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
//! let clone = shared.clone();
//! let mut a = 5;
//! let mut b = 3;
//! let mut c = shared;
//! c.accept(&mut a, &mut b);
//! assert_eq!(a, 8);
//!
//! // RcBiConsumer: Shared ownership, cloneable, single-threaded
//! let rc = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
//! let clone = rc.clone();
//! let mut a = 5;
//! let mut b = 3;
//! let mut c = rc;
//! c.accept(&mut a, &mut b);
//! assert_eq!(a, 8);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{BiConsumer, BoxBiConsumer, ArcBiConsumer};
//!
//! // BoxBiConsumer: Consumes self
//! let mut chained = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y)
//!     .and_then(|x: &mut i32, y: &mut i32| *y *= 2);
//! let mut a = 5;
//! let mut b = 3;
//! chained.accept(&mut a, &mut b);
//! assert_eq!(a, 8);  // 5 + 3
//! assert_eq!(b, 6);  // 3 * 2
//!
//! // ArcBiConsumer: Borrows &self, original still usable
//! let first = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
//! let second = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *y *= 2);
//! let combined = first.and_then(&second);
//! // first and second are still usable here
//! ```
//!
//! ## Working with Closures
//!
//! All closures automatically implement the `BiConsumer` trait:
//!
//! ```rust
//! use prism3_function::{BiConsumer, FnBiConsumerOps};
//!
//! // Closures can use .accept() directly
//! let mut closure = |x: &mut i32, y: &mut i32| *x += *y;
//! let mut a = 5;
//! let mut b = 3;
//! closure.accept(&mut a, &mut b);
//! assert_eq!(a, 8);
//!
//! // Closures can be chained, returning BoxBiConsumer
//! let mut chained = (|x: &mut i32, y: &mut i32| *x += *y)
//!     .and_then(|x: &mut i32, y: &mut i32| *y *= 2);
//! let mut a = 5;
//! let mut b = 3;
//! chained.accept(&mut a, &mut b);
//! assert_eq!(a, 8);
//! assert_eq!(b, 6);
//! ```
//!
//! ## Type Conversions
//!
//! ```rust
//! use prism3_function::BiConsumer;
//!
//! // Convert closure to concrete type
//! let closure = |x: &mut i32, y: &mut i32| *x += *y;
//! let mut box_consumer = closure.into_box();
//!
//! let closure = |x: &mut i32, y: &mut i32| *x += *y;
//! let mut rc_consumer = closure.into_rc();
//!
//! let closure = |x: &mut i32, y: &mut i32| *x += *y;
//! let mut arc_consumer = closure.into_arc();
//! ```
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ============================================================================
// 1. Type Aliases
// ============================================================================

/// Type alias for Arc-wrapped mutable bi-consumer function
type ArcMutBiConsumerFn<T, U> = Arc<Mutex<dyn FnMut(&mut T, &mut U) + Send>>;

/// Type alias for Rc-wrapped mutable bi-consumer function
type RcMutBiConsumerFn<T, U> = Rc<RefCell<dyn FnMut(&mut T, &mut U)>>;

// ============================================================================
// 2. BiConsumer Trait - Unified BiConsumer Interface
// ============================================================================

/// BiConsumer trait - Unified bi-consumer interface
///
/// Defines the core behavior of all bi-consumer types. Similar to Java's
/// `BiConsumer<T, U>` interface, it performs operations that accept two values
/// but return no result (side effects only).
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnMut(&mut T, &mut U)`
/// - `BoxBiConsumer<T, U>`, `ArcBiConsumer<T, U>`, and `RcBiConsumer<T, U>`
///
/// # Design Rationale
///
/// The trait provides a unified abstraction over different ownership models,
/// allowing generic code to work with any bi-consumer type. Type conversion
/// methods (`into_box`, `into_arc`, `into_rc`) enable flexible ownership
/// transitions based on usage requirements.
///
/// # Features
///
/// - **Unified Interface**: All bi-consumer types share the same `accept`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions that work with any bi-consumer
///   type
///
/// # Examples
///
/// ## Generic BiConsumer Function
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer, ArcBiConsumer};
///
/// fn apply_bi_consumer<C: BiConsumer<i32, i32>>(
///     consumer: &mut C,
///     a: i32,
///     b: i32
/// ) -> (i32, i32) {
///     let mut x = a;
///     let mut y = b;
///     consumer.accept(&mut x, &mut y);
///     (x, y)
/// }
///
/// // Works with any bi-consumer type
/// let mut box_con = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
/// assert_eq!(apply_bi_consumer(&mut box_con, 5, 3), (8, 3));
///
/// let mut arc_con = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
/// assert_eq!(apply_bi_consumer(&mut arc_con, 5, 3), (8, 3));
///
/// let mut closure = |x: &mut i32, y: &mut i32| *x += *y;
/// assert_eq!(apply_bi_consumer(&mut closure, 5, 3), (8, 3));
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::BiConsumer;
///
/// let closure = |x: &mut i32, y: &mut i32| *x += *y;
///
/// // Convert to different ownership models
/// let box_consumer = closure.into_box();
/// // let rc_consumer = closure.into_rc();  // closure moved
/// // let arc_consumer = closure.into_arc(); // closure moved
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait BiConsumer<T, U> {
    /// Performs the consumption operation
    ///
    /// Executes an operation on the given two mutable references. The operation
    /// typically modifies the input values or produces side effects.
    ///
    /// # Parameters
    ///
    /// * `first` - A mutable reference to the first value to be consumed
    /// * `second` - A mutable reference to the second value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let mut a = 5;
    /// let mut b = 3;
    /// consumer.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    fn accept(&mut self, first: &mut T, second: &mut U);

    /// Converts to BoxBiConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current bi-consumer to `BoxBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcBiConsumer`], [`RcBiConsumer`]),
    /// you can call `.clone()` first if you need to keep the original.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumer;
    ///
    /// let closure = |x: &mut i32, y: &mut i32| *x += *y;
    /// let mut box_consumer = closure.into_box();
    /// let mut a = 5;
    /// let mut b = 3;
    /// box_consumer.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;

    /// Converts to RcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current bi-consumer to `RcBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumer;
    ///
    /// let closure = |x: &mut i32, y: &mut i32| *x += *y;
    /// let mut rc_consumer = closure.into_rc();
    /// let mut a = 5;
    /// let mut b = 3;
    /// rc_consumer.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;

    /// Converts to ArcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current bi-consumer to `ArcBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumer;
    ///
    /// let closure = |x: &mut i32, y: &mut i32| *x += *y;
    /// let mut arc_consumer = closure.into_arc();
    /// let mut a = 5;
    /// let mut b = 3;
    /// arc_consumer.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static;

    /// Converts bi-consumer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the bi-consumer and returns a closure that can be
    /// directly used with methods like `zip().for_each()`. This provides
    /// a more ergonomic API when working with paired iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available. The returned closure captures the consumer by move.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut((&mut T, &mut U))`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let mut values1 = vec![1, 2, 3];
    /// let mut values2 = vec![10, 20, 30];
    ///
    /// values1.iter_mut()
    ///     .zip(values2.iter_mut())
    ///     .for_each(consumer.into_fn());
    ///
    /// assert_eq!(values1, vec![11, 22, 33]);
    /// ```
    fn into_fn(self) -> impl FnMut((&mut T, &mut U))
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;
}

// ============================================================================
// 3. BoxBiConsumer - Single Ownership Implementation
// ============================================================================

/// BoxBiConsumer struct
///
/// A bi-consumer implementation based on `Box<dyn FnMut(&mut T, &mut U)>` for
/// single ownership scenarios. This is the simplest and most efficient
/// bi-consumer type when sharing is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Builder Pattern**: Method chaining consumes `self` naturally
/// - **Factory Methods**: Convenient constructors for common patterns
///
/// # Comparison with Other BiConsumers
///
/// | Feature             | BoxBiConsumer | ArcBiConsumer | RcBiConsumer |
/// |---------------------|---------------|---------------|--------------|
/// | Ownership Model     | Single        | Shared        | Shared       |
/// | Cloneable           | No            | Yes           | Yes          |
/// | Thread-Safe         | No            | Yes           | No           |
/// | Reference Counting  | No            | Yes (Arc)     | Yes (Rc)     |
/// | Interior Mutability | No            | Mutex         | RefCell      |
/// | Lock Overhead       | None          | Yes           | None         |
/// | `and_then` API      | `self`        | `&self`       | `&self`      |
///
/// # Use Cases
///
/// Choose `BoxBiConsumer` when:
/// - The bi-consumer is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the consumer across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// let mut consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
/// let mut a = 5;
/// let mut b = 3;
/// consumer.accept(&mut a, &mut b);
/// assert_eq!(a, 8);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
///
/// let mut chained = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y)
///     .and_then(|x: &mut i32, y: &mut i32| *y *= 2)
///     .and_then(|x: &mut i32, y: &mut i32| *x -= 1);
/// let mut a = 5;
/// let mut b = 3;
/// chained.accept(&mut a, &mut b);
/// assert_eq!(a, 7);  // (5 + 3) - 1
/// assert_eq!(b, 6);  // 3 * 2
/// ```
///
/// 双参数消费者函数类型别名
type BiConsumerFn<T, U> = Box<dyn FnMut(&mut T, &mut U)>;

/// # Author
///
/// Haixing Hu
pub struct BoxBiConsumer<T, U> {
    func: BiConsumerFn<T, U>,
}

impl<T, U> BoxBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new BoxBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let mut a = 5;
    /// let mut b = 3;
    /// consumer.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T, &mut U) + 'static,
    {
        BoxBiConsumer { func: Box::new(f) }
    }

    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer that first executes the current operation, then
    /// executes the next operation. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut chained = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y)
    ///     .and_then(|x: &mut i32, y: &mut i32| *y *= 2);
    ///
    /// let mut a = 5;
    /// let mut b = 3;
    /// chained.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// assert_eq!(b, 6);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumer<T, U> + 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }

    /// Creates a no-op bi-consumer
    ///
    /// Returns a bi-consumer that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut noop = BoxBiConsumer::<i32, i32>::noop();
    /// let mut a = 42;
    /// let mut b = 10;
    /// noop.accept(&mut a, &mut b);
    /// assert_eq!(a, 42);
    /// assert_eq!(b, 10);
    /// ```
    pub fn noop() -> Self {
        BoxBiConsumer::new(|_, _| {})
    }

    /// Creates a print bi-consumer
    ///
    /// Returns a bi-consumer that prints the input values.
    ///
    /// # Returns
    ///
    /// Returns a print bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut print = BoxBiConsumer::<i32, i32>::print();
    /// let mut a = 42;
    /// let mut b = 10;
    /// print.accept(&mut a, &mut b); // Prints: (42, 10)
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        BoxBiConsumer::new(|t, u| {
            println!("({:?}, {:?})", t, u);
        })
    }

    /// Creates a print bi-consumer with prefix
    ///
    /// Returns a bi-consumer that prints the input values with a prefix.
    ///
    /// # Parameters
    ///
    /// * `prefix` - The prefix string
    ///
    /// # Returns
    ///
    /// Returns a print bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut print = BoxBiConsumer::<i32, i32>::print_with("Values: ");
    /// let mut a = 42;
    /// let mut b = 10;
    /// print.accept(&mut a, &mut b); // Prints: Values: (42, 10)
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxBiConsumer::new(move |t, u| {
            println!("{}{:?}, {:?}", prefix, t, u);
        })
    }

    /// Creates a conditional bi-consumer
    ///
    /// Returns a bi-consumer that only executes the operation when the
    /// predicate is true.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    /// * `C` - The consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `consumer` - The consumer to execute
    ///
    /// # Returns
    ///
    /// Returns a conditional bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut conditional = BoxBiConsumer::if_then(
    ///     |x: &i32, y: &i32| *x > 0 && *y > 0,
    ///     |x: &mut i32, y: &mut i32| *x += *y
    /// );
    ///
    /// let mut a = 5;
    /// let mut b = 3;
    /// conditional.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    ///
    /// let mut c = -5;
    /// let mut d = 3;
    /// conditional.accept(&mut c, &mut d);
    /// assert_eq!(c, -5); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T, &U) -> bool + 'static,
        C: FnMut(&mut T, &mut U) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred(t, u) {
                cons(t, u);
            }
        })
    }

    /// Creates a conditional branch bi-consumer
    ///
    /// Returns a bi-consumer that executes different operations based on the
    /// predicate.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    /// * `C1` - The then consumer type
    /// * `C2` - The else consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `then_consumer` - The consumer to execute when predicate is true
    /// * `else_consumer` - The consumer to execute when predicate is false
    ///
    /// # Returns
    ///
    /// Returns a conditional branch bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut conditional = BoxBiConsumer::if_then_else(
    ///     |x: &i32, y: &i32| *x > *y,
    ///     |x: &mut i32, y: &mut i32| *x += 1,
    ///     |x: &mut i32, y: &mut i32| *y += 1
    /// );
    ///
    /// let mut a = 5;
    /// let mut b = 3;
    /// conditional.accept(&mut a, &mut b);
    /// assert_eq!(a, 6);
    /// assert_eq!(b, 3);
    ///
    /// let mut c = 2;
    /// let mut d = 5;
    /// conditional.accept(&mut c, &mut d);
    /// assert_eq!(c, 2);
    /// assert_eq!(d, 6);
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnMut(&T, &U) -> bool + 'static,
        C1: FnMut(&mut T, &mut U) + 'static,
        C2: FnMut(&mut T, &mut U) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred(t, u) {
                then_cons(t, u);
            } else {
                else_cons(t, u);
            }
        })
    }
}

impl<T, U> BiConsumer<T, U> for BoxBiConsumer<T, U> {
    fn accept(&mut self, first: &mut T, second: &mut U) {
        (self.func)(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let mut func = self.func;
        RcBiConsumer::new(move |t, u| func(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        panic!("Cannot convert BoxBiConsumer to ArcBiConsumer: BoxBiConsumer's inner function may not be Send")
    }

    fn into_fn(mut self) -> impl FnMut((&mut T, &mut U))
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        move |(t, u): (&mut T, &mut U)| (self.func)(t, u)
    }
}

// ============================================================================
// 4. ArcBiConsumer - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcBiConsumer struct
///
/// A bi-consumer implementation based on `Arc<Mutex<dyn FnMut(&mut T, &mut U) + Send>>`
/// for thread-safe shared ownership scenarios. This consumer can be safely
/// cloned and shared across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
/// - **Cross-Thread Sharing**: Can be sent to and used by other threads
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
///
/// let consumer = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
/// let clone = consumer.clone();
///
/// let mut a = 5;
/// let mut b = 3;
/// let mut c = consumer;
/// c.accept(&mut a, &mut b);
/// assert_eq!(a, 8);
/// ```
///
/// ## Multi-Threaded Usage
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
/// use std::thread;
///
/// let shared = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
///
/// // Clone for another thread
/// let shared_clone = shared.clone();
/// let handle = thread::spawn(move || {
///     let mut a = 5;
///     let mut b = 3;
///     let mut consumer = shared_clone;
///     consumer.accept(&mut a, &mut b);
///     (a, b)
/// });
///
/// // Original consumer still usable
/// let mut c = 10;
/// let mut d = 2;
/// let mut consumer = shared;
/// consumer.accept(&mut c, &mut d);
/// assert_eq!(c, 12);
/// assert_eq!(handle.join().unwrap(), (8, 3));
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcBiConsumer<T, U> {
    func: ArcMutBiConsumerFn<T, U>,
}

impl<T, U> ArcBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    /// Creates a new ArcBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    ///
    /// let consumer = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let mut a = 5;
    /// let mut b = 3;
    /// let mut c = consumer;
    /// c.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T, &mut U) + Send + 'static,
    {
        ArcBiConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// Chains another ArcBiConsumer in sequence
    ///
    /// Returns a new consumer that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    ///
    /// let first = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let second = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| *y *= 2);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut a = 5;
    /// let mut b = 3;
    /// let mut c = chained;
    /// c.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);  // 5 + 3
    /// assert_eq!(b, 6);  // 3 * 2
    /// ```
    pub fn and_then(&self, next: &ArcBiConsumer<T, U>) -> ArcBiConsumer<T, U> {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcBiConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T, u: &mut U| {
                first.lock().unwrap()(t, u);
                second.lock().unwrap()(t, u);
            })),
        }
    }
}

impl<T, U> BiConsumer<T, U> for ArcBiConsumer<T, U> {
    fn accept(&mut self, first: &mut T, second: &mut U) {
        (self.func.lock().unwrap())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.func;
        BoxBiConsumer::new(move |t, u| func.lock().unwrap()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.func;
        RcBiConsumer::new(move |t, u| func.lock().unwrap()(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut((&mut T, &mut U))
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let func = self.func;
        move |(t, u): (&mut T, &mut U)| func.lock().unwrap()(t, u)
    }
}

impl<T, U> Clone for ArcBiConsumer<T, U> {
    /// Clones the ArcBiConsumer
    ///
    /// Creates a new ArcBiConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 5. RcBiConsumer - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcBiConsumer struct
///
/// A bi-consumer implementation based on `Rc<RefCell<dyn FnMut(&mut T, &mut U)>>`
/// for single-threaded shared ownership scenarios. This consumer provides the
/// benefits of shared ownership without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **No Lock Overhead**: More efficient than `ArcBiConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
///
/// let consumer = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
/// let clone = consumer.clone();
///
/// let mut a = 5;
/// let mut b = 3;
/// let mut c = consumer;
/// c.accept(&mut a, &mut b);
/// assert_eq!(a, 8);
/// ```
///
/// ## Sharing Within a Single Thread
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
///
/// let shared = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
///
/// // Clone for use in multiple places
/// let clone1 = shared.clone();
/// let clone2 = shared.clone();
///
/// let mut a = 5;
/// let mut b = 3;
/// let mut c1 = clone1;
/// c1.accept(&mut a, &mut b);
/// assert_eq!(a, 8);
///
/// let mut c = 2;
/// let mut d = 1;
/// let mut c2 = clone2;
/// c2.accept(&mut c, &mut d);
/// assert_eq!(c, 3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcBiConsumer<T, U> {
    func: RcMutBiConsumerFn<T, U>,
}

impl<T, U> RcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new RcBiConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let consumer = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let mut a = 5;
    /// let mut b = 3;
    /// let mut c = consumer;
    /// c.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T, &mut U) + 'static,
    {
        RcBiConsumer {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// Chains another RcBiConsumer in sequence
    ///
    /// Returns a new consumer that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    ///
    /// let first = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    /// let second = RcBiConsumer::new(|x: &mut i32, y: &mut i32| *y *= 2);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut a = 5;
    /// let mut b = 3;
    /// let mut c = chained;
    /// c.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);  // 5 + 3
    /// assert_eq!(b, 6);  // 3 * 2
    /// ```
    pub fn and_then(&self, next: &RcBiConsumer<T, U>) -> RcBiConsumer<T, U> {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcBiConsumer {
            func: Rc::new(RefCell::new(move |t: &mut T, u: &mut U| {
                first.borrow_mut()(t, u);
                second.borrow_mut()(t, u);
            })),
        }
    }
}

impl<T, U> BiConsumer<T, U> for RcBiConsumer<T, U> {
    fn accept(&mut self, first: &mut T, second: &mut U) {
        (self.func.borrow_mut())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.func;
        BoxBiConsumer::new(move |t, u| func.borrow_mut()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        panic!("Cannot convert RcBiConsumer to ArcBiConsumer (not Send)")
    }

    fn into_fn(self) -> impl FnMut((&mut T, &mut U))
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let func = self.func;
        move |(t, u): (&mut T, &mut U)| func.borrow_mut()(t, u)
    }
}

impl<T, U> Clone for RcBiConsumer<T, U> {
    /// Clones the RcBiConsumer
    ///
    /// Creates a new RcBiConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. Implement BiConsumer trait for closures
// ============================================================================

/// Implements BiConsumer for all FnMut(&mut T, &mut U)
impl<T, U, F> BiConsumer<T, U> for F
where
    F: FnMut(&mut T, &mut U),
{
    fn accept(&mut self, first: &mut T, second: &mut U) {
        self(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(self)
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcBiConsumer::new(self)
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        ArcBiConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut((&mut T, &mut U))
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self.into_box().into_fn()
    }
}

// ============================================================================
// 7. Provide extension methods for closures
// ============================================================================

/// Extension trait providing bi-consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnMut(&mut T, &mut U)`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Design Rationale
///
/// This trait allows closures to be composed naturally using method syntax,
/// similar to iterator combinators. Composition methods consume the closure
/// and return `BoxBiConsumer<T, U>`, which can be further chained.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxBiConsumer**: Composition results are `BoxBiConsumer<T, U>`
///   for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&mut T, &mut U)` closures get
///   these methods automatically
///
/// # Examples
///
/// ## Basic Chaining
///
/// ```rust
/// use prism3_function::{BiConsumer, FnBiConsumerOps};
///
/// let chained = (|x: &mut i32, y: &mut i32| *x += *y)
///     .and_then(|x: &mut i32, y: &mut i32| *y *= 2);
/// let mut a = 5;
/// let mut b = 3;
/// let mut result = chained;
/// result.accept(&mut a, &mut b);
/// assert_eq!(a, 8);  // 5 + 3
/// assert_eq!(b, 6);  // 3 * 2
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnBiConsumerOps<T, U>: FnMut(&mut T, &mut U) + Sized {
    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxBiConsumer<T, U>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, FnBiConsumerOps};
    ///
    /// let chained = (|x: &mut i32, y: &mut i32| *x += *y)
    ///     .and_then(|x: &mut i32, y: &mut i32| *y *= 2);
    ///
    /// let mut a = 5;
    /// let mut b = 3;
    /// let mut result = chained;
    /// result.accept(&mut a, &mut b);
    /// assert_eq!(a, 8);
    /// assert_eq!(b, 6);
    /// ```
    fn and_then<C>(self, next: C) -> BoxBiConsumer<T, U>
    where
        Self: 'static,
        C: BiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnBiConsumerOps for all closure types
impl<T, U, F> FnBiConsumerOps<T, U> for F where F: FnMut(&mut T, &mut U) {}
