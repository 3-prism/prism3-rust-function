// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # StatefulMutatingFunction Types
//!
//! Provides Java-like `StatefulMutatingFunction` interface implementations
//! for performing operations that accept a mutable reference, potentially
//! modify internal state, and return a result.
//!
//! It is similar to the `FnMut(&mut T) -> R` trait in the standard library.
//!
//! This module provides a unified `StatefulMutatingFunction` trait and three
//! concrete implementations based on different ownership models:
//!
//! - **`BoxStatefulMutatingFunction<T, R>`**: Box-based single ownership
//!   implementation
//! - **`ArcStatefulMutatingFunction<T, R>`**: Arc<Mutex<>>-based thread-safe
//!   shared ownership implementation
//! - **`RcStatefulMutatingFunction<T, R>`**: Rc<RefCell<>>-based
//!   single-threaded shared ownership implementation
//!
//! # Design Philosophy
//!
//! `StatefulMutatingFunction` extends `MutatingFunction` with the ability to
//! maintain internal state:
//!
//! - **MutatingFunction**: `Fn(&mut T) -> R` - stateless, immutable self
//! - **StatefulMutatingFunction**: `FnMut(&mut T) -> R` - stateful, mutable
//!   self
//!
//! ## Comparison with Related Types
//!
//! | Type | Self | Input | Modifies Self? | Modifies Input? | Returns? |
//! |------|------|-------|----------------|-----------------|----------|
//! | **StatefulFunction** | `&mut self` | `&T` | ✅ | ❌ | ✅ |
//! | **StatefulMutator** | `&mut self` | `&mut T` | ✅ | ✅ | ❌ |
//! | **StatefulMutatingFunction** | `&mut self` | `&mut T` | ✅ | ✅ | ✅ |
//!
//! **Key Insight**: Use `StatefulMutatingFunction` when you need to:
//! - Maintain internal state (counters, accumulators, etc.)
//! - Modify the input value
//! - Return information about the operation
//!
//! # Comparison Table
//!
//! | Feature          | Box | Arc | Rc |
//! |------------------|-----|-----|----|
//! | Ownership        | Single | Shared | Shared |
//! | Cloneable        | ❌ | ✅ | ✅ |
//! | Thread-Safe      | ❌ | ✅ | ❌ |
//! | Interior Mut.    | N/A | Mutex | RefCell |
//! | `and_then` API   | `self` | `&self` | `&self` |
//! | Lock Overhead    | None | Yes | None |
//!
//! # Use Cases
//!
//! ## Common Scenarios
//!
//! - **Stateful counters**: Increment and track modification count
//! - **Accumulators**: Collect statistics while modifying data
//! - **Rate limiters**: Track calls and conditionally modify
//! - **Validators**: Accumulate errors while fixing data
//! - **Stateful transformers**: Apply transformations based on history
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use qubit_function::{BoxStatefulMutatingFunction,
//!                       StatefulMutatingFunction};
//!
//! // Counter that increments value and tracks calls
//! let mut counter = {
//!     let mut call_count = 0;
//!     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
//!         call_count += 1;
//!         *x += 1;
//!         call_count
//!     })
//! };
//!
//! let mut value = 5;
//! assert_eq!(counter.apply(&mut value), 1);
//! assert_eq!(value, 6);
//! assert_eq!(counter.apply(&mut value), 2);
//! assert_eq!(value, 7);
//! ```
//!
//! ## Accumulator Pattern
//!
//! ```rust
//! use qubit_function::{BoxStatefulMutatingFunction,
//!                       StatefulMutatingFunction};
//!
//! // Accumulate sum while doubling values
//! let mut accumulator = {
//!     let mut sum = 0;
//!     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
//!         *x *= 2;
//!         sum += *x;
//!         sum
//!     })
//! };
//!
//! let mut value = 5;
//! assert_eq!(accumulator.apply(&mut value), 10);
//! assert_eq!(value, 10);
//!
//! let mut value2 = 3;
//! assert_eq!(accumulator.apply(&mut value2), 16); // 10 + 6
//! assert_eq!(value2, 6);
//! ```
#[cfg(feature = "rc")]
use std::cell::RefCell;
#[cfg(feature = "rc")]
use std::rc::Rc;
use std::sync::Arc;

use parking_lot::Mutex;

mod box_stateful_mutating_function;
pub use box_stateful_mutating_function::BoxStatefulMutatingFunction;
#[cfg(feature = "rc")]
mod rc_stateful_mutating_function;
#[cfg(feature = "rc")]
pub use rc_stateful_mutating_function::RcStatefulMutatingFunction;
mod arc_stateful_mutating_function;
pub use arc_stateful_mutating_function::ArcStatefulMutatingFunction;
mod box_conditional_stateful_mutating_function;
pub use box_conditional_stateful_mutating_function::BoxConditionalStatefulMutatingFunction;
#[cfg(feature = "rc")]
mod rc_conditional_stateful_mutating_function;
#[cfg(feature = "rc")]
pub use rc_conditional_stateful_mutating_function::RcConditionalStatefulMutatingFunction;
mod arc_conditional_stateful_mutating_function;
pub use arc_conditional_stateful_mutating_function::ArcConditionalStatefulMutatingFunction;

// =======================================================================
// 1. StatefulMutatingFunction Trait - Unified Interface
// =======================================================================

/// StatefulMutatingFunction trait - Unified stateful mutating function
/// interface
///
/// It is similar to the `FnMut(&mut T) -> R` trait in the standard library.
///
/// Defines the core behavior of all stateful mutating function types.
/// Performs operations that accept a mutable reference, potentially modify
/// both the function's internal state and the input, and return a result.
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnMut(&mut T) -> R`
/// - `BoxStatefulMutatingFunction<T, R>`, `ArcStatefulMutatingFunction<T, R>`,
///   and `RcStatefulMutatingFunction<T, R>`
///
/// # Design Rationale
///
/// The trait provides a unified abstraction over different ownership models
/// for operations that need to maintain state while modifying input and
/// returning results. This is useful for scenarios where you need to:
/// - Track statistics or counts during modifications
/// - Accumulate information across multiple calls
/// - Implement stateful validators or transformers
///
/// # Features
///
/// - **Unified Interface**: All stateful mutating function types share the same
///   `apply` method signature
/// - **Automatic Implementation**: Closures automatically implement this trait
/// - **Generic Programming**: Write functions that work with any stateful
///   mutating function type
///
/// # Examples
///
/// ## Generic Function
///
/// ```rust
/// use qubit_function::{StatefulMutatingFunction,
///                       BoxStatefulMutatingFunction};
///
/// fn apply_and_log<F: StatefulMutatingFunction<i32, i32>>(
///     func: &mut F,
///     value: i32
/// ) -> i32 {
///     let mut val = value;
///     let result = func.apply(&mut val);
///     println!("Modified: {} -> {}, returned: {}", value, val, result);
///     result
/// }
///
/// let mut counter = {
///     let mut count = 0;
///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x += 1;
///         count
///     })
/// };
/// assert_eq!(apply_and_log(&mut counter, 5), 1);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use qubit_function::{BoxStatefulMutatingFunction, StatefulMutatingFunction};
///
/// let mut count = 0;
/// let closure = move |x: &mut i32| {
///     count += 1;
///     *x *= 2;
///     count
/// };
///
/// // Convert to different ownership models
/// let mut box_func = BoxStatefulMutatingFunction::new(closure);
/// ```
pub trait StatefulMutatingFunction<T, R> {
    /// Applies the function to the mutable reference and returns a result
    ///
    /// Executes an operation on the given mutable reference, potentially
    /// modifying both the function's internal state and the input, and
    /// returns a result value.
    ///
    /// # Parameters
    ///
    /// * `t` - A mutable reference to the input value
    ///
    /// # Returns
    ///
    /// The computed result value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::{StatefulMutatingFunction,
    ///                       BoxStatefulMutatingFunction};
    ///
    /// let mut counter = {
    ///     let mut count = 0;
    ///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
    ///         count += 1;
    ///         let old = *x;
    ///         *x += 1;
    ///         (old, count)
    ///     })
    /// };
    ///
    /// let mut value = 5;
    /// let (old_value, call_count) = counter.apply(&mut value);
    /// assert_eq!(old_value, 5);
    /// assert_eq!(call_count, 1);
    /// assert_eq!(value, 6);
    /// ```
    #[must_use = "the computed callback result should be used"]
    fn apply(&mut self, t: &mut T) -> R;
}

impl<T, R, F> StatefulMutatingFunction<T, R> for F
where
    F: FnMut(&mut T) -> R,
{
    #[inline(always)]
    fn apply(&mut self, t: &mut T) -> R {
        self(t)
    }
}

// =======================================================================
// 2. Type Aliases
// =======================================================================

/// Type alias for Arc-wrapped stateful mutating function
type ArcStatefulMutatingFunctionFn<T, R> = Arc<Mutex<dyn FnMut(&mut T) -> R + Send + 'static>>;

/// Type alias for Rc-wrapped stateful mutating function
/// The erased callback representation used by this implementation.
#[cfg(feature = "rc")]
type RcStatefulMutatingFunctionFn<T, R> = Rc<RefCell<dyn FnMut(&mut T) -> R>>;
