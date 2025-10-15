/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Supplier Types
//!
//! Provides Java-like `Supplier` interface implementations for lazy
//! value production without input parameters.
//!
//! # Overview
//!
//! This module implements **Approach 3** from the design comparison:
//! a unified `Supplier` trait with three concrete implementations,
//! each optimized for different ownership and concurrency scenarios.
//!
//! Similar to Java's `Supplier<T>` interface, a supplier lazily
//! generates values without taking any input. Unlike `Predicate`
//! which typically has no side effects, suppliers commonly maintain
//! internal state (counters, sequences, etc.) and require mutable
//! access (`&mut self`).
//!
//! # Key Design Differences from Predicate
//!
//! | Aspect | Predicate | Supplier |
//! |--------|-----------|----------|
//! | Signature | `Fn(&T) -> bool` | `FnMut() -> T` |
//! | Mutability | Immutable (`&self`) | Mutable (`&mut self`) |
//! | Side Effects | Usually none | Usually has state changes |
//! | Sharing | Easy (`Arc<dyn Fn>`) | Complex (needs `Mutex`) |
//! | Common Use | Validation, filtering | Value generation, sequences |
//!
//! # Three Concrete Implementations
//!
//! - **`BoxSupplier<T>`**: Box-based single ownership for one-time
//!   use. Zero overhead, cannot be cloned. Best for builder patterns
//!   and simple sequential value generation.
//!
//! - **`ArcSupplier<T>`**: Thread-safe shared ownership using
//!   `Arc<Mutex<>>`. Can be cloned and sent across threads. Higher
//!   overhead due to locking. Best for multi-threaded scenarios.
//!
//! - **`RcSupplier<T>`**: Single-threaded shared ownership using
//!   `Rc<RefCell<>>`. Can be cloned but not sent across threads.
//!   Lower overhead than `ArcSupplier`. Best for single-threaded
//!   reuse.
//!
//! # Method Chaining Differences
//!
//! - `BoxSupplier`: Methods consume `self` (ownership transfer)
//! - `ArcSupplier` & `RcSupplier`: Methods borrow `&self` (can reuse)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use prism3_function::{BoxSupplier, Supplier};
//!
//! // Stateful counter
//! let mut counter = 0;
//! let mut supplier = BoxSupplier::new(move || {
//!     counter += 1;
//!     counter
//! });
//!
//! assert_eq!(supplier.get(), 1);
//! assert_eq!(supplier.get(), 2);
//! assert_eq!(supplier.get(), 3);
//! ```
//!
//! ## Method Chaining
//!
//! ```rust
//! use prism3_function::{BoxSupplier, Supplier};
//!
//! let mut supplier = BoxSupplier::new(|| 10)
//!     .map(|x| x * 2)
//!     .filter(|x| *x > 15);
//!
//! assert_eq!(supplier.get(), Some(20));
//! ```
//!
//! ## Thread-safe Sharing
//!
//! ```rust
//! use prism3_function::{ArcSupplier, Supplier};
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//!
//! let counter = Arc::new(Mutex::new(0));
//! let counter_clone = Arc::clone(&counter);
//!
//! let supplier = ArcSupplier::new(move || {
//!     let mut c = counter_clone.lock().unwrap();
//!     *c += 1;
//!     *c
//! });
//!
//! let mut s1 = supplier.clone();
//! let mut s2 = supplier.clone();
//!
//! let handle = thread::spawn(move || s2.get());
//! let value1 = s1.get();
//! let value2 = handle.join().unwrap();
//!
//! // Both get different sequential values
//! assert_ne!(value1, value2);
//! ```
//!
//! ## Memoization
//!
//! ```rust
//! use prism3_function::{BoxSupplier, Supplier};
//!
//! let mut expensive = BoxSupplier::new(|| {
//!     // Expensive computation
//!     42
//! }).memoize();
//!
//! let v1 = expensive.get(); // Computes
//! let v2 = expensive.get(); // Returns cached value
//! assert_eq!(v1, v2);
//! ```
//!
//! # Design Rationale
//!
//! This implementation chooses Approach 3 (trait + multiple
//! implementations) to provide:
//!
//! 1. **Clear semantics**: Type names directly indicate ownership
//!    model
//! 2. **Unified interface**: All types implement `Supplier<T>` trait
//! 3. **Performance options**: Choose the right type for your needs
//! 4. **Type safety**: Compiler enforces correct usage patterns
//!
//! # Performance Considerations
//!
//! | Type | Overhead | Cloneable | Thread-safe | Best For |
//! |------|----------|-----------|-------------|----------|
//! | `BoxSupplier` | None | No | No | One-time use |
//! | `RcSupplier` | `RefCell` | Yes | No | Single-thread reuse |
//! | `ArcSupplier` | `Mutex` | Yes | Yes | Multi-thread |
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ==========================================================================
// Supplier Trait - Unified Supplier Interface
// ==========================================================================

/// Unified supplier interface trait
///
/// Defines the core behavior of all supplier types. Similar to Java's
/// `Supplier<T>` interface, it lazily produces values without taking
/// any input parameters.
///
/// This trait is automatically implemented by all `FnMut() -> T`
/// closures and serves as the foundation for the three concrete
/// implementations: `BoxSupplier`, `ArcSupplier`, and `RcSupplier`.
///
/// # Key Characteristics
///
/// - **No input parameters**: Pure value generation
/// - **Mutable access**: Uses `&mut self` to allow state changes
/// - **Type conversions**: Provides `into_box()`, `into_arc()`,
///   `into_rc()` for flexible type conversion
/// - **State management**: Commonly used for counters, sequences,
///   and lazy initialization
///
/// # Difference from Predicate
///
/// Unlike `Predicate` which typically has no side effects and uses
/// immutable `&self`, `Supplier` commonly maintains internal state
/// and requires `&mut self`. This makes sharing suppliers more
/// complex, requiring `Mutex` (for `Arc`) or `RefCell` (for `Rc`).
///
/// # Implementation Notes
///
/// All closures implementing `FnMut() -> T` automatically implement
/// this trait, allowing seamless integration with both raw closures
/// and wrapped supplier types.
///
/// # Examples
///
/// ## Generic Function Accepting Any Supplier
///
/// ```rust
/// use prism3_function::{Supplier, BoxSupplier};
///
/// fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
///     supplier.get()
/// }
///
/// // Works with BoxSupplier
/// let mut box_sup = BoxSupplier::new(|| 42);
/// assert_eq!(use_any_supplier(&mut box_sup), 42);
///
/// // Works with raw closures
/// let mut closure = || 42;
/// assert_eq!(use_any_supplier(&mut closure), 42);
/// ```
///
/// ## Stateful Supplier
///
/// ```rust
/// use prism3_function::Supplier;
///
/// let mut counter = 0;
/// let mut stateful = || {
///     counter += 1;
///     counter
/// };
///
/// assert_eq!(stateful.get(), 1);
/// assert_eq!(stateful.get(), 2);
/// assert_eq!(stateful.get(), 3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Supplier<T> {
    /// Produces the next value
    ///
    /// Executes the underlying function and returns the generated value.
    /// The method takes `&mut self` because suppliers typically involve
    /// state changes (like counters or sequences).
    ///
    /// # Returns
    ///
    /// The generated value of type `T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// let mut supplier = BoxSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    fn get(&mut self) -> T;

    /// Converts to BoxSupplier
    ///
    /// Converts the current supplier to `BoxSupplier<T>`.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut box_supplier = closure.into_box();
    /// assert_eq!(box_supplier.get(), 42);
    /// ```
    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to RcSupplier
    ///
    /// Converts the current supplier to `RcSupplier<T>`.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut rc_supplier = closure.into_rc();
    /// assert_eq!(rc_supplier.get(), 42);
    /// ```
    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to ArcSupplier
    ///
    /// Converts the current supplier to `ArcSupplier<T>`.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut arc_supplier = closure.into_arc();
    /// assert_eq!(arc_supplier.get(), 42);
    /// ```
    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;

    /// Converts supplier to a closure for use with iterator methods
    ///
    /// **⚠️ Consumes `self`**: The original supplier becomes unavailable
    /// after calling this method.
    ///
    /// This method consumes the supplier and returns a closure that can be
    /// directly used with iterator methods like `map()`, `filter_map()`,
    /// etc. This provides a more ergonomic API when working with iterators.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the supplier (takes ownership of `self`).
    /// After calling this method, the original supplier is no longer
    /// available. The returned closure captures the supplier by move.
    ///
    /// **Tip**: For cloneable suppliers ([`ArcSupplier`], [`RcSupplier`]),
    /// you can call `.clone()` first if you need to keep the original:
    ///
    /// ```rust
    /// use prism3_function::{Supplier, ArcSupplier};
    /// use std::iter::repeat_with;
    ///
    /// let mut counter = 0;
    /// let supplier = ArcSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// });
    ///
    /// // Clone first to keep the original
    /// let func = supplier.clone().into_fn();
    /// let values: Vec<i32> = repeat_with(func).take(3).collect();
    ///
    /// // Original supplier is still available
    /// let mut original = supplier;
    /// assert_eq!(original.get(), 4);  // Counter continues from where it left off
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a closure that implements `FnMut() -> T`
    ///
    /// # Examples
    ///
    /// ## Using with Standard Iterator Methods
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxSupplier};
    /// use std::iter::repeat_with;
    ///
    /// let mut counter = 0;
    /// let supplier = BoxSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// });
    ///
    /// // Generate a sequence of 5 numbers using repeat_with
    /// let values: Vec<i32> = repeat_with(supplier.into_fn())
    ///     .take(5)
    ///     .collect();
    /// assert_eq!(values, vec![1, 2, 3, 4, 5]);
    /// ```
    ///
    /// ## Passing to Functions Expecting FnMut
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// fn call_n_times<F>(mut func: F, n: usize) -> Vec<i32>
    /// where
    ///     F: FnMut() -> i32,
    /// {
    ///     (0..n).map(|_| func()).collect()
    /// }
    ///
    /// let mut counter = 0;
    /// let supplier = BoxSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// });
    ///
    /// let result = call_n_times(supplier.into_fn(), 5);
    /// assert_eq!(result, vec![1, 2, 3, 4, 5]);
    /// ```
    ///
    /// ## With Standard Library Functions
    ///
    /// ```rust
    /// use prism3_function::{Supplier, RcSupplier};
    /// use std::iter::repeat_with;
    ///
    /// let mut counter = 0;
    /// let supplier = RcSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// });
    ///
    /// let values: Vec<i32> = repeat_with(supplier.into_fn())
    ///     .take(5)
    ///     .collect();
    /// assert_eq!(values, vec![1, 2, 3, 4, 5]);
    /// ```
    ///
    /// ## Clone Before Conversion (ArcSupplier)
    ///
    /// ```rust
    /// use prism3_function::{Supplier, ArcSupplier};
    /// use std::sync::{Arc, Mutex};
    /// use std::iter::repeat_with;
    ///
    /// let counter = Arc::new(Mutex::new(0));
    /// let counter_clone = Arc::clone(&counter);
    /// let supplier = ArcSupplier::new(move || {
    ///     let mut c = counter_clone.lock().unwrap();
    ///     *c += 1;
    ///     *c
    /// });
    ///
    /// // Clone before conversion to keep the original
    /// let func = supplier.clone().into_fn();
    /// let values: Vec<i32> = repeat_with(func).take(3).collect();
    /// assert_eq!(values, vec![1, 2, 3]);
    ///
    /// // Both are still usable and share the same state
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 4);  // Counter continues
    /// ```
    ///
    /// ## Clone Before Conversion (RcSupplier)
    ///
    /// ```rust
    /// use prism3_function::{Supplier, RcSupplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    /// use std::iter::repeat_with;
    ///
    /// let counter = Rc::new(RefCell::new(0));
    /// let counter_clone = Rc::clone(&counter);
    /// let supplier = RcSupplier::new(move || {
    ///     let mut c = counter_clone.borrow_mut();
    ///     *c += 1;
    ///     *c
    /// });
    ///
    /// // Clone before conversion to keep the original
    /// let func = supplier.clone().into_fn();
    /// let values: Vec<i32> = repeat_with(func).take(3).collect();
    /// assert_eq!(values, vec![1, 2, 3]);
    ///
    /// // Both are still usable and share the same state
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 4);  // Counter continues
    /// ```
    ///
    /// ## Ownership Behavior
    ///
    /// ```rust,compile_fail
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// let mut supplier = BoxSupplier::new(|| 42);
    /// let func = supplier.into_fn();
    ///
    /// // ❌ Error: supplier was moved in the call to into_fn()
    /// let value = supplier.get();
    /// ```
    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized + 'static,
        T: 'static;
}

// ==========================================================================
// Implement Supplier trait for closures
// ==========================================================================

/// Implements Supplier for all FnMut() -> T
impl<T, F> Supplier<T> for F
where
    F: FnMut() -> T,
{
    fn get(&mut self) -> T {
        self()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplier::new(self)
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcSupplier::new(self)
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcSupplier::new(self)
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ==========================================================================
// Provide extension methods for closures
// ==========================================================================

/// Extension trait providing supplier composition methods for closures
///
/// Provides `map`, `filter`, and other composition methods for all
/// closures that implement `FnMut() -> T`, allowing closures to be
/// composed directly using method chaining.
///
/// Composition methods consume the closure and return `BoxSupplier<T>`.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{FnSupplierOps, Supplier};
///
/// let mapped = (|| 10).map(|x| x * 2);
/// let mut result = mapped;
/// assert_eq!(result.get(), 20);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnSupplierOps<T>: FnMut() -> T + Sized {
    /// Maps the output using a mapping function
    ///
    /// Returns a new supplier that applies the mapper to the output
    /// of the current supplier. Consumes the current closure and
    /// returns `BoxSupplier<U>`.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The output type after transformation
    /// * `F` - The mapping function type
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to apply to the supplier's output
    ///
    /// # Returns
    ///
    /// Returns the mapped `BoxSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnSupplierOps, Supplier};
    ///
    /// let mapped = (|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    ///
    /// let mut result = mapped;
    /// assert_eq!(result.get(), 25);
    /// ```
    fn map<U, F>(self, mapper: F) -> BoxSupplier<U>
    where
        Self: 'static,
        F: FnMut(T) -> U + 'static,
        T: 'static,
        U: 'static,
    {
        let mut source = self;
        let mut map_fn = mapper;
        BoxSupplier::new(move || map_fn(source()))
    }

    /// Filters the output based on a predicate
    ///
    /// Returns a new supplier that returns Some(value) if the predicate
    /// is satisfied, None otherwise. Consumes the current closure and
    /// returns `BoxSupplier<Option<T>>`.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// Returns the filtered `BoxSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnSupplierOps, Supplier};
    ///
    /// let mut counter = 0;
    /// let filtered = (move || {
    ///     counter += 1;
    ///     counter
    /// }).filter(|x| x % 2 == 0);
    ///
    /// let mut result = filtered;
    /// assert_eq!(result.get(), None);     // 1 is odd
    /// assert_eq!(result.get(), Some(2));  // 2 is even
    /// ```
    fn filter<P>(self, predicate: P) -> BoxSupplier<Option<T>>
    where
        Self: 'static,
        P: FnMut(&T) -> bool + 'static,
        T: 'static,
    {
        let mut source = self;
        let mut pred = predicate;
        BoxSupplier::new(move || {
            let value = source();
            if pred(&value) {
                Some(value)
            } else {
                None
            }
        })
    }
}

/// Implements FnSupplierOps for all closure types
impl<T, F> FnSupplierOps<T> for F where F: FnMut() -> T {}

// ==========================================================================
// BoxSupplier - Single Ownership Implementation
// ==========================================================================

/// Box-based single ownership supplier implementation
///
/// A supplier implementation using `Box<dyn FnMut() -> T>` for
/// single ownership scenarios. This is the most lightweight supplier
/// type with zero reference counting overhead.
///
/// # Key Characteristics
///
/// - **Single ownership**: Cannot be cloned, ownership transfers
/// - **Zero overhead**: Direct `Box` without reference counting
/// - **Stateful**: Can modify captured variables via `FnMut`
/// - **Method chaining**: All methods consume `self` (ownership
///   transfer)
/// - **Not thread-safe**: Cannot be sent across threads (unless `T`
///   and closure are both `Send`)
///
/// # Ownership Model
///
/// Unlike `ArcSupplier` and `RcSupplier`, `BoxSupplier` follows
/// move semantics. When you call a method like `map()`, the original
/// supplier is consumed and you get a new one:
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let supplier = BoxSupplier::new(|| 10);
/// let mapped = supplier.map(|x| x * 2);
/// // `supplier` is no longer usable here
/// // Only `mapped` can be used
/// ```
///
/// # Performance
///
/// This is the fastest supplier type because:
/// - No reference counting (`Rc`/`Arc`)
/// - No interior mutability overhead (`RefCell`/`Mutex`)
/// - Direct function call through `Box`
///
/// # Use Cases
///
/// - **One-time value generation**: When you only need to call
///   `get()` a few times
/// - **Builder patterns**: Building complex suppliers through method
///   chaining
/// - **Sequential pipelines**: Transform-and-consume patterns
/// - **Simple stateful generation**: Counters, sequences, random
///   values
///
/// # When NOT to Use
///
/// - Need to share the same supplier across multiple places (use
///   `RcSupplier`)
/// - Need thread-safe sharing (use `ArcSupplier`)
/// - Need to reuse original supplier after transformation (use
///   `ArcSupplier` or `RcSupplier`)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let mut supplier = BoxSupplier::new(|| 42);
/// assert_eq!(supplier.get(), 42);
/// assert_eq!(supplier.get(), 42);
/// ```
///
/// ## Stateful Counter
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let mut counter = 0;
/// let mut supplier = BoxSupplier::new(move || {
///     counter += 1;
///     counter
/// });
///
/// assert_eq!(supplier.get(), 1);
/// assert_eq!(supplier.get(), 2);
/// assert_eq!(supplier.get(), 3);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let mut pipeline = BoxSupplier::new(|| 10)
///     .map(|x| x * 2)      // Multiply by 2
///     .filter(|x| *x > 15) // Only keep if > 15
///     .map(|opt| opt.unwrap_or(0));
///
/// assert_eq!(pipeline.get(), 20);
/// ```
///
/// ## Memoization
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let mut call_count = 0;
/// let mut memoized = BoxSupplier::new(move || {
///     call_count += 1;
///     println!("Expensive computation #{}", call_count);
///     42
/// }).memoize();
///
/// let v1 = memoized.get(); // Prints: Expensive computation #1
/// let v2 = memoized.get(); // No print (cached)
/// assert_eq!(v1, v2);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxSupplier<T> {
    func: Box<dyn FnMut() -> T>,
}

impl<T> BoxSupplier<T>
where
    T: 'static,
{
    /// Creates a new BoxSupplier
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
    /// Returns a new `BoxSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut supplier = BoxSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        BoxSupplier { func: Box::new(f) }
    }

    /// Creates a constant supplier
    ///
    /// Returns a supplier that always produces the same value.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// Returns a constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut constant = BoxSupplier::constant(42);
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        BoxSupplier::new(move || value.clone())
    }

    /// Maps the output using a mapping function
    ///
    /// Returns a new supplier that applies the mapper to the output.
    /// Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The output type after transformation
    /// * `F` - The mapping function type
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to apply to the supplier's output
    ///
    /// # Returns
    ///
    /// Returns a new mapped `BoxSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut mapped = BoxSupplier::new(|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    pub fn map<U, F>(mut self, mut mapper: F) -> BoxSupplier<U>
    where
        F: FnMut(T) -> U + 'static,
        U: 'static,
    {
        BoxSupplier::new(move || mapper(self.get()))
    }

    /// Filters the output based on a predicate
    ///
    /// Returns a new supplier that returns Some(value) if the
    /// predicate is satisfied, None otherwise. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// Returns a new filtered `BoxSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut counter = 0;
    /// let mut filtered = BoxSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// }).filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), None);     // 1 is odd
    /// assert_eq!(filtered.get(), Some(2));  // 2 is even
    /// ```
    pub fn filter<P>(mut self, mut predicate: P) -> BoxSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
    {
        BoxSupplier::new(move || {
            let value = self.get();
            if predicate(&value) {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Combines this supplier with another, producing a tuple
    ///
    /// Returns a new supplier that produces the outputs of both
    /// suppliers as a tuple. Consumes both suppliers.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The type of the other supplier's output
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with
    ///
    /// # Returns
    ///
    /// Returns a new `BoxSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let first = BoxSupplier::new(|| 42);
    /// let second = BoxSupplier::new(|| "hello");
    /// let mut zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    pub fn zip<U>(mut self, mut other: BoxSupplier<U>) -> BoxSupplier<(T, U)>
    where
        U: 'static,
    {
        BoxSupplier::new(move || (self.get(), other.get()))
    }

    /// Creates a memoizing supplier
    ///
    /// Returns a new supplier that caches the first value it produces.
    /// All subsequent calls return the cached value. Consumes self.
    ///
    /// # Returns
    ///
    /// Returns a new memoized `BoxSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut call_count = 0;
    /// let mut memoized = BoxSupplier::new(move || {
    ///     call_count += 1;
    ///     42
    /// }).memoize();
    ///
    /// assert_eq!(memoized.get(), 42); // Calls underlying function
    /// assert_eq!(memoized.get(), 42); // Returns cached value
    /// ```
    pub fn memoize(mut self) -> BoxSupplier<T>
    where
        T: Clone + 'static,
    {
        let mut cache: Option<T> = None;
        BoxSupplier::new(move || {
            if let Some(ref cached) = cache {
                cached.clone()
            } else {
                let value = self.get();
                cache = Some(value.clone());
                value
            }
        })
    }

    /// Creates a lazy supplier
    ///
    /// The factory function is not called until the first get() call.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The factory function type
    ///
    /// # Parameters
    ///
    /// * `factory` - A function that creates the actual supplier
    ///
    /// # Returns
    ///
    /// Returns a lazy `BoxSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut lazy = BoxSupplier::lazy(|| {
    ///     BoxSupplier::new(|| 42)
    /// });
    ///
    /// assert_eq!(lazy.get(), 42);
    /// ```
    pub fn lazy<F>(mut factory: F) -> BoxSupplier<T>
    where
        F: FnMut() -> BoxSupplier<T> + 'static,
    {
        let mut supplier: Option<BoxSupplier<T>> = None;
        BoxSupplier::new(move || {
            if supplier.is_none() {
                supplier = Some(factory());
            }
            supplier.as_mut().unwrap().get()
        })
    }

    /// Creates a supplier that alternates between two suppliers
    ///
    /// # Parameters
    ///
    /// * `first` - The first supplier
    /// * `second` - The second supplier
    ///
    /// # Returns
    ///
    /// Returns a new alternating `BoxSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let supplier1 = BoxSupplier::new(|| 1);
    /// let supplier2 = BoxSupplier::new(|| 2);
    /// let mut alternating = BoxSupplier::alternate(supplier1,
    ///                                               supplier2);
    ///
    /// assert_eq!(alternating.get(), 1);
    /// assert_eq!(alternating.get(), 2);
    /// assert_eq!(alternating.get(), 1);
    /// ```
    pub fn alternate(mut first: BoxSupplier<T>, mut second: BoxSupplier<T>) -> BoxSupplier<T> {
        let mut use_first = true;
        BoxSupplier::new(move || {
            let result = if use_first { first.get() } else { second.get() };
            use_first = !use_first;
            result
        })
    }
}

/// Additional methods for `BoxSupplier<Option<T>>`
impl<T> BoxSupplier<Option<T>>
where
    T: 'static,
{
    /// Uses a fallback function if this returns None
    ///
    /// # Type Parameters
    ///
    /// * `F` - The fallback function type
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback function to call if None
    ///
    /// # Returns
    ///
    /// Returns a new `BoxSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let primary = BoxSupplier::new(|| None);
    /// let mut supplier = primary.or_else(|| Some(42));
    /// assert_eq!(supplier.get(), Some(42));
    /// ```
    pub fn or_else<F>(mut self, mut fallback: F) -> BoxSupplier<Option<T>>
    where
        F: FnMut() -> Option<T> + 'static,
    {
        BoxSupplier::new(move || self.get().or_else(&mut fallback))
    }

    /// Uses a fallback supplier if this returns None
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback supplier
    ///
    /// # Returns
    ///
    /// Returns a new `BoxSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let primary = BoxSupplier::new(|| None);
    /// let fallback = BoxSupplier::new(|| Some(42));
    /// let mut supplier = primary.or_else_supplier(fallback);
    /// assert_eq!(supplier.get(), Some(42));
    /// ```
    pub fn or_else_supplier(
        mut self,
        mut fallback: BoxSupplier<Option<T>>,
    ) -> BoxSupplier<Option<T>> {
        BoxSupplier::new(move || self.get().or_else(|| fallback.get()))
    }
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&mut self) -> T {
        (self.func)()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        let func = self.func;
        RcSupplier::new(func)
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        // Note: BoxSupplier's inner function may not be Send,
        // so this conversion is unsafe. We panic here.
        panic!(
            "Cannot convert BoxSupplier to ArcSupplier: inner \
             function may not be Send"
        )
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut supplier = self;
        move || supplier.get()
    }
}

// ==========================================================================
// ArcSupplier - Thread-safe Shared Ownership Implementation
// ==========================================================================

/// Thread-safe shared ownership supplier implementation
///
/// A supplier implementation using `Arc<Mutex<dyn FnMut() -> T +
/// Send>>` for thread-safe shared ownership scenarios. This type can
/// be safely cloned and sent across threads.
///
/// # Key Characteristics
///
/// - **Shared ownership**: Can be cloned freely (`Arc`)
/// - **Thread-safe**: Can be sent across threads (`Send + Sync`)
/// - **Interior mutability**: Uses `Mutex` for safe concurrent access
/// - **Method chaining**: Methods borrow `&self` (original remains
///   usable)
/// - **Performance cost**: Locking overhead on every `get()` call
///
/// # Ownership Model
///
/// Unlike `BoxSupplier`, `ArcSupplier` uses shared ownership through
/// `Arc`. Methods like `map()` borrow `&self` instead of consuming
/// `self`, allowing the original supplier to be reused:
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let source = ArcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // `source` is still usable here!
/// let mut s1 = source;
/// let mut s2 = mapped;
/// ```
///
/// # Thread Safety
///
/// Each call to `get()` acquires the internal `Mutex`, ensuring safe
/// concurrent access from multiple threads. The mutex is held only
/// during the actual function execution:
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let counter = Arc::new(Mutex::new(0));
/// let counter_clone = Arc::clone(&counter);
///
/// let supplier = ArcSupplier::new(move || {
///     let mut c = counter_clone.lock().unwrap();
///     *c += 1;
///     *c
/// });
///
/// // Clone for multiple threads
/// let s1 = supplier.clone();
/// let s2 = supplier.clone();
///
/// // Both can safely call get() from different threads
/// ```
///
/// # Performance Considerations
///
/// - **Lock overhead**: Each `get()` call acquires and releases a
///   `Mutex`, which has measurable overhead
/// - **Contention**: High contention from multiple threads can
///   impact performance
/// - **Alternative**: If no sharing is needed, use `BoxSupplier` for
///   zero overhead
/// - **Single-thread**: If only single-threaded sharing is needed,
///   use `RcSupplier` (faster than `ArcSupplier`)
///
/// # Use Cases
///
/// - **Multi-threaded value generation**: Shared counter, ID
///   generator across threads
/// - **Concurrent task processing**: Same supplier used by multiple
///   worker threads
/// - **Global configuration**: Thread-safe lazy-loaded global values
/// - **Event systems**: Shared event generators in concurrent
///   systems
///
/// # When NOT to Use
///
/// - Single-threaded scenarios (use `RcSupplier` instead)
/// - One-time use without sharing (use `BoxSupplier` instead)
/// - Performance-critical hot paths (consider lock-free alternatives)
///
/// # Examples
///
/// ## Basic Sharing
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let supplier = ArcSupplier::new(|| 42);
///
/// // Clone creates shared reference
/// let clone1 = supplier.clone();
/// let clone2 = supplier.clone();
///
/// let mut s1 = clone1;
/// let mut s2 = clone2;
/// assert_eq!(s1.get(), 42);
/// assert_eq!(s2.get(), 42);
/// ```
///
/// ## Thread-safe Counter
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let counter = Arc::new(Mutex::new(0));
/// let counter_clone = Arc::clone(&counter);
///
/// let supplier = ArcSupplier::new(move || {
///     let mut c = counter_clone.lock().unwrap();
///     *c += 1;
///     *c
/// });
///
/// let mut s1 = supplier.clone();
/// let mut s2 = supplier.clone();
///
/// let h1 = thread::spawn(move || s1.get());
/// let h2 = thread::spawn(move || s2.get());
///
/// let v1 = h1.join().unwrap();
/// let v2 = h2.join().unwrap();
///
/// // Both get unique values
/// assert!(v1 != v2);
/// assert_eq!(*counter.lock().unwrap(), 2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let base = ArcSupplier::new(|| 10);
///
/// // Create multiple derived suppliers
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
///
/// // All remain usable
/// let mut b = base;
/// let mut d = doubled;
/// let mut t = tripled;
///
/// assert_eq!(b.get(), 10);
/// assert_eq!(d.get(), 20);
/// assert_eq!(t.get(), 30);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcSupplier<T> {
    func: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T>
where
    T: Send + 'static,
{
    /// Creates a new ArcSupplier
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
    /// Returns a new `ArcSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let supplier = ArcSupplier::new(|| 42);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcSupplier {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates a constant supplier
    ///
    /// Returns a supplier that always produces the same value.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// Returns a constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let constant = ArcSupplier::constant(42);
    /// let mut s = constant;
    /// assert_eq!(s.get(), 42);
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        ArcSupplier::new(move || value.clone())
    }

    /// Maps the output using a mapping function
    ///
    /// Returns a new supplier that applies the mapper to the output.
    /// Borrows &self, doesn't consume the original supplier.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The output type after transformation
    /// * `F` - The mapping function type
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to apply to the supplier's output
    ///
    /// # Returns
    ///
    /// Returns a new mapped `ArcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let source = ArcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    ///
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> ArcSupplier<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        let func = Arc::clone(&self.func);
        let mapper = Arc::new(Mutex::new(mapper));
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                let value = func.lock().unwrap()();
                mapper.lock().unwrap()(value)
            })),
        }
    }

    /// Filters the output based on a predicate
    ///
    /// Returns a new supplier that returns Some(value) if the
    /// predicate is satisfied, None otherwise.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// Returns a new filtered `ArcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let counter = Arc::new(Mutex::new(0));
    /// let counter_clone = Arc::clone(&counter);
    /// let source = ArcSupplier::new(move || {
    ///     let mut c = counter_clone.lock().unwrap();
    ///     *c += 1;
    ///     *c
    /// });
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// let mut s = filtered;
    /// assert_eq!(s.get(), None);     // 1 is odd
    /// assert_eq!(s.get(), Some(2));  // 2 is even
    /// ```
    pub fn filter<P>(&self, predicate: P) -> ArcSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + Send + 'static,
    {
        let func = Arc::clone(&self.func);
        let predicate = Arc::new(Mutex::new(predicate));
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                let value = func.lock().unwrap()();
                if predicate.lock().unwrap()(&value) {
                    Some(value)
                } else {
                    None
                }
            })),
        }
    }

    /// Combines this supplier with another, producing a tuple
    ///
    /// Returns a new supplier that produces the outputs of both
    /// suppliers as a tuple. Borrows &self.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The type of the other supplier's output
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with
    ///
    /// # Returns
    ///
    /// Returns a new `ArcSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let first = ArcSupplier::new(|| 42);
    /// let second = ArcSupplier::new(|| "hello");
    /// let zipped = first.zip(&second);
    ///
    /// // first and second are still usable
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    /// ```
    pub fn zip<U>(&self, other: &ArcSupplier<U>) -> ArcSupplier<(T, U)>
    where
        U: Send + 'static,
    {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&other.func);
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                (first.lock().unwrap()(), second.lock().unwrap()())
            })),
        }
    }

    /// Creates a memoizing supplier
    ///
    /// Returns a new supplier that caches the first value it produces.
    /// All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// Returns a new memoized `ArcSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let call_count = Arc::new(Mutex::new(0));
    /// let call_count_clone = Arc::clone(&call_count);
    /// let source = ArcSupplier::new(move || {
    ///     let mut c = call_count_clone.lock().unwrap();
    ///     *c += 1;
    ///     42
    /// });
    /// let memoized = source.memoize();
    ///
    /// let mut s = memoized;
    /// assert_eq!(s.get(), 42); // Calls underlying function
    /// assert_eq!(s.get(), 42); // Returns cached value
    /// assert_eq!(*call_count.lock().unwrap(), 1); // Only called once
    /// ```
    pub fn memoize(&self) -> ArcSupplier<T>
    where
        T: Clone + 'static,
    {
        let func = Arc::clone(&self.func);
        let cache: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                let mut cache_guard = cache.lock().unwrap();
                if let Some(ref cached) = *cache_guard {
                    cached.clone()
                } else {
                    let value = func.lock().unwrap()();
                    *cache_guard = Some(value.clone());
                    value
                }
            })),
        }
    }
}

/// Additional methods for `ArcSupplier<Option<T>>`
impl<T> ArcSupplier<Option<T>>
where
    T: Send + 'static,
{
    /// Uses a fallback function if this returns None
    ///
    /// # Type Parameters
    ///
    /// * `F` - The fallback function type
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback function to call if None
    ///
    /// # Returns
    ///
    /// Returns a new `ArcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let primary = ArcSupplier::new(|| None);
    /// let supplier = primary.or_else(|| Some(42));
    /// let mut s = supplier;
    /// assert_eq!(s.get(), Some(42));
    /// ```
    pub fn or_else<F>(&self, fallback: F) -> ArcSupplier<Option<T>>
    where
        F: FnMut() -> Option<T> + Send + 'static,
    {
        let func = Arc::clone(&self.func);
        let fallback = Arc::new(Mutex::new(fallback));
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                func.lock().unwrap()().or_else(|| fallback.lock().unwrap()())
            })),
        }
    }

    /// Uses a fallback supplier if this returns None
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback supplier
    ///
    /// # Returns
    ///
    /// Returns a new `ArcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let primary = ArcSupplier::new(|| None);
    /// let fallback = ArcSupplier::new(|| Some(42));
    /// let supplier = primary.or_else_supplier(&fallback);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), Some(42));
    /// ```
    pub fn or_else_supplier(&self, fallback: &ArcSupplier<Option<T>>) -> ArcSupplier<Option<T>> {
        let func = Arc::clone(&self.func);
        let fallback_func = Arc::clone(&fallback.func);
        ArcSupplier {
            func: Arc::new(Mutex::new(move || {
                func.lock().unwrap()().or_else(|| fallback_func.lock().unwrap()())
            })),
        }
    }
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.lock().unwrap())()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxSupplier::new(move || func.lock().unwrap()())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        let func = self.func;
        RcSupplier::new(move || func.lock().unwrap()())
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut supplier = self;
        move || supplier.get()
    }
}

impl<T> Clone for ArcSupplier<T> {
    /// Clones ArcSupplier
    ///
    /// Creates a new ArcSupplier that shares the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ==========================================================================
// RcSupplier - Single-threaded Shared Ownership Implementation
// ==========================================================================

/// Single-threaded shared ownership supplier implementation
///
/// A supplier implementation using `Rc<RefCell<dyn FnMut() -> T>>`
/// for single-threaded shared ownership scenarios. This type can be
/// cloned but cannot be sent across threads.
///
/// # Key Characteristics
///
/// - **Shared ownership**: Can be cloned freely (`Rc`)
/// - **Not thread-safe**: Cannot be sent across threads (no `Send`)
/// - **Interior mutability**: Uses `RefCell` for runtime borrow
///   checking
/// - **Method chaining**: Methods borrow `&self` (original remains
///   usable)
/// - **Lower overhead**: Faster than `ArcSupplier` (no `Mutex`)
///
/// # Ownership Model
///
/// Like `ArcSupplier`, `RcSupplier` uses shared ownership, but
/// through `Rc` instead of `Arc`. Methods borrow `&self` instead of
/// consuming `self`:
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let source = RcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // `source` is still usable here!
/// let mut s1 = source;
/// let mut s2 = mapped;
/// ```
///
/// # Interior Mutability
///
/// Each call to `get()` uses `RefCell::borrow_mut()`, which performs
/// runtime borrow checking. Unlike `Mutex`, this has lower overhead
/// but will panic if rules are violated:
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let supplier = RcSupplier::new(|| 42);
/// let mut s = supplier;
/// let value = s.get(); // OK: borrow_mut() succeeds
/// // If you try to hold multiple mutable borrows, it will panic
/// ```
///
/// # Performance vs ArcSupplier
///
/// `RcSupplier` is faster than `ArcSupplier` because:
/// - `Rc` has less overhead than `Arc` (no atomic operations)
/// - `RefCell` is faster than `Mutex` (no OS-level locking)
/// - No thread synchronization needed
///
/// However, it cannot be used across threads.
///
/// # Use Cases
///
/// - **Single-threaded sharing**: When you need to reuse the same
///   supplier in multiple places within one thread
/// - **UI event handling**: Shared event generators in single-
///   threaded UI frameworks
/// - **Callback systems**: Multiple callbacks sharing the same value
///   generator
/// - **State machines**: Shared state generators in single-threaded
///   state machines
///
/// # When NOT to Use
///
/// - Multi-threaded scenarios (use `ArcSupplier` instead)
/// - One-time use without sharing (use `BoxSupplier` instead)
/// - Need to send across threads (use `ArcSupplier` instead)
///
/// # Comparison Table
///
/// | Feature | BoxSupplier | RcSupplier | ArcSupplier |
/// |---------|-------------|------------|-------------|
/// | Overhead | None | Low | Medium |
/// | Cloneable | No | Yes | Yes |
/// | Thread-safe | No | No | Yes |
/// | Method chains | Consumes | Borrows | Borrows |
///
/// # Examples
///
/// ## Basic Sharing
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let supplier = RcSupplier::new(|| 42);
///
/// // Clone creates shared reference (same underlying supplier)
/// let clone1 = supplier.clone();
/// let clone2 = supplier.clone();
///
/// let mut s1 = clone1;
/// let mut s2 = clone2;
/// assert_eq!(s1.get(), 42);
/// assert_eq!(s2.get(), 42);
/// ```
///
/// ## Shared Counter
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let counter = Rc::new(RefCell::new(0));
/// let counter_clone = Rc::clone(&counter);
///
/// let supplier = RcSupplier::new(move || {
///     let mut c = counter_clone.borrow_mut();
///     *c += 1;
///     *c
/// });
///
/// let mut s1 = supplier.clone();
/// let mut s2 = supplier.clone();
///
/// assert_eq!(s1.get(), 1);
/// assert_eq!(s2.get(), 2);
/// assert_eq!(*counter.borrow(), 2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let base = RcSupplier::new(|| 10);
///
/// // Create multiple derived suppliers
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
/// let squared = base.map(|x| x * x);
///
/// // All remain usable
/// let mut b = base;
/// let mut d = doubled;
/// let mut t = tripled;
/// let mut s = squared;
///
/// assert_eq!(b.get(), 10);
/// assert_eq!(d.get(), 20);
/// assert_eq!(t.get(), 30);
/// assert_eq!(s.get(), 100);
/// ```
///
/// ## Callback Pattern
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// struct EventSystem {
///     id_generator: RcSupplier<u32>,
/// }
///
/// impl EventSystem {
///     fn new(generator: RcSupplier<u32>) -> Self {
///         Self { id_generator: generator }
///     }
///
///     fn generate_event_id(&mut self) -> u32 {
///         self.id_generator.get()
///     }
/// }
///
/// let mut counter = 0;
/// let generator = RcSupplier::new(move || {
///     counter += 1;
///     counter
/// });
///
/// let mut system1 = EventSystem::new(generator.clone());
/// let mut system2 = EventSystem::new(generator.clone());
///
/// // Both systems share the same ID generator
/// let id1 = system1.generate_event_id();
/// let id2 = system2.generate_event_id();
/// assert_ne!(id1, id2); // Different IDs
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcSupplier<T> {
    func: Rc<RefCell<dyn FnMut() -> T>>,
}

impl<T> RcSupplier<T>
where
    T: 'static,
{
    /// Creates a new RcSupplier
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
    /// Returns a new `RcSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let supplier = RcSupplier::new(|| 42);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        RcSupplier {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates a constant supplier
    ///
    /// Returns a supplier that always produces the same value.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// Returns a constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let constant = RcSupplier::constant(42);
    /// let mut s = constant;
    /// assert_eq!(s.get(), 42);
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        RcSupplier::new(move || value.clone())
    }

    /// Maps the output using a mapping function
    ///
    /// Returns a new supplier that applies the mapper to the output.
    /// Borrows &self, doesn't consume the original supplier.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The output type after transformation
    /// * `F` - The mapping function type
    ///
    /// # Parameters
    ///
    /// * `mapper` - The function to apply to the supplier's output
    ///
    /// # Returns
    ///
    /// Returns a new mapped `RcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let source = RcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    ///
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> RcSupplier<U>
    where
        F: FnMut(T) -> U + 'static,
        U: 'static,
    {
        let func = Rc::clone(&self.func);
        let mapper = Rc::new(RefCell::new(mapper));
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                let value = func.borrow_mut()();
                mapper.borrow_mut()(value)
            })),
        }
    }

    /// Filters the output based on a predicate
    ///
    /// Returns a new supplier that returns Some(value) if the
    /// predicate is satisfied, None otherwise.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// Returns a new filtered `RcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let counter = Rc::new(RefCell::new(0));
    /// let counter_clone = Rc::clone(&counter);
    /// let source = RcSupplier::new(move || {
    ///     let mut c = counter_clone.borrow_mut();
    ///     *c += 1;
    ///     *c
    /// });
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// let mut s = filtered;
    /// assert_eq!(s.get(), None);     // 1 is odd
    /// assert_eq!(s.get(), Some(2));  // 2 is even
    /// ```
    pub fn filter<P>(&self, predicate: P) -> RcSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
    {
        let func = Rc::clone(&self.func);
        let predicate = Rc::new(RefCell::new(predicate));
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                let value = func.borrow_mut()();
                if predicate.borrow_mut()(&value) {
                    Some(value)
                } else {
                    None
                }
            })),
        }
    }

    /// Combines this supplier with another, producing a tuple
    ///
    /// Returns a new supplier that produces the outputs of both
    /// suppliers as a tuple. Borrows &self.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The type of the other supplier's output
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with
    ///
    /// # Returns
    ///
    /// Returns a new `RcSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let first = RcSupplier::new(|| 42);
    /// let second = RcSupplier::new(|| "hello");
    /// let zipped = first.zip(&second);
    ///
    /// // first and second are still usable
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    /// ```
    pub fn zip<U>(&self, other: &RcSupplier<U>) -> RcSupplier<(T, U)>
    where
        U: 'static,
    {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&other.func);
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                (first.borrow_mut()(), second.borrow_mut()())
            })),
        }
    }

    /// Creates a memoizing supplier
    ///
    /// Returns a new supplier that caches the first value it produces.
    /// All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// Returns a new memoized `RcSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let call_count = Rc::new(RefCell::new(0));
    /// let call_count_clone = Rc::clone(&call_count);
    /// let source = RcSupplier::new(move || {
    ///     let mut c = call_count_clone.borrow_mut();
    ///     *c += 1;
    ///     42
    /// });
    /// let memoized = source.memoize();
    ///
    /// let mut s = memoized;
    /// assert_eq!(s.get(), 42); // Calls underlying function
    /// assert_eq!(s.get(), 42); // Returns cached value
    /// assert_eq!(*call_count.borrow(), 1); // Only called once
    /// ```
    pub fn memoize(&self) -> RcSupplier<T>
    where
        T: Clone + 'static,
    {
        let func = Rc::clone(&self.func);
        let cache: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                let mut cache_ref = cache.borrow_mut();
                if let Some(ref cached) = *cache_ref {
                    cached.clone()
                } else {
                    let value = func.borrow_mut()();
                    *cache_ref = Some(value.clone());
                    value
                }
            })),
        }
    }
}

/// Additional methods for `RcSupplier<Option<T>>`
impl<T> RcSupplier<Option<T>>
where
    T: 'static,
{
    /// Uses a fallback function if this returns None
    ///
    /// # Type Parameters
    ///
    /// * `F` - The fallback function type
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback function to call if None
    ///
    /// # Returns
    ///
    /// Returns a new `RcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let primary = RcSupplier::new(|| None);
    /// let supplier = primary.or_else(|| Some(42));
    /// let mut s = supplier;
    /// assert_eq!(s.get(), Some(42));
    /// ```
    pub fn or_else<F>(&self, fallback: F) -> RcSupplier<Option<T>>
    where
        F: FnMut() -> Option<T> + 'static,
    {
        let func = Rc::clone(&self.func);
        let fallback = Rc::new(RefCell::new(fallback));
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                func.borrow_mut()().or_else(|| fallback.borrow_mut()())
            })),
        }
    }

    /// Uses a fallback supplier if this returns None
    ///
    /// # Parameters
    ///
    /// * `fallback` - The fallback supplier
    ///
    /// # Returns
    ///
    /// Returns a new `RcSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let primary = RcSupplier::new(|| None);
    /// let fallback = RcSupplier::new(|| Some(42));
    /// let supplier = primary.or_else_supplier(&fallback);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), Some(42));
    /// ```
    pub fn or_else_supplier(&self, fallback: &RcSupplier<Option<T>>) -> RcSupplier<Option<T>> {
        let func = Rc::clone(&self.func);
        let fallback_func = Rc::clone(&fallback.func);
        RcSupplier {
            func: Rc::new(RefCell::new(move || {
                func.borrow_mut()().or_else(|| fallback_func.borrow_mut()())
            })),
        }
    }
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.func.borrow_mut())()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        let func = self.func;
        BoxSupplier::new(move || func.borrow_mut()())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcSupplier to ArcSupplier (not Send)")
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized + 'static,
        T: 'static,
    {
        let mut supplier = self;
        move || supplier.get()
    }
}

impl<T> Clone for RcSupplier<T> {
    /// Clones RcSupplier
    ///
    /// Creates a new RcSupplier that shares the underlying function
    /// with the original instance.
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
