/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Comparator Abstraction
//!
//! Provides a Rust implementation similar to Java's `Comparator` interface
//! for comparison operations and chaining.
//!
//! ## Design Overview
//!
//! This module adopts the **Trait + Multiple Implementations** design
//! pattern, which is the most flexible and elegant approach for
//! implementing comparators in Rust. It achieves a perfect balance
//! between semantic clarity, type safety, and API flexibility.
//!
//! ### Core Components
//!
//! 1. **`Comparator<T>` trait**: A minimalist unified interface that only
//!    defines the core `compare` method and type conversion methods
//!    (`into_*`). It does NOT include chaining methods like
//!    `then_comparing`, etc.
//!
//! 2. **Three Concrete Struct Implementations**:
//!    - [`BoxComparator<T>`]: Box-based single ownership implementation
//!      for one-time use scenarios
//!    - [`ArcComparator<T>`]: Arc-based thread-safe shared ownership
//!      implementation for multi-threaded scenarios
//!    - [`RcComparator<T>`]: Rc-based single-threaded shared ownership
//!      implementation for single-threaded reuse
//!
//! 3. **Specialized Composition Methods**: Each struct implements its own
//!    inherent methods (`reversed`, `then_comparing`, etc.) that return
//!    the same concrete type, preserving their specific characteristics
//!    (e.g., `ArcComparator` compositions remain `ArcComparator` and stay
//!    cloneable and thread-safe).
//!
//! 4. **Extension Trait for Closures**: The `FnComparatorOps<T>`
//!    extension trait provides composition methods for all closures and
//!    function pointers, returning `BoxComparator<T>` to initiate method
//!    chaining.
//!
//! 5. **Unified Trait Implementation**: All closures and the three
//!    structs implement the `Comparator<T>` trait, enabling them to be
//!    handled uniformly by generic functions.
//!
//! ## Ownership Model Coverage
//!
//! The three implementations correspond to three typical ownership
//! scenarios:
//!
//! | Type | Ownership | Clonable | Thread-Safe | API | Use Case |
//! |:-----|:----------|:---------|:------------|:----|:---------|
//! | [`BoxComparator`] | Single | ❌ | ❌ | consumes `self` | One-time |
//! | [`ArcComparator`] | Shared | ✅ | ✅ | borrows `&self` | Multi-thread |
//! | [`RcComparator`] | Shared | ✅ | ❌ | borrows `&self` | Single-thread |
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
//! use prism3_function::comparator::{Comparator, ArcComparator};
//! use std::cmp::Ordering;
//!
//! let arc_cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
//! let another = ArcComparator::new(|a: &i32, b: &i32| b.cmp(a));
//!
//! // Composition returns ArcComparator, preserving clonability and
//! // thread-safety
//! let combined = arc_cmp.then_comparing(&another);
//! let cloned = combined.clone();  // ✅ Still cloneable
//!
//! // Original comparators remain usable
//! assert_eq!(arc_cmp.compare(&5, &3), Ordering::Greater);
//! ```
//!
//! ### 2. Elegant API without Explicit Cloning
//!
//! `ArcComparator` and `RcComparator` use `&self` in their composition
//! methods, providing a natural experience without requiring explicit
//! `.clone()` calls:
//!
//! ```rust
//! use prism3_function::comparator::{Comparator, ArcComparator};
//!
//! let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
//!
//! // No need for explicit clone()
//! let reversed = cmp.reversed();
//! let chained = cmp.then_comparing(&ArcComparator::new(|a, b| b.cmp(a)));
//!
//! // cmp is still available
//! cmp.compare(&1, &2);
//! ```
//!
//! ### 3. Efficient Closure Composition
//!
//! The `FnComparatorOps` extension trait allows direct composition on
//! closures:
//!
//! ```rust
//! use prism3_function::comparator::{Comparator, FnComparatorOps};
//! use std::cmp::Ordering;
//!
//! let cmp = (|a: &i32, b: &i32| a.cmp(b))
//!     .reversed()
//!     .then_comparing(|a: &i32, b: &i32| b.cmp(a));
//!
//! assert_eq!(cmp.compare(&5, &3), Ordering::Less);
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Comparison
//!
//! ```rust
//! use prism3_function::comparator::{Comparator, BoxComparator};
//! use std::cmp::Ordering;
//!
//! let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
//! assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
//! ```
//!
//! ### Reversed Comparison
//!
//! ```rust
//! use prism3_function::comparator::{Comparator, BoxComparator};
//! use std::cmp::Ordering;
//!
//! let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
//! let rev = cmp.reversed();
//! assert_eq!(rev.compare(&5, &3), Ordering::Less);
//! ```
//!
//! ### Chained Comparison
//!
//! ```rust
//! use prism3_function::comparator::{Comparator, BoxComparator};
//! use std::cmp::Ordering;
//!
//! #[derive(Debug)]
//! struct Person {
//!     name: String,
//!     age: i32,
//! }
//!
//! let by_name = BoxComparator::new(|a: &Person, b: &Person| {
//!     a.name.cmp(&b.name)
//! });
//! let by_age = BoxComparator::new(|a: &Person, b: &Person| {
//!     a.age.cmp(&b.age)
//! });
//! let cmp = by_name.then_comparing(by_age);
//!
//! let p1 = Person { name: "Alice".to_string(), age: 30 };
//! let p2 = Person { name: "Alice".to_string(), age: 25 };
//! assert_eq!(cmp.compare(&p1, &p2), Ordering::Greater);
//! ```
//!
//! ## Author
//!
//! Haixing Hu

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;

/// A trait for comparison operations.
///
/// This trait defines the core comparison operation and conversion methods.
/// It does NOT include composition methods like `reversed` or
/// `then_comparing` to maintain a clean separation between the trait
/// interface and specialized implementations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::{Comparator, BoxComparator};
/// use std::cmp::Ordering;
///
/// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Comparator<T> {
    /// Compares two values and returns an ordering.
    ///
    /// # Parameters
    ///
    /// * `a` - The first value to compare
    /// * `b` - The second value to compare
    ///
    /// # Returns
    ///
    /// An `Ordering` indicating whether `a` is less than, equal to, or
    /// greater than `b`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    /// assert_eq!(cmp.compare(&3, &5), Ordering::Less);
    /// assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    /// ```
    fn compare(&self, a: &T, b: &T) -> Ordering;

    /// Converts this comparator into a `BoxComparator`.
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` wrapping this comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, BoxComparator};
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let boxed = cmp.into_box();
    /// ```
    fn into_box(self) -> BoxComparator<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxComparator::new(self)
    }

    /// Converts this comparator into an `ArcComparator`.
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` wrapping this comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, ArcComparator};
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let arc = cmp.into_arc();
    /// ```
    fn into_arc(self) -> ArcComparator<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: 'static,
    {
        ArcComparator::new(self)
    }

    /// Converts this comparator into an `RcComparator`.
    ///
    /// # Returns
    ///
    /// A new `RcComparator` wrapping this comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, RcComparator};
    ///
    /// let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rc = cmp.into_rc();
    /// ```
    fn into_rc(self) -> RcComparator<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcComparator::new(self)
    }
}

/// Blanket implementation of `Comparator` for all closures and function
/// pointers.
///
/// This allows any closure or function with the signature
/// `Fn(&T, &T) -> Ordering` to be used as a comparator.
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::Comparator;
/// use std::cmp::Ordering;
///
/// let cmp = |a: &i32, b: &i32| a.cmp(b);
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// ```
impl<T, F> Comparator<T> for F
where
    F: Fn(&T, &T) -> Ordering,
{
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self(a, b)
    }
}

/// A boxed comparator with single ownership.
///
/// `BoxComparator` wraps a comparator function in a `Box`, providing single
/// ownership semantics. It is not cloneable and consumes `self` in
/// composition operations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::{Comparator, BoxComparator};
/// use std::cmp::Ordering;
///
/// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxComparator<T> {
    inner: Box<dyn Comparator<T>>,
}

impl<T: 'static> BoxComparator<T> {
    /// Creates a new `BoxComparator` from a comparator.
    ///
    /// # Parameters
    ///
    /// * `comparator` - The comparator to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::BoxComparator;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// ```
    pub fn new<C>(comparator: C) -> Self
    where
        C: Comparator<T> + 'static,
    {
        Self {
            inner: Box::new(comparator),
        }
    }

    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rev = cmp.reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// ```
    pub fn reversed(self) -> Self {
        Self::new(ReverseComparator::new(self))
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to use for tie-breaking
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_name = BoxComparator::new(|a: &Person, b: &Person| {
    ///     a.name.cmp(&b.name)
    /// });
    /// let by_age = BoxComparator::new(|a: &Person, b: &Person| {
    ///     a.age.cmp(&b.age)
    /// });
    /// let cmp = by_name.then_comparing(by_age);
    ///
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Alice".to_string(), age: 25 };
    /// assert_eq!(cmp.compare(&p1, &p2), Ordering::Greater);
    /// ```
    pub fn then_comparing(self, other: Self) -> Self {
        Self::new(ChainedComparator::new(self, other))
    }

    /// Returns a comparator that compares values by a key extracted by the
    /// given function.
    ///
    /// # Parameters
    ///
    /// * `key_fn` - A function that extracts a comparable key from values
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that compares by the extracted key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_age = BoxComparator::comparing(|p: &Person| &p.age);
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 25 };
    /// assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    /// ```
    pub fn comparing<K, F>(key_fn: F) -> Self
    where
        K: Ord,
        F: Fn(&T) -> &K + 'static,
    {
        Self::new(move |a: &T, b: &T| key_fn(a).cmp(key_fn(b)))
    }
}

impl<T> Comparator<T> for BoxComparator<T> {
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.inner.compare(a, b)
    }
}

/// An Arc-based thread-safe comparator with shared ownership.
///
/// `ArcComparator` wraps a comparator function in an `Arc`, providing
/// thread-safe shared ownership semantics. It is cloneable and uses `&self`
/// in composition operations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::{Comparator, ArcComparator};
/// use std::cmp::Ordering;
///
/// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// let cloned = cmp.clone();
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Clone)]
pub struct ArcComparator<T> {
    inner: Arc<dyn Comparator<T> + Send + Sync>,
}

impl<T: 'static> ArcComparator<T> {
    /// Creates a new `ArcComparator` from a comparator.
    ///
    /// # Parameters
    ///
    /// * `comparator` - The comparator to wrap
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::ArcComparator;
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// ```
    pub fn new<C>(comparator: C) -> Self
    where
        C: Comparator<T> + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(comparator),
        }
    }

    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rev = cmp.reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// assert_eq!(cmp.compare(&5, &3), Ordering::Greater); // cmp still works
    /// ```
    pub fn reversed(&self) -> Self {
        let inner = self.inner.clone();
        Self {
            inner: Arc::new(move |a: &T, b: &T| inner.compare(b, a)),
        }
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to use for tie-breaking
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp1 = ArcComparator::new(|a: &i32, b: &i32| {
    ///     (a % 2).cmp(&(b % 2))
    /// });
    /// let cmp2 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let chained = cmp1.then_comparing(&cmp2);
    /// assert_eq!(chained.compare(&4, &2), Ordering::Greater);
    /// ```
    pub fn then_comparing(&self, other: &Self) -> Self {
        let first = self.inner.clone();
        let second = other.inner.clone();
        Self {
            inner: Arc::new(move |a: &T, b: &T| match first.compare(a, b) {
                Ordering::Equal => second.compare(a, b),
                other => other,
            }),
        }
    }

    /// Returns a comparator that compares values by a key extracted by the
    /// given function.
    ///
    /// # Parameters
    ///
    /// * `key_fn` - A function that extracts a comparable key from values
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` that compares by the extracted key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_age = ArcComparator::comparing(|p: &Person| &p.age);
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 25 };
    /// assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    /// ```
    pub fn comparing<K, F>(key_fn: F) -> Self
    where
        K: Ord,
        F: Fn(&T) -> &K + Send + Sync + 'static,
    {
        Self::new(move |a: &T, b: &T| key_fn(a).cmp(key_fn(b)))
    }
}

impl<T> Comparator<T> for ArcComparator<T> {
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.inner.compare(a, b)
    }
}

/// An Rc-based single-threaded comparator with shared ownership.
///
/// `RcComparator` wraps a comparator function in an `Rc`, providing
/// single-threaded shared ownership semantics. It is cloneable and uses
/// `&self` in composition operations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::{Comparator, RcComparator};
/// use std::cmp::Ordering;
///
/// let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// let cloned = cmp.clone();
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Clone)]
pub struct RcComparator<T> {
    inner: Rc<dyn Comparator<T>>,
}

impl<T: 'static> RcComparator<T> {
    /// Creates a new `RcComparator` from a comparator.
    ///
    /// # Parameters
    ///
    /// * `comparator` - The comparator to wrap
    ///
    /// # Returns
    ///
    /// A new `RcComparator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::RcComparator;
    ///
    /// let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// ```
    pub fn new<C>(comparator: C) -> Self
    where
        C: Comparator<T> + 'static,
    {
        Self {
            inner: Rc::new(comparator),
        }
    }

    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `RcComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, RcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rev = cmp.reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// assert_eq!(cmp.compare(&5, &3), Ordering::Greater); // cmp still works
    /// ```
    pub fn reversed(&self) -> Self {
        let inner = self.inner.clone();
        Self {
            inner: Rc::new(move |a: &T, b: &T| inner.compare(b, a)),
        }
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to use for tie-breaking
    ///
    /// # Returns
    ///
    /// A new `RcComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, RcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp1 = RcComparator::new(|a: &i32, b: &i32| {
    ///     (a % 2).cmp(&(b % 2))
    /// });
    /// let cmp2 = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let chained = cmp1.then_comparing(&cmp2);
    /// assert_eq!(chained.compare(&4, &2), Ordering::Greater);
    /// ```
    pub fn then_comparing(&self, other: &Self) -> Self {
        let first = self.inner.clone();
        let second = other.inner.clone();
        Self {
            inner: Rc::new(move |a: &T, b: &T| match first.compare(a, b) {
                Ordering::Equal => second.compare(a, b),
                other => other,
            }),
        }
    }

    /// Returns a comparator that compares values by a key extracted by the
    /// given function.
    ///
    /// # Parameters
    ///
    /// * `key_fn` - A function that extracts a comparable key from values
    ///
    /// # Returns
    ///
    /// A new `RcComparator` that compares by the extracted key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, RcComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_age = RcComparator::comparing(|p: &Person| &p.age);
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 25 };
    /// assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    /// ```
    pub fn comparing<K, F>(key_fn: F) -> Self
    where
        K: Ord,
        F: Fn(&T) -> &K + 'static,
    {
        Self::new(move |a: &T, b: &T| key_fn(a).cmp(key_fn(b)))
    }
}

impl<T> Comparator<T> for RcComparator<T> {
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.inner.compare(a, b)
    }
}

/// Extension trait providing composition methods for closures and function
/// pointers.
///
/// This trait is automatically implemented for all closures and function
/// pointers with the signature `Fn(&T, &T) -> Ordering`, allowing them to
/// be composed directly without explicit wrapping.
///
/// # Examples
///
/// ```rust
/// use prism3_function::comparator::{Comparator, FnComparatorOps};
/// use std::cmp::Ordering;
///
/// let cmp = (|a: &i32, b: &i32| a.cmp(b))
///     .reversed()
///     .then_comparing(BoxComparator::new(|a, b| b.cmp(a)));
///
/// assert_eq!(cmp.compare(&5, &3), Ordering::Less);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnComparatorOps<T>: Fn(&T, &T) -> Ordering + Sized {
    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, FnComparatorOps};
    /// use std::cmp::Ordering;
    ///
    /// let rev = (|a: &i32, b: &i32| a.cmp(b)).reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// ```
    fn reversed(self) -> BoxComparator<T>
    where
        Self: 'static,
        T: 'static,
    {
        BoxComparator::new(self).reversed()
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to use for tie-breaking
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::comparator::{Comparator, FnComparatorOps,
    ///                                   BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = (|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)))
    ///     .then_comparing(BoxComparator::new(|a, b| a.cmp(b)));
    /// assert_eq!(cmp.compare(&4, &2), Ordering::Greater);
    /// ```
    fn then_comparing(self, other: BoxComparator<T>) -> BoxComparator<T>
    where
        Self: 'static,
        T: 'static,
    {
        BoxComparator::new(self).then_comparing(other)
    }
}

impl<T, F> FnComparatorOps<T> for F where F: Fn(&T, &T) -> Ordering {}

/// A comparator that reverses the ordering of another comparator.
///
/// This struct is used internally by the `reversed` method.
struct ReverseComparator<T, C> {
    inner: C,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, C> ReverseComparator<T, C>
where
    C: Comparator<T>,
{
    fn new(inner: C) -> Self {
        Self {
            inner,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, C> Comparator<T> for ReverseComparator<T, C>
where
    C: Comparator<T>,
{
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.inner.compare(b, a)
    }
}

/// A comparator that chains two comparators.
///
/// This struct is used internally by the `then_comparing` method. It first
/// compares using the first comparator, and if the result is `Equal`, it
/// uses the second comparator.
struct ChainedComparator<T, A, B> {
    first: A,
    second: B,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, A, B> ChainedComparator<T, A, B>
where
    A: Comparator<T>,
    B: Comparator<T>,
{
    fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, A, B> Comparator<T> for ChainedComparator<T, A, B>
where
    A: Comparator<T>,
    B: Comparator<T>,
{
    fn compare(&self, a: &T, b: &T) -> Ordering {
        match self.first.compare(a, b) {
            Ordering::Equal => self.second.compare(a, b),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_as_comparator() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_box_comparator_basic() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_box_comparator_reversed() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
    }

    #[test]
    fn test_arc_comparator_clone() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cloned = cmp.clone();
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_arc_comparator_reversed() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(cmp2);
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        assert_eq!(chained.compare(&3, &1), Ordering::Greater);
    }

    #[test]
    fn test_fn_ops_reversed() {
        let rev = (|a: &i32, b: &i32| a.cmp(b)).reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
    }
}
