/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiPredicate Abstraction
//!
//! Provides a Rust implementation similar to Java's `BiPredicate`
//! interface for testing whether two values satisfy a condition.
//!
//! ## Core Semantics
//!
//! A **BiPredicate** is fundamentally a pure judgment operation that
//! tests whether two values satisfy a specific condition. It should
//! be:
//!
//! - **Read-only**: Does not modify the tested values
//! - **Side-effect free**: Does not change external state (from the
//!   user's perspective)
//! - **Repeatable**: Same inputs should produce the same result
//! - **Deterministic**: Judgment logic should be predictable
//!
//! ## Design Philosophy
//!
//! This module follows the same principles as the `Predicate` module:
//!
//! 1. **Single Trait**: Only one `BiPredicate<T, U>` trait with
//!    `&self`, keeping the API simple and semantically clear
//! 2. **No BiPredicateMut**: All stateful scenarios use interior
//!    mutability (`RefCell`, `Cell`, `Mutex`) instead of `&mut self`
//! 3. **No BiPredicateOnce**: Violates bi-predicate semantics -
//!    judgments should be repeatable
//! 4. **Three Implementations**: `BoxBiPredicate`, `RcBiPredicate`,
//!    and `ArcBiPredicate` cover all ownership scenarios
//!
//! ## Type Selection Guide
//!
//! | Scenario | Recommended Type | Reason |
//! |----------|------------------|--------|
//! | One-time use | `BoxBiPredicate` | Single ownership, no overhead |
//! | Multi-threaded | `ArcBiPredicate` | Thread-safe, clonable |
//! | Single-threaded reuse | `RcBiPredicate` | Better performance |
//! | Stateful predicate | Any type + `RefCell`/`Cell`/`Mutex` | Interior mutability |
//!
//! ## Examples
//!
//! ### Basic Usage with Closures
//!
//! ```rust
//! use prism3_function::bi_predicate::BiPredicate;
//!
//! let is_sum_positive = |x: &i32, y: &i32| x + y > 0;
//! assert!(is_sum_positive.test(&5, &3));
//! assert!(!is_sum_positive.test(&-3, &-7));
//! ```
//!
//! ### BoxBiPredicate - Single Ownership
//!
//! ```rust
//! use prism3_function::bi_predicate::{BiPredicate, BoxBiPredicate};
//!
//! let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0)
//!     .and(BoxBiPredicate::new(|x, y| x > y));
//! assert!(pred.test(&10, &5));
//! ```
//!
//! ### Closure Composition with Extension Methods
//!
//! Closures automatically gain `and`, `or`, `not` methods through the
//! `FnBiPredicateOps` extension trait, returning `BoxBiPredicate`:
//!
//! ```rust
//! use prism3_function::bi_predicate::{BiPredicate,
//!     FnBiPredicateOps};
//!
//! // Compose closures directly - result is BoxBiPredicate
//! let is_sum_positive = |x: &i32, y: &i32| x + y > 0;
//! let first_larger = |x: &i32, y: &i32| x > y;
//!
//! let combined = is_sum_positive.and(first_larger);
//! assert!(combined.test(&10, &5));
//! assert!(!combined.test(&3, &8));
//!
//! // Use `or` for disjunction
//! let negative_sum = |x: &i32, y: &i32| x + y < 0;
//! let both_large = |x: &i32, y: &i32| *x > 100 && *y > 100;
//! let either = negative_sum.or(both_large);
//! assert!(either.test(&-10, &5));
//! assert!(either.test(&200, &150));
//! ```
//!
//! ### RcBiPredicate - Single-threaded Reuse
//!
//! ```rust
//! use prism3_function::bi_predicate::{BiPredicate, RcBiPredicate};
//!
//! let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
//! let combined1 = pred.and(RcBiPredicate::new(|x, y| x > y));
//! let combined2 = pred.or(RcBiPredicate::new(|x, y| *x > 100));
//!
//! // Original predicate is still usable
//! assert!(pred.test(&5, &3));
//! ```
//!
//! ### ArcBiPredicate - Thread-safe Sharing
//!
//! ```rust
//! use prism3_function::bi_predicate::{BiPredicate, ArcBiPredicate};
//! use std::thread;
//!
//! let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
//! let pred_clone = pred.clone();
//!
//! let handle = thread::spawn(move || {
//!     pred_clone.test(&10, &5)
//! });
//!
//! assert!(handle.join().unwrap());
//! assert!(pred.test(&3, &7));  // Original still usable
//! ```
//!
//! ### Stateful BiPredicates with Interior Mutability
//!
//! ```rust
//! use prism3_function::bi_predicate::{BiPredicate, BoxBiPredicate};
//! use std::cell::Cell;
//!
//! let count = Cell::new(0);
//! let pred = BoxBiPredicate::new(move |x: &i32, y: &i32| {
//!     count.set(count.get() + 1);
//!     x + y > 0
//! });
//!
//! // No need for `mut` - interior mutability handles state
//! assert!(pred.test(&5, &3));
//! assert!(!pred.test(&-8, &-3));
//! ```
//!
//! ## Author
//!
//! Haixing Hu

use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::Arc;

/// Type alias for bi-predicate function to simplify complex types.
///
/// This type alias represents a function that takes two references and returns a boolean.
/// It is used to reduce type complexity in struct definitions.
type BiPredicateFn<T, U> = dyn Fn(&T, &U) -> bool;

/// Type alias for thread-safe bi-predicate function to simplify complex types.
///
/// This type alias represents a function that takes two references and returns a boolean,
/// with Send + Sync bounds for thread-safe usage. It is used to reduce type complexity
/// in Arc-based struct definitions.
type SendSyncBiPredicateFn<T, U> = dyn Fn(&T, &U) -> bool + Send + Sync;

/// BiPredicate name constant for always-true bi-predicates
const ALWAYS_TRUE_NAME: &str = "always_true";

/// BiPredicate name constant for always-false bi-predicates
const ALWAYS_FALSE_NAME: &str = "always_false";

/// A bi-predicate trait for testing whether two values satisfy a
/// condition.
///
/// This trait represents a **pure judgment operation** - it tests
/// whether two given values meet certain criteria without modifying
/// either the values or the bi-predicate itself (from the user's
/// perspective). This semantic clarity distinguishes bi-predicates
/// from consumers or transformers.
///
/// ## Design Rationale
///
/// This is a **minimal trait** that only defines:
/// - The core `test` method using `&self` (immutable borrow)
/// - Type conversion methods (`into_box`, `into_rc`, `into_arc`)
/// - Closure conversion method (`into_fn`)
///
/// Logical composition methods (`and`, `or`, `not`, `xor`, `nand`,
/// `nor`) are intentionally **not** part of the trait. Instead, they
/// are implemented on concrete types (`BoxBiPredicate`,
/// `RcBiPredicate`, `ArcBiPredicate`), allowing each implementation
/// to maintain its specific ownership characteristics:
///
/// - `BoxBiPredicate`: Methods consume `self` (single ownership)
/// - `RcBiPredicate`: Methods borrow `&self` (shared ownership)
/// - `ArcBiPredicate`: Methods borrow `&self` (thread-safe shared
///   ownership)
///
/// ## Why `&self` Instead of `&mut self`?
///
/// Bi-predicates use `&self` because:
///
/// 1. **Semantic Clarity**: A bi-predicate is a judgment, not a
///    mutation
/// 2. **Flexibility**: Can be used in immutable contexts
/// 3. **Simplicity**: No need for `mut` in user code
/// 4. **Interior Mutability**: State (if needed) can be managed with
///    `RefCell`, `Cell`, or `Mutex`
///
/// ## Automatic Implementation for Closures
///
/// Any closure matching `Fn(&T, &U) -> bool` automatically implements
/// this trait, providing seamless integration with Rust's closure
/// system.
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```rust
/// use prism3_function::bi_predicate::BiPredicate;
///
/// let is_sum_positive = |x: &i32, y: &i32| x + y > 0;
/// assert!(is_sum_positive.test(&5, &3));
/// assert!(!is_sum_positive.test(&-5, &-3));
/// ```
///
/// ### Type Conversion
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate,
///     BoxBiPredicate};
///
/// let closure = |x: &i32, y: &i32| x + y > 0;
/// let boxed: BoxBiPredicate<i32, i32> = closure.into_box();
/// assert!(boxed.test(&5, &3));
/// ```
///
/// ### Stateful BiPredicate with Interior Mutability
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate,
///     BoxBiPredicate};
/// use std::cell::Cell;
///
/// let count = Cell::new(0);
/// let counting_pred = BoxBiPredicate::new(move |x: &i32, y: &i32| {
///     count.set(count.get() + 1);
///     x + y > 0
/// });
///
/// // Note: No `mut` needed - interior mutability handles state
/// assert!(counting_pred.test(&5, &3));
/// assert!(!counting_pred.test(&-5, &-3));
/// ```
///
/// ## Author
///
/// Haixing Hu
pub trait BiPredicate<T, U> {
    /// Tests whether the given values satisfy this bi-predicate.
    ///
    /// # Parameters
    ///
    /// * `first` - The first value to test.
    /// * `second` - The second value to test.
    ///
    /// # Returns
    ///
    /// `true` if the values satisfy this bi-predicate, `false`
    /// otherwise.
    fn test(&self, first: &T, second: &U) -> bool;

    /// Converts this bi-predicate into a `BoxBiPredicate`.
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` wrapping this bi-predicate.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps the bi-predicate in a
    /// closure that calls `test`, providing automatic conversion
    /// for custom types that only implement the core `test`
    /// method.
    fn into_box(self) -> BoxBiPredicate<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first: &T, second: &U| self.test(first, second))
    }

    /// Converts this bi-predicate into an `RcBiPredicate`.
    ///
    /// # Returns
    ///
    /// An `RcBiPredicate` wrapping this bi-predicate.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps the bi-predicate in a
    /// closure that calls `test`, providing automatic conversion
    /// for custom types that only implement the core `test`
    /// method.
    fn into_rc(self) -> RcBiPredicate<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcBiPredicate::new(move |first: &T, second: &U| self.test(first, second))
    }

    /// Converts this bi-predicate into an `ArcBiPredicate`.
    ///
    /// # Returns
    ///
    /// An `ArcBiPredicate` wrapping this bi-predicate.
    ///
    /// # Default Implementation
    ///
    /// The default implementation wraps the bi-predicate in a
    /// closure that calls `test`, providing automatic conversion
    /// for custom types that only implement the core `test`
    /// method. Note that this requires `Send + Sync` bounds for
    /// thread-safe sharing.
    fn into_arc(self) -> ArcBiPredicate<T, U>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        ArcBiPredicate::new(move |first: &T, second: &U| self.test(first, second))
    }

    /// Converts this bi-predicate into a closure that can be used
    /// directly with standard library methods.
    ///
    /// This method consumes the bi-predicate and returns a closure
    /// with signature `Fn(&T, &U) -> bool`. Since `Fn` is a subtrait
    /// of `FnMut`, the returned closure can be used in any context
    /// that requires either `Fn(&T, &U) -> bool` or
    /// `FnMut(&T, &U) -> bool`.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T, &U) -> bool` (also usable as
    /// `FnMut(&T, &U) -> bool`).
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns a closure that calls the
    /// `test` method, providing automatic conversion for custom
    /// types.
    ///
    /// # Examples
    ///
    /// ## Using with Iterator Methods
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate,
    ///     BoxBiPredicate};
    ///
    /// let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
    ///
    /// let pairs = vec![(1, 2), (-1, 3), (5, -6)];
    /// let mut closure = pred.into_fn();
    /// let positives: Vec<_> = pairs.iter()
    ///     .filter(|(x, y)| closure(x, y))
    ///     .collect();
    /// assert_eq!(positives, vec![&(1, 2), &(-1, 3)]);
    /// ```
    fn into_fn(self) -> impl Fn(&T, &U) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        move |first: &T, second: &U| self.test(first, second)
    }
}

/// A Box-based bi-predicate with single ownership.
///
/// This type is suitable for one-time use scenarios where the
/// bi-predicate does not need to be cloned or shared. Composition
/// methods consume `self`, reflecting the single-ownership model.
///
/// # Examples
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate, BoxBiPredicate};
///
/// let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
/// assert!(pred.test(&5, &3));
///
/// // Chaining consumes the bi-predicate
/// let combined = pred.and(BoxBiPredicate::new(|x, y| x > y));
/// assert!(combined.test(&10, &5));
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxBiPredicate<T, U> {
    function: Box<BiPredicateFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxBiPredicate<T, U> {
    /// Creates a new `BoxBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + 'static,
    {
        Self {
            function: Box::new(f),
            name: None,
        }
    }

    /// Creates a named `BoxBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this bi-predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `BoxBiPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + 'static,
    {
        Self {
            function: Box::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, BoxBiPredicate};
    ///
    /// let pred: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_true();
    /// assert!(pred.test(&42, &10));
    /// assert!(pred.test(&-1, &5));
    /// assert!(pred.test(&0, &0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Box::new(|_, _| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, BoxBiPredicate};
    ///
    /// let pred: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_false();
    /// assert!(!pred.test(&42, &10));
    /// assert!(!pred.test(&-1, &5));
    /// assert!(!pred.test(&0, &0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Box::new(|_, _| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this bi-predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the bi-predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this bi-predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this bi-predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a bi-predicate that represents the logical AND of this
    /// bi-predicate and another.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - Another `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical AND.
    pub fn and<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| {
                (self.function)(first, second) && other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical OR of this
    /// bi-predicate and another.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - Another `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical OR.
    pub fn or<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| {
                (self.function)(first, second) || other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical negation of
    /// this bi-predicate.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| !(self.function)(first, second)),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NAND (NOT
    /// AND) of this bi-predicate and another.
    ///
    /// NAND returns `true` unless both bi-predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - Another `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical NAND.
    pub fn nand<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| {
                !((self.function)(first, second) && other.test(first, second))
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical XOR
    /// (exclusive OR) of this bi-predicate and another.
    ///
    /// XOR returns `true` if exactly one of the bi-predicates is
    /// `true`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - Another `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical XOR.
    pub fn xor<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| {
                (self.function)(first, second) ^ other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NOR (NOT
    /// OR) of this bi-predicate and another.
    ///
    /// NOR returns `true` only if both bi-predicates are `false`.
    /// Equivalent to `!(self OR other)`.
    ///
    /// This method consumes `self` due to single-ownership semantics.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - Another `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `BoxBiPredicate` representing the logical NOR.
    pub fn nor<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| {
                !((self.function)(first, second) || other.test(first, second))
            }),
            name: None,
        }
    }
}

impl<T, U> BiPredicate<T, U> for BoxBiPredicate<T, U> {
    fn test(&self, first: &T, second: &U) -> bool {
        (self.function)(first, second)
    }

    // Use optimized zero-cost conversion for into_box
    fn into_box(self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    // Use optimized conversion for into_rc that preserves the
    // existing Box
    fn into_rc(self) -> RcBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        RcBiPredicate {
            function: Rc::from(self.function),
            name: self.name,
        }
    }

    // BoxBiPredicate cannot be converted to ArcBiPredicate
    // because Box<dyn Fn> is not Send + Sync
    fn into_arc(self) -> ArcBiPredicate<T, U>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        panic!("BoxBiPredicate cannot be converted to ArcBiPredicate - use ArcBiPredicate::new directly")
    }

    // Use optimized conversion for into_fn that preserves the
    // existing Box
    fn into_fn(self) -> impl Fn(&T, &U) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        move |first: &T, second: &U| (self.function)(first, second)
    }
}

impl<T, U> Display for BoxBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "BoxBiPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T, U> Debug for BoxBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BoxBiPredicate")
            .field("name", &self.name)
            .finish()
    }
}

/// An Rc-based bi-predicate with single-threaded shared ownership.
///
/// This type is suitable for scenarios where the bi-predicate needs
/// to be reused in a single-threaded context. Composition methods
/// borrow `&self`, allowing the original bi-predicate to remain
/// usable after composition.
///
/// # Examples
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate, RcBiPredicate};
///
/// let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
/// assert!(pred.test(&5, &3));
///
/// // Original bi-predicate remains usable after composition
/// let combined = pred.and(RcBiPredicate::new(|x, y| x > y));
/// assert!(pred.test(&5, &3));  // Still works
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcBiPredicate<T, U> {
    function: Rc<BiPredicateFn<T, U>>,
    name: Option<String>,
}

impl<T, U> RcBiPredicate<T, U> {
    /// Creates a new `RcBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + 'static,
    {
        Self {
            function: Rc::new(f),
            name: None,
        }
    }

    /// Creates a named `RcBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this bi-predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `RcBiPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + 'static,
    {
        Self {
            function: Rc::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, RcBiPredicate};
    ///
    /// let pred: RcBiPredicate<i32, i32> = RcBiPredicate::always_true();
    /// assert!(pred.test(&42, &10));
    /// assert!(pred.test(&-1, &5));
    /// assert!(pred.test(&0, &0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Rc::new(|_, _| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, RcBiPredicate};
    ///
    /// let pred: RcBiPredicate<i32, i32> = RcBiPredicate::always_false();
    /// assert!(!pred.test(&42, &10));
    /// assert!(!pred.test(&-1, &5));
    /// assert!(!pred.test(&0, &0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Rc::new(|_, _| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this bi-predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the bi-predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this bi-predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this bi-predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a bi-predicate that represents the logical AND of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Another `RcBiPredicate<T, U>` (will be moved)
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical AND.
    pub fn and<P>(&self, other: P) -> RcBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| {
                self_fn(first, second) && other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical OR of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Another `RcBiPredicate<T, U>` (will be moved)
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical OR.
    pub fn or<P>(&self, other: P) -> RcBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| {
                self_fn(first, second) || other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical negation of
    /// this bi-predicate.
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> RcBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| !self_fn(first, second)),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NAND (NOT
    /// AND) of this bi-predicate and another.
    ///
    /// NAND returns `true` unless both bi-predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Another `RcBiPredicate<T, U>` (will be moved)
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical NAND.
    pub fn nand<P>(&self, other: P) -> RcBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| {
                !(self_fn(first, second) && other.test(first, second))
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical XOR
    /// (exclusive OR) of this bi-predicate and another.
    ///
    /// XOR returns `true` if exactly one of the bi-predicates is
    /// `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Another `RcBiPredicate<T, U>` (will be moved)
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical XOR.
    pub fn xor<P>(&self, other: P) -> RcBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| {
                self_fn(first, second) ^ other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NOR (NOT
    /// OR) of this bi-predicate and another.
    ///
    /// NOR returns `true` only if both bi-predicates are `false`.
    /// Equivalent to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Another `RcBiPredicate<T, U>` (will be moved)
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A new `RcBiPredicate` representing the logical NOR.
    pub fn nor<P>(&self, other: P) -> RcBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| {
                !(self_fn(first, second) || other.test(first, second))
            }),
            name: None,
        }
    }

    /// Converts this bi-predicate to a `BoxBiPredicate`.
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` wrapping this bi-predicate.
    pub fn to_box(&self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| self_fn(first, second)),
            name: self.name.clone(),
        }
    }

    /// Converts this bi-predicate to a closure that can be used
    /// directly with standard library methods.
    ///
    /// This method creates a new closure without consuming the
    /// original bi-predicate, since `RcBiPredicate` uses shared
    /// ownership. The returned closure has signature
    /// `Fn(&T, &U) -> bool`.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T, &U) -> bool`.
    pub fn to_fn(&self) -> impl Fn(&T, &U) -> bool
    where
        T: 'static,
        U: 'static,
    {
        let function = Rc::clone(&self.function);
        move |first: &T, second: &U| function(first, second)
    }
}

impl<T, U> BiPredicate<T, U> for RcBiPredicate<T, U> {
    fn test(&self, first: &T, second: &U) -> bool {
        (self.function)(first, second)
    }

    // Use optimized conversion for into_box that preserves the
    // existing Rc
    fn into_box(self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| self_fn(first, second)),
            name: self.name,
        }
    }

    // Use optimized zero-cost conversion for into_rc
    fn into_rc(self) -> RcBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    // RcBiPredicate cannot be converted to ArcBiPredicate because
    // Rc is not Send + Sync
    fn into_arc(self) -> ArcBiPredicate<T, U>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        panic!("RcBiPredicate cannot be converted to ArcBiPredicate - use ArcBiPredicate::new directly")
    }

    // Use optimized conversion for into_fn that preserves the
    // existing Rc
    fn into_fn(self) -> impl Fn(&T, &U) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        move |first: &T, second: &U| self_fn(first, second)
    }
}

impl<T, U> Clone for RcBiPredicate<T, U> {
    /// Clones this bi-predicate.
    ///
    /// Creates a new instance that shares the underlying function with
    /// the original, allowing multiple references to the same
    /// bi-predicate logic.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> Display for RcBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RcBiPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T, U> Debug for RcBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("RcBiPredicate")
            .field("name", &self.name)
            .finish()
    }
}

/// An Arc-based bi-predicate with thread-safe shared ownership.
///
/// This type is suitable for scenarios where the bi-predicate needs
/// to be shared across threads. Composition methods borrow `&self`,
/// allowing the original bi-predicate to remain usable after
/// composition.
///
/// # Examples
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate, ArcBiPredicate};
///
/// let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
/// assert!(pred.test(&5, &3));
///
/// // Original bi-predicate remains usable after composition
/// let combined = pred.and(ArcBiPredicate::new(|x, y| x > y));
/// assert!(pred.test(&5, &3));  // Still works
///
/// // Can be cloned and sent across threads
/// let pred_clone = pred.clone();
/// std::thread::spawn(move || {
///     assert!(pred_clone.test(&10, &5));
/// }).join().unwrap();
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcBiPredicate<T, U> {
    function: Arc<SendSyncBiPredicateFn<T, U>>,
    name: Option<String>,
}

impl<T, U> ArcBiPredicate<T, U> {
    /// Creates a new `ArcBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(f),
            name: None,
        }
    }

    /// Creates a named `ArcBiPredicate` from a closure.
    ///
    /// # Parameters
    ///
    /// * `name` - The name for this bi-predicate.
    /// * `f` - The closure to wrap.
    ///
    /// # Returns
    ///
    /// A new named `ArcBiPredicate` instance.
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: Fn(&T, &U) -> bool + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(f),
            name: Some(name.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `true`.
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` that always returns `true`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, ArcBiPredicate};
    ///
    /// let pred: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_true();
    /// assert!(pred.test(&42, &10));
    /// assert!(pred.test(&-1, &5));
    /// assert!(pred.test(&0, &0));
    /// ```
    pub fn always_true() -> Self {
        Self {
            function: Arc::new(|_, _| true),
            name: Some(ALWAYS_TRUE_NAME.to_string()),
        }
    }

    /// Creates a bi-predicate that always returns `false`.
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` that always returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::bi_predicate::{BiPredicate, ArcBiPredicate};
    ///
    /// let pred: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_false();
    /// assert!(!pred.test(&42, &10));
    /// assert!(!pred.test(&-1, &5));
    /// assert!(!pred.test(&0, &0));
    /// ```
    pub fn always_false() -> Self {
        Self {
            function: Arc::new(|_, _| false),
            name: Some(ALWAYS_FALSE_NAME.to_string()),
        }
    }

    /// Returns the name of this bi-predicate, if set.
    ///
    /// # Returns
    ///
    /// An `Option` containing the bi-predicate's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this bi-predicate.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for this bi-predicate.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    /// Returns a bi-predicate that represents the logical AND of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - Another `ArcBiPredicate<T, U>` (will be moved)
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical AND.
    pub fn and<P>(&self, other: P) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| {
                self_fn(first, second) && other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical OR of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - Another `ArcBiPredicate<T, U>` (will be moved)
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical OR.
    /// Thread-safe.
    pub fn or<P>(&self, other: P) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| {
                self_fn(first, second) || other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical negation of
    /// this bi-predicate.
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical negation.
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| !self_fn(first, second)),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NAND (NOT
    /// AND) of this bi-predicate and another.
    ///
    /// NAND returns `true` unless both bi-predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - Another `ArcBiPredicate<T, U>` (will be moved)
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical NAND.
    /// Thread-safe.
    pub fn nand<P>(&self, other: P) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| {
                !(self_fn(first, second) && other.test(first, second))
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical XOR
    /// (exclusive OR) of this bi-predicate and another.
    ///
    /// XOR returns `true` if exactly one of the bi-predicates is
    /// `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - Another `ArcBiPredicate<T, U>` (will be moved)
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical XOR.
    pub fn xor<P>(&self, other: P) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| {
                self_fn(first, second) ^ other.test(first, second)
            }),
            name: None,
        }
    }

    /// Returns a bi-predicate that represents the logical NOR (NOT
    /// OR) of this bi-predicate and another.
    ///
    /// NOR returns `true` only if both bi-predicates are `false`.
    /// Equivalent to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - Another `ArcBiPredicate<T, U>` (will be moved)
    ///   - Any type implementing `BiPredicate<T, U> + Send + Sync`
    ///
    /// # Returns
    ///
    /// A new `ArcBiPredicate` representing the logical NOR.
    /// Thread-safe.
    pub fn nor<P>(&self, other: P) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
        P: BiPredicate<T, U> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcBiPredicate {
            function: Arc::new(move |first: &T, second: &U| {
                !(self_fn(first, second) || other.test(first, second))
            }),
            name: None,
        }
    }

    /// Converts this bi-predicate to a `BoxBiPredicate`.
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` wrapping this bi-predicate.
    pub fn to_box(&self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = Arc::clone(&self.function);
        BoxBiPredicate {
            function: Box::new(move |first: &T, second: &U| self_fn(first, second)),
            name: self.name.clone(),
        }
    }

    /// Converts this bi-predicate to an `RcBiPredicate`.
    ///
    /// # Returns
    ///
    /// An `RcBiPredicate` wrapping this bi-predicate.
    pub fn to_rc(&self) -> RcBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let self_fn = Arc::clone(&self.function);
        RcBiPredicate {
            function: Rc::new(move |first: &T, second: &U| self_fn(first, second)),
            name: self.name.clone(),
        }
    }

    /// Converts this bi-predicate to a closure that can be used
    /// directly with standard library methods.
    ///
    /// This method creates a new closure without consuming the
    /// original bi-predicate, since `ArcBiPredicate` uses shared
    /// ownership. The returned closure has signature
    /// `Fn(&T, &U) -> bool + Send + Sync` and is thread-safe.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T, &U) -> bool + Send + Sync`.
    pub fn to_fn(&self) -> impl Fn(&T, &U) -> bool + Send + Sync
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        move |first: &T, second: &U| self_fn(first, second)
    }
}

impl<T, U> BiPredicate<T, U> for ArcBiPredicate<T, U> {
    fn test(&self, first: &T, second: &U) -> bool {
        (self.function)(first, second)
    }

    // Use optimized conversion for into_box that preserves the
    // existing Arc
    fn into_box(self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let name = self.name.clone();
        BoxBiPredicate {
            function: Box::new(move |first, second| self.test(first, second)),
            name,
        }
    }

    // Use optimized conversion for into_rc that preserves the
    // existing Arc
    fn into_rc(self) -> RcBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let name = self.name.clone();
        RcBiPredicate {
            function: Rc::new(move |first, second| self.test(first, second)),
            name,
        }
    }

    // Use optimized zero-cost conversion for into_arc
    fn into_arc(self) -> ArcBiPredicate<T, U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        self
    }

    // Use optimized conversion for into_fn that preserves the
    // existing Arc
    fn into_fn(self) -> impl Fn(&T, &U) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        let self_fn = self.function;
        move |first: &T, second: &U| self_fn(first, second)
    }
}

impl<T, U> Clone for ArcBiPredicate<T, U> {
    /// Clones this bi-predicate.
    ///
    /// Creates a new instance that shares the underlying function with
    /// the original, allowing multiple references to the same
    /// bi-predicate logic.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> Display for ArcBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ArcBiPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T, U> Debug for ArcBiPredicate<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ArcBiPredicate")
            .field("name", &self.name)
            .finish()
    }
}

// Blanket implementation for all closures that match
// Fn(&T, &U) -> bool. This provides optimal implementations for
// closures by wrapping them directly into the target type.
impl<T: 'static, U: 'static, F> BiPredicate<T, U> for F
where
    F: Fn(&T, &U) -> bool + 'static,
{
    fn test(&self, first: &T, second: &U) -> bool {
        self(first, second)
    }

    // Optimal implementation for closures: wrap directly in Box
    fn into_box(self) -> BoxBiPredicate<T, U> {
        BoxBiPredicate::new(self)
    }

    // Optimal implementation for closures: wrap directly in Rc
    fn into_rc(self) -> RcBiPredicate<T, U> {
        RcBiPredicate::new(self)
    }

    // Optimal implementation for closures: wrap directly in Arc
    fn into_arc(self) -> ArcBiPredicate<T, U>
    where
        Self: Send + Sync,
        T: Send + Sync,
        U: Send + Sync,
    {
        ArcBiPredicate::new(self)
    }

    // Optimal implementation for closures: return self (zero-cost)
    fn into_fn(self) -> impl Fn(&T, &U) -> bool {
        self
    }
}

/// Extension trait providing logical composition methods for closures.
///
/// This trait is automatically implemented for all closures and
/// function pointers that match `Fn(&T, &U) -> bool`, enabling method
/// chaining starting from a closure.
///
/// # Examples
///
/// ```rust
/// use prism3_function::bi_predicate::{BiPredicate, FnBiPredicateOps};
///
/// let is_sum_positive = |x: &i32, y: &i32| x + y > 0;
/// let first_larger = |x: &i32, y: &i32| x > y;
///
/// // Combine bi-predicates using extension methods
/// let pred = is_sum_positive.and(first_larger);
/// assert!(pred.test(&10, &5));
/// assert!(!pred.test(&3, &8));
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnBiPredicateOps<T, U>: Fn(&T, &U) -> bool + Sized + 'static {
    /// Returns a bi-predicate that represents the logical AND of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical AND.
    fn and<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| self(first, second) && other.test(first, second))
    }

    /// Returns a bi-predicate that represents the logical OR of this
    /// bi-predicate and another.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical OR.
    fn or<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| self(first, second) || other.test(first, second))
    }

    /// Returns a bi-predicate that represents the logical negation of
    /// this bi-predicate.
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical negation.
    fn not(self) -> BoxBiPredicate<T, U>
    where
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| !self(first, second))
    }

    /// Returns a bi-predicate that represents the logical NAND (NOT
    /// AND) of this bi-predicate and another.
    ///
    /// NAND returns `true` unless both bi-predicates are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical NAND.
    fn nand<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| {
            !(self(first, second) && other.test(first, second))
        })
    }

    /// Returns a bi-predicate that represents the logical XOR
    /// (exclusive OR) of this bi-predicate and another.
    ///
    /// XOR returns `true` if exactly one of the bi-predicates is
    /// `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical XOR.
    fn xor<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| self(first, second) ^ other.test(first, second))
    }

    /// Returns a bi-predicate that represents the logical NOR (NOT
    /// OR) of this bi-predicate and another.
    ///
    /// NOR returns `true` only if both bi-predicates are `false`.
    /// Equivalent to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other bi-predicate to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original bi-predicate, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - An `RcBiPredicate<T, U>`
    ///   - An `ArcBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// A `BoxBiPredicate` representing the logical NOR.
    fn nor<P>(self, other: P) -> BoxBiPredicate<T, U>
    where
        P: BiPredicate<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiPredicate::new(move |first, second| {
            !(self(first, second) || other.test(first, second))
        })
    }
}

// Blanket implementation for all closures
impl<T, U, F> FnBiPredicateOps<T, U> for F where F: Fn(&T, &U) -> bool + 'static {}
