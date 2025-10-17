/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Mutator Types
//!
//! Provides Java-like `Mutator` interface implementations for performing
//! operations that accept a single mutable input parameter and return no result.
//!
//! This module provides a unified `Mutator` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxMutator<T>`**: Box-based single ownership implementation for
//!   one-time use scenarios and builder patterns
//! - **`ArcMutator<T>`**: Arc<Mutex<>>-based thread-safe shared ownership
//!   implementation for multi-threaded scenarios
//! - **`RcMutator<T>`**: Rc<RefCell<>>-based single-threaded shared
//!   ownership implementation with no lock overhead
//!
//! # Design Philosophy
//!
//! Unlike `Consumer` which observes values without modifying them (`FnMut(&T)`),
//! `Mutator` is designed to **modify input values** using `FnMut(&mut T)`.
//!
//! ## Mutator vs Consumer
//!
//! | Type | Input | Modifies Input? | Modifies Self? | Use Cases |
//! |------|-------|----------------|----------------|-----------|
//! | **Consumer** | `&T` | ❌ | ✅ | Observe, log, count, notify |
//! | **Mutator** | `&mut T` | ✅ | ✅ | Modify, transform, update |
//!
//! **Key Insight**: If you need to modify input values, use `Mutator`.
//! If you only need to observe or accumulate state, use `Consumer`.
//!
//! # Comparison Table
//!
//! | Feature          | BoxMutator | ArcMutator | RcMutator |
//! |------------------|------------|------------|-----------|
//! | Ownership        | Single     | Shared     | Shared    |
//! | Cloneable        | ❌         | ✅         | ✅        |
//! | Thread-Safe      | ❌         | ✅         | ❌        |
//! | Interior Mut.    | N/A        | Mutex      | RefCell   |
//! | `and_then` API   | `self`     | `&self`    | `&self`   |
//! | Lock Overhead    | None       | Yes        | None      |
//!
//! # Use Cases
//!
//! ## BoxMutator
//!
//! - One-time operations that don't require sharing
//! - Builder patterns where ownership naturally flows
//! - Simple scenarios with no reuse requirements
//!
//! ## ArcMutator
//!
//! - Multi-threaded shared operations
//! - Concurrent task processing (e.g., thread pools)
//! - Situations requiring the same mutator across threads
//!
//! ## RcMutator
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
//! use prism3_function::{BoxMutator, ArcMutator, RcMutator, Mutator};
//!
//! // BoxMutator: Single ownership, consumes self
//! let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
//! let mut value = 5;
//! mutator.mutate(&mut value);
//! assert_eq!(value, 10);
//!
//! // ArcMutator: Shared ownership, cloneable, thread-safe
//! let shared = ArcMutator::new(|x: &mut i32| *x *= 2);
//! let clone = shared.clone();
//! let mut value = 5;
//! let mut m = shared;
//! m.mutate(&mut value);
//! assert_eq!(value, 10);
//!
//! // RcMutator: Shared ownership, cloneable, single-threaded
//! let rc = RcMutator::new(|x: &mut i32| *x *= 2);
//! let clone = rc.clone();
//! let mut value = 5;
//! let mut m = rc;
//! m.mutate(&mut value);
//! assert_eq!(value, 10);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{Mutator, BoxMutator, ArcMutator};
//!
//! // BoxMutator: Consumes self
//! let mut chained = BoxMutator::new(|x: &mut i32| *x *= 2)
//!     .and_then(|x: &mut i32| *x += 10);
//! let mut value = 5;
//! chained.mutate(&mut value);
//! assert_eq!(value, 20); // (5 * 2) + 10
//!
//! // ArcMutator: Borrows &self, original still usable
//! let first = ArcMutator::new(|x: &mut i32| *x *= 2);
//! let second = ArcMutator::new(|x: &mut i32| *x += 10);
//! let combined = first.and_then(&second);
//! // first and second are still usable here
//! ```
//!
//! ## Working with Closures
//!
//! All closures automatically implement the `Mutator` trait:
//!
//! ```rust
//! use prism3_function::{Mutator, FnMutatorOps};
//!
//! // Closures can use .mutate() directly
//! let mut closure = |x: &mut i32| *x *= 2;
//! let mut value = 5;
//! closure.mutate(&mut value);
//! assert_eq!(value, 10);
//!
//! // Closures can be chained, returning BoxMutator
//! let mut chained = (|x: &mut i32| *x *= 2)
//!     .and_then(|x: &mut i32| *x += 10);
//! let mut value = 5;
//! chained.mutate(&mut value);
//! assert_eq!(value, 20);
//! ```
//!
//! ## Type Conversions
//!
//! ```rust
//! use prism3_function::Mutator;
//!
//! // Convert closure to concrete type
//! let closure = |x: &mut i32| *x *= 2;
//! let mut box_mutator = closure.into_box();
//!
//! let closure = |x: &mut i32| *x *= 2;
//! let mut rc_mutator = closure.into_rc();
//!
//! let closure = |x: &mut i32| *x *= 2;
//! let mut arc_mutator = closure.into_arc();
//! ```
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ============================================================================
// 1. Mutator Trait - Unified Mutator Interface
// ============================================================================

/// Mutator trait - Unified mutator interface
///
/// Defines the core behavior of all mutator types. Performs operations that
/// accept a mutable reference and modify the input value (not just side effects).
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnMut(&mut T)`
/// - `BoxMutator<T>`, `ArcMutator<T>`, and `RcMutator<T>`
///
/// # Design Rationale
///
/// The trait provides a unified abstraction over different ownership models,
/// allowing generic code to work with any mutator type. Type conversion
/// methods (`into_box`, `into_arc`, `into_rc`) enable flexible ownership
/// transitions based on usage requirements.
///
/// # Features
///
/// - **Unified Interface**: All mutator types share the same `mutate`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions that work with any mutator
///   type
///
/// # Examples
///
/// ## Generic Mutator Function
///
/// ```rust
/// use prism3_function::{Mutator, BoxMutator, ArcMutator};
///
/// fn apply_mutator<M: Mutator<i32>>(
///     mutator: &mut M,
///     value: i32
/// ) -> i32 {
///     let mut val = value;
///     mutator.mutate(&mut val);
///     val
/// }
///
/// // Works with any mutator type
/// let mut box_mut = BoxMutator::new(|x: &mut i32| *x *= 2);
/// assert_eq!(apply_mutator(&mut box_mut, 5), 10);
///
/// let mut arc_mut = ArcMutator::new(|x: &mut i32| *x *= 2);
/// assert_eq!(apply_mutator(&mut arc_mut, 5), 10);
///
/// let mut closure = |x: &mut i32| *x *= 2;
/// assert_eq!(apply_mutator(&mut closure, 5), 10);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::Mutator;
///
/// let closure = |x: &mut i32| *x *= 2;
///
/// // Convert to different ownership models
/// let box_mutator = closure.into_box();
/// // let rc_mutator = closure.into_rc();  // closure moved
/// // let arc_mutator = closure.into_arc(); // closure moved
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Mutator<T> {
    /// Performs the mutation operation
    ///
    /// Executes an operation on the given mutable reference. The operation
    /// typically modifies the input value or produces side effects.
    ///
    /// # Parameters
    ///
    /// * `value` - A mutable reference to the value to be mutated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
    /// let mut value = 5;
    /// mutator.mutate(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    fn mutate(&mut self, value: &mut T);

    /// Converts to BoxMutator
    ///
    /// **⚠️ Consumes `self`**: The original mutator becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current mutator to `BoxMutator<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the mutator (takes ownership of `self`).
    /// After calling this method, the original mutator is no longer
    /// available.
    ///
    /// **Tip**: For cloneable mutators ([`ArcMutator`], [`RcMutator`]),
    /// you can call `.clone()` first if you need to keep the original.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxMutator<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::Mutator;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut box_mutator = closure.into_box();
    /// let mut value = 5;
    /// box_mutator.mutate(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    fn into_box(self) -> BoxMutator<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to RcMutator
    ///
    /// **⚠️ Consumes `self`**: The original mutator becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Mutator;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut rc_mutator = closure.into_rc();
    /// let mut value = 5;
    /// rc_mutator.mutate(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    fn into_rc(self) -> RcMutator<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to ArcMutator
    ///
    /// **⚠️ Consumes `self`**: The original mutator becomes unavailable
    /// after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Mutator;
    ///
    /// let closure = |x: &mut i32| *x *= 2;
    /// let mut arc_mutator = closure.into_arc();
    /// let mut value = 5;
    /// arc_mutator.mutate(&mut value);
    /// assert_eq!(value, 10);
    /// ```
    fn into_arc(self) -> ArcMutator<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;

    /// Converts mutator to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original mutator becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the mutator and returns a closure that can be
    /// directly used with iterator methods like `for_each()`.
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(&mut T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
    /// let mut values = vec![1, 2, 3, 4, 5];
    ///
    /// values.iter_mut().for_each(mutator.into_fn());
    ///
    /// assert_eq!(values, vec![2, 4, 6, 8, 10]);
    /// ```
    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. Type Aliases
// ============================================================================

/// Type alias for Arc-wrapped mutable mutator function
type ArcMutMutatorFn<T> = Arc<Mutex<dyn FnMut(&mut T) + Send>>;

/// Type alias for Rc-wrapped mutable mutator function
type RcMutMutatorFn<T> = Rc<RefCell<dyn FnMut(&mut T)>>;

// ============================================================================
// 3. BoxMutator - Single Ownership Implementation
// ============================================================================

/// BoxMutator struct
///
/// A mutator implementation based on `Box<dyn FnMut(&mut T)>` for single
/// ownership scenarios. This is the simplest and most efficient mutator
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
/// # Use Cases
///
/// Choose `BoxMutator` when:
/// - The mutator is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the mutator across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Performance
///
/// `BoxMutator` has the best performance among the three mutator types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mutator, BoxMutator};
///
/// let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
/// let mut value = 5;
/// mutator.mutate(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxMutator<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> BoxMutator<T>
where
    T: 'static,
{
    /// Creates a new BoxMutator
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutator<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut mutator = BoxMutator::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// mutator.mutate(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        BoxMutator { func: Box::new(f) }
    }

    /// Creates a no-op mutator
    ///
    /// Returns a mutator that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut noop = BoxMutator::<i32>::noop();
    /// let mut value = 42;
    /// noop.mutate(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxMutator::new(|_| {})
    }

    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut chained = BoxMutator::new(|x: &mut i32| *x *= 2)
    ///     .and_then(|x: &mut i32| *x += 10)
    ///     .and_then(|x: &mut i32| println!("Result: {}", x));
    ///
    /// let mut value = 5;
    /// chained.mutate(&mut value); // Prints: Result: 20
    /// assert_eq!(value, 20);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: Mutator<T> + 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxMutator::new(move |t| {
            first(t);
            second.mutate(t);
        })
    }

    /// Creates a conditional mutator
    ///
    /// Returns a mutator that only executes the operation when the predicate
    /// is true.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `mutator` - The mutator to execute
    ///
    /// # Returns
    ///
    /// Returns a conditional mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut conditional = BoxMutator::if_then(
    ///     |x: &i32| *x > 0,
    ///     |x: &mut i32| *x += 1
    /// );
    ///
    /// let mut positive = 5;
    /// conditional.mutate(&mut positive);
    /// assert_eq!(positive, 6);
    ///
    /// let mut negative = -5;
    /// conditional.mutate(&mut negative);
    /// assert_eq!(negative, -5); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, mutator: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut mut_fn = mutator;
        BoxMutator::new(move |t| {
            if pred(t) {
                mut_fn(t);
            }
        })
    }

    /// Creates a conditional branch mutator
    ///
    /// Returns a mutator that executes different operations based on the
    /// predicate.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `then_mutator` - The mutator to execute when predicate is true
    /// * `else_mutator` - The mutator to execute when predicate is false
    ///
    /// # Returns
    ///
    /// Returns a conditional branch mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, BoxMutator};
    ///
    /// let mut conditional = BoxMutator::if_then_else(
    ///     |x: &i32| *x > 0,
    ///     |x: &mut i32| *x += 1,
    ///     |x: &mut i32| *x -= 1
    /// );
    ///
    /// let mut positive = 5;
    /// conditional.mutate(&mut positive);
    /// assert_eq!(positive, 6);
    ///
    /// let mut negative = -5;
    /// conditional.mutate(&mut negative);
    /// assert_eq!(negative, -6);
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_mutator: C1, else_mutator: C2) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&mut T) + 'static,
        C2: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut then_mut = then_mutator;
        let mut else_mut = else_mutator;
        BoxMutator::new(move |t| {
            if pred(t) {
                then_mut(t);
            } else {
                else_mut(t);
            }
        })
    }
}

impl<T> Mutator<T> for BoxMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func)(value)
    }

    fn into_box(self) -> BoxMutator<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcMutator<T>
    where
        T: 'static,
    {
        let mut func = self.func;
        RcMutator::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcMutator<T>
    where
        T: Send + 'static,
    {
        panic!(
            "Cannot convert BoxMutator to ArcMutator: BoxMutator's inner function may not be Send"
        )
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
// 4. ArcMutator - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcMutator struct
///
/// A mutator implementation based on `Arc<Mutex<dyn FnMut(&mut T) + Send>>`
/// for thread-safe shared ownership scenarios.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains usable
///
/// # Use Cases
///
/// Choose `ArcMutator` when:
/// - The mutator needs to be shared across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Thread safety is required (Send + Sync)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mutator, ArcMutator};
///
/// let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
/// let clone = mutator.clone();
///
/// let mut value = 5;
/// let mut m = mutator;
/// m.mutate(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcMutator<T> {
    func: ArcMutMutatorFn<T>,
}

impl<T> ArcMutator<T>
where
    T: Send + 'static,
{
    /// Creates a new ArcMutator
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `ArcMutator<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, ArcMutator};
    ///
    /// let mutator = ArcMutator::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// let mut m = mutator;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        ArcMutator {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates a no-op mutator
    ///
    /// Returns a mutator that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, ArcMutator};
    ///
    /// let noop = ArcMutator::<i32>::noop();
    /// let mut value = 42;
    /// let mut m = noop;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        ArcMutator::new(|_| {})
    }

    /// Chains another ArcMutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original mutator.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, ArcMutator};
    ///
    /// let first = ArcMutator::new(|x: &mut i32| *x *= 2);
    /// let second = ArcMutator::new(|x: &mut i32| *x += 10);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// let mut m = chained;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 20); // (5 * 2) + 10
    /// ```
    pub fn and_then(&self, next: &ArcMutator<T>) -> ArcMutator<T> {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcMutator {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

impl<T> Mutator<T> for ArcMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxMutator<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxMutator::new(move |t| func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcMutator<T>
    where
        T: 'static,
    {
        let func = self.func;
        RcMutator::new(move |t| func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcMutator<T>
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

impl<T> Clone for ArcMutator<T> {
    /// Clones the ArcMutator
    ///
    /// Creates a new ArcMutator that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 5. RcMutator - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcMutator struct
///
/// A mutator implementation based on `Rc<RefCell<dyn FnMut(&mut T)>>` for
/// single-threaded shared ownership scenarios.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **No Lock Overhead**: More efficient than `ArcMutator` for single-threaded use
///
/// # Use Cases
///
/// Choose `RcMutator` when:
/// - The mutator needs to be shared within a single thread
/// - Thread safety is not required
/// - Performance is important (avoiding lock overhead)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mutator, RcMutator};
///
/// let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
/// let clone = mutator.clone();
///
/// let mut value = 5;
/// let mut m = mutator;
/// m.mutate(&mut value);
/// assert_eq!(value, 10);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcMutator<T> {
    func: RcMutMutatorFn<T>,
}

impl<T> RcMutator<T>
where
    T: 'static,
{
    /// Creates a new RcMutator
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `RcMutator<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, RcMutator};
    ///
    /// let mutator = RcMutator::new(|x: &mut i32| *x += 1);
    /// let mut value = 5;
    /// let mut m = mutator;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 6);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        RcMutator {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates a no-op mutator
    ///
    /// Returns a mutator that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op mutator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, RcMutator};
    ///
    /// let noop = RcMutator::<i32>::noop();
    /// let mut value = 42;
    /// let mut m = noop;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        RcMutator::new(|_| {})
    }

    /// Chains another RcMutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Borrows &self, does not consume the
    /// original mutator.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, RcMutator};
    ///
    /// let first = RcMutator::new(|x: &mut i32| *x *= 2);
    /// let second = RcMutator::new(|x: &mut i32| *x += 10);
    ///
    /// let chained = first.and_then(&second);
    ///
    /// // first and second are still usable
    /// let mut value = 5;
    /// let mut m = chained;
    /// m.mutate(&mut value);
    /// assert_eq!(value, 20); // (5 * 2) + 10
    /// ```
    pub fn and_then(&self, next: &RcMutator<T>) -> RcMutator<T> {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcMutator {
            func: Rc::new(RefCell::new(move |t: &mut T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
        }
    }
}

impl<T> Mutator<T> for RcMutator<T> {
    fn mutate(&mut self, value: &mut T) {
        (self.func.borrow_mut())(value)
    }

    fn into_box(self) -> BoxMutator<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxMutator::new(move |t| func.borrow_mut()(t))
    }

    fn into_rc(self) -> RcMutator<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcMutator<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcMutator to ArcMutator (not Send)")
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

impl<T> Clone for RcMutator<T> {
    /// Clones the RcMutator
    ///
    /// Creates a new RcMutator that shares the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. Implement Mutator trait for closures
// ============================================================================

/// Implements Mutator for all FnMut(&mut T)
impl<T, F> Mutator<T> for F
where
    F: FnMut(&mut T),
{
    fn mutate(&mut self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxMutator<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxMutator::new(self)
    }

    fn into_rc(self) -> RcMutator<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcMutator::new(self)
    }

    fn into_arc(self) -> ArcMutator<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcMutator::new(self)
    }

    fn into_fn(self) -> impl FnMut(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 7. Provide extension methods for closures
// ============================================================================

/// Extension trait providing mutator composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnMut(&mut T)`, enabling direct method chaining on closures
/// without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxMutator**: Composition results are `BoxMutator<T>` for
///   continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&mut T)` closures get these
///   methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Mutator, FnMutatorOps};
///
/// let chained = (|x: &mut i32| *x *= 2)
///     .and_then(|x: &mut i32| *x += 10);
/// let mut value = 5;
/// let mut result = chained;
/// result.mutate(&mut value);
/// assert_eq!(value, 20); // (5 * 2) + 10
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnMutatorOps<T>: FnMut(&mut T) + Sized {
    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxMutator<T>`.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMutator<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Mutator, FnMutatorOps};
    ///
    /// let chained = (|x: &mut i32| *x *= 2)
    ///     .and_then(|x: &mut i32| *x += 10)
    ///     .and_then(|x: &mut i32| println!("Result: {}", x));
    ///
    /// let mut value = 5;
    /// let mut result = chained;
    /// result.mutate(&mut value); // Prints: Result: 20
    /// assert_eq!(value, 20);
    /// ```
    fn and_then<C>(self, next: C) -> BoxMutator<T>
    where
        Self: 'static,
        C: Mutator<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxMutator::new(move |t| {
            first(t);
            second.mutate(t);
        })
    }
}

/// Implements FnMutatorOps for all closure types
impl<T, F> FnMutatorOps<T> for F where F: FnMut(&mut T) {}
