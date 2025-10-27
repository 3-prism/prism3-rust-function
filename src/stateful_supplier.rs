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
//! Provides supplier implementations that generate and return values
//! without taking any input parameters.
//!
//! # Overview
//!
//! A **Supplier** is a functional abstraction that generates and
//! provides a value without accepting input. It can produce new
//! values each time (like a factory) or return fixed values
//! (like constants).
//!
//! This module implements **Approach 3** from the design document: a
//! unified `Supplier` trait with multiple concrete implementations
//! optimized for different ownership and concurrency scenarios.
//!
//! # Core Design Principles
//!
//! 1. **Returns Ownership**: `Supplier` returns `T` (not `&T`) to
//!    avoid lifetime issues
//! 2. **Uses `&mut self`**: Typical scenarios (counters, generators)
//!    require state modification
//! 3. **No ReadonlySupplier**: Main use cases require state
//!    modification; value is extremely low
//!
//! # Three Implementations
//!
//! - **`BoxStatefulSupplier<T>`**: Single ownership using `Box<dyn FnMut()
//!   -> T>`. Zero overhead, cannot be cloned. Best for one-time use
//!   and builder patterns.
//!
//! - **`ArcStatefulSupplier<T>`**: Thread-safe shared ownership using
//!   `Arc<Mutex<dyn FnMut() -> T + Send>>`. Can be cloned and sent
//!   across threads. Higher overhead due to locking.
//!
//! - **`RcStatefulSupplier<T>`**: Single-threaded shared ownership using
//!   `Rc<RefCell<dyn FnMut() -> T>>`. Can be cloned but not sent
//!   across threads. Lower overhead than `ArcStatefulSupplier`.
//!
//! # Comparison with Other Functional Abstractions
//!
//! | Type      | Input | Output | self      | Modifies? | Use Case      |
//! |-----------|-------|--------|-----------|-----------|---------------|
//! | Supplier  | None  | `T`    | `&mut`    | Yes       | Factory       |
//! | Consumer  | `&T`  | `()`   | `&mut`    | Yes       | Observer      |
//! | Predicate | `&T`  | `bool` | `&self`   | No        | Filter        |
//! | Function  | `&T`  | `R`    | `&self`   | No        | Transform     |
//!
//! # Examples
//!
//! ## Basic Counter
//!
//! ```rust
//! use prism3_function::{BoxStatefulSupplier, Supplier};
//!
//! let mut counter = 0;
//! let mut supplier = BoxStatefulSupplier::new(move || {
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
//! use prism3_function::{BoxStatefulSupplier, Supplier};
//!
//! let mut pipeline = BoxStatefulSupplier::new(|| 10)
//!     .map(|x| x * 2)
//!     .map(|x| x + 5);
//!
//! assert_eq!(pipeline.get(), 25);
//! ```
//!
//! ## Thread-safe Sharing
//!
//! ```rust
//! use prism3_function::{ArcStatefulSupplier, Supplier};
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//!
//! let counter = Arc::new(Mutex::new(0));
//! let counter_clone = Arc::clone(&counter);
//!
//! let supplier = ArcStatefulSupplier::new(move || {
//!     let mut c = counter_clone.lock().unwrap();
//!     *c += 1;
//!     *c
//! });
//!
//! let mut s1 = supplier.clone();
//! let mut s2 = supplier.clone();
//!
//! let h1 = thread::spawn(move || s1.get());
//! let h2 = thread::spawn(move || s2.get());
//!
//! let v1 = h1.join().unwrap();
//! let v2 = h2.join().unwrap();
//!
//! assert!(v1 != v2);
//! assert_eq!(*counter.lock().unwrap(), 2);
//! ```
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

use crate::stateful_transformer::StatefulTransformer;
use crate::supplier_once::{
    BoxSupplierOnce,
    SupplierOnce,
};

// ==========================================================================
// Supplier Trait
// ==========================================================================

/// Supplier trait: generates and returns values without input.
///
/// The core abstraction for value generation. Similar to Java's
/// `Supplier<T>` interface, it produces values without taking any
/// input parameters.
///
/// # Key Characteristics
///
/// - **No input parameters**: Pure value generation
/// - **Mutable access**: Uses `&mut self` to allow state changes
/// - **Returns ownership**: Returns `T` (not `&T`) to avoid lifetime
///   issues
/// - **Can modify state**: Commonly used for counters, sequences,
///   and generators
///
/// # Automatically Implemented for Closures
///
/// All `FnMut() -> T` closures automatically implement this trait,
/// enabling seamless integration with both raw closures and wrapped
/// supplier types.
///
/// # Examples
///
/// ## Using with Generic Functions
///
/// ```rust
/// use prism3_function::{Supplier, BoxStatefulSupplier};
///
/// fn call_twice<S: StatefulSupplier<i32>>(supplier: &mut S) -> (i32, i32) {
///     (supplier.get(), supplier.get())
/// }
///
/// let mut s = BoxStatefulSupplier::new(|| 42);
/// assert_eq!(call_twice(&mut s), (42, 42));
///
/// let mut closure = || 100;
/// assert_eq!(call_twice(&mut closure), (100, 100));
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
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait StatefulSupplier<T> {
    /// Generates and returns the next value.
    ///
    /// Executes the underlying function and returns the generated
    /// value. Uses `&mut self` because suppliers typically involve
    /// state changes (counters, sequences, etc.).
    ///
    /// # Returns
    ///
    /// The generated value of type `T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxStatefulSupplier};
    ///
    /// let mut supplier = BoxStatefulSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    fn get(&mut self) -> T;

    /// Converts to `BoxStatefulSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in a `BoxStatefulSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut boxed = closure.into_box();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn into_box(mut self) -> BoxStatefulSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxStatefulSupplier::new(move || self.get())
    }

    /// Converts to `RcStatefulSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `RcStatefulSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `RcStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut rc = closure.into_rc();
    /// assert_eq!(rc.get(), 42);
    /// ```
    fn into_rc(mut self) -> RcStatefulSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcStatefulSupplier::new(move || self.get())
    }

    /// Converts to `ArcStatefulSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `ArcStatefulSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut arc = closure.into_arc();
    /// assert_eq!(arc.get(), 42);
    /// ```
    fn into_arc(mut self) -> ArcStatefulSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcStatefulSupplier::new(move || self.get())
    }

    /// Converts to a closure `FnMut() -> T`.
    ///
    /// This method wraps the supplier in a closure that calls the
    /// `get()` method when invoked. This allows using suppliers
    /// in contexts that expect `FnMut()` closures.
    ///
    /// # Returns
    ///
    /// A closure `impl FnMut() -> T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxStatefulSupplier};
    ///
    /// let supplier = BoxStatefulSupplier::new(|| 42);
    /// let mut closure = supplier.into_fn();
    /// assert_eq!(closure(), 42);
    /// assert_eq!(closure(), 42);
    /// ```
    ///
    /// ## Using with functions that expect FnMut
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxStatefulSupplier};
    ///
    /// fn call_fn_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
    ///     (f(), f())
    /// }
    ///
    /// let supplier = BoxStatefulSupplier::new(|| 100);
    /// let closure = supplier.into_fn();
    /// assert_eq!(call_fn_twice(closure), (100, 100));
    /// ```
    fn into_fn(mut self) -> impl FnMut() -> T
    where
        Self: Sized,
    {
        move || self.get()
    }

    /// Creates a `BoxStatefulSupplier` from a cloned supplier.
    ///
    /// Uses `Clone` to obtain an owned copy and converts it into a
    /// `BoxStatefulSupplier`. Implementations can override this for a more
    /// efficient conversion.
    fn to_box(&self) -> BoxStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Creates an `RcStatefulSupplier` from a cloned supplier.
    ///
    /// Uses `Clone` to obtain an owned copy and converts it into an
    /// `RcStatefulSupplier`. Implementations can override it for better
    /// performance.
    fn to_rc(&self) -> RcStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Creates an `ArcStatefulSupplier` from a cloned supplier.
    ///
    /// Requires the supplier and produced values to be `Send` so the
    /// resulting supplier can be shared across threads.
    fn to_arc(&self) -> ArcStatefulSupplier<T>
    where
        Self: Clone + Sized + Send + 'static,
        T: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Creates a closure from a cloned supplier.
    ///
    /// The default implementation clones `self` and consumes the clone
    /// to produce a closure. Concrete suppliers can override it to
    /// avoid the additional clone.
    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone + Sized,
    {
        self.clone().into_fn()
    }
}

// ==========================================================================
// BoxStatefulSupplier - Single Ownership Implementation
// ==========================================================================

/// Box-based single ownership supplier.
///
/// Uses `Box<dyn FnMut() -> T>` for single ownership scenarios.
/// This is the most lightweight supplier with zero reference
/// counting overhead.
///
/// # Ownership Model
///
/// Methods consume `self` (move semantics). When you call a method
/// like `map()`, the original supplier is consumed and you get a new
/// one:
///
/// ```rust
/// use prism3_function::{BoxStatefulSupplier, Supplier};
///
/// let supplier = BoxStatefulSupplier::new(|| 10);
/// let mapped = supplier.map(|x| x * 2);
/// // supplier is no longer usable here
/// ```
///
/// # Examples
///
/// ## Counter
///
/// ```rust
/// use prism3_function::{BoxStatefulSupplier, Supplier};
///
/// let mut counter = 0;
/// let mut supplier = BoxStatefulSupplier::new(move || {
///     counter += 1;
///     counter
/// });
///
/// assert_eq!(supplier.get(), 1);
/// assert_eq!(supplier.get(), 2);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BoxStatefulSupplier, Supplier};
///
/// let mut pipeline = BoxStatefulSupplier::new(|| 10)
///     .map(|x| x * 2)
///     .map(|x| x + 5);
///
/// assert_eq!(pipeline.get(), 25);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxStatefulSupplier<T> {
    function: Box<dyn FnMut() -> T>,
}

impl<T> BoxStatefulSupplier<T>
where
    T: 'static,
{
    /// Creates a new `BoxStatefulSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let mut supplier = BoxStatefulSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        BoxStatefulSupplier {
            function: Box::new(f),
        }
    }

    /// Creates a constant supplier.
    ///
    /// Returns a supplier that always produces the same value (via
    /// cloning).
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// A constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let mut constant = BoxStatefulSupplier::constant(42);
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        BoxStatefulSupplier::new(move || value.clone())
    }

    /// Maps the output using a transformation function.
    ///
    /// Consumes self and returns a new supplier that applies the
    /// mapper to each output.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The mapper to apply to the output. Can be:
    ///   - A closure: `|x: T| -> U`
    ///   - A function pointer: `fn(T) -> U`
    ///   - A `BoxMapper<T, U>`, `RcMapper<T, U>`, `ArcMapper<T, U>`
    ///   - Any type implementing `StatefulTransformer<T, U>`
    ///
    /// # Returns
    ///
    /// A new mapped `BoxStatefulSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let mut mapped = BoxStatefulSupplier::new(|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    ///
    /// ## Using with StatefulTransformer object
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, BoxMapper, Supplier, StatefulTransformer};
    ///
    /// let mapper = BoxMapper::new(|x: i32| x * 2);
    /// let mut supplier = BoxStatefulSupplier::new(|| 10)
    ///     .map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(mut self, mut mapper: F) -> BoxStatefulSupplier<U>
    where
        F: StatefulTransformer<T, U> + 'static,
        U: 'static,
    {
        BoxStatefulSupplier::new(move || mapper.apply(StatefulSupplier::get(&mut self)))
    }

    /// Filters output based on a predicate.
    ///
    /// Returns a new supplier that returns `Some(value)` if the
    /// predicate is satisfied, `None` otherwise.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// A new filtered `BoxStatefulSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let mut counter = 0;
    /// let mut filtered = BoxStatefulSupplier::new(move || {
    ///     counter += 1;
    ///     counter
    /// }).filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), None);     // 1 is odd
    /// assert_eq!(filtered.get(), Some(2));  // 2 is even
    /// ```
    pub fn filter<P>(mut self, mut predicate: P) -> BoxStatefulSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
    {
        BoxStatefulSupplier::new(move || {
            let value = StatefulSupplier::get(&mut self);
            if predicate(&value) {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// Consumes both suppliers and returns a new supplier that
    /// produces `(T, U)` tuples.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. Can be any type
    ///   implementing `Supplier<U>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let first = BoxStatefulSupplier::new(|| 42);
    /// let second = BoxStatefulSupplier::new(|| "hello");
    /// let mut zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    pub fn zip<S, U>(mut self, mut other: S) -> BoxStatefulSupplier<(T, U)>
    where
        S: StatefulSupplier<U> + 'static,
        U: 'static,
    {
        BoxStatefulSupplier::new(move || {
            (
                StatefulSupplier::get(&mut self),
                StatefulSupplier::get(&mut other),
            )
        })
    }

    /// Creates a memoizing supplier.
    ///
    /// Returns a new supplier that caches the first value it
    /// produces. All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// A new memoized `BoxStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxStatefulSupplier, Supplier};
    ///
    /// let mut call_count = 0;
    /// let mut memoized = BoxStatefulSupplier::new(move || {
    ///     call_count += 1;
    ///     42
    /// }).memoize();
    ///
    /// assert_eq!(memoized.get(), 42); // Calls underlying function
    /// assert_eq!(memoized.get(), 42); // Returns cached value
    /// ```
    pub fn memoize(mut self) -> BoxStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        let mut cache: Option<T> = None;
        BoxStatefulSupplier::new(move || {
            if let Some(ref cached) = cache {
                cached.clone()
            } else {
                let value = StatefulSupplier::get(&mut self);
                cache = Some(value.clone());
                value
            }
        })
    }
}

impl<T> StatefulSupplier<T> for BoxStatefulSupplier<T> {
    fn get(&mut self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxStatefulSupplier<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcStatefulSupplier<T>
    where
        T: 'static,
    {
        RcStatefulSupplier::new(self.function)
    }

    // into_arc cannot be implemented because the inner function may not be Send.
    // Attempting to call this method will result in a compiler error due to missing Send bound.
    // Use ArcStatefulSupplier::new directly with a Send closure instead.
    // compile_error!("Cannot convert BoxStatefulSupplier to ArcStatefulSupplier: inner function may not implement Send");

    fn into_fn(self) -> impl FnMut() -> T {
        self.function
    }

    // NOTE: `BoxStatefulSupplier` is not `Clone`, so it cannot offer optimized
    // `to_box`, `to_rc`, `to_arc`, or `to_fn` implementations. Invoking
    // the default trait methods will not compile because the required
    // `Clone` bound is not satisfied.
}

impl<T> SupplierOnce<T> for BoxStatefulSupplier<T>
where
    T: 'static,
{
    fn get_once(mut self) -> T {
        StatefulSupplier::get(&mut self)
    }

    fn into_box_once(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
    {
        BoxSupplierOnce::new(self.function)
    }

    fn into_fn_once(self) -> impl FnOnce() -> T
    where
        Self: Sized + 'static,
    {
        let mut f = self.function;
        move || f()
    }

    // NOTE: `BoxStatefulSupplier` is not `Clone`, so it cannot offer
    // `to_box_once` or `to_fn_once` implementations. Invoking the default
    // trait methods will not compile because the required `Clone`
    // bound is not satisfied.
}

// ==========================================================================
// ArcStatefulSupplier - Thread-safe Shared Ownership Implementation
// ==========================================================================

/// Thread-safe shared ownership supplier.
///
/// Uses `Arc<Mutex<dyn FnMut() -> T + Send>>` for thread-safe
/// shared ownership. Can be cloned and sent across threads.
///
/// # Ownership Model
///
/// Methods borrow `&self` instead of consuming `self`. The original
/// supplier remains usable after method calls:
///
/// ```rust
/// use prism3_function::{ArcStatefulSupplier, Supplier};
///
/// let source = ArcStatefulSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Examples
///
/// ## Thread-safe Counter
///
/// ```rust
/// use prism3_function::{ArcStatefulSupplier, Supplier};
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let counter = Arc::new(Mutex::new(0));
/// let counter_clone = Arc::clone(&counter);
///
/// let supplier = ArcStatefulSupplier::new(move || {
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
/// assert!(v1 != v2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{ArcStatefulSupplier, Supplier};
///
/// let base = ArcStatefulSupplier::new(|| 10);
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
///
/// // All remain usable
/// let mut b = base;
/// let mut d = doubled;
/// let mut t = tripled;
/// assert_eq!(b.get(), 10);
/// assert_eq!(d.get(), 20);
/// assert_eq!(t.get(), 30);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcStatefulSupplier<T> {
    function: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcStatefulSupplier<T>
where
    T: Send + 'static,
{
    /// Creates a new `ArcStatefulSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    ///
    /// let supplier = ArcStatefulSupplier::new(|| 42);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        ArcStatefulSupplier {
            function: Arc::new(Mutex::new(f)),
        }
    }

    /// Creates a constant supplier.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// A constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    ///
    /// let constant = ArcStatefulSupplier::constant(42);
    /// let mut s = constant;
    /// assert_eq!(s.get(), 42);
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        ArcStatefulSupplier::new(move || value.clone())
    }

    /// Maps the output using a transformation function.
    ///
    /// Borrows `&self`, doesn't consume the original supplier.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The mapper to apply to the output. Can be:
    ///   - A closure: `|x: T| -> U` (must be `Send`)
    ///   - A function pointer: `fn(T) -> U`
    ///   - A `BoxMapper<T, U>`, `RcMapper<T, U>`, `ArcMapper<T, U>`
    ///   - Any type implementing `StatefulTransformer<T, U> + Send`
    ///
    /// # Returns
    ///
    /// A new mapped `ArcStatefulSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    ///
    /// let source = ArcStatefulSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    ///
    /// ## Using with StatefulTransformer object
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, ArcMapper, Supplier, StatefulTransformer};
    ///
    /// let mapper = ArcMapper::new(|x: i32| x * 2);
    /// let source = ArcStatefulSupplier::new(|| 10);
    /// let mut supplier = source.map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> ArcStatefulSupplier<U>
    where
        F: StatefulTransformer<T, U> + Send + 'static,
        U: Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let mapper = Arc::new(Mutex::new(mapper));
        ArcStatefulSupplier {
            function: Arc::new(Mutex::new(move || {
                let value = self_fn.lock().unwrap()();
                mapper.lock().unwrap().apply(value)
            })),
        }
    }

    /// Filters output based on a predicate.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// A new filtered `ArcStatefulSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let counter = Arc::new(Mutex::new(0));
    /// let counter_clone = Arc::clone(&counter);
    /// let source = ArcStatefulSupplier::new(move || {
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
    pub fn filter<P>(&self, predicate: P) -> ArcStatefulSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let predicate = Arc::new(Mutex::new(predicate));
        ArcStatefulSupplier {
            function: Arc::new(Mutex::new(move || {
                let value = self_fn.lock().unwrap()();
                if predicate.lock().unwrap()(&value) {
                    Some(value)
                } else {
                    None
                }
            })),
        }
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. Can be any type
    ///   implementing `Supplier<U> + Send`. The supplier is consumed.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    ///
    /// let first = ArcStatefulSupplier::new(|| 42);
    /// let second = ArcStatefulSupplier::new(|| "hello");
    ///
    /// let zipped = first.zip(second.clone());
    ///
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    ///
    /// // second is still usable because it was cloned
    /// let mut s = second;
    /// assert_eq!(s.get(), "hello");
    /// ```
    pub fn zip<S, U>(&self, mut other: S) -> ArcStatefulSupplier<(T, U)>
    where
        S: StatefulSupplier<U> + Send + 'static,
        U: Send + 'static,
    {
        let first = Arc::clone(&self.function);
        ArcStatefulSupplier {
            function: Arc::new(Mutex::new(move || (first.lock().unwrap()(), other.get()))),
        }
    }

    /// Creates a memoizing supplier.
    ///
    /// # Returns
    ///
    /// A new memoized `ArcStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcStatefulSupplier, Supplier};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let call_count = Arc::new(Mutex::new(0));
    /// let call_count_clone = Arc::clone(&call_count);
    /// let source = ArcStatefulSupplier::new(move || {
    ///     let mut c = call_count_clone.lock().unwrap();
    ///     *c += 1;
    ///     42
    /// });
    /// let memoized = source.memoize();
    ///
    /// let mut s = memoized;
    /// assert_eq!(s.get(), 42); // Calls underlying function
    /// assert_eq!(s.get(), 42); // Returns cached value
    /// assert_eq!(*call_count.lock().unwrap(), 1);
    /// ```
    pub fn memoize(&self) -> ArcStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let cache: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        ArcStatefulSupplier {
            function: Arc::new(Mutex::new(move || {
                let mut cache_guard = cache.lock().unwrap();
                if let Some(ref cached) = *cache_guard {
                    cached.clone()
                } else {
                    let value = self_fn.lock().unwrap()();
                    *cache_guard = Some(value.clone());
                    value
                }
            })),
        }
    }
}

impl<T> StatefulSupplier<T> for ArcStatefulSupplier<T> {
    fn get(&mut self) -> T {
        (self.function.lock().unwrap())()
    }

    fn into_box(self) -> BoxStatefulSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxStatefulSupplier::new(move || self_fn.lock().unwrap()())
    }

    fn into_rc(self) -> RcStatefulSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        RcStatefulSupplier::new(move || self_fn.lock().unwrap()())
    }

    fn into_arc(self) -> ArcStatefulSupplier<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function.lock().unwrap()()
    }

    fn to_box(&self) -> BoxStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Arc::clone(&self.function);
        BoxStatefulSupplier::new(move || function.lock().unwrap()())
    }

    fn to_rc(&self) -> RcStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Arc::clone(&self.function);
        RcStatefulSupplier::new(move || function.lock().unwrap()())
    }

    fn to_arc(&self) -> ArcStatefulSupplier<T>
    where
        Self: Clone + Sized + Send + 'static,
        T: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone + Sized,
    {
        let function = Arc::clone(&self.function);
        move || function.lock().unwrap()()
    }
}

impl<T> Clone for ArcStatefulSupplier<T> {
    /// Clones the `ArcStatefulSupplier`.
    ///
    /// Creates a new instance that shares the underlying function
    /// with the original.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
        }
    }
}

impl<T> SupplierOnce<T> for ArcStatefulSupplier<T>
where
    T: Send + 'static,
{
    fn get_once(mut self) -> T {
        StatefulSupplier::get(&mut self)
    }

    fn into_box_once(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
    {
        let f = self.function;
        BoxSupplierOnce::new(move || f.lock().unwrap()())
    }

    fn into_fn_once(self) -> impl FnOnce() -> T
    where
        Self: Sized + 'static,
    {
        let f = self.function;
        move || f.lock().unwrap()()
    }

    fn to_box_once(&self) -> BoxSupplierOnce<T>
    where
        Self: Clone + Sized + 'static,
    {
        let f = Arc::clone(&self.function);
        BoxSupplierOnce::new(move || f.lock().unwrap()())
    }

    fn to_fn_once(&self) -> impl FnOnce() -> T
    where
        Self: Clone + Sized + 'static,
    {
        let f = Arc::clone(&self.function);
        move || f.lock().unwrap()()
    }
}

// ==========================================================================
// RcStatefulSupplier - Single-threaded Shared Ownership Implementation
// ==========================================================================

/// Single-threaded shared ownership supplier.
///
/// Uses `Rc<RefCell<dyn FnMut() -> T>>` for single-threaded shared
/// ownership. Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcStatefulSupplier`, methods borrow `&self` instead of consuming
/// `self`:
///
/// ```rust
/// use prism3_function::{RcStatefulSupplier, Supplier};
///
/// let source = RcStatefulSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Examples
///
/// ## Shared Counter
///
/// ```rust
/// use prism3_function::{RcStatefulSupplier, Supplier};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let counter = Rc::new(RefCell::new(0));
/// let counter_clone = Rc::clone(&counter);
///
/// let supplier = RcStatefulSupplier::new(move || {
///     let mut c = counter_clone.borrow_mut();
///     *c += 1;
///     *c
/// });
///
/// let mut s1 = supplier.clone();
/// let mut s2 = supplier.clone();
/// assert_eq!(s1.get(), 1);
/// assert_eq!(s2.get(), 2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{RcStatefulSupplier, Supplier};
///
/// let base = RcStatefulSupplier::new(|| 10);
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
///
/// let mut b = base;
/// let mut d = doubled;
/// let mut t = tripled;
/// assert_eq!(b.get(), 10);
/// assert_eq!(d.get(), 20);
/// assert_eq!(t.get(), 30);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcStatefulSupplier<T> {
    function: Rc<RefCell<dyn FnMut() -> T>>,
}

impl<T> RcStatefulSupplier<T>
where
    T: 'static,
{
    /// Creates a new `RcStatefulSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `RcStatefulSupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    ///
    /// let supplier = RcStatefulSupplier::new(|| 42);
    /// let mut s = supplier;
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        RcStatefulSupplier {
            function: Rc::new(RefCell::new(f)),
        }
    }

    /// Creates a constant supplier.
    ///
    /// # Parameters
    ///
    /// * `value` - The constant value to return
    ///
    /// # Returns
    ///
    /// A constant supplier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    ///
    /// let constant = RcStatefulSupplier::constant(42);
    /// let mut s = constant;
    /// assert_eq!(s.get(), 42);
    /// assert_eq!(s.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        RcStatefulSupplier::new(move || value.clone())
    }

    /// Maps the output using a transformation function.
    ///
    /// Borrows `&self`, doesn't consume the original supplier.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The mapper to apply to the output. Can be:
    ///   - A closure: `|x: T| -> U`
    ///   - A function pointer: `fn(T) -> U`
    ///   - A `BoxMapper<T, U>`, `RcMapper<T, U>`, `ArcMapper<T, U>`
    ///   - Any type implementing `StatefulTransformer<T, U>`
    ///
    /// # Returns
    ///
    /// A new mapped `RcStatefulSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    ///
    /// let source = RcStatefulSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    ///
    /// ## Using with StatefulTransformer object
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, RcMapper, Supplier, StatefulTransformer};
    ///
    /// let mapper = RcMapper::new(|x: i32| x * 2);
    /// let source = RcStatefulSupplier::new(|| 10);
    /// let mut supplier = source.map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> RcStatefulSupplier<U>
    where
        F: StatefulTransformer<T, U> + 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let mapper = Rc::new(RefCell::new(mapper));
        RcStatefulSupplier {
            function: Rc::new(RefCell::new(move || {
                let value = self_fn.borrow_mut()();
                mapper.borrow_mut().apply(value)
            })),
        }
    }

    /// Filters output based on a predicate.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// A new filtered `RcStatefulSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let counter = Rc::new(RefCell::new(0));
    /// let counter_clone = Rc::clone(&counter);
    /// let source = RcStatefulSupplier::new(move || {
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
    pub fn filter<P>(&self, predicate: P) -> RcStatefulSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let predicate = Rc::new(RefCell::new(predicate));
        RcStatefulSupplier {
            function: Rc::new(RefCell::new(move || {
                let value = self_fn.borrow_mut()();
                if predicate.borrow_mut()(&value) {
                    Some(value)
                } else {
                    None
                }
            })),
        }
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. Can be any type
    ///   implementing `Supplier<U>`. The supplier is consumed.
    ///
    /// # Returns
    ///
    /// A new `RcStatefulSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    ///
    /// let first = RcStatefulSupplier::new(|| 42);
    /// let second = RcStatefulSupplier::new(|| "hello");
    ///
    /// let zipped = first.zip(second.clone());
    ///
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    ///
    /// // second is still usable because it was cloned
    /// let mut s = second;
    /// assert_eq!(s.get(), "hello");
    /// ```
    pub fn zip<S, U>(&self, mut other: S) -> RcStatefulSupplier<(T, U)>
    where
        S: StatefulSupplier<U> + 'static,
        U: 'static,
    {
        let first = Rc::clone(&self.function);
        RcStatefulSupplier {
            function: Rc::new(RefCell::new(move || (first.borrow_mut()(), other.get()))),
        }
    }

    /// Creates a memoizing supplier.
    ///
    /// # Returns
    ///
    /// A new memoized `RcStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcStatefulSupplier, Supplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let call_count = Rc::new(RefCell::new(0));
    /// let call_count_clone = Rc::clone(&call_count);
    /// let source = RcStatefulSupplier::new(move || {
    ///     let mut c = call_count_clone.borrow_mut();
    ///     *c += 1;
    ///     42
    /// });
    /// let memoized = source.memoize();
    ///
    /// let mut s = memoized;
    /// assert_eq!(s.get(), 42); // Calls underlying function
    /// assert_eq!(s.get(), 42); // Returns cached value
    /// assert_eq!(*call_count.borrow(), 1);
    /// ```
    pub fn memoize(&self) -> RcStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let cache: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
        RcStatefulSupplier {
            function: Rc::new(RefCell::new(move || {
                let mut cache_ref = cache.borrow_mut();
                if let Some(ref cached) = *cache_ref {
                    cached.clone()
                } else {
                    let value = self_fn.borrow_mut()();
                    *cache_ref = Some(value.clone());
                    value
                }
            })),
        }
    }
}

impl<T> StatefulSupplier<T> for RcStatefulSupplier<T> {
    fn get(&mut self) -> T {
        (self.function.borrow_mut())()
    }

    fn into_box(self) -> BoxStatefulSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxStatefulSupplier::new(move || self_fn.borrow_mut()())
    }

    fn into_rc(self) -> RcStatefulSupplier<T>
    where
        T: 'static,
    {
        self
    }

    // into_arc cannot be implemented because RcStatefulSupplier does not implement Send.
    // Attempting to call this method will result in a compiler error due to missing Send bound.
    // Use ArcStatefulSupplier::new directly instead.
    // compile_error!("Cannot convert RcStatefulSupplier to ArcStatefulSupplier: RcStatefulSupplier does not implement Send");

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function.borrow_mut()()
    }

    fn to_box(&self) -> BoxStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Rc::clone(&self.function);
        BoxStatefulSupplier::new(move || function.borrow_mut()())
    }

    fn to_rc(&self) -> RcStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone()
    }

    // NOTE: `RcStatefulSupplier` cannot be converted to `ArcStatefulSupplier` because it
    // is not `Send`. Calling the default `to_arc` would fail compilation
    // due to the missing `Send` bound.

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone + Sized,
    {
        let function = Rc::clone(&self.function);
        move || function.borrow_mut()()
    }
}

impl<T> Clone for RcStatefulSupplier<T> {
    /// Clones the `RcStatefulSupplier`.
    ///
    /// Creates a new instance that shares the underlying function
    /// with the original.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
        }
    }
}

impl<T> SupplierOnce<T> for RcStatefulSupplier<T>
where
    T: 'static,
{
    fn get_once(mut self) -> T {
        StatefulSupplier::get(&mut self)
    }

    fn into_box_once(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
    {
        let f = self.function;
        BoxSupplierOnce::new(move || f.borrow_mut()())
    }

    fn into_fn_once(self) -> impl FnOnce() -> T
    where
        Self: Sized + 'static,
    {
        let f = self.function;
        move || f.borrow_mut()()
    }

    fn to_box_once(&self) -> BoxSupplierOnce<T>
    where
        Self: Clone + Sized + 'static,
    {
        let f = Rc::clone(&self.function);
        BoxSupplierOnce::new(move || f.borrow_mut()())
    }

    fn to_fn_once(&self) -> impl FnOnce() -> T
    where
        Self: Clone + Sized + 'static,
    {
        let f = Rc::clone(&self.function);
        move || f.borrow_mut()()
    }
}

// ==========================================================================
// Implement Supplier for Closures
// ==========================================================================

impl<T, F> StatefulSupplier<T> for F
where
    F: FnMut() -> T,
{
    fn get(&mut self) -> T {
        self()
    }

    fn into_box(self) -> BoxStatefulSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxStatefulSupplier::new(self)
    }

    fn into_rc(self) -> RcStatefulSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcStatefulSupplier::new(self)
    }

    fn into_arc(self) -> ArcStatefulSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcStatefulSupplier::new(self)
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized,
    {
        self
    }

    fn to_box(&self) -> BoxStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    fn to_rc(&self) -> RcStatefulSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    fn to_arc(&self) -> ArcStatefulSupplier<T>
    where
        Self: Clone + Sized + Send + 'static,
        T: Send + 'static,
    {
        self.clone().into_arc()
    }

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone + Sized,
    {
        self.clone()
    }
}

// ==========================================================================
// Extension Trait for Closure Operations
// ==========================================================================

/// Extension trait providing supplier operations for closures
///
/// Provides composition methods (`map`, `filter`, `zip`, `memoize`) for
/// closures implementing `FnMut() -> T` without requiring explicit
/// wrapping in `BoxStatefulSupplier`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnMut() -> T`.
///
/// # Design Rationale
///
/// While closures automatically implement `Supplier<T>` through blanket
/// implementation, they don't have access to instance methods like
/// `map`, `filter`, and `zip`. This extension trait provides those
/// methods, returning `BoxStatefulSupplier` for maximum flexibility.
///
/// # Examples
///
/// ## Map transformation
///
/// ```rust
/// use prism3_function::{Supplier, FnStatefulSupplierOps};
///
/// let mut counter = 0;
/// let mut mapped = (move || {
///     counter += 1;
///     counter
/// }).map(|x| x * 2);
///
/// assert_eq!(mapped.get(), 2);
/// assert_eq!(mapped.get(), 4);
/// ```
///
/// ## Filter values
///
/// ```rust
/// use prism3_function::{Supplier, FnStatefulSupplierOps};
///
/// let mut counter = 0;
/// let mut filtered = (move || {
///     counter += 1;
///     counter
/// }).filter(|x| x % 2 == 0);
///
/// assert_eq!(filtered.get(), None);     // 1 is odd
/// assert_eq!(filtered.get(), Some(2));  // 2 is even
/// ```
///
/// ## Combine with zip
///
/// ```rust
/// use prism3_function::{Supplier, FnStatefulSupplierOps, BoxStatefulSupplier};
///
/// let first = || 42;
/// let second = BoxStatefulSupplier::new(|| "hello");
/// let mut zipped = first.zip(second);
///
/// assert_eq!(zipped.get(), (42, "hello"));
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnStatefulSupplierOps<T>: FnMut() -> T + Sized + 'static {
    /// Maps the output using a transformation function.
    ///
    /// Consumes the closure and returns a new supplier that applies
    /// the mapper to each output.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The mapper to apply to the output
    ///
    /// # Returns
    ///
    /// A new mapped `BoxStatefulSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnStatefulSupplierOps};
    ///
    /// let mut mapped = (|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    fn map<U, M>(self, mapper: M) -> BoxStatefulSupplier<U>
    where
        M: StatefulTransformer<T, U> + 'static,
        U: 'static,
        T: 'static,
    {
        BoxStatefulSupplier::new(self).map(mapper)
    }

    /// Filters output based on a predicate.
    ///
    /// Returns a new supplier that returns `Some(value)` if the
    /// predicate is satisfied, `None` otherwise.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate to test the supplied value
    ///
    /// # Returns
    ///
    /// A new filtered `BoxStatefulSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnStatefulSupplierOps};
    ///
    /// let mut counter = 0;
    /// let mut filtered = (move || {
    ///     counter += 1;
    ///     counter
    /// }).filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), None);     // 1 is odd
    /// assert_eq!(filtered.get(), Some(2));  // 2 is even
    /// ```
    fn filter<P>(self, predicate: P) -> BoxStatefulSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
        T: 'static,
    {
        BoxStatefulSupplier::new(self).filter(predicate)
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// Consumes both suppliers and returns a new supplier that
    /// produces `(T, U)` tuples.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. Can be any type
    ///   implementing `Supplier<U>`
    ///
    /// # Returns
    ///
    /// A new `BoxStatefulSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnStatefulSupplierOps, BoxStatefulSupplier};
    ///
    /// let first = || 42;
    /// let second = BoxStatefulSupplier::new(|| "hello");
    /// let mut zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    fn zip<S, U>(self, other: S) -> BoxStatefulSupplier<(T, U)>
    where
        S: StatefulSupplier<U> + 'static,
        U: 'static,
        T: 'static,
    {
        BoxStatefulSupplier::new(self).zip(other)
    }

    /// Creates a memoizing supplier.
    ///
    /// Returns a new supplier that caches the first value it
    /// produces. All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// A new memoized `BoxStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnStatefulSupplierOps};
    ///
    /// let mut call_count = 0;
    /// let mut memoized = (move || {
    ///     call_count += 1;
    ///     42
    /// }).memoize();
    ///
    /// assert_eq!(memoized.get(), 42); // Calls underlying function
    /// assert_eq!(memoized.get(), 42); // Returns cached value
    /// ```
    fn memoize(self) -> BoxStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        BoxStatefulSupplier::new(self).memoize()
    }
}

// Implement the extension trait for all closures
impl<T, F> FnStatefulSupplierOps<T> for F where F: FnMut() -> T + Sized + 'static {}
