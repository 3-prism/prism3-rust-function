/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Types
//!
//! Provides Java-like `Consumer` interface implementations for performing
//! operations that accept a single input parameter and return no result.
//!
//! This module provides a unified `Consumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxConsumer<T>`**: Box-based single ownership implementation for
//!   one-time use scenarios and builder patterns
//! - **`ArcConsumer<T>`**: Arc<Mutex<>>-based thread-safe shared ownership
//!   implementation for multi-threaded scenarios
//! - **`RcConsumer<T>`**: Rc<RefCell<>>-based single-threaded shared
//!   ownership implementation with no lock overhead
//!
//! # Design Philosophy
//!
//! Unlike `Predicate` which uses `Fn(&T) -> bool`, `Consumer` requires
//! `FnMut(&mut T)` for mutability. This introduces interior mutability
//! challenges when sharing across contexts:
//!
//! - **Single Ownership**: `BoxConsumer` uses `Box<dyn FnMut>` with no
//!   sharing overhead
//! - **Thread-Safe Sharing**: `ArcConsumer` uses `Arc<Mutex<dyn FnMut>>`
//!   for safe concurrent access
//! - **Single-Threaded Sharing**: `RcConsumer` uses `Rc<RefCell<dyn FnMut>>`
//!   for efficient single-threaded reuse
//!
//! # Comparison Table
//!
//! | Feature          | BoxConsumer | ArcConsumer | RcConsumer |
//! |------------------|-------------|-------------|------------|
//! | Ownership        | Single      | Shared      | Shared     |
//! | Cloneable        | ❌          | ✅          | ✅         |
//! | Thread-Safe      | ❌          | ✅          | ❌         |
//! | Interior Mut.    | N/A         | Mutex       | RefCell    |
//! | `and_then` API   | `self`      | `&self`     | `&self`    |
//! | Lock Overhead    | None        | Yes         | None       |
//!
//! # Use Cases
//!
//! ## BoxConsumer
//!
//! - One-time operations that don't require sharing
//! - Builder patterns where ownership naturally flows
//! - Simple scenarios with no reuse requirements
//!
//! ## ArcConsumer
//!
//! - Multi-threaded shared operations
//! - Concurrent task processing (e.g., thread pools)
//! - Situations requiring the same consumer across threads
//!
//! ## RcConsumer
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
//! use prism3_function::{BoxConsumer, ArcConsumer, RcConsumer, Consumer};
//!
//! // BoxConsumer: Single ownership, consumes self
//! let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
//! let mut value = 5;
//! consumer.accept(&mut value);
//! assert_eq!(value, 10);
//!
//! // ArcConsumer: Shared ownership, cloneable, thread-safe
//! let shared = ArcConsumer::new(|x: &mut i32| *x *= 2);
//! let clone = shared.clone();
//! let mut value = 5;
//! let mut c = shared;
//! c.accept(&mut value);
//! assert_eq!(value, 10);
//!
//! // RcConsumer: Shared ownership, cloneable, single-threaded
//! let rc = RcConsumer::new(|x: &mut i32| *x *= 2);
//! let clone = rc.clone();
//! let mut value = 5;
//! let mut c = rc;
//! c.accept(&mut value);
//! assert_eq!(value, 10);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{Consumer, BoxConsumer, ArcConsumer};
//!
//! // BoxConsumer: Consumes self
//! let mut chained = BoxConsumer::new(|x: &mut i32| *x *= 2)
//!     .and_then(|x: &mut i32| *x += 10);
//! let mut value = 5;
//! chained.accept(&mut value);
//! assert_eq!(value, 20); // (5 * 2) + 10
//!
//! // ArcConsumer: Borrows &self, original still usable
//! let first = ArcConsumer::new(|x: &mut i32| *x *= 2);
//! let second = ArcConsumer::new(|x: &mut i32| *x += 10);
//! let combined = first.and_then(&second);
//! // first and second are still usable here
//! ```
//!
//! ## Working with Closures
//!
//! All closures automatically implement the `Consumer` trait:
//!
//! ```rust
//! use prism3_function::{Consumer, FnConsumerOps};
//!
//! // Closures can use .accept() directly
//! let mut closure = |x: &mut i32| *x *= 2;
//! let mut value = 5;
//! closure.accept(&mut value);
//! assert_eq!(value, 10);
//!
//! // Closures can be chained, returning BoxConsumer
//! let mut chained = (|x: &mut i32| *x *= 2)
//!     .and_then(|x: &mut i32| *x += 10);
//! let mut value = 5;
//! chained.accept(&mut value);
//! assert_eq!(value, 20);
//! ```
//!
//! ## Type Conversions
//!
//! ```rust
//! use prism3_function::Consumer;
//!
//! // Convert closure to concrete type
//! let closure = |x: &mut i32| *x *= 2;
//! let mut box_consumer = closure.into_box();
//!
//! let closure = |x: &mut i32| *x *= 2;
//! let mut rc_consumer = closure.into_rc();
//!
//! let closure = |x: &mut i32| *x *= 2;
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
// Consumer Trait - Unified Consumer Interface
// ============================================================================

/// Consumer trait - Unified consumer interface
///
/// Defines the core behavior of all consumer types. Similar to Java's
/// `Consumer<T>` interface, it performs operations that accept a value but
/// return no result (side effects only).
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnMut(&mut T)`
/// - `BoxConsumer<T>`, `ArcConsumer<T>`, and `RcConsumer<T>`
///
/// # Design Rationale
///
/// The trait provides a unified abstraction over different ownership models,
/// allowing generic code to work with any consumer type. Type conversion
/// methods (`into_box`, `into_arc`, `into_rc`) enable flexible ownership
/// transitions based on usage requirements.
///
/// # Features
///
/// - **Unified Interface**: All consumer types share the same `accept`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions that work with any consumer
///   type
///
/// # Examples
///
/// ## Generic Consumer Function
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer, ArcConsumer};
///
/// fn apply_consumer<C: Consumer<i32>>(
///     consumer: &mut C,
///     value: i32
/// ) -> i32 {
///     let mut val = value;
///     consumer.accept(&mut val);
///     val
/// }
///
/// // Works with any consumer type
/// let mut box_con = BoxConsumer::new(|x: &mut i32| *x *= 2);
/// assert_eq!(apply_consumer(&mut box_con, 5), 10);
///
/// let mut arc_con = ArcConsumer::new(|x: &mut i32| *x *= 2);
/// assert_eq!(apply_consumer(&mut arc_con, 5), 10);
///
/// let mut closure = |x: &mut i32| *x *= 2;
/// assert_eq!(apply_consumer(&mut closure, 5), 10);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::Consumer;
///
/// let closure = |x: &mut i32| *x *= 2;
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
pub trait Consumer<T> {
    /// Performs the consumption operation
    ///
    /// Executes an operation on the given mutable reference. The operation
    /// typically modifies the input value or produces side effects.
    ///
    /// # Parameters
    ///
    /// * `value` - A mutable reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut value = 5;
    /// consumer.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    fn accept(&mut self, value: &mut T);

    /// Converts to BoxConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current consumer to `BoxConsumer<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcConsumer`], [`RcConsumer`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc_consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut box_consumer = arc_consumer.clone().into_box();  // Clone first
    ///
    /// // Original still available
    /// let mut value1 = 5;
    /// let mut c = arc_consumer;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// // Converted BoxConsumer also works
    /// let mut value2 = 3;
    /// box_consumer.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::Consumer;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut box_consumer = closure.into_box();
    /// let mut value = 5;
    /// box_consumer.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    ///
    /// ## Clone Before Conversion (ArcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc = ArcConsumer::new(|x: &mut i32| *x *= 2);
    ///
    /// // Clone before conversion to keep the original
    /// let mut boxed = arc.clone().into_box();
    ///
    /// // Both are still usable
    /// let mut value1 = 5;
    /// let mut c = arc;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// let mut value2 = 3;
    /// boxed.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    ///
    /// ## Clone Before Conversion (RcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let rc = RcConsumer::new(|x: &mut i32| *x *= 2);
    ///
    /// // Clone before conversion to keep the original
    /// let mut boxed = rc.clone().into_box();
    ///
    /// // Both are still usable
    /// let mut value1 = 5;
    /// let mut c = rc;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// let mut value2 = 3;
    /// boxed.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to RcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current consumer to `RcConsumer<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcConsumer`], [`RcConsumer`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc_consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut rc_consumer = arc_consumer.clone().into_rc();  // Clone first
    ///
    /// // Original still available
    /// let mut value1 = 5;
    /// let mut c = arc_consumer;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// // Converted RcConsumer also works
    /// let mut value2 = 3;
    /// rc_consumer.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::Consumer;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut rc_consumer = closure.into_rc();
    /// let mut value = 5;
    /// rc_consumer.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    ///
    /// ## Clone Before Conversion (ArcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc = ArcConsumer::new(|x: &mut i32| *x *= 2);
    ///
    /// // Clone before conversion to keep the original
    /// let mut rc = arc.clone().into_rc();
    ///
    /// // Both are still usable
    /// let mut value1 = 5;
    /// let mut c = arc;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// let mut value2 = 3;
    /// rc.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    ///
    /// ## Clone Before Conversion (RcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let rc1 = RcConsumer::new(|x: &mut i32| *x *= 2);
    ///
    /// // Clone before conversion to keep the original
    /// let mut rc2 = rc1.clone().into_rc();
    ///
    /// // Both are still usable
    /// let mut value1 = 5;
    /// let mut c1 = rc1;
    /// c1.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// let mut value2 = 3;
    /// rc2.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to ArcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current consumer to `ArcConsumer<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcConsumer`], [`RcConsumer`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc_consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut another = arc_consumer.clone().into_arc();  // Clone first
    ///
    /// // Original still available
    /// let mut value1 = 5;
    /// let mut c = arc_consumer;
    /// c.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// // Converted ArcConsumer also works
    /// let mut value2 = 3;
    /// another.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::Consumer;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut arc_consumer = closure.into_arc();
    /// let mut value = 5;
    /// arc_consumer.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    ///
    /// ## Clone Before Conversion (ArcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let arc1 = ArcConsumer::new(|x: &mut i32| *x *= 2);
    ///
    /// // Clone before conversion to keep the original
    /// let mut arc2 = arc1.clone().into_arc();
    ///
    /// // Both are still usable
    /// let mut value1 = 5;
    /// let mut c1 = arc1;
    /// c1.accept(&mut value1);
    /// assert_eq!(value1, 10);
    ///
    /// let mut value2 = 3;
    /// arc2.accept(&mut value2);
    /// assert_eq!(value2, 6);
    /// ```
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;

    /// Converts consumer to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original consumer becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the consumer and returns a closure that can be
    /// directly used with iterator methods like `for_each()`. This provides
    /// a more ergonomic API when working with iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer
    /// available. The returned closure captures the consumer by move.
    ///
    /// **Tip**: For cloneable consumers ([`ArcConsumer`], [`RcConsumer`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut values = vec![1, 2, 3];
    ///
    /// // Clone before into_fn to keep the original
    /// values.iter_mut().for_each(consumer.clone().into_fn());
    ///
    /// // Original consumer is still available
    /// let mut value = 5;
    /// let mut c = consumer;
    /// c.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(&mut T)`
    ///
    /// # Examples
    ///
    /// ## Basic Usage with Iterator
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut values = vec![1, 2, 3, 4, 5];
    ///
    /// values.iter_mut().for_each(consumer.into_fn());
    ///
    /// assert_eq!(values, vec![2, 4, 6, 8, 10]);
    /// ```
    ///
    /// ## With Complex Consumer
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let processor = BoxConsumer::new(|x: &mut i32| *x *= 2)
    ///     .and_then(|x: &mut i32| *x += 10);
    ///
    /// let mut values = vec![1, 2, 3];
    /// values.iter_mut().for_each(processor.into_fn());
    ///
    /// assert_eq!(values, vec![12, 14, 16]); // (1*2)+10, (2*2)+10, (3*2)+10
    /// ```
    ///
    /// ## With ArcConsumer
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut values = vec![1, 2, 3];
    ///
    /// values.iter_mut().for_each(consumer.into_fn());
    ///
    /// assert_eq!(values, vec![2, 4, 6]);
    /// ```
    ///
    /// ## Clone Before Conversion (ArcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut values1 = vec![1, 2, 3];
    /// let mut values2 = vec![4, 5, 6];
    ///
    /// // Clone to use in multiple places
    /// values1.iter_mut().for_each(consumer.clone().into_fn());
    /// values2.iter_mut().for_each(consumer.clone().into_fn());
    ///
    /// assert_eq!(values1, vec![2, 4, 6]);
    /// assert_eq!(values2, vec![8, 10, 12]);
    ///
    /// // Original still usable
    /// let mut value = 7;
    /// let mut c = consumer;
    /// c.accept(&mut value);
    /// assert_eq!(value, 14);
    /// ```
    ///
    /// ## Clone Before Conversion (RcConsumer)
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let consumer = RcConsumer::new(|x: &mut i32| *x += 10);
    /// let mut values = vec![1, 2, 3];
    ///
    /// // Clone before into_fn to keep the original
    /// values.iter_mut().for_each(consumer.clone().into_fn());
    ///
    /// assert_eq!(values, vec![11, 12, 13]);
    ///
    /// // Original still available
    /// let mut value = 5;
    /// let mut c = consumer;
    /// c.accept(&mut value);
    /// assert_eq!(value, 15);
    /// ```
    ///
    /// ## Ownership Behavior
    ///
    /// ```rust,compile_fail
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
    /// let mut values = vec![1, 2, 3];
    ///
    /// values.iter_mut().for_each(consumer.into_fn());
    ///
    /// // ❌ Error: consumer was moved in the call to into_fn()
    /// let mut value = 5;
    /// consumer.accept(&mut value);
    /// ```
    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// Implement Consumer trait for closures
// ============================================================================

/// Implements Consumer for all FnMut(&mut T)
impl<T, F> Consumer<T> for F
where
    F: FnMut(&mut T),
{
    fn accept(&mut self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(self)
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcConsumer::new(self)
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self.into_box().into_fn()
    }
}

// ============================================================================
// Provide extension methods for closures
// ============================================================================

/// Extension trait providing consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnMut(&mut T)`, enabling direct method chaining on closures
/// without explicit wrapper types.
///
/// # Design Rationale
///
/// This trait allows closures to be composed naturally using method syntax,
/// similar to iterator combinators. Composition methods consume the closure
/// and return `BoxConsumer<T>`, which can be further chained.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxConsumer**: Composition results are `BoxConsumer<T>` for
///   continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&mut T)` closures get these
///   methods automatically
///
/// # Examples
///
/// ## Basic Chaining
///
/// ```rust
/// use prism3_function::{Consumer, FnConsumerOps};
///
/// let chained = (|x: &mut i32| *x *= 2)
///     .and_then(|x: &mut i32| *x += 10);
/// let mut value = 5;
/// let mut result = chained;
/// result.accept(&mut value);
/// assert_eq!(value, 20); // (5 * 2) + 10
/// ```
///
/// ## Multi-Step Composition
///
/// ```rust
/// use prism3_function::{Consumer, FnConsumerOps};
///
/// let pipeline = (|x: &mut i32| *x *= 2)
///     .and_then(|x: &mut i32| *x += 10)
///     .and_then(|x: &mut i32| *x /= 2);
///
/// let mut value = 5;
/// let mut result = pipeline;
/// result.accept(&mut value);
/// assert_eq!(value, 10); // ((5 * 2) + 10) / 2
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnConsumerOps<T>: FnMut(&mut T) + Sized {
    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxConsumer<T>`.
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
    /// Returns the composed `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnConsumerOps};
    ///
    /// let chained = (|x: &mut i32| *x *= 2)
    ///     .and_then(|x: &mut i32| *x += 10)
    ///     .and_then(|x: &mut i32| println!("Result: {}", x));
    ///
    /// let mut value = 5;
    /// let mut result = chained;
    /// result.accept(&mut value); // Prints: Result: 20
    /// assert_eq!(value, 20);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// Implements FnConsumerOps for all closure types
impl<T, F> FnConsumerOps<T> for F where F: FnMut(&mut T) {}

// ============================================================================
// BoxConsumer - Single Ownership Implementation
// ============================================================================

/// BoxConsumer struct
///
/// A consumer implementation based on `Box<dyn FnMut(&mut T)>` for single
/// ownership scenarios. This is the simplest and most efficient consumer
/// type when sharing is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Builder Pattern**: Method chaining consumes `self` naturally
/// - **Factory Methods**: Convenient constructors for common patterns
///
/// # Comparison with Other Consumers
///
/// | Feature             | BoxConsumer | ArcConsumer | RcConsumer |
/// |---------------------|-------------|-------------|------------|
/// | Ownership Model     | Single      | Shared      | Shared     |
/// | Cloneable           | No          | Yes         | Yes        |
/// | Thread-Safe         | No          | Yes         | No         |
/// | Reference Counting  | No          | Yes (Arc)   | Yes (Rc)   |
/// | Interior Mutability | No          | Mutex       | RefCell    |
/// | Lock Overhead       | None        | Yes         | None       |
/// | `and_then` API      | `self`      | `&self`     | `&self`    |
///
/// # Use Cases
///
/// Choose `BoxConsumer` when:
/// - The consumer is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the consumer across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Performance
///
/// `BoxConsumer` has the best performance among the three consumer types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
/// let mut value = 5;
/// consumer.accept(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// let mut chained = BoxConsumer::new(|x: &mut i32| *x *= 2)
///     .and_then(|x: &mut i32| *x += 10)
///     .and_then(|x: &mut i32| *x -= 1);
/// let mut value = 5;
/// chained.accept(&mut value);
/// assert_eq!(value, 19); // ((5 * 2) + 10) - 1
/// ```
///
/// ## Using Factory Methods
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
///
/// // No-op consumer
/// let mut noop = BoxConsumer::<i32>::noop();
/// let mut value = 42;
/// noop.accept(&mut value);
/// assert_eq!(value, 42);
///
/// // Print consumer
/// let mut print = BoxConsumer::<i32>::print();
/// let mut value = 42;
/// print.accept(&mut value); // Prints: 42
///
/// // Conditional consumer
/// let mut conditional = BoxConsumer::if_then(
///     |x: &i32| *x > 0,
///     |x: &mut i32| *x *= 2
/// );
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConsumer<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> BoxConsumer<T>
where
    T: 'static,
{
    /// Creates a new BoxConsumer
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
    /// Returns a new `BoxConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut consumer = BoxConsumer::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// consumer.accept(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        BoxConsumer { func: Box::new(f) }
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
    /// Returns a new composed `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut chained = BoxConsumer::new(|x: &mut i32| *x *= 2)
    ///     .and_then(|x: &mut i32| *x += 10)
    ///     .and_then(|x: &mut i32| println!("Result: {}", x));
    ///
    /// let mut value = 5;
    /// chained.accept(&mut value); // Prints: Result: 20
    /// assert_eq!(value, 20);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: Consumer<T> + 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// Creates a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut noop = BoxConsumer::<i32>::noop();
    /// let mut value = 42;
    /// noop.accept(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxConsumer::new(|_| {})
    }

    /// Creates a print consumer
    ///
    /// Returns a consumer that prints the input value.
    ///
    /// # Returns
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print();
    /// let mut value = 42;
    /// print.accept(&mut value); // Prints: 42
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        BoxConsumer::new(|t| {
            println!("{:?}", t);
        })
    }

    /// Creates a print consumer with prefix
    ///
    /// Returns a consumer that prints the input value with a prefix.
    ///
    /// # Parameters
    ///
    /// * `prefix` - The prefix string
    ///
    /// # Returns
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print_with("Value: ");
    /// let mut value = 42;
    /// print.accept(&mut value); // Prints: Value: 42
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxConsumer::new(move |t| {
            println!("{}{:?}", prefix, t);
        })
    }

    /// Creates a transform consumer
    ///
    /// Returns a consumer that applies a transformation function.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The transformation function type
    ///
    /// # Parameters
    ///
    /// * `f` - The transformation function
    ///
    /// # Returns
    ///
    /// Returns a transform consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut double = BoxConsumer::transform(|x: &i32| x * 2);
    /// let mut value = 5;
    /// double.accept(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    pub fn transform<F>(f: F) -> Self
    where
        F: FnMut(&T) -> T + 'static,
    {
        let mut func = f;
        BoxConsumer::new(move |t| {
            *t = func(t);
        })
    }

    /// Creates a conditional consumer
    ///
    /// Returns a consumer that only executes the operation when the predicate
    /// is true.
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
    /// Returns a conditional consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut conditional = BoxConsumer::if_then(
    ///     |x: &i32| *x > 0,
    ///     |x: &mut i32| *x += 1
    /// );
    ///
    /// let mut positive = 5;
    /// conditional.accept(&mut positive);
    /// assert_eq!(positive, 6);
    ///
    /// let mut negative = -5;
    /// conditional.accept(&mut negative);
    /// assert_eq!(negative, -5); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }

    /// Creates a conditional branch consumer
    ///
    /// Returns a consumer that executes different operations based on the
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
    /// Returns a conditional branch consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut conditional = BoxConsumer::if_then_else(
    ///     |x: &i32| *x > 0,
    ///     |x: &mut i32| *x += 1,
    ///     |x: &mut i32| *x -= 1
    /// );
    ///
    /// let mut positive = 5;
    /// conditional.accept(&mut positive);
    /// assert_eq!(positive, 6);
    ///
    /// let mut negative = -5;
    /// conditional.accept(&mut negative);
    /// assert_eq!(negative, -6);
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&mut T) + 'static,
        C2: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                then_cons(t);
            } else {
                else_cons(t);
            }
        })
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let mut func = self.func;
        RcConsumer::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        // 注意：BoxConsumer 的 func 不一定是 Send，所以无法安全转换为 ArcConsumer
        // 这里我们 panic，因为这个转换在类型系统上是不安全的
        panic!("Cannot convert BoxConsumer to ArcConsumer: BoxConsumer's inner function may not be Send")
    }

    fn into_fn(mut self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t: &mut T| (self.func)(t)
    }
}

// ============================================================================
// ArcConsumer - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcConsumer struct
///
/// A consumer implementation based on `Arc<Mutex<dyn FnMut(&mut T) + Send>>`
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
/// # Comparison with Other Consumers
///
/// | Feature             | BoxConsumer | ArcConsumer | RcConsumer |
/// |---------------------|-------------|-------------|------------|
/// | Ownership Model     | Single      | Shared      | Shared     |
/// | Cloneable           | No          | Yes         | Yes        |
/// | Thread-Safe         | No          | Yes         | No         |
/// | Reference Counting  | No          | Yes (Arc)   | Yes (Rc)   |
/// | Interior Mutability | No          | Mutex       | RefCell    |
/// | Lock Overhead       | None        | Yes         | None       |
/// | `and_then` API      | `self`      | `&self`     | `&self`    |
///
/// # Use Cases
///
/// Choose `ArcConsumer` when:
/// - The consumer needs to be shared across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - The same consumer is used in multiple places simultaneously
/// - Thread safety is required (Send + Sync)
///
/// # Performance Considerations
///
/// `ArcConsumer` has some performance overhead compared to `BoxConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call acquires a lock
/// - **Lock Contention**: High concurrency may cause contention
///
/// However, this overhead is necessary for safe concurrent access. If thread
/// safety is not required, consider `RcConsumer` for single-threaded
/// sharing with less overhead.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
///
/// let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
/// let clone = consumer.clone();
///
/// let mut value = 5;
/// let mut c = consumer;
/// c.accept(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// ## Multi-Threaded Usage
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
/// use std::thread;
///
/// let shared = ArcConsumer::new(|x: &mut i32| *x *= 2);
///
/// // Clone for another thread
/// let shared_clone = shared.clone();
/// let handle = thread::spawn(move || {
///     let mut value = 5;
///     let mut consumer = shared_clone;
///     consumer.accept(&mut value);
///     value
/// });
///
/// // Original consumer still usable
/// let mut value = 3;
/// let mut consumer = shared;
/// consumer.accept(&mut value);
/// assert_eq!(value, 6);
/// assert_eq!(handle.join().unwrap(), 10);
/// ```
///
/// ## Method Chaining with Shared Ownership
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
///
/// let first = ArcConsumer::new(|x: &mut i32| *x *= 2);
/// let second = ArcConsumer::new(|x: &mut i32| *x += 10);
///
/// // Both consumers remain usable after chaining
/// let chained = first.and_then(&second);
///
/// let mut value = 5;
/// let mut c = chained;
/// c.accept(&mut value);
/// assert_eq!(value, 20); // (5 * 2) + 10
///
/// // first and second are still usable here
/// let mut v = 3;
/// let mut f = first;
/// f.accept(&mut v);
/// assert_eq!(v, 6);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&mut T) + Send>>,
}

impl<T> ArcConsumer<T>
where
    T: Send + 'static,
{
    /// Creates a new ArcConsumer
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
    /// Returns a new `ArcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let consumer = ArcConsumer::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// let mut c = consumer;
    /// c.accept(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        ArcConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// Chains another ArcConsumer in sequence
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
    /// Returns a new composed `ArcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let first = ArcConsumer::new(|x: &mut i32| *x *= 2);
    /// let second = ArcConsumer::new(|x: &mut i32| *x += 10);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// let mut c = chained;
    /// c.accept(&mut value);
    /// assert_eq!(value, 20); // (5 * 2) + 10
    /// ```
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T> {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let func = self.func;
        RcConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let func = self.func;
        move |t: &mut T| func.lock().unwrap()(t)
    }
}

impl<T> Clone for ArcConsumer<T> {
    /// Clones the ArcConsumer
    ///
    /// Creates a new ArcConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// RcConsumer - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcConsumer struct
///
/// A consumer implementation based on `Rc<RefCell<dyn FnMut(&mut T)>>` for
/// single-threaded shared ownership scenarios. This consumer provides the
/// benefits of shared ownership without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **No Lock Overhead**: More efficient than `ArcConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Comparison with Other Consumers
///
/// | Feature             | BoxConsumer | ArcConsumer | RcConsumer |
/// |---------------------|-------------|-------------|------------|
/// | Ownership Model     | Single      | Shared      | Shared     |
/// | Cloneable           | No          | Yes         | Yes        |
/// | Thread-Safe         | No          | Yes         | No         |
/// | Reference Counting  | No          | Yes (Arc)   | Yes (Rc)   |
/// | Interior Mutability | No          | Mutex       | RefCell    |
/// | Lock Overhead       | None        | Yes         | None       |
/// | `and_then` API      | `self`      | `&self`     | `&self`    |
///
/// # Use Cases
///
/// Choose `RcConsumer` when:
/// - The consumer needs to be shared within a single thread
/// - Thread safety is not required
/// - Performance is important (avoiding lock overhead)
/// - UI event handling in single-threaded frameworks
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcConsumer` provides better performance than `ArcConsumer` for
/// single-threaded scenarios:
/// - **Non-Atomic Counting**: Cheaper clone/drop than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checks, not locks
/// - **Better Cache Locality**: No atomic operations means better CPU cache
///   behavior
///
/// However, it has slight overhead compared to `BoxConsumer`:
/// - **Reference Counting**: Non-atomic but still present
/// - **Runtime Borrow Checking**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcConsumer` is NOT thread-safe and does NOT implement `Send` or `Sync`.
/// Attempting to send it to another thread will result in a compile error.
/// For thread-safe sharing, use `ArcConsumer` instead.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let consumer = RcConsumer::new(|x: &mut i32| *x *= 2);
/// let clone = consumer.clone();
///
/// let mut value = 5;
/// let mut c = consumer;
/// c.accept(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// ## Sharing Within a Single Thread
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let shared = RcConsumer::new(|x: &mut i32| *x *= 2);
///
/// // Clone for use in multiple places
/// let clone1 = shared.clone();
/// let clone2 = shared.clone();
///
/// let mut value1 = 5;
/// let mut c1 = clone1;
/// c1.accept(&mut value1);
/// assert_eq!(value1, 10);
///
/// let mut value2 = 3;
/// let mut c2 = clone2;
/// c2.accept(&mut value2);
/// assert_eq!(value2, 6);
///
/// // Original is still usable
/// let mut value3 = 7;
/// let mut c3 = shared;
/// c3.accept(&mut value3);
/// assert_eq!(value3, 14);
/// ```
///
/// ## Method Chaining with Shared Ownership
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
///
/// let first = RcConsumer::new(|x: &mut i32| *x *= 2);
/// let second = RcConsumer::new(|x: &mut i32| *x += 10);
///
/// // Both consumers remain usable after chaining
/// let chained = first.and_then(&second);
///
/// let mut value = 5;
/// let mut c = chained;
/// c.accept(&mut value);
/// assert_eq!(value, 20); // (5 * 2) + 10
///
/// // first and second are still usable here
/// let mut v = 3;
/// let mut f = first;
/// f.accept(&mut v);
/// assert_eq!(v, 6);
/// ```
///
/// ## Event Handler Pattern
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
/// use std::collections::HashMap;
///
/// struct EventSystem {
///     handlers: HashMap<String, Vec<RcConsumer<i32>>>,
/// }
///
/// impl EventSystem {
///     fn new() -> Self {
///         Self {
///             handlers: HashMap::new(),
///         }
///     }
///
///     fn register(&mut self, event: &str, handler: RcConsumer<i32>) {
///         self.handlers.entry(event.to_string())
///             .or_insert_with(Vec::new)
///             .push(handler);
///     }
///
///     fn trigger(&mut self, event: &str, value: &mut i32) {
///         if let Some(handlers) = self.handlers.get_mut(event) {
///             for handler in handlers.iter_mut() {
///                 handler.accept(value);
///             }
///         }
///     }
/// }
///
/// let mut system = EventSystem::new();
/// let handler = RcConsumer::new(|x: &mut i32| *x *= 2);
/// system.register("double", handler.clone());
/// system.register("double", handler.clone());
///
/// let mut value = 5;
/// system.trigger("double", &mut value);
/// assert_eq!(value, 20); // Applied twice: 5 * 2 * 2
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcConsumer<T> {
    func: Rc<RefCell<dyn FnMut(&mut T)>>,
}

impl<T> RcConsumer<T>
where
    T: 'static,
{
    /// Creates a new RcConsumer
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
    /// Returns a new `RcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let consumer = RcConsumer::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// let mut c = consumer;
    /// c.accept(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        RcConsumer {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// Chains another RcConsumer in sequence
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
    /// Returns a new composed `RcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let first = RcConsumer::new(|x: &mut i32| *x *= 2);
    /// let second = RcConsumer::new(|x: &mut i32| *x += 10);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// let mut c = chained;
    /// c.accept(&mut value);
    /// assert_eq!(value, 20); // (5 * 2) + 10
    /// ```
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T> {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcConsumer {
            func: Rc::new(RefCell::new(move |t: &mut T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.borrow_mut())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxConsumer::new(move |t| func.borrow_mut()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcConsumer to ArcConsumer (not Send)")
    }

    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let func = self.func;
        move |t: &mut T| func.borrow_mut()(t)
    }
}

impl<T> Clone for RcConsumer<T> {
    /// Clones the RcConsumer
    ///
    /// Creates a new RcConsumer that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
