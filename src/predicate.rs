/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Predicate Abstraction
//!
//! Provides a Rust implementation similar to Java's `Predicate` interface
//! for condition testing and logical composition.
//!
//! ## Core Semantics
//!
//! A **Predicate** is fundamentally a pure judgment operation that tests
//! whether a value satisfies a specific condition. It should be:
//!
//! - **Read-only**: Does not modify the tested value
//! - **Side-effect free**: Does not change external state (from the user's
//!   perspective)
//! - **Repeatable**: Same input should produce the same result
//! - **Deterministic**: Judgment logic should be predictable
//!
//! ## Design Philosophy
//!
//! This module follows these principles:
//!
//! 1. **Single Trait**: Only one `Predicate<T>` trait with `&self`, keeping
//!    the API simple and semantically clear
//! 2. **No PredicateMut**: All stateful scenarios use interior mutability
//!    (`RefCell`, `Cell`, `Mutex`) instead of `&mut self`
//! 3. **No PredicateOnce**: Violates predicate semantics - judgments should
//!    be repeatable
//! 4. **Three Implementations**: `BoxPredicate`, `RcPredicate`, and
//!    `ArcPredicate` cover all ownership scenarios
//!
//! ## Type Selection Guide
//!
//! | Scenario | Recommended Type | Reason |
//! |----------|------------------|--------|
//! | One-time use | `BoxPredicate` | Single ownership, no overhead |
//! | Multi-threaded | `ArcPredicate` | Thread-safe, clonable |
//! | Single-threaded reuse | `RcPredicate` | Better performance |
//! | Stateful predicate | Any type + `RefCell`/`Cell`/`Mutex` | Interior mutability |
//!
//! ## Examples
//!
//! ### Basic Usage with Closures
//!
//! ```rust
//! use prism3_function::predicate::Predicate;
//!
//! let is_positive = |x: &i32| *x > 0;
//! assert!(is_positive.test(&5));
//! assert!(!is_positive.test(&-3));
//! ```
//!
//! ### BoxPredicate - Single Ownership
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, BoxPredicate};
//!
//! let pred = BoxPredicate::new(|x: &i32| *x > 0)
//!     .and(BoxPredicate::new(|x| x % 2 == 0));
//! assert!(pred.test(&4));
//! ```
//!
//! ### Closure Composition with Extension Methods
//!
//! Closures automatically gain `and`, `or`, `not` methods through the
//! `FnPredicateOps` extension trait, returning `BoxPredicate`:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, FnPredicateOps, BoxPredicate};
//!
//! // Compose closures directly - result is BoxPredicate
//! let is_positive = |x: &i32| *x > 0;
//! let is_even = |x: &i32| x % 2 == 0;
//!
//! let positive_and_even = is_positive.and(is_even);
//! assert!(positive_and_even.test(&4));
//! assert!(!positive_and_even.test(&3));
//!
//! // Can chain multiple operations
//! let pred = (|x: &i32| *x > 0)
//!     .and(|x: &i32| x % 2 == 0)
//!     .and(BoxPredicate::new(|x: &i32| *x < 100));
//! assert!(pred.test(&42));
//!
//! // Use `or` for disjunction
//! let negative_or_large = (|x: &i32| *x < 0)
//!     .or(|x: &i32| *x > 100);
//! assert!(negative_or_large.test(&-5));
//! assert!(negative_or_large.test(&200));
//!
//! // Use `not` for negation
//! let not_zero = (|x: &i32| *x == 0).not();
//! assert!(not_zero.test(&5));
//! assert!(!not_zero.test(&0));
//! ```
//!
//! ### Complex Predicate Composition
//!
//! Build complex predicates by mixing closures and predicate types:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, BoxPredicate, FnPredicateOps};
//!
//! // Start with a closure, compose with BoxPredicate
//! let in_range = (|x: &i32| *x >= 0)
//!     .and(BoxPredicate::new(|x| *x <= 100));
//!
//! // Use in filtering
//! let numbers = vec![-10, 5, 50, 150, 75];
//! let filtered: Vec<_> = numbers.iter()
//!     .copied()
//!     .filter(in_range.into_fn())
//!     .collect();
//! assert_eq!(filtered, vec![5, 50, 75]);
//! ```
//!
//! ### RcPredicate - Single-threaded Reuse
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, RcPredicate};
//!
//! let pred = RcPredicate::new(|x: &i32| *x > 0);
//! let combined1 = pred.and(RcPredicate::new(|x| x % 2 == 0));
//! let combined2 = pred.or(RcPredicate::new(|x| *x > 100));
//!
//! // Original predicate is still usable
//! assert!(pred.test(&5));
//! ```
//!
//! ### ArcPredicate - Thread-safe Sharing
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, ArcPredicate};
//! use std::thread;
//!
//! let pred = ArcPredicate::new(|x: &i32| *x > 0);
//! let pred_clone = pred.clone();
//!
//! let handle = thread::spawn(move || {
//!     pred_clone.test(&10)
//! });
//!
//! assert!(handle.join().unwrap());
//! assert!(pred.test(&5));  // Original still usable
//! ```
//!
//! ### Stateful Predicates with Interior Mutability
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, BoxPredicate};
//! use std::cell::Cell;
//!
//! let count = Cell::new(0);
//! let pred = BoxPredicate::new(move |x: &i32| {
//!     count.set(count.get() + 1);
//!     *x > 0
//! });
//!
//! // No need for `mut` - interior mutability handles state
//! assert!(pred.test(&5));
//! assert!(!pred.test(&-3));
//! ```
//!
//! ## Author
//!
//! Haixing Hu

use std::fmt::{
    Debug,
    Display,
};
use std::rc::Rc;
use std::sync::Arc;

/// Predicate name constant for always-true predicates
const ALWAYS_TRUE_NAME: &str = "always_true";

/// Predicate name constant for always-false predicates
const ALWAYS_FALSE_NAME: &str = "always_false";

/// A predicate trait for testing whether a value satisfies a condition.
///
/// This trait represents a **pure judgment operation** - it tests whether
/// a given value meets certain criteria without modifying either the value
/// or the predicate itself (from the user's perspective). This semantic
/// clarity distinguishes predicates from consumers or transformers.
///
/// ## Design Rationale
///
/// This is a **minimal trait** that only defines:
/// - The core `test` method using `&self` (immutable borrow)
/// - Type conversion methods (`into_box`, `into_rc`, `into_arc`)
/// - Closure conversion method (`into_fn`)
///
/// Logical composition methods (`and`, `or`, `not`) are intentionally
/// **not** part of the trait. Instead, they are implemented on concrete
/// types (`BoxPredicate`, `RcPredicate`, `ArcPredicate`), allowing each
/// implementation to maintain its specific ownership characteristics:
///
/// - `BoxPredicate`: Methods consume `self` (single ownership)
/// - `RcPredicate`: Methods borrow `&self` (shared ownership)
/// - `ArcPredicate`: Methods borrow `&self` (thread-safe shared ownership)
///
/// ## Why `&self` Instead of `&mut self`?
///
/// Predicates use `&self` because:
///
/// 1. **Semantic Clarity**: A predicate is a judgment, not a mutation
/// 2. **Flexibility**: Can be used in immutable contexts
/// 3. **Simplicity**: No need for `mut` in user code
/// 4. **Interior Mutability**: State (if needed) can be managed with
///    `RefCell`, `Cell`, or `Mutex`
///
/// ## Automatic Implementation for Closures
///
/// Any closure matching `Fn(&T) -> bool` automatically implements this
/// trait, providing seamless integration with Rust's closure system.
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```rust
/// use prism3_function::predicate::Predicate;
///
/// let is_positive = |x: &i32| *x > 0;
/// assert!(is_positive.test(&5));
/// assert!(!is_positive.test(&-3));
/// ```
///
/// ### Type Conversion
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let closure = |x: &i32| *x > 0;
/// let boxed: BoxPredicate<i32> = closure.into_box();
/// assert!(boxed.test(&5));
/// ```
///
/// ### Stateful Predicate with Interior Mutability
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
/// use std::cell::Cell;
///
/// let count = Cell::new(0);
/// let counting_pred = BoxPredicate::new(move |x: &i32| {
///     count.set(count.get() + 1);
///     *x > 0
/// });
///
/// // Note: No `mut` needed - interior mutability handles state
/// assert!(counting_pred.test(&5));
/// assert!(!counting_pred.test(&-3));
/// ```
///
/// ## Author
///
/// Haixing Hu
pub trait Predicate<T> {
    /// Tests whether the given value satisfies this predicate.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to test.
    ///
    /// # Returns
    ///
    /// `true` if the value satisfies this predicate, `false` otherwise.
    fn test(&self, value: &T) -> bool;

    /// Converts this predicate into a `BoxPredicate`.
    ///
    /// The default implementation wraps the predicate in a closure that
    /// calls the `test` method. Concrete types may override this with
    /// more efficient implementations.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` wrapping this predicate.
    fn into_box(self) -> BoxPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| self.test(value))
    }

    /// Converts this predicate into an `RcPredicate`.
    ///
    /// The default implementation wraps the predicate in a closure that
    /// calls the `test` method. Concrete types may override this with
    /// more efficient implementations.
    ///
    /// # Returns
    ///
    /// An `RcPredicate` wrapping this predicate.
    fn into_rc(self) -> RcPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcPredicate::new(move |value: &T| self.test(value))
    }

    /// Converts this predicate into an `ArcPredicate`.
    ///
    /// The default implementation wraps the predicate in a closure that
    /// calls the `test` method. Concrete types may override this with
    /// more efficient implementations.
    ///
    /// # Returns
    ///
    /// An `ArcPredicate` wrapping this predicate.
    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: 'static,
    {
        ArcPredicate::new(move |value: &T| self.test(value))
    }

    /// Converts this predicate into a closure that can be used directly
    /// with standard library methods.
    ///
    /// This method consumes the predicate and returns a closure with
    /// signature `Fn(&T) -> bool`. Since `Fn` is a subtrait of `FnMut`,
    /// the returned closure can be used in any context that requires
    /// either `Fn(&T) -> bool` or `FnMut(&T) -> bool`, making it
    /// compatible with methods like `Iterator::filter`,
    /// `Iterator::filter_map`, `Vec::retain`, and similar standard
    /// library APIs.
    ///
    /// The default implementation returns a closure that calls the
    /// `test` method. Concrete types may override this with more
    /// efficient implementations.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T) -> bool` (also usable as
    /// `FnMut(&T) -> bool`).
    ///
    /// # Examples
    ///
    /// ## Using with `Iterator::filter` (requires `FnMut`)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0);
    ///
    /// let numbers = vec![-2, -1, 0, 1, 2, 3];
    /// let positives: Vec<_> = numbers.iter()
    ///     .copied()
    ///     .filter(pred.into_fn())
    ///     .collect();
    /// assert_eq!(positives, vec![1, 2, 3]);
    /// ```
    ///
    /// ## Using with `Vec::retain` (requires `FnMut`)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x % 2 == 0);
    /// let mut numbers = vec![1, 2, 3, 4, 5, 6];
    /// numbers.retain(pred.into_fn());
    /// assert_eq!(numbers, vec![2, 4, 6]);
    /// ```
    fn into_fn(self) -> impl Fn(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |value: &T| self.test(value)
    }

    /// Converts a reference to this predicate into a `BoxPredicate`.
    ///
    /// This method clones the predicate and then converts it to a
    /// `BoxPredicate`. The original predicate remains usable after this call.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` wrapping a clone of this predicate.
    fn to_box(&self) -> BoxPredicate<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Converts a reference to this predicate into an `RcPredicate`.
    ///
    /// This method clones the predicate and then converts it to an
    /// `RcPredicate`. The original predicate remains usable after this call.
    ///
    /// # Returns
    ///
    /// An `RcPredicate` wrapping a clone of this predicate.
    fn to_rc(&self) -> RcPredicate<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts a reference to this predicate into an `ArcPredicate`.
    ///
    /// This method clones the predicate and then converts it to an
    /// `ArcPredicate`. The original predicate remains usable after this call.
    ///
    /// # Returns
    ///
    /// An `ArcPredicate` wrapping a clone of this predicate.
    fn to_arc(&self) -> ArcPredicate<T>
    where
        Self: Clone + Sized + Send + Sync + 'static,
        T: 'static,
    {
        self.clone().into_arc()
    }

    /// Converts a reference to this predicate into a closure that can be
    /// used directly with standard library methods.
    ///
    /// This method clones the predicate and then converts it to a closure.
    /// The original predicate remains usable after this call.
    ///
    /// The returned closure has signature `Fn(&T) -> bool`. Since `Fn` is a
    /// subtrait of `FnMut`, it can be used in any context that requires
    /// either `Fn(&T) -> bool` or `FnMut(&T) -> bool`, making it compatible
    /// with methods like `Iterator::filter`, `Iterator::filter_map`,
    /// `Vec::retain`, and similar standard library APIs.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T) -> bool` (also usable as
    /// `FnMut(&T) -> bool`).
    fn to_fn(&self) -> impl Fn(&T) -> bool
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_fn()
    }
}

/// A Box-based predicate with single ownership.
///
/// This type is suitable for one-time use scenarios where the predicate does
/// not need to be cloned or shared. Composition methods consume `self`,
/// reflecting the single-ownership model.
///
/// # Examples
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Chaining consumes the predicate
/// let combined = pred.and(BoxPredicate::new(|x| x % 2 == 0));
/// assert!(combined.test(&4));
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxPredicate<T> {
    function: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,
}

impl<T: 'static> BoxPredicate<T> {
    /// Creates a new `BoxPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            function: Box::new(f),
            name: None,
        }
    }

    /// Creates a named `BoxPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `BoxPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            function: Box::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred: BoxPredicate<i32> = BoxPredicate::always_true();
    /// assert!(pred.test(&42));
    /// assert!(pred.test(&-1));
    /// assert!(pred.test(&0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Box::new(|_| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred: BoxPredicate<i32> = BoxPredicate::always_false();
    /// assert!(!pred.test(&42));
    /// assert!(!pred.test(&-1));
    /// assert!(!pred.test(&0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Box::new(|_| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a predicate that represents the logical AND of this predicate
    /// and another.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - Another `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical AND.
    ///
    /// # Examples
    ///
    /// ## Combining with closures
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// // Note: is_positive is moved here, so it's no longer usable
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));   // positive and even
    /// assert!(!combined.test(&3));  // positive but odd
    /// assert!(!combined.test(&-2)); // even but negative
    /// // is_positive.test(&5); // This would not compile - is_positive was moved
    /// ```
    ///
    /// ## Combining with function pointers
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// fn is_even(x: &i32) -> bool { x % 2 == 0 }
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let combined = is_positive.and(is_even);
    ///
    /// assert!(combined.test(&4));
    /// assert!(!combined.test(&3));
    /// ```
    ///
    /// ## Combining with other BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));
    /// assert!(!combined.test(&3));
    /// ```
    ///
    /// ## Chained composition
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0)
    ///     .and(|x: &i32| x % 2 == 0)
    ///     .and(|x: &i32| *x < 100);
    ///
    /// assert!(pred.test(&42));  // positive, even, less than 100
    /// assert!(!pred.test(&101)); // does not satisfy less than 100
    /// ```
    ///
    /// ## Note on ownership
    ///
    /// `BoxPredicate` uses single ownership semantics. If you need to reuse
    /// predicates after composition, consider using `RcPredicate` instead,
    /// which supports cloning and shared ownership.
    pub fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxPredicate {
            function: Box::new(move |value: &T| (self.function)(value) && other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical OR of this predicate
    /// and another.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - Another `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical OR.
    ///
    /// # Examples
    ///
    /// ## Combining with closures
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_negative = BoxPredicate::new(|x: &i32| *x < 0);
    /// let is_large = |x: &i32| *x > 100;
    ///
    /// // Note: is_negative is moved here, so it's no longer usable
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));  // negative
    /// assert!(combined.test(&150)); // greater than 100
    /// assert!(!combined.test(&50)); // neither negative nor greater than 100
    /// // is_negative.test(&-10); // This would not compile - is_negative was moved
    /// ```
    ///
    /// ## Combining with function pointers
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// fn is_large(x: &i32) -> bool { *x > 100 }
    ///
    /// let is_negative = BoxPredicate::new(|x: &i32| *x < 0);
    /// let combined = is_negative.or(is_large);
    ///
    /// assert!(combined.test(&-5));
    /// assert!(combined.test(&150));
    /// ```
    ///
    /// ## Combining with other BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_negative = BoxPredicate::new(|x: &i32| *x < 0);
    /// let is_large = BoxPredicate::new(|x: &i32| *x > 100);
    ///
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));
    /// assert!(combined.test(&150));
    /// ```
    ///
    /// ## Note on ownership
    ///
    /// `BoxPredicate` uses single ownership semantics. If you need to reuse
    /// predicates after composition, consider using `RcPredicate` instead,
    /// which supports cloning and shared ownership.
    pub fn or<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxPredicate {
            function: Box::new(move |value: &T| (self.function)(value) || other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical negation of this
    /// predicate.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> BoxPredicate<T> {
        BoxPredicate {
            function: Box::new(move |value: &T| !(self.function)(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NAND (NOT AND) of this
    /// predicate and another.
    ///
    /// NAND returns `true` unless both predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - Another `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical NAND.
    ///
    /// # Examples
    ///
    /// ## Combining with closures
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// // Note: is_positive is moved here, so it's no longer usable
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // positive but odd: !(true && false) = true
    /// assert!(nand.test(&-2));  // even but negative: !(false && true) = true
    /// assert!(!nand.test(&4));  // positive and even: !(true && true) = false
    /// // is_positive.test(&5); // This would not compile - is_positive was moved
    /// ```
    ///
    /// ## Combining with function pointers
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// fn is_even(x: &i32) -> bool { x % 2 == 0 }
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let nand = is_positive.nand(is_even);
    ///
    /// assert!(nand.test(&3));
    /// assert!(!nand.test(&4));
    /// ```
    ///
    /// ## Combining with other BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // returns true when only one condition is met
    /// assert!(!nand.test(&4));  // returns false when both conditions are met
    /// ```
    ///
    /// ## Note on ownership
    ///
    /// `BoxPredicate` uses single ownership semantics. If you need to reuse
    /// predicates after composition, consider using `RcPredicate` instead,
    /// which supports cloning and shared ownership.
    pub fn nand<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxPredicate {
            function: Box::new(move |value: &T| !((self.function)(value) && other.test(value))),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical XOR (exclusive OR) of
    /// this predicate and another.
    ///
    /// XOR returns `true` if exactly one of the predicates is `true`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - Another `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical XOR.
    ///
    /// # Examples
    ///
    /// ## Combining with closures
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// // Note: is_positive is moved here, so it's no longer usable
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // positive but odd: true ^ false = true
    /// assert!(xor.test(&-2));   // even but negative: false ^ true = true
    /// assert!(!xor.test(&4));   // positive and even: true ^ true = false
    /// assert!(!xor.test(&-1));  // negative and odd: false ^ false = false
    /// // is_positive.test(&5); // This would not compile - is_positive was moved
    /// ```
    ///
    /// ## Combining with function pointers
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// fn is_even(x: &i32) -> bool { x % 2 == 0 }
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let xor = is_positive.xor(is_even);
    ///
    /// assert!(xor.test(&3));
    /// assert!(!xor.test(&4));
    /// ```
    ///
    /// ## Combining with other BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // returns true when only one condition is met
    /// assert!(!xor.test(&4));   // returns false when both conditions are met
    /// assert!(!xor.test(&-1));  // returns false when neither condition is met
    /// ```
    ///
    /// ## Note on ownership
    ///
    /// `BoxPredicate` uses single ownership semantics. If you need to reuse
    /// predicates after composition, consider using `RcPredicate` instead,
    /// which supports cloning and shared ownership.
    pub fn xor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxPredicate {
            function: Box::new(move |value: &T| (self.function)(value) ^ other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NOR (NOT OR) of this
    /// predicate and another.
    ///
    /// NOR returns `true` only when both predicates are `false`. Equivalent
    /// to `!(self OR other)`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - Another `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxPredicate` representing the logical NOR.
    ///
    /// # Examples
    ///
    /// ## Combining with closures
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// // Note: is_positive is moved here, so it's no longer usable
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // Neither positive nor even:
    ///                           // !(false || false) = true
    /// assert!(!nor.test(&4));   // Both positive and even:
    ///                           // !(true || true) = false
    /// assert!(!nor.test(&3));   // Positive but not even:
    ///                           // !(true || false) = false
    /// assert!(!nor.test(&-2));  // Even but not positive:
    ///                           // !(false || true) = false
    /// // is_positive.test(&5); // This would not compile - is_positive was moved
    /// ```
    ///
    /// ## Combining with function pointers
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// fn is_even(x: &i32) -> bool { x % 2 == 0 }
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let nor = is_positive.nor(is_even);
    ///
    /// assert!(nor.test(&-3));
    /// assert!(!nor.test(&4));
    /// ```
    ///
    /// ## Combining with other BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // Returns true only when both are false
    /// assert!(!nor.test(&4));   // Returns false when at least one is true
    /// ```
    ///
    /// ## Note on ownership
    ///
    /// `BoxPredicate` uses single ownership semantics. If you need to reuse
    /// predicates after composition, consider using `RcPredicate` instead,
    /// which supports cloning and shared ownership.
    pub fn nor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxPredicate {
            function: Box::new(move |value: &T| !((self.function)(value) || other.test(value))),
            name: None,
        }
    }
}

impl<T: 'static> Predicate<T> for BoxPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }

    fn into_box(self) -> BoxPredicate<T> {
        self
    }

    fn into_rc(self) -> RcPredicate<T> {
        RcPredicate {
            function: Rc::from(self.function),
            name: self.name,
        }
    }

    // do NOT override Predicate::into_arc() because BoxPredicate is not Send + Sync
    // and calling BoxPredicate::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T) -> bool {
        move |value: &T| (self.function)(value)
    }

    // do NOT override Predicate::to_xxx() because BoxPredicate is not Clone
    // and calling BoxPredicate::to_xxx() will cause a compile error
}

impl<T> Display for BoxPredicate<T> {
    /// Implements Display trait for BoxPredicate
    ///
    /// Shows the predicate name if available, or "unnamed" as default.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BoxPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> Debug for BoxPredicate<T> {
    /// Implements Debug trait for BoxPredicate
    ///
    /// Shows the predicate name in debug struct format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxPredicate")
            .field("name", &self.name)
            .finish()
    }
}

/// An Rc-based predicate with single-threaded shared ownership.
///
/// This type is suitable for scenarios where the predicate needs to be
/// reused in a single-threaded context. Composition methods borrow `&self`,
/// allowing the original predicate to remain usable after composition.
///
/// # Examples
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Original predicate remains usable after composition
/// let combined = pred.and(RcPredicate::new(|x| x % 2 == 0));
/// assert!(pred.test(&5));  // Still works
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcPredicate<T> {
    function: Rc<dyn Fn(&T) -> bool>,
    name: Option<String>,
}

impl<T: 'static> RcPredicate<T> {
    /// Creates a new `RcPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            function: Rc::new(f),
            name: None,
        }
    }

    /// Creates a named `RcPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `RcPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            function: Rc::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred: RcPredicate<i32> = RcPredicate::always_true();
    /// assert!(pred.test(&42));
    /// assert!(pred.test(&-1));
    /// assert!(pred.test(&0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Rc::new(|_| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred: RcPredicate<i32> = RcPredicate::always_false();
    /// assert!(!pred.test(&42));
    /// assert!(!pred.test(&-1));
    /// assert!(!pred.test(&0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Rc::new(|_| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a predicate that represents the logical AND of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - Another `RcPredicate<T>` (will be moved)
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical AND.
    ///
    /// # Examples
    ///
    /// ## Combining with closures (original predicate remains usable)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// // Note: is_positive is borrowed (&self), so it remains usable
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));
    /// assert!(!combined.test(&3));
    ///
    /// // original predicate remains usable because RcPredicate uses &self
    /// assert!(is_positive.test(&5));
    /// ```
    ///
    /// ## Combining with other RcPredicate (requires clone)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// // Note: is_even parameter is passed by value and will be moved
    /// // If you need to continue using is_even, you should clone it
    /// let combined = is_positive.and(is_even.clone());
    /// assert!(combined.test(&4));
    ///
    /// // both original predicates remain usable
    /// assert!(is_positive.test(&5));
    /// assert!(is_even.test(&6));
    /// ```
    ///
    /// ## Reusing the same predicate multiple times
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Note: is_positive is borrowed (&self), so it can be reused
    /// let positive_and_even = is_positive.and(|x: &i32| x % 2 == 0);
    /// let positive_and_small = is_positive.and(|x: &i32| *x < 100);
    ///
    /// // is_positive can be combined multiple times because RcPredicate uses &self
    /// assert!(positive_and_even.test(&4));
    /// assert!(positive_and_small.test(&5));
    /// assert!(is_positive.test(&10));
    /// ```
    pub fn and<P>(&self, other: P) -> RcPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value) && other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical OR of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - Another `RcPredicate<T>` (will be moved)
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical OR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_negative = RcPredicate::new(|x: &i32| *x < 0);
    /// let is_large = |x: &i32| *x > 100;
    ///
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));
    /// assert!(combined.test(&150));
    /// assert!(!combined.test(&50));
    ///
    /// // original predicate remains usable
    /// assert!(is_negative.test(&-10));
    /// ```
    pub fn or<P>(&self, other: P) -> RcPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value) || other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical negation of this
    /// predicate.
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> RcPredicate<T> {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| !self_fn(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NAND (NOT AND) of this
    /// predicate and another.
    ///
    /// NAND returns `true` unless both predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - Another `RcPredicate<T>` (will be moved)
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical NAND.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // !(true && false) = true
    /// assert!(!nand.test(&4));  // !(true && true) = false
    ///
    /// // original predicate remains usable
    /// assert!(is_positive.test(&5));
    /// ```
    pub fn nand<P>(&self, other: P) -> RcPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| !(self_fn(value) && other.test(value))),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical XOR (exclusive OR) of
    /// this predicate and another.
    ///
    /// XOR returns `true` if exactly one of the predicates is `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - Another `RcPredicate<T>` (will be moved)
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical XOR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // true ^ false = true
    /// assert!(!xor.test(&4));   // true ^ true = false
    /// assert!(!xor.test(&-1));  // false ^ false = false
    ///
    /// // original predicate remains usable
    /// assert!(is_positive.test(&5));
    /// ```
    pub fn xor<P>(&self, other: P) -> RcPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value) ^ other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NOR (NOT OR) of this
    /// predicate and another.
    ///
    /// NOR returns `true` only when both predicates are `false`. Equivalent
    /// to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - Another `RcPredicate<T>` (will be moved)
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A new `RcPredicate` representing the logical NOR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // !(false || false) = true
    /// assert!(!nor.test(&4));   // !(true || true) = false
    /// assert!(!nor.test(&3));   // !(true || false) = false
    ///
    /// // Original predicate remains usable
    /// assert!(is_positive.test(&5));
    /// ```
    pub fn nor<P>(&self, other: P) -> RcPredicate<T>
    where
        P: Predicate<T> + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| !(self_fn(value) || other.test(value))),
            name: None,
        }
    }
}

impl<T: 'static> Predicate<T> for RcPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }

    fn into_box(self) -> BoxPredicate<T> {
        let self_fn = self.function;
        BoxPredicate {
            function: Box::new(move |value: &T| self_fn(value)),
            name: self.name,
        }
    }

    fn into_rc(self) -> RcPredicate<T> {
        self
    }

    // do NOT override Predicate::into_arc() because RcPredicate is not Send + Sync
    // and calling RcPredicate::into_arc() will cause a compile error

    fn into_fn(self) -> impl Fn(&T) -> bool {
        let self_fn = self.function;
        move |value: &T| self_fn(value)
    }

    fn to_box(&self) -> BoxPredicate<T> {
        let self_fn = self.function.clone();
        BoxPredicate {
            function: Box::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    fn to_rc(&self) -> RcPredicate<T> {
        self.clone()
    }

    // do NOT override Predicate::to_arc() because RcPredicate is not Send + Sync
    // and calling RcPredicate::to_arc() will cause a compile error

    fn to_fn(&self) -> impl Fn(&T) -> bool {
        let self_fn = self.function.clone();
        move |value: &T| self_fn(value)
    }
}

impl<T> Clone for RcPredicate<T> {
    /// Clones this predicate.
    ///
    /// Creates a new instance that shares the underlying function with the
    /// original, allowing multiple references to the same predicate logic.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> Display for RcPredicate<T> {
    /// Implements Display trait for RcPredicate
    ///
    /// Shows the predicate name if available, or "unnamed" as default.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RcPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> Debug for RcPredicate<T> {
    /// Implements Debug trait for RcPredicate
    ///
    /// Shows the predicate name in debug struct format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcPredicate")
            .field("name", &self.name)
            .finish()
    }
}

/// An Arc-based predicate with thread-safe shared ownership.
///
/// This type is suitable for scenarios where the predicate needs to be
/// shared across threads. Composition methods borrow `&self`, allowing the
/// original predicate to remain usable after composition.
///
/// # Examples
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let pred = ArcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Original predicate remains usable after composition
/// let combined = pred.and(ArcPredicate::new(|x| x % 2 == 0));
/// assert!(pred.test(&5));  // Still works
///
/// // Can be cloned and sent across threads
/// let pred_clone = pred.clone();
/// std::thread::spawn(move || {
///     assert!(pred_clone.test(&10));
/// }).join().unwrap();
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcPredicate<T> {
    function: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    name: Option<String>,
}

impl<T: 'static> ArcPredicate<T> {
    /// Creates a new `ArcPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(f),
            name: None,
        }
    }

    /// Creates a named `ArcPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `ArcPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred: ArcPredicate<i32> = ArcPredicate::always_true();
    /// assert!(pred.test(&42));
    /// assert!(pred.test(&-1));
    /// assert!(pred.test(&0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Arc::new(|_| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred: ArcPredicate<i32> = ArcPredicate::always_false();
    /// assert!(!pred.test(&42));
    /// assert!(!pred.test(&-1));
    /// assert!(!pred.test(&0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Arc::new(|_| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a predicate that represents the logical AND of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - Another `ArcPredicate<T>` (will be moved)
    ///   - Any type implementing `Predicate<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical AND.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    /// use std::thread;
    ///
    /// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let combined = is_positive.and(is_even);
    ///
    /// // can be used across threads
    /// let handle = thread::spawn(move || {
    ///     combined.test(&4)
    /// });
    ///
    /// assert!(handle.join().unwrap());
    /// assert!(is_positive.test(&5)); // original predicate still usable
    /// ```
    pub fn and<P>(&self, other: P) -> ArcPredicate<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| self_fn(value) && other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical OR of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - Another `ArcPredicate<T>` (will be moved)
    ///   - Any type implementing `Predicate<T> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical OR. Thread-safe.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let is_negative = ArcPredicate::new(|x: &i32| *x < 0);
    /// let is_large = |x: &i32| *x > 100;
    ///
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));
    /// assert!(combined.test(&150));
    /// assert!(is_negative.test(&-10)); // original predicate still usable
    /// ```
    pub fn or<P>(&self, other: P) -> ArcPredicate<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| self_fn(value) || other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical negation of this
    /// predicate.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> ArcPredicate<T> {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| !self_fn(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NAND (NOT AND) of this
    /// predicate and another.
    ///
    /// NAND returns `true` unless both predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Predicate<T> + Send + Sync` implementation.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical NAND. Thread-safe.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // !(true && false) = true
    /// assert!(!nand.test(&4));  // !(true && true) = false
    /// ```
    pub fn nand<P>(&self, other: P) -> ArcPredicate<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| !(self_fn(value) && other.test(value))),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical XOR (exclusive OR) of
    /// this predicate and another.
    ///
    /// XOR returns `true` if exactly one of the predicates is `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`).
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical XOR.
    pub fn xor<P>(&self, other: P) -> ArcPredicate<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| self_fn(value) ^ other.test(value)),
            name: None,
        }
    }

    /// Returns a predicate that represents the logical NOR (NOT OR) of this
    /// predicate and another.
    ///
    /// NOR returns `true` only when both predicates are `false`. Equivalent
    /// to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Predicate<T> + Send + Sync` implementation.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical NOR. Thread-safe.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // !(false || false) = true
    /// assert!(!nor.test(&4));   // !(true || true) = false
    /// assert!(!nor.test(&3));   // !(true || false) = false
    /// ```
    pub fn nor<P>(&self, other: P) -> ArcPredicate<T>
    where
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| !(self_fn(value) || other.test(value))),
            name: None,
        }
    }
}

impl<T: 'static> Predicate<T> for ArcPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }

    fn into_box(self) -> BoxPredicate<T> {
        BoxPredicate {
            function: Box::new(move |value: &T| (self.function)(value)),
            name: self.name,
        }
    }

    fn into_rc(self) -> RcPredicate<T> {
        RcPredicate {
            function: Rc::new(move |value: &T| (self.function)(value)),
            name: self.name,
        }
    }

    fn into_arc(self) -> ArcPredicate<T> {
        self
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        move |value: &T| (self.function)(value)
    }

    fn to_box(&self) -> BoxPredicate<T> {
        let self_fn = self.function.clone();
        BoxPredicate {
            function: Box::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    fn to_rc(&self) -> RcPredicate<T> {
        let self_fn = self.function.clone();
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    fn to_arc(&self) -> ArcPredicate<T> {
        self.clone()
    }

    fn to_fn(&self) -> impl Fn(&T) -> bool {
        let self_fn = self.function.clone();
        move |value: &T| self_fn(value)
    }
}

impl<T> Clone for ArcPredicate<T> {
    /// Clones this predicate.
    ///
    /// Creates a new instance that shares the underlying function with the
    /// original, allowing multiple references to the same predicate logic.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> Display for ArcPredicate<T> {
    /// Implements Display trait for ArcPredicate
    ///
    /// Shows the predicate name if available, or "unnamed" as default.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ArcPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> Debug for ArcPredicate<T> {
    /// Implements Debug trait for ArcPredicate
    ///
    /// Shows the predicate name in debug struct format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcPredicate")
            .field("name", &self.name)
            .finish()
    }
}

// Blanket implementation for all closures that match Fn(&T) -> bool
impl<T: 'static, F> Predicate<T> for F
where
    F: Fn(&T) -> bool + 'static,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }

    fn into_box(self) -> BoxPredicate<T> {
        BoxPredicate::new(self)
    }

    fn into_rc(self) -> RcPredicate<T> {
        RcPredicate::new(self)
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Send + Sync,
    {
        ArcPredicate::new(self)
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        self
    }

    fn to_box(&self) -> BoxPredicate<T>
    where
        Self: Clone + 'static,
    {
        let self_fn = self.clone();
        BoxPredicate::new(self_fn)
    }

    fn to_rc(&self) -> RcPredicate<T>
    where
        Self: Clone + 'static,
    {
        let self_fn = self.clone();
        RcPredicate::new(self_fn)
    }

    fn to_arc(&self) -> ArcPredicate<T>
    where
        Self: Clone + Send + Sync,
    {
        let self_fn = self.clone();
        ArcPredicate::new(self_fn)
    }

    fn to_fn(&self) -> impl Fn(&T) -> bool
    where
        Self: Clone + 'static,
    {
        self.clone()
    }
}

/// Extension trait providing logical composition methods for closures.
///
/// This trait is automatically implemented for all closures and function
/// pointers that match `Fn(&T) -> bool`, enabling method chaining starting
/// from a closure.
///
/// # Examples
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps};
///
/// let is_positive = |x: &i32| *x > 0;
/// let is_even = |x: &i32| x % 2 == 0;
///
/// // Combine predicates using extension methods
/// let pred = is_positive.and(is_even);
/// assert!(pred.test(&4));
/// assert!(!pred.test(&3));
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnPredicateOps<T>: Fn(&T) -> bool + Sized + 'static {
    /// Returns a predicate that represents the logical AND of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure
    ///   - A function pointer
    ///   - A `BoxPredicate<T>`, `RcPredicate<T>`, or `ArcPredicate<T>`
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical AND.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let is_positive = |x: &i32| *x > 0;
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));
    /// assert!(!combined.test(&3));
    /// ```
    fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| self.test(value) && other.test(value))
    }

    /// Returns a predicate that represents the logical OR of this predicate
    /// and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure
    ///   - A function pointer
    ///   - A `BoxPredicate<T>`, `RcPredicate<T>`, or `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical OR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let is_negative = |x: &i32| *x < 0;
    /// let is_large = |x: &i32| *x > 100;
    ///
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));
    /// assert!(combined.test(&150));
    /// assert!(!combined.test(&50));
    /// ```
    fn or<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| self.test(value) || other.test(value))
    }

    /// Returns a predicate that represents the logical negation of this
    /// predicate.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical negation.
    fn not(self) -> BoxPredicate<T>
    where
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| !self.test(value))
    }

    /// Returns a predicate that represents the logical NAND (NOT AND) of this
    /// predicate and another.
    ///
    /// NAND returns `true` unless both predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Predicate<T>` implementation.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical NAND.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let is_positive = |x: &i32| *x > 0;
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // !(true && false) = true
    /// assert!(!nand.test(&4));  // !(true && true) = false
    /// ```
    fn nand<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| !(self.test(value) && other.test(value)))
    }

    /// Returns a predicate that represents the logical XOR (exclusive OR) of
    /// this predicate and another.
    ///
    /// XOR returns `true` if exactly one of the predicates is `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Predicate<T>` implementation.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical XOR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let is_positive = |x: &i32| *x > 0;
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // true ^ false = true
    /// assert!(!xor.test(&4));   // true ^ true = false
    /// assert!(!xor.test(&-1));  // false ^ false = false
    /// ```
    fn xor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| self.test(value) ^ other.test(value))
    }

    /// Returns a predicate that represents the logical NOR (NOT OR) of this
    /// predicate and another.
    ///
    /// NOR returns `true` only when both predicates are `false`. Equivalent
    /// to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original predicate, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Predicate<T>` implementation.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` representing the logical NOR.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let is_positive = |x: &i32| *x > 0;
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // !(false || false) = true
    /// assert!(!nor.test(&4));   // !(true || true) = false
    /// assert!(!nor.test(&3));   // !(true || false) = false
    /// ```
    fn nor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |value: &T| !(self.test(value) || other.test(value)))
    }
}

// Blanket implementation for all closures
impl<T, F> FnPredicateOps<T> for F where F: Fn(&T) -> bool + 'static {}
