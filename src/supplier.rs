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
    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to `RcSupplier`.
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
    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Converts to `ArcSupplier`.
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
    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
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
        BoxSupplier::new(move || mapper.map(self.get()))
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

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        // Note: This conversion may fail if the inner function is
        // not Send. We panic here to indicate the error at runtime.
        panic!(
            "Cannot convert BoxSupplier to ArcSupplier: inner \
             function may not be Send. Create ArcSupplier directly \
             with Send closures."
        )
    }
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
                mapper.lock().unwrap().map(value)
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
                mapper.borrow_mut().map(value)
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

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcSupplier to ArcSupplier (not Send)")
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
}
