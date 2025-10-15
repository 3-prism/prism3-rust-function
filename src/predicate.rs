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
//! ## Design Overview
//!
//! This module adopts the **Trait + Multiple Implementations** design
//! pattern, which is the most flexible and elegant approach for
//! implementing predicates in Rust. It achieves a perfect balance between
//! semantic clarity, type safety, and API flexibility.
//!
//! ### Core Components
//!
//! 1. **`Predicate<T>` trait**: A minimalist unified interface that only
//!    defines the core `test` method and type conversion methods
//!    (`into_*`). It does NOT include logical composition methods like
//!    `and`, `or`, etc.
//!
//! 2. **Three Concrete Struct Implementations**:
//!    - [`BoxPredicate<T>`]: Box-based single ownership implementation
//!      for one-time use scenarios
//!    - [`ArcPredicate<T>`]: Arc-based thread-safe shared ownership
//!      implementation for multi-threaded scenarios
//!    - [`RcPredicate<T>`]: Rc-based single-threaded shared ownership
//!      implementation for single-threaded reuse
//!
//! 3. **Specialized Composition Methods**: Each struct implements its own
//!    inherent methods (`and`, `or`, `not`, etc.) that return the same
//!    concrete type, preserving their specific characteristics (e.g.,
//!    `ArcPredicate` compositions remain `ArcPredicate` and stay
//!    cloneable and thread-safe).
//!
//! 4. **Extension Trait for Closures**: The `FnPredicateOps<T>`
//!    extension trait provides composition methods for all closures and
//!    function pointers, returning `BoxPredicate<T>` to initiate method
//!    chaining.
//!
//! 5. **Unified Trait Implementation**: All closures and the three
//!    structs implement the `Predicate<T>` trait, enabling them to be
//!    handled uniformly by generic functions.
//!
//! ## Ownership Model Coverage
//!
//! The three implementations correspond to three typical ownership
//! scenarios:
//!
//! | Type | Ownership | Clonable | Thread-Safe | API | Use Case |
//! |:-----|:----------|:---------|:------------|:----|:---------|
//! | [`BoxPredicate`] | Single | ❌ | ❌ | consumes `self` | One-time use |
//! | [`ArcPredicate`] | Shared | ✅ | ✅ | borrows `&self` | Multi-threaded |
//! | [`RcPredicate`] | Shared | ✅ | ❌ | borrows `&self` | Single-threaded |
//!
//! ## Key Design Advantages
//!
//! ### 1. Type Preservation through Specialization
//!
//! By implementing composition methods on concrete structs rather than in
//! the trait, each type maintains its specific characteristics through
//! composition:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, ArcPredicate};
//!
//! let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
//! let another = ArcPredicate::new(|x: &i32| x % 2 == 0);
//!
//! // Composition returns ArcPredicate, preserving clonability and
//! // thread-safety
//! let combined = arc_pred.and(&another);
//! let cloned = combined.clone();  // ✅ Still cloneable
//!
//! // Original predicates remain usable
//! assert!(arc_pred.test(&5));
//! ```
//!
//! ### 2. Elegant API without Explicit Cloning
//!
//! `ArcPredicate` and `RcPredicate` use `&self` in their composition
//! methods, providing a natural experience without requiring explicit
//! `.clone()` calls:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, ArcPredicate};
//!
//! let pred = ArcPredicate::new(|x: &i32| *x > 0);
//!
//! // No need for explicit clone()
//! let combined1 = pred.and(&ArcPredicate::new(|x| x % 2 == 0));
//! let combined2 = pred.or(&ArcPredicate::new(|x| *x > 100));
//!
//! // pred is still available
//! assert!(pred.test(&42));
//! ```
//!
//! ### 3. Unified Trait Interface
//!
//! All predicate types implement the `Predicate<T>` trait, enabling polymorphic usage:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, BoxPredicate, ArcPredicate, RcPredicate};
//!
//! fn use_any_predicate<P: Predicate<i32>>(pred: &P, value: i32) -> bool {
//!     pred.test(&value)
//! }
//!
//! let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
//! let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
//! let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
//! let closure = |x: &i32| *x > 0;
//!
//! // All types can be passed to the same function
//! assert!(use_any_predicate(&box_pred, 5));
//! assert!(use_any_predicate(&arc_pred, 5));
//! assert!(use_any_predicate(&rc_pred, 5));
//! assert!(use_any_predicate(&closure, 5));
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Usage with Closures
//!
//! Closures automatically implement both `Predicate` and `FnPredicateOps`:
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, FnPredicateOps};
//!
//! let is_positive = |x: &i32| *x > 0;
//! assert!(is_positive.test(&5));  // Direct use of .test()
//!
//! // Method chaining returns BoxPredicate
//! let positive_even = is_positive.and(|x: &i32| x % 2 == 0);
//! assert!(positive_even.test(&4));
//! ```
//!
//! ### BoxPredicate - One-time Use
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, BoxPredicate};
//!
//! let pred = BoxPredicate::new(|x: &i32| *x > 0)
//!     .with_name("is_positive");
//!
//! assert!(pred.test(&5));
//!
//! // Method chaining consumes ownership
//! let combined = pred.and(|x: &i32| x % 2 == 0);
//! assert!(combined.test(&4));
//! // pred is no longer available
//! ```
//!
//! ### ArcPredicate - Multi-threaded Sharing
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, ArcPredicate};
//! use std::thread;
//!
//! let shared = ArcPredicate::new(|x: &i32| *x > 0);
//!
//! // Can be cloned and sent across threads
//! let clone = shared.clone();
//! let handle = thread::spawn(move || clone.test(&5));
//! assert!(handle.join().unwrap());
//!
//! // Original still available for composition
//! let combined = shared.and(&ArcPredicate::new(|x| x % 2 == 0));
//! assert!(combined.test(&4));
//! ```
//!
//! ### RcPredicate - Single-threaded Reuse
//!
//! ```rust
//! use prism3_function::predicate::{Predicate, RcPredicate};
//!
//! let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
//!
//! // Efficient reuse without thread-safety overhead
//! let combined1 = rc_pred.and(&RcPredicate::new(|x| x % 2 == 0));
//! let combined2 = rc_pred.or(&RcPredicate::new(|x| *x > 100));
//!
//! // Original predicate remains usable
//! assert!(rc_pred.test(&7));
//! ```
//!
//! ## Choosing the Right Implementation
//!
//! - **Use [`BoxPredicate`]** when:
//!   - The predicate is used only once
//!   - You're building a predicate through method chaining and consuming it immediately
//!   - You want to avoid the overhead of reference counting
//!
//! - **Use [`ArcPredicate`]** when:
//!   - The predicate needs to be shared across multiple threads
//!   - You need to clone the predicate for use in multiple places
//!   - Thread safety is required (e.g., in async contexts with Send + Sync bounds)
//!
//! - **Use [`RcPredicate`]** when:
//!   - You need to share the predicate within a single thread
//!   - You want better performance than Arc (no atomic operations)
//!   - Thread safety is not required (e.g., UI validation logic)
//!
//! ## Design Philosophy
//!
//! This design follows Rust's philosophy of "zero-cost abstractions" and
//! aligns perfectly with the standard library's design patterns (similar
//! to how `Deref` trait works with `Box/Rc/Arc`). It provides maximum
//! flexibility while maintaining type safety and clear semantics.
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// 1. Predicate Trait - Unified predicate interface
// ============================================================================

/// Predicate trait - Unified predicate interface
///
/// The `Predicate<T>` trait provides a minimalist, unified interface for
/// all predicate types in this module. Similar to Java's `Predicate<T>`
/// interface, it is used to test whether a value satisfies a specific
/// condition.
///
/// ## Design Philosophy
///
/// This trait intentionally keeps a minimal surface area, defining only:
/// - The core `test` method for condition evaluation
/// - Type conversion methods (`into_box`, `into_rc`, `into_arc`) for
///   flexibility
///
/// **Importantly**: Logical composition methods (such as `and`, `or`,
/// `not`, etc.) are **NOT** defined in this trait. Instead, they are
/// implemented as inherent methods on each concrete type
/// ([`BoxPredicate`], [`ArcPredicate`], [`RcPredicate`]) and as
/// extension methods for closures ([`FnPredicateOps`]). This design
/// allows each implementation to:
///
/// 1. **Return its own concrete type**, preserving type-specific
///    characteristics (clonability, thread-safety, etc.)
/// 2. **Use appropriate ownership semantics** (`self` vs `&self`) based
///    on the underlying pointer type
/// 3. **Avoid trait object limitations** and enable method chaining with
///    zero-cost abstractions
///
/// ## Universal Implementation
///
/// All closures, function pointers, and predicate structs implement this
/// trait automatically:
///
/// - **Closures and function pointers**: Any `F where F: Fn(&T) -> bool`
///   automatically implements `Predicate<T>`
/// - **Predicate structs**: [`BoxPredicate`], [`ArcPredicate`], and
///   [`RcPredicate`] all implement this trait
///
/// This enables polymorphic usage where any predicate type can be
/// accepted uniformly via trait bounds.
///
/// ## Logical Composition Methods
///
/// Each implementation provides its own logical composition methods with
/// different signatures:
///
/// | Type | Composition Methods | Ownership | Returns | After |
/// |:-----|:-------------------|:----------|:--------|:------|
/// | Closures via [`FnPredicateOps`] | `and(self, ...)` | Consumes | [`BoxPredicate`] | ❌ |
/// | [`BoxPredicate`] | `and(self, ...)` | Consumes | [`BoxPredicate`] | ❌ |
/// | [`ArcPredicate`] | `and(&self, ...)` | Borrows | [`ArcPredicate`] | ✅ |
/// | [`RcPredicate`] | `and(&self, ...)` | Borrows | [`RcPredicate`] | ✅ |
///
/// ## Type Conversion Methods
///
/// The trait provides methods to convert between different predicate
/// types. **All conversion methods consume `self`** (take ownership):
///
/// | Method | Converts To | Consumes Self | Zero-cost When | Requirements |
/// |:-------|:------------|:--------------|:---------------|:-------------|
/// | `into_box()` | [`BoxPredicate<T>`] | ✅ Yes | Already `BoxPredicate` | `'static` |
/// | `into_rc()` | [`RcPredicate<T>`] | ✅ Yes | Already `RcPredicate` | `'static` |
/// | `into_arc()` | [`ArcPredicate<T>`] | ✅ Yes | Already `ArcPredicate` | `Send + Sync + 'static` |
/// | `into_fn()` | `impl FnMut(&T) -> bool` | ✅ Yes | N/A | `'static` |
///
/// **Important**: After calling any conversion method, the original
/// predicate is moved and becomes unavailable. If you need to reuse a
/// predicate, consider using [`ArcPredicate`] or [`RcPredicate`] which
/// can be cloned before conversion.
///
/// These methods enable flexible interoperability between different
/// predicate types when needed.
///
/// ## Examples
///
/// ### Basic Usage with Different Types
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps, BoxPredicate, ArcPredicate};
///
/// // Closures automatically implement Predicate
/// let closure = |x: &i32| *x > 0;
/// assert!(closure.test(&5));
///
/// // BoxPredicate implements Predicate
/// let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
/// assert!(box_pred.test(&5));
///
/// // ArcPredicate implements Predicate
/// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
/// assert!(arc_pred.test(&5));
/// ```
///
/// ### Polymorphic Function Usage
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
/// use prism3_function::predicate::{ArcPredicate, RcPredicate};
///
/// // Function accepting any predicate type
/// fn filter_positive<P>(values: Vec<i32>, predicate: &P)
///     -> Vec<i32>
/// where
///     P: Predicate<i32>,
/// {
///     values.into_iter().filter(|v| predicate.test(v)).collect()
/// }
///
/// let values = vec![1, -2, 3, -4, 5];
///
/// // Works with closures
/// let result = filter_positive(values.clone(), &|x: &i32| *x > 0);
/// assert_eq!(result, vec![1, 3, 5]);
///
/// // Works with BoxPredicate
/// let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
/// let result = filter_positive(values.clone(), &box_pred);
/// assert_eq!(result, vec![1, 3, 5]);
///
/// // Works with ArcPredicate
/// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
/// let result = filter_positive(values.clone(), &arc_pred);
/// assert_eq!(result, vec![1, 3, 5]);
/// ```
///
/// ### Composition with Different Ownership Semantics
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps};
/// use prism3_function::predicate::{BoxPredicate, ArcPredicate};
///
/// // Closure composition consumes the closure, returns BoxPredicate
/// let closure = |x: &i32| *x > 0;
/// let combined = closure.and(|x: &i32| x % 2 == 0);
/// assert!(combined.test(&4));
/// // closure is no longer available
///
/// // BoxPredicate composition consumes self
/// let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
/// let combined = box_pred.and(|x: &i32| x % 2 == 0);
/// assert!(combined.test(&4));
/// // box_pred is no longer available
///
/// // ArcPredicate composition borrows &self
/// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
/// let combined = arc_pred.and(&ArcPredicate::new(|x| x % 2 == 0));
/// assert!(combined.test(&4));
/// // arc_pred is still available!
/// assert!(arc_pred.test(&10));
/// ```
///
/// ### Type Conversion
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
/// use prism3_function::predicate::{RcPredicate, ArcPredicate};
///
/// // Convert closure to BoxPredicate
/// let closure = |x: &i32| *x > 0;
/// let box_pred: BoxPredicate<i32> = closure.into_box();
/// assert!(box_pred.test(&5));
///
/// // Convert BoxPredicate to RcPredicate
/// let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
/// let rc_pred: RcPredicate<i32> = box_pred.into_rc();
/// assert!(rc_pred.test(&5));
///
/// // Convert closure to ArcPredicate (requires Send + Sync)
/// let closure = |x: &i32| *x > 0;
/// let arc_pred: ArcPredicate<i32> = closure.into_arc();
/// assert!(arc_pred.test(&5));
/// ```
///
/// ### Using with Iterators via `into_fn()`
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate, ArcPredicate};
///
/// // Simple filtering
/// let predicate = BoxPredicate::new(|x: &i32| *x > 0);
/// let values = vec![1, -2, 3, -4, 5];
/// let result: Vec<i32> = values.into_iter()
///     .filter(predicate.into_fn())
///     .collect();
/// assert_eq!(result, vec![1, 3, 5]);
///
/// // Complex predicate composition
/// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
/// let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
/// let predicate = is_positive.and(&is_even);
///
/// let values = vec![1, 2, 3, 4, 5, 6];
/// let result: Vec<i32> = values.into_iter()
///     .filter(predicate.into_fn())
///     .collect();
/// assert_eq!(result, vec![2, 4, 6]);
///
/// // Using with other iterator methods
/// let predicate = BoxPredicate::new(|x: &i32| *x > 0);
/// let values = vec![1, 2, -3, 4];
/// let result: Vec<i32> = values.iter()
///     .copied()
///     .take_while(predicate.into_fn())
///     .collect();
/// assert_eq!(result, vec![1, 2]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Predicate<T> {
    /// Tests whether the value satisfies the predicate condition
    ///
    /// # Parameters
    ///
    /// * `value` - A reference to the value to be tested
    ///
    /// # Returns
    ///
    /// Returns `true` if the value satisfies the condition, otherwise
    /// returns `false`
    fn test(&self, value: &T) -> bool;

    /// Converts to BoxPredicate
    ///
    /// **⚠️ Consumes `self`**: The original predicate becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current predicate to `BoxPredicate<T>`. For types
    /// that are already `BoxPredicate`, this is a zero-cost operation.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the predicate (takes ownership of `self`).
    /// After calling this method, the original predicate is no longer
    /// available.
    ///
    /// **Tip**: For cloneable predicates ([`ArcPredicate`], [`RcPredicate`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let box_pred = arc_pred.clone().into_box();  // Clone first
    ///
    /// // Original still available
    /// assert!(arc_pred.test(&5));
    /// assert!(box_pred.test(&5));
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `BoxPredicate<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let closure = |x: &i32| *x > 0;
    /// let box_pred: BoxPredicate<i32> = closure.into_box();
    /// assert!(box_pred.test(&5));
    /// // closure is consumed and no longer available
    /// ```
    ///
    /// ## Zero-cost for BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0);
    /// let same_pred = pred.into_box();  // Zero-cost, just returns self
    /// assert!(same_pred.test(&5));
    /// // pred is consumed (moved)
    /// ```
    ///
    /// ## Clone Before Conversion (ArcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone before conversion to keep the original
    /// let box_pred = arc_pred.clone().into_box();
    ///
    /// // Both are still usable
    /// assert!(arc_pred.test(&5));
    /// assert!(box_pred.test(&5));
    /// ```
    ///
    /// ## Clone Before Conversion (RcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone before conversion to keep the original
    /// let box_pred = rc_pred.clone().into_box();
    ///
    /// // Both are still usable
    /// assert!(rc_pred.test(&5));
    /// assert!(box_pred.test(&5));
    /// ```
    fn into_box(self) -> BoxPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to RcPredicate
    ///
    /// **⚠️ Consumes `self`**: The original predicate becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current predicate to `RcPredicate<T>`. For types
    /// that are already `RcPredicate`, this is a zero-cost operation.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the predicate (takes ownership of `self`).
    /// After calling this method, the original predicate is no longer
    /// available.
    ///
    /// **Tip**: For cloneable predicates ([`ArcPredicate`], [`RcPredicate`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let rc_pred = arc_pred.clone().into_rc();  // Clone first
    ///
    /// // Original still available
    /// assert!(arc_pred.test(&5));
    /// assert!(rc_pred.test(&5));
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `RcPredicate<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let closure = |x: &i32| *x > 0;
    /// let rc_pred: RcPredicate<i32> = closure.into_rc();
    /// assert!(rc_pred.test(&5));
    /// // closure is consumed and no longer available
    /// ```
    ///
    /// ## Zero-cost for RcPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred = RcPredicate::new(|x: &i32| *x > 0);
    /// let same_pred = pred.into_rc();  // Zero-cost, just returns self
    /// assert!(same_pred.test(&5));
    /// // pred is consumed (moved)
    /// ```
    ///
    /// ## Converting from BoxPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
    /// let rc_pred = box_pred.into_rc();  // Efficient conversion
    /// assert!(rc_pred.test(&5));
    /// // box_pred is consumed
    /// ```
    ///
    /// ## Clone Before Conversion (ArcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone before conversion to keep the original
    /// let rc_pred = arc_pred.clone().into_rc();
    ///
    /// // Both are still usable
    /// assert!(arc_pred.test(&5));
    /// assert!(rc_pred.test(&5));
    /// ```
    ///
    /// ## Clone Before Conversion (RcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone before conversion to keep the original
    /// let rc_pred2 = rc_pred.clone().into_rc();
    ///
    /// // Both are still usable
    /// assert!(rc_pred.test(&5));
    /// assert!(rc_pred2.test(&5));
    /// ```
    fn into_rc(self) -> RcPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to ArcPredicate
    ///
    /// **⚠️ Consumes `self`**: The original predicate becomes unavailable
    /// after calling this method.
    ///
    /// Converts the current predicate to `ArcPredicate<T>`. For types
    /// that are already `ArcPredicate`, this is a zero-cost operation.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the predicate (takes ownership of `self`).
    /// After calling this method, the original predicate is no longer
    /// available.
    ///
    /// **Tip**: For cloneable predicates ([`ArcPredicate`], [`RcPredicate`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let arc_pred2 = arc_pred.clone().into_arc();  // Clone first
    ///
    /// // Original still available
    /// assert!(arc_pred.test(&5));
    /// assert!(arc_pred2.test(&5));
    /// ```
    ///
    /// # Thread Safety Requirements
    ///
    /// Since `ArcPredicate` requires thread safety:
    /// - Closures must implement `Send + Sync`
    /// - `BoxPredicate` usually cannot be converted (not `Send + Sync`)
    /// - `RcPredicate` cannot be converted (Rc is not `Send + Sync`)
    ///
    /// # Returns
    ///
    /// Returns `ArcPredicate<T>`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let closure = |x: &i32| *x > 0;
    /// let arc_pred: ArcPredicate<i32> = closure.into_arc();
    /// assert!(arc_pred.test(&5));
    /// // closure is consumed and no longer available
    /// ```
    ///
    /// ## Zero-cost for ArcPredicate
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let same_pred = pred.into_arc();  // Zero-cost, just returns self
    /// assert!(same_pred.test(&5));
    /// // pred is consumed (moved)
    /// ```
    ///
    /// ## Thread-safe Closure Conversion
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    /// use std::thread;
    ///
    /// let closure = |x: &i32| *x > 0;
    /// let arc_pred = closure.into_arc();  // Closure must be Send + Sync
    ///
    /// let clone = arc_pred.clone();
    /// let handle = thread::spawn(move || clone.test(&5));
    /// assert!(handle.join().unwrap());
    /// ```
    ///
    /// ## Clone Before Conversion (ArcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    ///
    /// // Clone before conversion to keep the original
    /// let arc_pred2 = arc_pred.clone().into_arc();
    ///
    /// // Both are still usable
    /// assert!(arc_pred.test(&5));
    /// assert!(arc_pred2.test(&5));
    /// ```
    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static;

    /// Converts predicate to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original predicate becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the predicate and returns a closure that can be
    /// directly used with iterator methods like `filter()`, `take_while()`,
    /// etc. This provides a more ergonomic API when working with iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the predicate (takes ownership of `self`).
    /// After calling this method, the original predicate is no longer
    /// available. The returned closure captures the predicate by move.
    ///
    /// **Tip**: For cloneable predicates ([`ArcPredicate`], [`RcPredicate`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, -2, 3];
    ///
    /// // Clone before conversion to keep the original
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(arc_pred.clone().into_fn())
    ///     .collect();
    ///
    /// // Original still available
    /// assert!(arc_pred.test(&5));
    /// assert_eq!(result, vec![1, 3]);
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut(&T) -> bool`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let predicate = BoxPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, -2, 3, -4, 5];
    ///
    /// // Before: verbose lambda wrapper
    /// // let result: Vec<i32> = values.into_iter()
    /// //     .filter(|v| predicate.test(v))
    /// //     .collect();
    ///
    /// // After: direct use with into_fn()
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(predicate.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 3, 5]);
    /// ```
    ///
    /// ## With Complex Predicates
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    /// let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
    /// let predicate = is_positive.and(&is_even);
    ///
    /// let values = vec![1, 2, 3, 4, 5, 6];
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(predicate.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6]);
    /// ```
    ///
    /// ## With Other Iterator Methods
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let predicate = RcPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, 2, -3, 4, -5, 6];
    ///
    /// // Use with take_while
    /// let result: Vec<i32> = values.iter()
    ///     .copied()
    ///     .take_while(predicate.into_fn())
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2]);
    /// // predicate is consumed and no longer available here
    /// ```
    ///
    /// ## Clone Before Conversion (ArcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, -2, 3, -4, 5];
    ///
    /// // Clone before conversion to keep the original
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(arc_pred.clone().into_fn())
    ///     .collect();
    ///
    /// // Original still available
    /// assert!(arc_pred.test(&10));
    /// assert_eq!(result, vec![1, 3, 5]);
    /// ```
    ///
    /// ## Clone Before Conversion (RcPredicate)
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, -2, 3, -4, 5];
    ///
    /// // Clone before conversion to keep the original
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(rc_pred.clone().into_fn())
    ///     .collect();
    ///
    /// // Original still available
    /// assert!(rc_pred.test(&10));
    /// assert_eq!(result, vec![1, 3, 5]);
    /// ```
    ///
    /// ## Ownership Behavior
    ///
    /// ```rust,compile_fail
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let predicate = BoxPredicate::new(|x: &i32| *x > 0);
    /// let values = vec![1, -2, 3];
    ///
    /// let result: Vec<i32> = values.into_iter()
    ///     .filter(predicate.into_fn())
    ///     .collect();
    ///
    /// // ❌ Error: predicate was moved in the call to into_fn()
    /// assert!(predicate.test(&5));
    /// ```
    fn into_fn(self) -> impl FnMut(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. Implement Predicate trait for closures
// ============================================================================

/// Implements Predicate trait for all Fn(&T) -> bool types
///
/// This allows all closures and function pointers to directly use the
/// Predicate `test` method.
impl<T, F> Predicate<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }

    fn into_box(self) -> BoxPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxPredicate::new(self)
    }

    fn into_rc(self) -> RcPredicate<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcPredicate::new(self)
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        ArcPredicate::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t: &T| self(t)
    }
}

// ============================================================================
// 2.5. FnPredicateOps - Extension trait for providing logical composition
//      methods to closures
// ============================================================================

/// Extension trait providing logical composition methods for closures
///
/// `FnPredicateOps<T>` is an extension trait that adds logical
/// composition methods (`and`, `or`, `not`, `xor`, `nand`, `nor`) to all
/// closures and function pointers that implement `Fn(&T) -> bool`.
///
/// ## Purpose
///
/// This trait enables method chaining on closures, allowing them to be
/// composed using a fluent API similar to the predicate structs. Without
/// this trait, closures would only have the `test` method from the
/// [`Predicate`] trait.
///
/// ## Automatic Implementation
///
/// This trait is **automatically implemented for all closure types**.
/// You don't need to do anything special - just write a closure and the
/// methods are available:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps};
///
/// // Closure automatically has composition methods
/// let is_positive = |x: &i32| *x > 0;
/// let combined = is_positive.and(|x: &i32| x % 2 == 0);
/// ```
///
/// ## Composition Behavior
///
/// All composition methods:
/// - **Consume the closure** (take ownership of `self`)
/// - **Return `BoxPredicate<T>`** (not another closure)
/// - **Cannot be used after composition** (closure is moved)
///
/// ```rust
/// use prism3_function::predicate::{FnPredicateOps, BoxPredicate};
///
/// let closure = |x: &i32| *x > 0;
/// let result: BoxPredicate<i32> = closure.and(|x: &i32| x % 2 == 0);
/// // closure is no longer available here
/// ```
///
/// ## Why Return BoxPredicate?
///
/// The composition methods return [`BoxPredicate`] rather than keeping
/// the closure type because:
///
/// 1. **Type erasure**: Composed closures would have complex,
///    unnameable types
/// 2. **Consistent API**: Enables further method chaining with
///    `BoxPredicate` methods
/// 3. **Simplicity**: Users don't need to worry about complex closure
///    types
///
/// ## When to Use Closures vs Structs
///
/// - **Use closures with `FnPredicateOps`** when:
///   - Writing quick, inline predicates
///   - One-time use or immediate composition
///   - Don't need shared ownership
///
/// - **Use [`ArcPredicate`] or [`RcPredicate`]** when:
///   - Need to reuse predicates after composition
///   - Want to avoid consuming ownership
///   - Need to share predicates across multiple locations
///
/// ## Available Composition Methods
///
/// - **`and(self, other)`**: Logical AND - both must be true
/// - **`or(self, other)`**: Logical OR - at least one must be true
/// - **`not(self)`**: Logical NOT - inverts the result
/// - **`xor(self, other)`**: Logical XOR - exactly one must be true
/// - **`nand(self, other)`**: Logical NAND - NOT (both true)
/// - **`nor(self, other)`**: Logical NOR - NOT (any true)
///
/// ## Examples
///
/// ### Basic Composition
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps};
///
/// let is_positive = |x: &i32| *x > 0;
/// let is_even = |x: &i32| x % 2 == 0;
///
/// // Compose using method chaining
/// let positive_and_even = is_positive.and(is_even);
/// assert!(positive_and_even.test(&4));
/// assert!(!positive_and_even.test(&3));
/// ```
///
/// ### Complex Composition
///
/// ```rust
/// use prism3_function::predicate::{Predicate, FnPredicateOps};
///
/// let complex = (|x: &i32| *x > 0)
///     .and(|x: &i32| x % 2 == 0)
///     .or(|x: &i32| *x > 100);
///
/// assert!(complex.test(&4));   // Positive and even
/// assert!(complex.test(&150)); // Large
/// assert!(!complex.test(&3));  // Positive but odd and not large
/// ```
///
/// ### With Different Predicate Types
///
/// ```rust
/// use prism3_function::predicate::{FnPredicateOps, BoxPredicate};
/// use prism3_function::predicate::ArcPredicate;
///
/// let closure = |x: &i32| *x > 0;
///
/// // Can combine with BoxPredicate
/// let box_pred = BoxPredicate::new(|x: &i32| x % 2 == 0);
/// let combined1 = closure.and(box_pred);
///
/// // Can combine with another closure
/// let closure2 = |x: &i32| *x > 0;
/// let combined2 = closure2.or(|x: &i32| *x < -100);
/// ```
pub trait FnPredicateOps<T>: Fn(&T) -> bool + Sized {
    /// AND composition - returns BoxPredicate
    fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self(t) && other.test(t))
    }

    /// OR composition - returns BoxPredicate
    fn or<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self(t) || other.test(t))
    }

    /// NOT composition - returns BoxPredicate
    fn not(self) -> BoxPredicate<T>
    where
        Self: 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| !self(t))
    }

    /// XOR composition - returns BoxPredicate
    fn xor<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self(t) ^ other.test(t))
    }

    /// NAND composition - returns BoxPredicate
    fn nand<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| !(self(t) && other.test(t)))
    }

    /// NOR composition - returns BoxPredicate
    fn nor<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| !(self(t) || other.test(t)))
    }
}

/// Implements FnPredicateOps for all closure types
impl<T, F> FnPredicateOps<T> for F where F: Fn(&T) -> bool {}

// ============================================================================
// 3. BoxPredicate - Single ownership implementation
// ============================================================================

/// Box-based predicate implementation
///
/// `BoxPredicate<T>` encapsulates predicate functions using `Box<dyn Fn>`,
/// providing single ownership semantics suitable for one-time use or
/// ownership transfer scenarios. This is the simplest predicate type with
/// minimal overhead.
///
/// # Ownership Model
///
/// - **Single ownership**: Cannot be cloned (predicate is consumed on use)
/// - **No reference counting overhead**: Direct `Box` allocation without
///   atomic operations
/// - **Move semantics**: Composition methods consume `self` and return new
///   `BoxPredicate`
/// - **Zero-cost**: When already a `BoxPredicate`, `into_box()` is a
///   no-op
///
/// # Features
///
/// - **Metadata support**: Can attach a name via `with_name()` for
///   debugging
/// - **Method chaining**: Supports fluent API through composition methods
///   (`and`, `or`, `not`, etc.)
/// - **Type safety**: Distinct type from raw `Box<dyn Fn>` in the type
///   system
/// - **Implements standard traits**: `Display`, `Debug` for better
///   developer experience
///
/// # Composition Behavior
///
/// All composition methods (`and`, `or`, `not`, `xor`, `nand`, `nor`)
/// **consume** the predicate by taking ownership of `self`. The predicate
/// cannot be used after composition:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0);
/// let combined = pred.and(|x: &i32| x % 2 == 0);
/// // pred is no longer available here - it was moved
/// assert!(combined.test(&4));
/// ```
///
/// If you need to reuse a predicate after composition, consider using
/// [`ArcPredicate`] or [`RcPredicate`] instead.
///
/// # Thread Safety
///
/// `BoxPredicate` is **not** thread-safe and does not implement
/// `Send + Sync` by default (though the contained closure might).
/// For thread-safe predicates, use [`ArcPredicate`].
///
/// # Conversion Limitations
///
/// Since `BoxPredicate` uses `Box<dyn Fn(&T) -> bool>` internally
/// (without `Send + Sync` bounds), instances created through `new()`
/// **cannot** call the `into_arc()` method. The compiler will reject
/// such attempts because `BoxPredicate<T>` does not satisfy the
/// `Send + Sync` constraint.
///
/// To create an [`ArcPredicate`] from a closure, use the closure's
/// `into_arc()` method directly instead:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// // ✅ Correct way to create ArcPredicate from closure
/// let closure = |x: &i32| *x > 0;
/// let arc_pred = closure.into_arc();
///
/// // ❌ This won't compile:
/// // let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
/// // let arc_pred = box_pred.into_arc(); // Error!
/// ```
///
/// # When to Use BoxPredicate
///
/// Use `BoxPredicate` when:
///
/// - **One-time consumption**: The predicate is used once and discarded
/// - **Builder pattern**: Building complex predicates through method
///   chaining
/// - **Minimal overhead**: Want to avoid reference counting costs
/// - **No sharing needed**: The predicate doesn't need to be cloned or
///   shared
///
/// For scenarios requiring predicate reuse, see [`ArcPredicate`] or
/// [`RcPredicate`].
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
/// assert!(!pred.test(&-3));
/// ```
///
/// ## With Metadata
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0)
///     .with_name("is_positive");
///
/// println!("Testing with: {}", pred); // Prints: BoxPredicate(is_positive)
/// assert!(pred.test(&5));
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// let complex = BoxPredicate::new(|x: &i32| *x > 0)
///     .with_name("positive")
///     .and(|x: &i32| x % 2 == 0)      // Must be even
///     .or(|x: &i32| *x > 100);        // Or very large
///
/// assert!(complex.test(&4));   // Positive and even
/// assert!(complex.test(&150)); // Very large
/// assert!(!complex.test(&3));  // Positive but odd and not large
/// ```
///
/// ## Accepting Any Predicate
///
/// ```rust
/// use prism3_function::predicate::{Predicate, BoxPredicate};
///
/// fn filter_values<P>(values: Vec<i32>, pred: &P) -> Vec<i32>
/// where
///     P: Predicate<i32>,
/// {
///     values.into_iter().filter(|v| pred.test(v)).collect()
/// }
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0);
/// let result = filter_values(vec![1, -2, 3], &pred);
/// assert_eq!(result, vec![1, 3]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxPredicate<T> {
    inner: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,
}

impl<T> BoxPredicate<T> {
    /// Creates a new BoxPredicate
    ///
    /// # Parameters
    ///
    /// * `f` - Predicate function or closure
    ///
    /// # Returns
    ///
    /// Returns a new BoxPredicate instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0);
    /// assert!(pred.test(&5));
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            inner: Box::new(f),
            name: None,
        }
    }

    /// Sets the predicate name
    ///
    /// # Parameters
    ///
    /// * `name` - Predicate name
    ///
    /// # Returns
    ///
    /// Returns self to support method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, BoxPredicate};
    ///
    /// let pred = BoxPredicate::new(|x: &i32| *x > 0)
    ///     .with_name("is_positive");
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Gets the predicate name
    ///
    /// # Returns
    ///
    /// Returns a reference to the predicate name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// AND composition
    ///
    /// Performs AND composition with another predicate, returning a new
    /// BoxPredicate.
    ///
    /// # Parameters
    ///
    /// * `other` - Another predicate
    ///
    /// # Returns
    ///
    /// Returns the composed BoxPredicate
    pub fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self.test(t) && other.test(t))
    }

    /// OR composition
    pub fn or<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self.test(t) || other.test(t))
    }

    /// NOT composition
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> BoxPredicate<T>
    where
        T: 'static,
    {
        BoxPredicate::new(move |t| !self.test(t))
    }

    /// XOR composition
    pub fn xor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self.test(t) ^ other.test(t))
    }

    /// NAND composition
    pub fn nand<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| !(self.test(t) && other.test(t)))
    }

    /// NOR composition
    pub fn nor<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| !(self.test(t) || other.test(t)))
    }
}

impl<T> Predicate<T> for BoxPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    fn into_box(self) -> BoxPredicate<T>
    where
        T: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_rc(self) -> RcPredicate<T>
    where
        T: 'static,
    {
        // Directly convert from Box to Rc, avoiding extra closure
        // wrapping
        RcPredicate {
            inner: Rc::from(self.inner),
            name: self.name,
        }
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Send + Sync,
        T: Send + Sync + 'static,
    {
        // Since BoxPredicate internally uses Box<dyn Fn(&T) -> bool>
        // (without Send + Sync bounds), BoxPredicate<T> will never
        // satisfy the Send + Sync constraint. Therefore this method
        // exists but can never be called. The compiler will reject
        // compilation at the call site.
        unreachable!(
            "BoxPredicate<T> does not implement Send + Sync, so this \
             method can never be called"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t: &T| self.test(t)
    }
}

impl<T> std::fmt::Display for BoxPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "BoxPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> std::fmt::Debug for BoxPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BoxPredicate")
            .field("name", &self.name)
            .finish()
    }
}

// ============================================================================
// 4. ArcPredicate - Thread-safe shared ownership implementation
// ============================================================================

/// Arc-based predicate implementation
///
/// `ArcPredicate<T>` encapsulates predicate functions using `Arc<dyn Fn>`,
/// providing **thread-safe shared ownership** semantics. This is the most
/// flexible predicate type for concurrent scenarios where predicates need
/// to be shared across multiple threads or cloned for multiple uses.
///
/// # Ownership Model
///
/// - **Shared ownership**: Can be cloned cheaply (only increments
///   reference count)
/// - **Thread-safe**: Implements `Send + Sync`, safe to share across
///   threads
/// - **Borrow semantics**: Composition methods borrow `&self`, preserving
///   the original predicate
/// - **Zero-cost conversion**: When already an `ArcPredicate`,
///   `into_arc()` is a no-op
///
/// # Key Advantages
///
/// The most significant advantage of `ArcPredicate` is its **elegant API
/// without explicit cloning**. Composition methods use `&self` rather
/// than consuming `self`, which means:
///
/// 1. **No explicit `.clone()` calls needed** in method chains
/// 2. **Original predicate remains available** after composition
/// 3. **Natural and intuitive usage** aligned with Rust's reference
///    counting patterns
///
/// This design makes `ArcPredicate` particularly pleasant to use in
/// scenarios where predicates are reused or shared.
///
/// # Composition Behavior
///
/// All composition methods (`and`, `or`, `not`, `xor`, `nand`, `nor`)
/// **borrow** the predicate via `&self`. The predicate remains available
/// after composition:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let pred = ArcPredicate::new(|x: &i32| *x > 0);
/// let combined = pred.and(&ArcPredicate::new(|x| x % 2 == 0));
///
/// // ✅ pred is still available - it was only borrowed
/// assert!(pred.test(&5));
/// assert!(combined.test(&4));
///
/// // Can continue composing with the original predicate
/// let another = pred.or(&ArcPredicate::new(|x| *x < -100));
/// assert!(another.test(&150));
/// ```
///
/// # Thread Safety
///
/// `ArcPredicate` is fully thread-safe and can be sent across thread
/// boundaries. The internal closure must implement `Send + Sync`:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
/// use std::thread;
///
/// let shared = ArcPredicate::new(|x: &i32| *x > 0);
///
/// // Clone and send to another thread
/// let clone = shared.clone();
/// let handle = thread::spawn(move || {
///     clone.test(&42)
/// });
///
/// // Original still usable in this thread
/// assert!(shared.test(&10));
/// assert!(handle.join().unwrap());
/// ```
///
/// # Cloning Behavior
///
/// Cloning an `ArcPredicate` is cheap - it only increments the reference
/// count without duplicating the underlying closure. All clones share the
/// same predicate logic:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let original = ArcPredicate::new(|x: &i32| *x > 0);
/// let clone1 = original.clone();
/// let clone2 = original.clone();
///
/// // All share the same predicate logic
/// assert!(original.test(&5));
/// assert!(clone1.test(&5));
/// assert!(clone2.test(&5));
/// ```
///
/// # When to Use ArcPredicate
///
/// Use `ArcPredicate` when:
///
/// - **Multi-threaded sharing**: Predicate needs to be used in multiple
///   threads
/// - **Async contexts**: Working with async/await and need `Send + Sync`
///   bounds
/// - **Configuration/registry**: Storing predicates in shared
///   configuration
/// - **Multiple consumers**: Multiple parts of code need access to the
///   same predicate
/// - **Reuse in composition**: Building multiple predicates from the same
///   base
///
/// For single-threaded scenarios, [`RcPredicate`] provides the same API
/// with better performance (no atomic operations). For one-time use,
/// [`BoxPredicate`] is simpler.
///
/// # Performance Characteristics
///
/// - **Creation**: Single `Arc` allocation
/// - **Cloning**: Atomic reference count increment (cheap)
/// - **Method calls**: Single indirect function call
/// - **Composition**: Creates new `Arc` wrapping both predicates
/// - **Drop**: Atomic reference count decrement
///
/// The atomic operations make `ArcPredicate` slightly slower than
/// [`RcPredicate`] in single-threaded contexts, but the overhead is
/// minimal for most use cases.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let pred = ArcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
/// assert!(!pred.test(&-3));
/// ```
///
/// ## Cloning and Thread Safety
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
/// use std::thread;
///
/// let pred = ArcPredicate::new(|x: &i32| *x > 0);
///
/// // Clone for use in another thread
/// let clone = pred.clone();
/// let handle = thread::spawn(move || {
///     clone.test(&5)
/// });
///
/// // Original still available
/// assert!(pred.test(&10));
/// assert!(handle.join().unwrap());
/// ```
///
/// ## Composition Without Explicit Cloning
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
/// let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
///
/// // No need for explicit .clone() calls
/// let positive_even = is_positive.and(&is_even);
/// let positive_or_even = is_positive.or(&is_even);
///
/// // Both original predicates still available
/// assert!(is_positive.test(&7));
/// assert!(is_even.test(&8));
/// assert!(positive_even.test(&4));
/// assert!(positive_or_even.test(&3));
/// ```
///
/// ## Shared Configuration
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
/// use std::collections::HashMap;
///
/// // Store predicates in a shared configuration
/// let mut validators: HashMap<&str, ArcPredicate<i32>> = HashMap::new();
///
/// validators.insert(
///     "age_adult",
///     ArcPredicate::new(|age| *age >= 18)
/// );
/// validators.insert(
///     "age_senior",
///     ArcPredicate::new(|age| *age >= 65)
/// );
///
/// // Retrieve and use
/// let adult_check = validators.get("age_adult").unwrap();
/// assert!(adult_check.test(&25));
/// ```
///
/// ## Building Complex Predicates
///
/// ```rust
/// use prism3_function::predicate::{Predicate, ArcPredicate};
///
/// let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
/// let is_small = ArcPredicate::new(|x: &i32| *x < 100);
/// let is_large = ArcPredicate::new(|x: &i32| *x >= 1000);
///
/// // Build multiple derived predicates
/// let valid_medium = is_positive.and(&is_small);
/// let extreme = is_positive.not().or(&is_large);
///
/// // All base predicates still usable
/// assert!(valid_medium.test(&50));
/// assert!(extreme.test(&-10));
/// assert!(extreme.test(&2000));
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcPredicate<T> {
    inner: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    name: Option<String>,
}

impl<T> ArcPredicate<T> {
    /// Creates a new ArcPredicate
    ///
    /// # Parameters
    ///
    /// * `f` - Predicate function or closure (must implement Send + Sync)
    ///
    /// # Returns
    ///
    /// Returns a new ArcPredicate instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, ArcPredicate};
    ///
    /// let pred = ArcPredicate::new(|x: &i32| *x > 0);
    /// assert!(pred.test(&5));
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(f),
            name: None,
        }
    }

    /// Sets the predicate name
    ///
    /// # Parameters
    ///
    /// * `name` - Predicate name
    ///
    /// # Returns
    ///
    /// Returns self to support method chaining
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Gets the predicate name
    ///
    /// # Returns
    ///
    /// Returns a reference to the predicate name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// AND composition
    ///
    /// Performs AND composition with another ArcPredicate.
    ///
    /// # Parameters
    ///
    /// * `other` - Another ArcPredicate
    ///
    /// # Returns
    ///
    /// Returns the composed ArcPredicate
    pub fn and(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate {
            inner: Arc::new(move |t| self_clone.test(t) && other_clone.test(t)),
            name: None,
        }
    }

    /// OR composition
    ///
    /// Performs OR composition with another ArcPredicate.
    pub fn or(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate {
            inner: Arc::new(move |t| self_clone.test(t) || other_clone.test(t)),
            name: None,
        }
    }

    /// NOT composition
    ///
    /// Negates the current predicate.
    pub fn not(&self) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        ArcPredicate {
            inner: Arc::new(move |t| !self_clone.test(t)),
            name: None,
        }
    }

    /// XOR composition
    ///
    /// Performs XOR composition with another ArcPredicate.
    pub fn xor(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate {
            inner: Arc::new(move |t| self_clone.test(t) ^ other_clone.test(t)),
            name: None,
        }
    }

    /// NAND composition
    ///
    /// Performs NAND composition with another ArcPredicate.
    pub fn nand(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate {
            inner: Arc::new(move |t| !(self_clone.test(t) && other_clone.test(t))),
            name: None,
        }
    }

    /// NOR composition
    ///
    /// Performs NOR composition with another ArcPredicate.
    pub fn nor(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate {
            inner: Arc::new(move |t| !(self_clone.test(t) || other_clone.test(t))),
            name: None,
        }
    }
}

impl<T> Predicate<T> for ArcPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    fn into_box(self) -> BoxPredicate<T>
    where
        T: 'static,
    {
        // First extract name, then wrap as Box
        let name = self.name.clone();
        BoxPredicate {
            inner: Box::new(move |x| self.test(x)),
            name,
        }
    }

    fn into_rc(self) -> RcPredicate<T>
    where
        T: 'static,
    {
        // First extract name, then wrap as Rc
        let name = self.name.clone();
        RcPredicate {
            inner: Rc::new(move |x| self.test(x)),
            name,
        }
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_fn(self) -> impl FnMut(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t: &T| self.test(t)
    }
}

impl<T> Clone for ArcPredicate<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            name: self.name.clone(),
        }
    }
}

impl<T> std::fmt::Display for ArcPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ArcPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> std::fmt::Debug for ArcPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ArcPredicate")
            .field("name", &self.name)
            .finish()
    }
}

// ============================================================================
// 5. RcPredicate - Single-threaded shared ownership implementation
// ============================================================================

/// Rc-based predicate implementation
///
/// `RcPredicate<T>` encapsulates predicate functions using `Rc<dyn Fn>`,
/// providing **single-threaded shared ownership** semantics. This type
/// offers the same elegant API as [`ArcPredicate`] but with better
/// performance in single-threaded contexts (no atomic operations).
///
/// # Ownership Model
///
/// - **Shared ownership**: Can be cloned cheaply (only increments
///   reference count)
/// - **Not thread-safe**: Does NOT implement `Send` or `Sync`
/// - **Borrow semantics**: Composition methods borrow `&self`, preserving
///   the original predicate
/// - **Zero-cost conversion**: When already an `RcPredicate`, `into_rc()`
///   is a no-op
///
/// # Key Advantages
///
/// `RcPredicate` provides the same elegant API as [`ArcPredicate`]
/// without the overhead of atomic operations:
///
/// 1. **No explicit `.clone()` calls needed** in method chains
/// 2. **Original predicate remains available** after composition
/// 3. **Better performance than Arc** (no atomic reference counting)
/// 4. **Same intuitive API** as `ArcPredicate` for single-threaded code
///
/// For single-threaded applications, `RcPredicate` is often the best
/// choice when you need shared ownership of predicates.
///
/// # Composition Behavior
///
/// All composition methods (`and`, `or`, `not`, `xor`, `nand`, `nor`)
/// **borrow** the predicate via `&self`. The predicate remains available
/// after composition:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
/// let combined = pred.and(&RcPredicate::new(|x| x % 2 == 0));
///
/// // ✅ pred is still available - it was only borrowed
/// assert!(pred.test(&5));
/// assert!(combined.test(&4));
///
/// // Can continue composing with the original predicate
/// let another = pred.or(&RcPredicate::new(|x| *x > 100));
/// assert!(another.test(&150));
/// ```
///
/// # Thread Safety
///
/// `RcPredicate` is **NOT** thread-safe. It cannot be sent across thread
/// boundaries or shared between threads. Attempting to do so will result
/// in a compile-time error:
///
/// ```rust,compile_fail
/// use prism3_function::predicate::{Predicate, RcPredicate};
/// use std::thread;
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
///
/// // ❌ This won't compile - RcPredicate is not Send
/// let handle = thread::spawn(move || {
///     pred.test(&5)
/// });
/// ```
///
/// For thread-safe predicates, use [`ArcPredicate`] instead.
///
/// # Cloning Behavior
///
/// Cloning an `RcPredicate` is cheap - it only increments the reference
/// count (non-atomically) without duplicating the underlying closure:
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let original = RcPredicate::new(|x: &i32| *x > 0);
/// let clone1 = original.clone();
/// let clone2 = original.clone();
///
/// // All share the same predicate logic
/// assert!(original.test(&5));
/// assert!(clone1.test(&5));
/// assert!(clone2.test(&5));
/// ```
///
/// # When to Use RcPredicate
///
/// Use `RcPredicate` when:
///
/// - **Single-threaded context**: Your application or component runs in
///   a single thread
/// - **Performance matters**: Want better performance than `ArcPredicate`
/// - **Shared ownership needed**: Multiple parts of code need to access
///   the same predicate
/// - **Reuse in composition**: Building multiple predicates from the same
///   base
/// - **UI validation**: Form validators, input filters in single-threaded
///   UI
///
/// For multi-threaded scenarios, use [`ArcPredicate`]. For one-time use,
/// [`BoxPredicate`] is simpler.
///
/// # Performance Characteristics
///
/// - **Creation**: Single `Rc` allocation
/// - **Cloning**: Non-atomic reference count increment (very cheap)
/// - **Method calls**: Single indirect function call
/// - **Composition**: Creates new `Rc` wrapping both predicates
/// - **Drop**: Non-atomic reference count decrement
///
/// `RcPredicate` is faster than `ArcPredicate` because it uses
/// non-atomic reference counting. In single-threaded benchmarks, the
/// difference is measurable though usually not significant for most
/// applications.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
/// assert!(!pred.test(&-3));
/// ```
///
/// ## Cloning and Reuse
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
///
/// // Clone for use in multiple places
/// let clone1 = pred.clone();
/// let clone2 = pred.clone();
///
/// assert!(pred.test(&5));
/// assert!(clone1.test(&10));
/// assert!(clone2.test(&15));
/// ```
///
/// ## Composition Without Explicit Cloning
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
/// let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);
///
/// // No need for explicit .clone() calls
/// let positive_even = is_positive.and(&is_even);
/// let positive_or_even = is_positive.or(&is_even);
///
/// // Both original predicates still available
/// assert!(is_positive.test(&7));
/// assert!(is_even.test(&8));
/// assert!(positive_even.test(&4));
/// assert!(positive_or_even.test(&3));
/// ```
///
/// ## Form Validation Example
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
/// use std::collections::HashMap;
///
/// struct FormValidator {
///     rules: HashMap<String, RcPredicate<String>>,
/// }
///
/// impl FormValidator {
///     fn new() -> Self {
///         let mut rules = HashMap::new();
///
///         // Define validation rules
///         rules.insert(
///             "email".to_string(),
///             RcPredicate::new(|s: &String| s.contains('@'))
///         );
///         rules.insert(
///             "password".to_string(),
///             RcPredicate::new(|s: &String| s.len() >= 8)
///         );
///
///         Self { rules }
///     }
///
///     fn validate(&self, field: &str, value: &String) -> bool {
///         self.rules.get(field)
///             .map(|rule| rule.test(value))
///             .unwrap_or(true)
///     }
/// }
///
/// let validator = FormValidator::new();
/// assert!(validator.validate("email", &"user@example.com".to_string()));
/// assert!(!validator.validate("email", &"invalid".to_string()));
/// assert!(validator.validate("password", &"secure123".to_string()));
/// ```
///
/// ## Building Complex Predicates
///
/// ```rust
/// use prism3_function::predicate::{Predicate, RcPredicate};
///
/// let is_positive = RcPredicate::new(|x: &i32| *x > 0);
/// let is_small = RcPredicate::new(|x: &i32| *x < 100);
/// let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);
///
/// // Build multiple derived predicates
/// let valid_small_positive = is_positive.and(&is_small);
/// let positive_even = is_positive.and(&is_even);
/// let small_or_even = is_small.or(&is_even);
///
/// // All base predicates still usable
/// assert!(valid_small_positive.test(&50));
/// assert!(positive_even.test(&42));
/// assert!(small_or_even.test(&3));
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcPredicate<T> {
    inner: Rc<dyn Fn(&T) -> bool>,
    name: Option<String>,
}

impl<T> RcPredicate<T> {
    /// Creates a new RcPredicate
    ///
    /// # Parameters
    ///
    /// * `f` - Predicate function or closure
    ///
    /// # Returns
    ///
    /// Returns a new RcPredicate instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::predicate::{Predicate, RcPredicate};
    ///
    /// let pred = RcPredicate::new(|x: &i32| *x > 0);
    /// assert!(pred.test(&5));
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            inner: Rc::new(f),
            name: None,
        }
    }

    /// Sets the predicate name
    ///
    /// # Parameters
    ///
    /// * `name` - Predicate name
    ///
    /// # Returns
    ///
    /// Returns self to support method chaining
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Gets the predicate name
    ///
    /// # Returns
    ///
    /// Returns a reference to the predicate name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// AND composition
    ///
    /// Performs AND composition with another RcPredicate.
    ///
    /// # Parameters
    ///
    /// * `other` - Another RcPredicate
    ///
    /// # Returns
    ///
    /// Returns the composed RcPredicate
    pub fn and(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate {
            inner: Rc::new(move |t| self_clone.test(t) && other_clone.test(t)),
            name: None,
        }
    }

    /// OR composition
    ///
    /// Performs OR composition with another RcPredicate.
    pub fn or(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate {
            inner: Rc::new(move |t| self_clone.test(t) || other_clone.test(t)),
            name: None,
        }
    }

    /// NOT composition
    ///
    /// Negates the current predicate.
    pub fn not(&self) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        RcPredicate {
            inner: Rc::new(move |t| !self_clone.test(t)),
            name: None,
        }
    }

    /// XOR composition
    ///
    /// Performs XOR composition with another RcPredicate.
    pub fn xor(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate {
            inner: Rc::new(move |t| self_clone.test(t) ^ other_clone.test(t)),
            name: None,
        }
    }

    /// NAND composition
    ///
    /// Performs NAND composition with another RcPredicate.
    pub fn nand(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate {
            inner: Rc::new(move |t| !(self_clone.test(t) && other_clone.test(t))),
            name: None,
        }
    }

    /// NOR composition
    ///
    /// Performs NOR composition with another RcPredicate.
    pub fn nor(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate {
            inner: Rc::new(move |t| !(self_clone.test(t) || other_clone.test(t))),
            name: None,
        }
    }
}

impl<T> Predicate<T> for RcPredicate<T> {
    fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    fn into_box(self) -> BoxPredicate<T>
    where
        T: 'static,
    {
        // First extract name, then wrap as Box
        let name = self.name.clone();
        BoxPredicate {
            inner: Box::new(move |x| self.test(x)),
            name,
        }
    }

    fn into_rc(self) -> RcPredicate<T>
    where
        T: 'static,
    {
        // Zero-cost: directly return itself
        self
    }

    fn into_arc(self) -> ArcPredicate<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        // Note: This method will never be called, because
        // RcPredicate<T> does not satisfy the Send + Sync constraint.
        // If attempted to call, the compiler will report an error. But we
        // must provide an implementation to satisfy the trait definition.
        unreachable!(
            "RcPredicate cannot be converted to ArcPredicate because Rc \
             is not Send + Sync"
        )
    }

    fn into_fn(self) -> impl FnMut(&T) -> bool
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t: &T| self.test(t)
    }
}

impl<T> Clone for RcPredicate<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            name: self.name.clone(),
        }
    }
}

impl<T> std::fmt::Display for RcPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RcPredicate({})",
            self.name.as_deref().unwrap_or("unnamed")
        )
    }
}

impl<T> std::fmt::Debug for RcPredicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("RcPredicate")
            .field("name", &self.name)
            .finish()
    }
}
