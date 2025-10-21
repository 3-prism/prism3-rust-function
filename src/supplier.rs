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
//! - **`BoxSupplier<T>`**: Single ownership using `Box<dyn FnMut()
//!   -> T>`. Zero overhead, cannot be cloned. Best for one-time use
//!   and builder patterns.
//!
//! - **`ArcSupplier<T>`**: Thread-safe shared ownership using
//!   `Arc<Mutex<dyn FnMut() -> T + Send>>`. Can be cloned and sent
//!   across threads. Higher overhead due to locking.
//!
//! - **`RcSupplier<T>`**: Single-threaded shared ownership using
//!   `Rc<RefCell<dyn FnMut() -> T>>`. Can be cloned but not sent
//!   across threads. Lower overhead than `ArcSupplier`.
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
//! use prism3_function::{BoxSupplier, Supplier};
//!
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
//! let mut pipeline = BoxSupplier::new(|| 10)
//!     .map(|x| x * 2)
//!     .map(|x| x + 5);
//!
//! assert_eq!(pipeline.get(), 25);
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
use std::sync::{Arc, Mutex};

use crate::mapper::Mapper;

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
/// use prism3_function::{Supplier, BoxSupplier};
///
/// fn call_twice<S: Supplier<i32>>(supplier: &mut S) -> (i32, i32) {
///     (supplier.get(), supplier.get())
/// }
///
/// let mut s = BoxSupplier::new(|| 42);
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
pub trait Supplier<T> {
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
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// let mut supplier = BoxSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    fn get(&mut self) -> T;

    /// Converts to `BoxSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in a `BoxSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `BoxSupplier<T>` instance
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
    fn into_box(mut self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplier::new(move || self.get())
    }

    /// Converts to `RcSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `RcSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `RcSupplier<T>` instance
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
    fn into_rc(mut self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcSupplier::new(move || self.get())
    }

    /// Converts to `ArcSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `ArcSupplier`. Custom implementations can
    /// override this for more efficient conversions.
    ///
    /// # Returns
    ///
    /// A new `ArcSupplier<T>` instance
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
    fn into_arc(mut self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcSupplier::new(move || self.get())
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
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// let supplier = BoxSupplier::new(|| 42);
    /// let mut closure = supplier.into_fn();
    /// assert_eq!(closure(), 42);
    /// assert_eq!(closure(), 42);
    /// ```
    ///
    /// ## Using with functions that expect FnMut
    ///
    /// ```rust
    /// use prism3_function::{Supplier, BoxSupplier};
    ///
    /// fn call_fn_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
    ///     (f(), f())
    /// }
    ///
    /// let supplier = BoxSupplier::new(|| 100);
    /// let closure = supplier.into_fn();
    /// assert_eq!(call_fn_twice(closure), (100, 100));
    /// ```
    fn into_fn(mut self) -> impl FnMut() -> T
    where
        Self: Sized,
    {
        move || self.get()
    }

    /// Creates a `BoxSupplier` from a cloned supplier.
    ///
    /// Uses `Clone` to obtain an owned copy and converts it into a
    /// `BoxSupplier`. Implementations can override this for a more
    /// efficient conversion.
    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Creates an `RcSupplier` from a cloned supplier.
    ///
    /// Uses `Clone` to obtain an owned copy and converts it into an
    /// `RcSupplier`. Implementations can override it for better
    /// performance.
    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Creates an `ArcSupplier` from a cloned supplier.
    ///
    /// Requires the supplier and produced values to be `Send` so the
    /// resulting supplier can be shared across threads.
    fn to_arc(&self) -> ArcSupplier<T>
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
// BoxSupplier - Single Ownership Implementation
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
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let supplier = BoxSupplier::new(|| 10);
/// let mapped = supplier.map(|x| x * 2);
/// // supplier is no longer usable here
/// ```
///
/// # Examples
///
/// ## Counter
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
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let mut pipeline = BoxSupplier::new(|| 10)
///     .map(|x| x * 2)
///     .map(|x| x + 5);
///
/// assert_eq!(pipeline.get(), 25);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxSupplier<T> {
    function: Box<dyn FnMut() -> T>,
}

impl<T> BoxSupplier<T>
where
    T: 'static,
{
    /// Creates a new `BoxSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxSupplier<T>` instance
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
        BoxSupplier {
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
    ///   - Any type implementing `Mapper<T, U>`
    ///
    /// # Returns
    ///
    /// A new mapped `BoxSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mut mapped = BoxSupplier::new(|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    ///
    /// ## Using with Mapper object
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, BoxMapper, Supplier, Mapper};
    ///
    /// let mapper = BoxMapper::new(|x: i32| x * 2);
    /// let mut supplier = BoxSupplier::new(|| 10)
    ///     .map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(mut self, mut mapper: F) -> BoxSupplier<U>
    where
        F: Mapper<T, U> + 'static,
        U: 'static,
    {
        BoxSupplier::new(move || mapper.apply(self.get()))
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
    /// A new filtered `BoxSupplier<Option<T>>`
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

    /// Combines this supplier with another, producing a tuple.
    ///
    /// Consumes both suppliers and returns a new supplier that
    /// produces `(T, U)` tuples.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with
    ///
    /// # Returns
    ///
    /// A new `BoxSupplier<(T, U)>`
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

    /// Creates a memoizing supplier.
    ///
    /// Returns a new supplier that caches the first value it
    /// produces. All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// A new memoized `BoxSupplier<T>`
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
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&mut self) -> T {
        (self.function)()
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
        RcSupplier::new(self.function)
    }

    // into_arc cannot be implemented because the inner function may not be Send.
    // Attempting to call this method will result in a compiler error due to missing Send bound.
    // Use ArcSupplier::new directly with a Send closure instead.
    // compile_error!("Cannot convert BoxSupplier to ArcSupplier: inner function may not implement Send");

    fn into_fn(self) -> impl FnMut() -> T {
        self.function
    }

    // NOTE: `BoxSupplier` is not `Clone`, so it cannot offer optimized
    // `to_box`, `to_rc`, `to_arc`, or `to_fn` implementations. Invoking
    // the default trait methods will not compile because the required
    // `Clone` bound is not satisfied.
}

// ==========================================================================
// ArcSupplier - Thread-safe Shared Ownership Implementation
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
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let source = ArcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Examples
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
/// assert!(v1 != v2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let base = ArcSupplier::new(|| 10);
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
pub struct ArcSupplier<T> {
    function: Arc<Mutex<dyn FnMut() -> T + Send>>,
}

impl<T> ArcSupplier<T>
where
    T: Send + 'static,
{
    /// Creates a new `ArcSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `ArcSupplier<T>` instance
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
    ///   - Any type implementing `Mapper<T, U> + Send`
    ///
    /// # Returns
    ///
    /// A new mapped `ArcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let source = ArcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    ///
    /// ## Using with Mapper object
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, ArcMapper, Supplier, Mapper};
    ///
    /// let mapper = ArcMapper::new(|x: i32| x * 2);
    /// let source = ArcSupplier::new(|| 10);
    /// let mut supplier = source.map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> ArcSupplier<U>
    where
        F: Mapper<T, U> + Send + 'static,
        U: Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let mapper = Arc::new(Mutex::new(mapper));
        ArcSupplier {
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
    /// A new filtered `ArcSupplier<Option<T>>`
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
        let self_fn = Arc::clone(&self.function);
        let predicate = Arc::new(Mutex::new(predicate));
        ArcSupplier {
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
    /// * `other` - The other supplier to combine with. **Note: This parameter
    ///   is passed by reference, so the original supplier remains usable.**
    ///   Can be:
    ///   - An `ArcSupplier<U>` (passed by reference)
    ///   - Any type implementing `Supplier<U> + Send`
    ///
    /// # Returns
    ///
    /// A new `ArcSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let first = ArcSupplier::new(|| 42);
    /// let second = ArcSupplier::new(|| "hello");
    ///
    /// // second is passed by reference, so it remains usable
    /// let zipped = first.zip(&second);
    ///
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    ///
    /// // Both first and second still usable
    /// let mut f = first;
    /// let mut s = second;
    /// assert_eq!(f.get(), 42);
    /// assert_eq!(s.get(), "hello");
    /// ```
    pub fn zip<U>(&self, other: &ArcSupplier<U>) -> ArcSupplier<(T, U)>
    where
        U: Send + 'static,
    {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&other.function);
        ArcSupplier {
            function: Arc::new(Mutex::new(move || {
                (first.lock().unwrap()(), second.lock().unwrap()())
            })),
        }
    }

    /// Creates a memoizing supplier.
    ///
    /// # Returns
    ///
    /// A new memoized `ArcSupplier<T>`
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
    /// assert_eq!(*call_count.lock().unwrap(), 1);
    /// ```
    pub fn memoize(&self) -> ArcSupplier<T>
    where
        T: Clone + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let cache: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        ArcSupplier {
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

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&mut self) -> T {
        (self.function.lock().unwrap())()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxSupplier::new(move || self_fn.lock().unwrap()())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        RcSupplier::new(move || self_fn.lock().unwrap()())
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function.lock().unwrap()()
    }

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Arc::clone(&self.function);
        BoxSupplier::new(move || function.lock().unwrap()())
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Arc::clone(&self.function);
        RcSupplier::new(move || function.lock().unwrap()())
    }

    fn to_arc(&self) -> ArcSupplier<T>
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

impl<T> Clone for ArcSupplier<T> {
    /// Clones the `ArcSupplier`.
    ///
    /// Creates a new instance that shares the underlying function
    /// with the original.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
        }
    }
}

// ==========================================================================
// RcSupplier - Single-threaded Shared Ownership Implementation
// ==========================================================================

/// Single-threaded shared ownership supplier.
///
/// Uses `Rc<RefCell<dyn FnMut() -> T>>` for single-threaded shared
/// ownership. Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcSupplier`, methods borrow `&self` instead of consuming
/// `self`:
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let source = RcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Examples
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
/// assert_eq!(s1.get(), 1);
/// assert_eq!(s2.get(), 2);
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let base = RcSupplier::new(|| 10);
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
pub struct RcSupplier<T> {
    function: Rc<RefCell<dyn FnMut() -> T>>,
}

impl<T> RcSupplier<T>
where
    T: 'static,
{
    /// Creates a new `RcSupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `RcSupplier<T>` instance
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
    ///   - Any type implementing `Mapper<T, U>`
    ///
    /// # Returns
    ///
    /// A new mapped `RcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ## Using with closure
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let source = RcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// let mut s = mapped;
    /// assert_eq!(s.get(), 20);
    /// ```
    ///
    /// ## Using with Mapper object
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, RcMapper, Supplier, Mapper};
    ///
    /// let mapper = RcMapper::new(|x: i32| x * 2);
    /// let source = RcSupplier::new(|| 10);
    /// let mut supplier = source.map(mapper);
    /// assert_eq!(supplier.get(), 20);
    /// ```
    pub fn map<U, F>(&self, mapper: F) -> RcSupplier<U>
    where
        F: Mapper<T, U> + 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let mapper = Rc::new(RefCell::new(mapper));
        RcSupplier {
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
    /// A new filtered `RcSupplier<Option<T>>`
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
        let self_fn = Rc::clone(&self.function);
        let predicate = Rc::new(RefCell::new(predicate));
        RcSupplier {
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
    /// * `other` - The other supplier to combine with. **Note: This parameter
    ///   is passed by reference, so the original supplier remains usable.**
    ///   Can be:
    ///   - An `RcSupplier<U>` (passed by reference)
    ///   - Any type implementing `Supplier<U>`
    ///
    /// # Returns
    ///
    /// A new `RcSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let first = RcSupplier::new(|| 42);
    /// let second = RcSupplier::new(|| "hello");
    ///
    /// // second is passed by reference, so it remains usable
    /// let zipped = first.zip(&second);
    ///
    /// let mut z = zipped;
    /// assert_eq!(z.get(), (42, "hello"));
    ///
    /// // Both first and second still usable
    /// let mut f = first;
    /// let mut s = second;
    /// assert_eq!(f.get(), 42);
    /// assert_eq!(s.get(), "hello");
    /// ```
    pub fn zip<U>(&self, other: &RcSupplier<U>) -> RcSupplier<(T, U)>
    where
        U: 'static,
    {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&other.function);
        RcSupplier {
            function: Rc::new(RefCell::new(move || {
                (first.borrow_mut()(), second.borrow_mut()())
            })),
        }
    }

    /// Creates a memoizing supplier.
    ///
    /// # Returns
    ///
    /// A new memoized `RcSupplier<T>`
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
    /// assert_eq!(*call_count.borrow(), 1);
    /// ```
    pub fn memoize(&self) -> RcSupplier<T>
    where
        T: Clone + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let cache: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
        RcSupplier {
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

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&mut self) -> T {
        (self.function.borrow_mut())()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxSupplier::new(move || self_fn.borrow_mut()())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        self
    }

    // into_arc cannot be implemented because RcSupplier does not implement Send.
    // Attempting to call this method will result in a compiler error due to missing Send bound.
    // Use ArcSupplier::new directly instead.
    // compile_error!("Cannot convert RcSupplier to ArcSupplier: RcSupplier does not implement Send");

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function.borrow_mut()()
    }

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        let function = Rc::clone(&self.function);
        BoxSupplier::new(move || function.borrow_mut()())
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone()
    }

    // NOTE: `RcSupplier` cannot be converted to `ArcSupplier` because it
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

impl<T> Clone for RcSupplier<T> {
    /// Clones the `RcSupplier`.
    ///
    /// Creates a new instance that shares the underlying function
    /// with the original.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
        }
    }
}

// ==========================================================================
// Implement Supplier for Closures
// ==========================================================================

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
        Self: Sized,
    {
        self
    }

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + Sized + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    fn to_arc(&self) -> ArcSupplier<T>
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
/// wrapping in `BoxSupplier`.
///
/// This trait is automatically implemented for all closures and function
/// pointers that implement `FnMut() -> T`.
///
/// # Design Rationale
///
/// While closures automatically implement `Supplier<T>` through blanket
/// implementation, they don't have access to instance methods like
/// `map`, `filter`, and `zip`. This extension trait provides those
/// methods, returning `BoxSupplier` for maximum flexibility.
///
/// # Examples
///
/// ## Map transformation
///
/// ```rust
/// use prism3_function::{Supplier, FnSupplierOps};
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
/// use prism3_function::{Supplier, FnSupplierOps};
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
/// use prism3_function::{Supplier, FnSupplierOps, BoxSupplier};
///
/// let first = || 42;
/// let second = BoxSupplier::new(|| "hello");
/// let mut zipped = first.zip(second);
///
/// assert_eq!(zipped.get(), (42, "hello"));
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnSupplierOps<T>: FnMut() -> T + Sized + 'static {
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
    /// A new mapped `BoxSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnSupplierOps};
    ///
    /// let mut mapped = (|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    fn map<U, M>(self, mapper: M) -> BoxSupplier<U>
    where
        M: Mapper<T, U> + 'static,
        U: 'static,
        T: 'static,
    {
        BoxSupplier::new(self).map(mapper)
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
    /// A new filtered `BoxSupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnSupplierOps};
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
    fn filter<P>(self, predicate: P) -> BoxSupplier<Option<T>>
    where
        P: FnMut(&T) -> bool + 'static,
        T: 'static,
    {
        BoxSupplier::new(self).filter(predicate)
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// Consumes both suppliers and returns a new supplier that
    /// produces `(T, U)` tuples.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with
    ///
    /// # Returns
    ///
    /// A new `BoxSupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnSupplierOps, BoxSupplier};
    ///
    /// let first = || 42;
    /// let second = BoxSupplier::new(|| "hello");
    /// let mut zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    fn zip<U>(self, other: BoxSupplier<U>) -> BoxSupplier<(T, U)>
    where
        U: 'static,
        T: 'static,
    {
        BoxSupplier::new(self).zip(other)
    }

    /// Creates a memoizing supplier.
    ///
    /// Returns a new supplier that caches the first value it
    /// produces. All subsequent calls return the cached value.
    ///
    /// # Returns
    ///
    /// A new memoized `BoxSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Supplier, FnSupplierOps};
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
    fn memoize(self) -> BoxSupplier<T>
    where
        T: Clone + 'static,
    {
        BoxSupplier::new(self).memoize()
    }
}

// Implement the extension trait for all closures
impl<T, F> FnSupplierOps<T> for F where F: FnMut() -> T + Sized + 'static {}
