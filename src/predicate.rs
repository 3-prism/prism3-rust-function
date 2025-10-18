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

use std::fmt::{Debug, Display};
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
    /// # Returns
    ///
    /// A `BoxPredicate` wrapping this predicate.
    fn into_box(self) -> BoxPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts this predicate into an `RcPredicate`.
    ///
    /// # Returns
    ///
    /// An `RcPredicate` wrapping this predicate.
    fn into_rc(self) -> RcPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts this predicate into an `ArcPredicate`.
    ///
    /// # Returns
    ///
    /// An `ArcPredicate` wrapping this predicate.
    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static;

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
        T: 'static;
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// ## 与闭包组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));   // 正数且偶数
    /// assert!(!combined.test(&3));  // 正数但奇数
    /// assert!(!combined.test(&-2)); // 偶数但负数
    /// ```
    ///
    /// ## 与函数指针组合
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
    /// ## 与其他 BoxPredicate 组合
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
    /// ## 链式组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0)
    ///     .and(|x: &i32| x % 2 == 0)
    ///     .and(|x: &i32| *x < 100);
    ///
    /// assert!(pred.test(&42));  // 正数、偶数、小于100
    /// assert!(!pred.test(&101)); // 不满足小于100
    /// ```
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// ## 与闭包组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_negative = BoxPredicate::new(|x: &i32| *x < 0);
    /// let is_large = |x: &i32| *x > 100;
    ///
    /// let combined = is_negative.or(is_large);
    /// assert!(combined.test(&-5));  // 负数
    /// assert!(combined.test(&150)); // 大于100
    /// assert!(!combined.test(&50)); // 既不是负数也不大于100
    /// ```
    ///
    /// ## 与函数指针组合
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
    /// ## 与其他 BoxPredicate 组合
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// ## 与闭包组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // 正数但奇数: !(true && false) = true
    /// assert!(nand.test(&-2));  // 偶数但负数: !(false && true) = true
    /// assert!(!nand.test(&4));  // 正数且偶数: !(true && true) = false
    /// ```
    ///
    /// ## 与函数指针组合
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
    /// ## 与其他 BoxPredicate 组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let nand = is_positive.nand(is_even);
    /// assert!(nand.test(&3));   // 只满足一个条件时返回 true
    /// assert!(!nand.test(&4));  // 两个都满足时返回 false
    /// ```
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// ## 与闭包组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // 正数但奇数: true ^ false = true
    /// assert!(xor.test(&-2));   // 偶数但负数: false ^ true = true
    /// assert!(!xor.test(&4));   // 正数且偶数: true ^ true = false
    /// assert!(!xor.test(&-1));  // 负数且奇数: false ^ false = false
    /// ```
    ///
    /// ## 与函数指针组合
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
    /// ## 与其他 BoxPredicate 组合
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    /// let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// let xor = is_positive.xor(is_even);
    /// assert!(xor.test(&3));    // 只满足一个条件时返回 true
    /// assert!(!xor.test(&4));   // 两个都满足时返回 false
    /// assert!(!xor.test(&-1));  // 两个都不满足时返回 false
    /// ```
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// let nor = is_positive.nor(is_even);
    /// assert!(nor.test(&-3));   // Neither positive nor even:
    ///                           // !(false || false) = true
    /// assert!(!nor.test(&4));   // Both positive and even:
    ///                           // !(true || true) = false
    /// assert!(!nor.test(&3));   // Positive but not even:
    ///                           // !(true || false) = false
    /// assert!(!nor.test(&-2));  // Even but not positive:
    ///                           // !(false || true) = false
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

    fn into_arc(self) -> ArcPredicate<T>
    where
        T: Send + Sync,
    {
        // This is a best-effort conversion. If the underlying function
        // is not Send + Sync, this will fail at compile time.
        // Users should use ArcPredicate::new directly if they need
        // guaranteed Send + Sync.
        panic!("BoxPredicate cannot be converted to ArcPredicate - use ArcPredicate::new directly")
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        move |value: &T| (self.function)(value)
    }
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// ## 与闭包组合（原 predicate 可继续使用）
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = |x: &i32| x % 2 == 0;
    ///
    /// let combined = is_positive.and(is_even);
    /// assert!(combined.test(&4));
    /// assert!(!combined.test(&3));
    ///
    /// // 原 predicate 仍然可用
    /// assert!(is_positive.test(&5));
    /// ```
    ///
    /// ## 与其他 RcPredicate 组合（需要 clone）
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);
    ///
    /// // 如果需要继续使用 is_even，应该 clone
    /// let combined = is_positive.and(is_even.clone());
    /// assert!(combined.test(&4));
    ///
    /// // 两个原 predicate 都可继续使用
    /// assert!(is_positive.test(&5));
    /// assert!(is_even.test(&6));
    /// ```
    ///
    /// ## 多次重用同一个 predicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// let positive_and_even = is_positive.and(|x: &i32| x % 2 == 0);
    /// let positive_and_small = is_positive.and(|x: &i32| *x < 100);
    ///
    /// // is_positive 可以被多次组合
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// // 原 predicate 仍然可用
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// // 原 predicate 仍然可用
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// // 原 predicate 仍然可用
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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

    /// Converts this predicate to a `BoxPredicate`.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` wrapping this predicate.
    pub fn to_box(&self) -> BoxPredicate<T> {
        let self_fn = Rc::clone(&self.function);
        BoxPredicate {
            function: Box::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    /// Converts this predicate to a closure that can be used directly with
    /// standard library methods.
    ///
    /// This method creates a new closure without consuming the original
    /// predicate, since `RcPredicate` uses shared ownership. The returned
    /// closure has signature `Fn(&T) -> bool`. Since `Fn` is a subtrait of
    /// `FnMut`, it can be used in any context that requires either
    /// `Fn(&T) -> bool` or `FnMut(&T) -> bool`, making it compatible with
    /// methods like `Iterator::filter`, `Iterator::filter_map`,
    /// `Vec::retain`, and similar standard library APIs.
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
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred = RcPredicate::new(|x: &i32| *x > 0);
    /// let closure = pred.to_fn();
    ///
    /// let numbers = vec![-2, -1, 0, 1, 2, 3];
    /// let positives: Vec<_> = numbers.iter()
    ///     .copied()
    ///     .filter(closure)
    ///     .collect();
    /// assert_eq!(positives, vec![1, 2, 3]);
    ///
    /// // Original predicate is still usable
    /// assert!(pred.test(&5));
    /// ```
    ///
    /// ## Using with `Vec::retain` (requires `FnMut`)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred = RcPredicate::new(|x: &i32| *x % 2 == 0);
    /// let mut numbers = vec![1, 2, 3, 4, 5, 6];
    /// numbers.retain(pred.to_fn());
    /// assert_eq!(numbers, vec![2, 4, 6]);
    ///
    /// // Original predicate is still usable
    /// assert!(pred.test(&2));
    /// ```
    ///
    /// ## Passing to functions that require `FnMut`
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// fn count_matching<F>(items: &[i32], mut predicate: F) -> usize
    /// where
    ///     F: FnMut(&i32) -> bool,
    /// {
    ///     items.iter().filter(|x| predicate(x)).count()
    /// }
    ///
    /// let pred = RcPredicate::new(|x: &i32| *x > 10);
    /// let count = count_matching(&[5, 15, 8, 20], pred.to_fn());
    /// assert_eq!(count, 2);
    ///
    /// // Original predicate can be reused
    /// let count2 = count_matching(&[12, 3, 18], pred.to_fn());
    /// assert_eq!(count2, 2);
    /// ```
    pub fn to_fn(&self) -> impl Fn(&T) -> bool {
        let function = Rc::clone(&self.function);
        move |value: &T| function(value)
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

    fn into_arc(self) -> ArcPredicate<T>
    where
        T: Send + Sync,
    {
        // RcPredicate cannot be converted to ArcPredicate because Rc is not Send.
        // Users should use ArcPredicate::new directly if they need thread-safety.
        panic!("RcPredicate cannot be converted to ArcPredicate - use ArcPredicate::new directly")
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        let self_fn = self.function;
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T> + Send + Sync` implementation.
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
    /// // 可以跨线程使用
    /// let handle = thread::spawn(move || {
    ///     combined.test(&4)
    /// });
    ///
    /// assert!(handle.join().unwrap());
    /// assert!(is_positive.test(&5)); // 原 predicate 仍可用
    /// ```
    pub fn and<P>(&self, other: P) -> ArcPredicate<T>
    where
        T: Send + Sync,
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T> + Send + Sync` implementation.
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
    /// assert!(is_negative.test(&-10)); // 原 predicate 仍可用
    /// ```
    pub fn or<P>(&self, other: P) -> ArcPredicate<T>
    where
        T: Send + Sync,
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
    pub fn not(&self) -> ArcPredicate<T>
    where
        T: Send + Sync,
    {
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T> + Send + Sync` implementation.
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
        T: Send + Sync,
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
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcPredicate` representing the logical XOR.
    pub fn xor<P>(&self, other: P) -> ArcPredicate<T>
    where
        T: Send + Sync,
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T> + Send + Sync`
    ///   implementation.
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
        T: Send + Sync,
        P: Predicate<T> + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        ArcPredicate {
            function: Arc::new(move |value: &T| !(self_fn(value) || other.test(value))),
            name: None,
        }
    }

    /// Converts this predicate to a `BoxPredicate`.
    ///
    /// # Returns
    ///
    /// A `BoxPredicate` wrapping this predicate.
    pub fn to_box(&self) -> BoxPredicate<T> {
        let self_fn = Arc::clone(&self.function);
        BoxPredicate {
            function: Box::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    /// Converts this predicate to an `RcPredicate`.
    ///
    /// # Returns
    ///
    /// An `RcPredicate` wrapping this predicate.
    pub fn to_rc(&self) -> RcPredicate<T> {
        let self_fn = Arc::clone(&self.function);
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value)),
            name: self.name.clone(),
        }
    }

    /// Converts this predicate to a closure that can be used directly with
    /// standard library methods.
    ///
    /// This method creates a new closure without consuming the original
    /// predicate, since `ArcPredicate` uses shared ownership. The returned
    /// closure has signature `Fn(&T) -> bool + Send + Sync` and is
    /// thread-safe. Since `Fn` is a subtrait of `FnMut`, it can be used in
    /// any context that requires either `Fn(&T) -> bool` or
    /// `FnMut(&T) -> bool`, making it compatible with methods like
    /// `Iterator::filter`, `Iterator::filter_map`, `Vec::retain`, and
    /// similar standard library APIs.
    ///
    /// # Returns
    ///
    /// A closure implementing `Fn(&T) -> bool + Send + Sync` (also usable
    /// as `FnMut(&T) -> bool`).
    ///
    /// # Examples
    ///
    /// ## Using with `Iterator::filter` (requires `FnMut`)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let closure = pred.to_fn();
    ///
    /// let numbers = vec![-2, -1, 0, 1, 2, 3];
    /// let positives: Vec<_> = numbers.iter()
    ///     .copied()
    ///     .filter(closure)
    ///     .collect();
    /// assert_eq!(positives, vec![1, 2, 3]);
    ///
    /// // Original predicate is still usable
    /// assert!(pred.test(&5));
    /// ```
    ///
    /// ## Using with `Vec::retain` (requires `FnMut`)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x % 2 == 0);
    /// let mut numbers = vec![1, 2, 3, 4, 5, 6];
    /// numbers.retain(pred.to_fn());
    /// assert_eq!(numbers, vec![2, 4, 6]);
    ///
    /// // Original predicate is still usable
    /// assert!(pred.test(&2));
    /// ```
    ///
    /// ## Passing to functions that require `FnMut`
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// fn count_matching<F>(items: &[i32], mut predicate: F) -> usize
    /// where
    ///     F: FnMut(&i32) -> bool,
    /// {
    ///     items.iter().filter(|x| predicate(x)).count()
    /// }
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x > 10);
    /// let count = count_matching(&[5, 15, 8, 20], pred.to_fn());
    /// assert_eq!(count, 2);
    ///
    /// // Original predicate can be reused
    /// let count2 = count_matching(&[12, 3, 18], pred.to_fn());
    /// assert_eq!(count2, 2);
    /// ```
    ///
    /// ## Thread-safe usage
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    /// use std::thread;
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let closure = pred.to_fn();
    ///
    /// // Closure can be sent across threads
    /// let handle = thread::spawn(move || {
    ///     let numbers = vec![-2, -1, 0, 1, 2, 3];
    ///     numbers.iter().copied().filter(closure).count()
    /// });
    ///
    /// assert_eq!(handle.join().unwrap(), 3);
    /// // Original predicate is still usable
    /// assert!(pred.test(&5));
    /// ```
    pub fn to_fn(&self) -> impl Fn(&T) -> bool + Send + Sync
    where
        T: Send + Sync,
    {
        let self_fn = Arc::clone(&self.function);
        move |value: &T| self_fn(value)
    }
}

impl<T: 'static> Predicate<T> for ArcPredicate<T> {
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
        let self_fn = self.function;
        RcPredicate {
            function: Rc::new(move |value: &T| self_fn(value)),
            name: self.name,
        }
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        T: Send + Sync,
    {
        self
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        let self_fn = self.function;
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
        T: Send + Sync,
    {
        ArcPredicate::new(self)
    }

    fn into_fn(self) -> impl Fn(&T) -> bool {
        self
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
    /// * `other` - The other predicate to combine with. Can be:
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
    /// * `other` - The other predicate to combine with. Accepts closures,
    ///   function pointers, or any `Predicate<T>` implementation.
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
