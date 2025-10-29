/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Types
//!
//! Provides Java-style one-time `Mutator` interface implementations for performing
//! operations that consume self and modify the input value.
//!
//! This module provides a unified `MutatorOnce` trait and a Box-based single
//! ownership implementation:
//!
//! - **`BoxMutatorOnce<T>`**: Box-based single ownership implementation for
//!   one-time use scenarios
//!
//! # Design Philosophy
//!
//! The key difference between `MutatorOnce` and `Mutator`:
//!
//! - **Mutator**: `&mut self`, can be called multiple times, uses `FnMut(&mut T)`
//! - **MutatorOnce**: `self`, can only be called once, uses `FnOnce(&mut T)`
//!
//! ## MutatorOnce vs Mutator
//!
//! | Feature | Mutator | MutatorOnce |
//! |---------|---------|-------------|
//! | **Self Parameter** | `&mut self` | `self` |
//! | **Call Count** | Multiple | Once |
//! | **Closure Type** | `FnMut(&mut T)` | `FnOnce(&mut T)` |
//! | **Use Cases** | Repeatable modifications | One-time resource transfers, init callbacks |
//!
//! # Why MutatorOnce?
//!
//! Core value of MutatorOnce:
//!
//! 1. **Store FnOnce closures**: Allows moving captured variables
//! 2. **Delayed execution**: Store in data structures, execute later
//! 3. **Resource transfer**: Suitable for scenarios requiring ownership transfer
//!
//! # Why Only Box Variant?
//!
//! - **Arc/Rc conflicts with FnOnce semantics**: FnOnce can only be called once,
//!   while shared ownership implies multiple references
//! - **Box is perfect match**: Single ownership aligns perfectly with one-time
//!   call semantics
//!
//! # Use Cases
//!
//! ## BoxMutatorOnce
//!
//! - Post-initialization callbacks (moving data)
//! - Resource transfer (moving Vec, String, etc.)
//! - One-time complex operations (requiring moved capture variables)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxMutatorOnce, MutatorOnce};
//!
//! let data = vec![1, 2, 3];
//! let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
//!     x.extend(data); // Move data
//! });
//!
//! let mut target = vec![0];
//! mutator.mutate_once(&mut target);
//! assert_eq!(target, vec![0, 1, 2, 3]);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{BoxMutatorOnce, MutatorOnce};
//!
//! let data1 = vec![1, 2];
//! let data2 = vec![3, 4];
//!
//! let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
//!     x.extend(data1);
//! })
//! .and_then(move |x: &mut Vec<i32>| {
//!     x.extend(data2);
//! });
//!
//! let mut target = vec![0];
//! chained.mutate_once(&mut target);
//! assert_eq!(target, vec![0, 1, 2, 3, 4]);
//! ```
//!
//! ## Initialization Callback
//!
//! ```rust
//! use prism3_function::{BoxMutatorOnce, MutatorOnce};
//!
//! struct Initializer {
//!     on_complete: Option<BoxMutatorOnce<Vec<i32>>>,
//! }
//!
//! impl Initializer {
//!     fn new<F>(callback: F) -> Self
//!     where
//!         F: FnOnce(&mut Vec<i32>) + 'static
//!     {
//!         Self {
//!             on_complete: Some(BoxMutatorOnce::new(callback))
//!         }
//!     }
//!
//!     fn run(mut self, data: &mut Vec<i32>) {
//!         // Execute initialization logic
//!         data.push(42);
//!
//!         // Call callback
//!         if let Some(callback) = self.on_complete.take() {
//!             callback.mutate_once(data);
//!         }
//!     }
//! }
//!
//! let data_to_add = vec![1, 2, 3];
//! let init = Initializer::new(move |x| {
//!     x.extend(data_to_add); // Move data_to_add
//! });
//!
//! let mut result = Vec::new();
//! init.run(&mut result);
//! assert_eq!(result, vec![42, 1, 2, 3]);
//! ```
//!
//! # Author
//!
//! Haixing Hu

use crate::predicates::predicate::{
    BoxPredicate,
    Predicate,
};

// ============================================================================
// 1. MutatorOnce Trait - One-time Mutator Interface
// ============================================================================

/// MutatorOnce trait - One-time mutator interface
///
/// Defines the core behavior of all one-time mutator types. Performs operations
/// that consume self and modify the input value.
///
/// This trait is automatically implemented by:
/// - All closures implementing `FnOnce(&mut T)`
/// - `BoxMutatorOnce<T>`
///
/// # Design Rationale
///
/// This trait provides a unified abstraction for one-time mutation operations.
/// The key difference from `Mutator`:
/// - `Mutator` uses `&mut self`, can be called multiple times
/// - `MutatorOnce` uses `self`, can only be called once
///
/// # Features
///
/// - **Unified Interface**: All one-time mutators share the same `mutate`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Provides `into_box` method for type conversion
/// - **Generic Programming**: Write functions that work with any one-time
///   mutator type
///
/// # Examples
///
/// ## Generic Function
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// fn apply_once<M: MutatorOnce<Vec<i32>>>(
///     mutator: M,
///     initial: Vec<i32>
/// ) -> Vec<i32> {
///     let mut val = initial;
///     mutator.mutate_once(&mut val);
///     val
/// }
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data);
/// });
/// let result = apply_once(mutator, vec![0]);
/// assert_eq!(result, vec![0, 1, 2, 3]);
/// ```
///
/// ## Type Conversion
///
/// ```rust
/// use prism3_function::MutatorOnce;
///
/// let data = vec![1, 2, 3];
/// let closure = move |x: &mut Vec<i32>| x.extend(data);
/// let box_mutator = closure.into_box_once();
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait MutatorOnce<T> {
    /// Performs the one-time mutation operation
    ///
    /// Consumes self and executes an operation on the given mutable reference.
    /// The operation typically modifies the input value or produces side effects,
    /// and can only be called once.
    ///
    /// # Parameters
    ///
    /// * `value` - A mutable reference to the value to be mutated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data = vec![1, 2, 3];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data);
    /// });
    ///
    /// let mut target = vec![0];
    /// mutator.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    /// ```
    fn mutate_once(self, value: &mut T);

    /// Converts to `BoxMutatorOnce` (consuming)
    ///
    /// Consumes `self` and returns an owned `BoxMutatorOnce<T>`. The default
    /// implementation simply wraps the consuming `mutate_once(self, &mut T)` call
    /// in a `Box<dyn FnOnce(&mut T)>`. Types that can provide a cheaper or
    /// identity conversion (for example `BoxMutatorOnce` itself) should
    /// override this method.
    ///
    /// # Note
    ///
    /// - This method consumes the source value.
    /// - Implementors may return `self` directly when `Self` is already a
    ///   `BoxMutatorOnce<T>` to avoid the extra wrapper allocation.
    fn into_box_once(self) -> BoxMutatorOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxMutatorOnce::new(move |t| self.mutate_once(t))
    }

    /// Converts to a consuming closure `FnOnce(&mut T)`
    ///
    /// Consumes `self` and returns a closure that, when invoked, calls
    /// `mutate_once(self, &mut T)`. This is the default, straightforward
    /// implementation; types that can produce a more direct function pointer
    /// or avoid additional captures may override it.
    fn into_fn_once(self) -> impl FnOnce(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        move |t| self.mutate_once(t)
    }

    /// Non-consuming adapter to `BoxMutatorOnce`
    ///
    /// Creates a `BoxMutatorOnce<T>` that does not consume `self`. The default
    /// implementation requires `Self: Clone` and clones the receiver for the
    /// stored closure; the clone is consumed when the boxed mutator is invoked.
    /// Types that can provide a zero-cost adapter (for example clonable
    /// closures) should override this method to avoid unnecessary allocations.
    fn to_box_once(&self) -> BoxMutatorOnce<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone().into_box_once()
    }

    /// Non-consuming adapter to a callable `FnOnce(&mut T)`
    ///
    /// Returns a closure that does not consume `self`. The default requires
    /// `Self: Clone` and clones `self` for the captured closure; the clone is
    /// consumed when the returned closure is invoked. Implementors may provide
    /// more efficient adapters for specific types.
    fn to_fn_once(&self) -> impl FnOnce(&mut T)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone().into_fn_once()
    }
}

// ============================================================================
// 2. BoxMutatorOnce - Single Ownership Implementation
// ============================================================================

/// BoxMutatorOnce struct
///
/// A one-time mutator implementation based on `Box<dyn FnOnce(&mut T)>` for
/// single ownership scenarios. This is the only MutatorOnce implementation type
/// because FnOnce conflicts with shared ownership semantics.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes self on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Move Semantics**: Can capture and move variables
/// - **Method Chaining**: Compose multiple operations via `and_then`
///
/// # Use Cases
///
/// Choose `BoxMutatorOnce` when:
/// - Need to store FnOnce closures (with moved captured variables)
/// - One-time resource transfer operations
/// - Post-initialization callbacks
/// - Complex operations requiring ownership transfer
///
/// # Performance
///
/// `BoxMutatorOnce` performance characteristics:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Why No Arc/Rc Variants?
///
/// FnOnce can only be called once, which conflicts with Arc/Rc shared ownership
/// semantics:
/// - Arc/Rc implies multiple owners might need to call
/// - FnOnce is consumed after calling, cannot be called again
/// - This semantic incompatibility makes Arc/Rc variants meaningless
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data); // Move data
/// });
///
/// let mut target = vec![0];
/// mutator.mutate_once(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data1);
/// })
/// .and_then(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
///
/// let mut target = vec![0];
/// chained.mutate_once(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3, 4]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxMutatorOnce<T> {
    function: Box<dyn FnOnce(&mut T)>,
}

impl<T> BoxMutatorOnce<T>
where
    T: 'static,
{
    /// Creates a new BoxMutatorOnce
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatorOnce<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data = String::from("world");
    /// let mutator = BoxMutatorOnce::new(move |x: &mut String| {
    ///     x.push_str(" ");
    ///     x.push_str(&data); // Move data
    /// });
    ///
    /// let mut target = String::from("hello");
    /// mutator.mutate_once(&mut target);
    /// assert_eq!(target, "hello world");
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut T) + 'static,
    {
        BoxMutatorOnce {
            function: Box::new(f),
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
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let noop = BoxMutatorOnce::<i32>::noop();
    /// let mut value = 42;
    /// noop.mutate_once(&mut value);
    /// assert_eq!(value, 42); // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxMutatorOnce::new(|_| {})
    }

    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation. **Note:
    ///   This parameter is passed by value and will transfer ownership.** Since
    ///   `BoxMutatorOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: &mut T|`
    ///   - A `BoxMutatorOnce<T>`
    ///   - Any type implementing `MutatorOnce<T>`
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    /// let data3 = vec![5, 6];
    ///
    /// let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data1);
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    /// })
    /// .and_then(move |x: &mut Vec<i32>| {
    ///     x.extend(data3);
    /// });
    ///
    /// let mut target = vec![0];
    /// chained.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: MutatorOnce<T> + 'static,
    {
        let first = self.function;
        BoxMutatorOnce::new(move |t| {
            first(t);
            next.mutate_once(t);
        })
    }

    /// Creates a conditional mutator
    ///
    /// Returns a mutator that only executes when a predicate is satisfied.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T| -> bool`
    ///   - A function pointer: `fn(&T) -> bool`
    ///   - A `BoxPredicate<T>`
    ///   - An `RcPredicate<T>`
    ///   - An `ArcPredicate<T>`
    ///   - Any type implementing `Predicate<T>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data = vec![1, 2, 3];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data);
    /// });
    /// let conditional = mutator.when(|x: &Vec<i32>| !x.is_empty());
    ///
    /// let mut target = vec![0];
    /// conditional.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    ///
    /// let mut empty = Vec::new();
    /// let data2 = vec![4, 5];
    /// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    /// });
    /// let conditional2 = mutator2.when(|x: &Vec<i32>| x.len() > 5);
    /// conditional2.mutate_once(&mut empty);
    /// assert_eq!(empty, Vec::<i32>::new()); // Unchanged
    /// ```
    ///
    /// ## Preserving predicate with clone
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce, RcPredicate};
    /// use prism3_function::predicate::Predicate;
    ///
    /// let data = vec![1, 2, 3];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data);
    /// });
    /// let predicate = RcPredicate::new(|x: &Vec<i32>| !x.is_empty());
    ///
    /// // Clone to preserve original predicate
    /// let conditional = mutator.when(predicate.clone());
    ///
    /// let mut target = vec![0];
    /// conditional.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    ///
    /// // Original predicate still usable
    /// assert!(predicate.test(&vec![1, 2]));
    /// ```
    ///
    /// ## Using composed predicate
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    /// use prism3_function::predicate::{Predicate, FnPredicateOps};
    ///
    /// let pred = (|x: &Vec<i32>| !x.is_empty())
    ///     .and(|x: &Vec<i32>| x.len() < 10);
    /// let data = vec![1, 2, 3];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data);
    /// });
    /// let conditional = mutator.when(pred);
    ///
    /// let mut target = vec![0];
    /// conditional.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]);
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalMutatorOnce<T>
    where
        P: Predicate<T> + 'static,
    {
        BoxConditionalMutatorOnce {
            mutator: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T> MutatorOnce<T> for BoxMutatorOnce<T> {
    fn mutate_once(self, value: &mut T) {
        (self.function)(value)
    }

    fn into_box_once(self) -> BoxMutatorOnce<T>
    where
        T: 'static,
    {
        self
    }

    fn into_fn_once(self) -> impl FnOnce(&mut T)
    where
        T: 'static,
    {
        move |t| (self.function)(t)
    }
}

// ============================================================================
// 3. BoxConditionalMutatorOnce - Box-based Conditional Mutator
// ============================================================================

/// BoxConditionalMutatorOnce struct
///
/// A conditional one-time mutator that only executes when a predicate is satisfied.
/// Uses `BoxMutatorOnce` and `BoxPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxMutatorOnce::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only mutates when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements MutatorOnce**: Can be used anywhere a `MutatorOnce` is expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data);
/// });
/// let conditional = mutator.when(|x: &Vec<i32>| !x.is_empty());
///
/// let mut target = vec![0];
/// conditional.mutate_once(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]); // Executed
///
/// let mut empty = Vec::new();
/// let data2 = vec![4, 5];
/// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
/// let conditional2 = mutator2.when(|x: &Vec<i32>| x.len() > 5);
/// conditional2.mutate_once(&mut empty);
/// assert_eq!(empty, Vec::<i32>::new()); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data1 = vec![1, 2, 3];
/// let data2 = vec![99];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data1);
/// })
/// .when(|x: &Vec<i32>| !x.is_empty())
/// .or_else(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
///
/// let mut target = vec![0];
/// mutator.mutate_once(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]); // when branch executed
///
/// let data3 = vec![4, 5];
/// let data4 = vec![99];
/// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data3);
/// })
/// .when(|x: &Vec<i32>| x.is_empty())
/// .or_else(move |x: &mut Vec<i32>| {
///     x.extend(data4);
/// });
///
/// let mut target2 = vec![0];
/// mutator2.mutate_once(&mut target2);
/// assert_eq!(target2, vec![0, 99]); // or_else branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalMutatorOnce<T> {
    mutator: BoxMutatorOnce<T>,
    predicate: BoxPredicate<T>,
}

impl<T> MutatorOnce<T> for BoxConditionalMutatorOnce<T>
where
    T: 'static,
{
    fn mutate_once(self, value: &mut T) {
        if self.predicate.test(value) {
            self.mutator.mutate_once(value);
        }
    }

    fn into_box_once(self) -> BoxMutatorOnce<T> {
        let pred = self.predicate;
        let mutator = self.mutator;
        BoxMutatorOnce::new(move |t| {
            if pred.test(t) {
                mutator.mutate_once(t);
            }
        })
    }

    fn into_fn_once(self) -> impl FnOnce(&mut T) {
        let pred = self.predicate;
        let mutator = self.mutator;
        move |t: &mut T| {
            if pred.test(t) {
                mutator.mutate_once(t);
            }
        }
    }
}

impl<T> BoxConditionalMutatorOnce<T>
where
    T: 'static,
{
    /// Chains another mutator in sequence
    ///
    /// Combines the current conditional mutator with another mutator into a new
    /// mutator. The current conditional mutator executes first, followed by the
    /// next mutator.
    ///
    /// # Parameters
    ///
    /// * `next` - The next mutator to execute. **Note: This parameter is passed
    ///   by value and will transfer ownership.** Since `BoxMutatorOnce` cannot
    ///   be cloned, the parameter will be consumed. Can be:
    ///   - A closure: `|x: &mut T|`
    ///   - A `BoxMutatorOnce<T>`
    ///   - Any type implementing `MutatorOnce<T>`
    ///
    /// # Returns
    ///
    /// Returns a new `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data1 = vec![1, 2];
    /// let cond1 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data1);
    /// }).when(|x: &Vec<i32>| !x.is_empty());
    ///
    /// let data2 = vec![3, 4];
    /// let cond2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    /// }).when(|x: &Vec<i32>| x.len() < 10);
    ///
    /// // Both cond1 and cond2 are moved and consumed
    /// let chained = cond1.and_then(cond2);
    ///
    /// let mut target = vec![0];
    /// chained.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4]);
    /// // cond1.mutate_once(&mut target); // Would not compile - moved
    /// // cond2.mutate_once(&mut target); // Would not compile - moved
    /// ```
    pub fn and_then<C>(self, next: C) -> BoxMutatorOnce<T>
    where
        C: MutatorOnce<T> + 'static,
    {
        let first = self;
        BoxMutatorOnce::new(move |t| {
            first.mutate_once(t);
            next.mutate_once(t);
        })
    }

    /// Adds an else branch
    ///
    /// Executes the original mutator when the condition is satisfied, otherwise
    /// executes else_mutator.
    ///
    /// # Parameters
    ///
    /// * `else_mutator` - The mutator for the else branch. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since `BoxMutatorOnce`
    ///   cannot be cloned, the parameter will be consumed. Can be:
    ///   - A closure: `|x: &mut T|`
    ///   - A `BoxMutatorOnce<T>`
    ///   - Any type implementing `MutatorOnce<T>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, BoxMutatorOnce};
    ///
    /// let data1 = vec![1, 2, 3];
    /// let data2 = vec![99];
    /// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data1);
    /// })
    /// .when(|x: &Vec<i32>| !x.is_empty())
    /// .or_else(move |x: &mut Vec<i32>| {
    ///     x.extend(data2);
    /// });
    ///
    /// let mut target = vec![0];
    /// mutator.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3]); // Condition satisfied, execute when branch
    ///
    /// let data3 = vec![4, 5];
    /// let data4 = vec![99];
    /// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
    ///     x.extend(data3);
    /// })
    /// .when(|x: &Vec<i32>| x.is_empty())
    /// .or_else(move |x: &mut Vec<i32>| {
    ///     x.extend(data4);
    /// });
    ///
    /// let mut target2 = vec![0];
    /// mutator2.mutate_once(&mut target2);
    /// assert_eq!(target2, vec![0, 99]); // Condition not satisfied, execute or_else branch
    /// ```
    pub fn or_else<C>(self, else_mutator: C) -> BoxMutatorOnce<T>
    where
        C: MutatorOnce<T> + 'static,
    {
        let pred = self.predicate;
        let then_mut = self.mutator;
        BoxMutatorOnce::new(move |t| {
            if pred.test(t) {
                then_mut.mutate_once(t);
            } else {
                else_mutator.mutate_once(t);
            }
        })
    }
}

// ============================================================================
// 4. Implement MutatorOnce trait for closures
// ============================================================================

impl<T, F> MutatorOnce<T> for F
where
    F: FnOnce(&mut T),
{
    fn mutate_once(self, value: &mut T) {
        self(value)
    }

    fn into_box_once(self) -> BoxMutatorOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxMutatorOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(&mut T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }

    // Provide specialized non-consuming conversions for closures that
    // implement `Clone`. Many simple closures are zero-sized and `Clone`,
    // allowing non-consuming adapters to be cheaply produced.
    fn to_box_once(&self) -> BoxMutatorOnce<T>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        BoxMutatorOnce::new(move |t| cloned.mutate_once(t))
    }

    fn to_fn_once(&self) -> impl FnOnce(&mut T)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        self.clone()
    }
}

// ============================================================================
// 4. Provide extension methods for closures
// ============================================================================

/// Extension trait providing one-time mutator composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures that
/// implement `FnOnce(&mut T)`, enabling direct method chaining on closures
/// without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxMutatorOnce**: Composition results are `BoxMutatorOnce<T>`
///   for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnOnce(&mut T)` closures get these
///   methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{MutatorOnce, FnMutatorOnceOps};
///
/// let data1 = vec![1, 2];
/// let data2 = vec![3, 4];
///
/// let chained = (move |x: &mut Vec<i32>| x.extend(data1))
///     .and_then(move |x: &mut Vec<i32>| x.extend(data2));
///
/// let mut target = vec![0];
/// chained.mutate_once(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3, 4]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnMutatorOnceOps<T>: FnOnce(&mut T) + Sized {
    /// Chains another mutator in sequence
    ///
    /// Returns a new mutator that first executes the current operation, then
    /// executes the next operation. Consumes the current closure and returns
    /// `BoxMutatorOnce<T>`.
    ///
    /// # Parameters
    ///
    /// * `next` - The mutator to execute after the current operation. **Note: This
    ///   parameter is passed by value and will transfer ownership.** Since
    ///   `BoxMutatorOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: &mut T|`
    ///   - A `BoxMutatorOnce<T>`
    ///   - Any type implementing `MutatorOnce<T>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxMutatorOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{MutatorOnce, FnMutatorOnceOps};
    ///
    /// let data1 = vec![1, 2];
    /// let data2 = vec![3, 4];
    ///
    /// // Both closures are moved and consumed
    /// let chained = (move |x: &mut Vec<i32>| x.extend(data1))
    ///     .and_then(move |x: &mut Vec<i32>| x.extend(data2));
    ///
    /// let mut target = vec![0];
    /// chained.mutate_once(&mut target);
    /// assert_eq!(target, vec![0, 1, 2, 3, 4]);
    /// // The original closures are consumed and no longer usable
    /// ```
    fn and_then<C>(self, next: C) -> BoxMutatorOnce<T>
    where
        Self: 'static,
        C: MutatorOnce<T> + 'static,
        T: 'static,
    {
        BoxMutatorOnce::new(move |t| {
            self(t);
            next.mutate_once(t);
        })
    }
}

/// Implements FnMutatorOnceOps for all closure types
impl<T, F> FnMutatorOnceOps<T> for F where F: FnOnce(&mut T) {}
