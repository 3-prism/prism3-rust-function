/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Read-only Supplier Types
//!
//! Provides read-only supplier implementations that generate and
//! return values without modifying their own state.
//!
//! # Overview
//!
//! A **Supplier** is a functional abstraction that
//! generates values without accepting input or modifying its own
//! state. Unlike `Supplier`, it uses `&self` instead of `&mut
//! self`, enabling usage in read-only contexts and lock-free
//! concurrent access.
//!
//! # Key Differences from Supplier
//!
//! | Aspect | Supplier | Supplier |
//! |--------|----------|------------------|
//! | self signature | `&mut self` | `&self` |
//! | Closure type | `FnMut() -> T` | `Fn() -> T` |
//! | Can modify state | Yes | No |
//! | Arc implementation | `Arc<Mutex<FnMut>>` | `Arc<Fn>` (lock-free!) |
//! | Use cases | Counter, generator | Factory, constant, high concurrency |
//!
//! # Three Implementations
//!
//! - **`BoxSupplier<T>`**: Single ownership using `Box<dyn
//!   Fn() -> T>`. Zero overhead, cannot be cloned. Best for
//!   one-time use in read-only contexts.
//!
//! - **`ArcSupplier<T>`**: Thread-safe shared ownership
//!   using `Arc<dyn Fn() -> T + Send + Sync>`. **Lock-free** - no
//!   Mutex needed! Can be cloned and sent across threads with
//!   excellent performance.
//!
//! - **`RcSupplier<T>`**: Single-threaded shared ownership
//!   using `Rc<dyn Fn() -> T>`. Can be cloned but not sent across
//!   threads. Lightweight alternative to `ArcSupplier`.
//!
//! # Use Cases
//!
//! ## 1. Calling in `&self` Methods
//!
//! ```rust
//! use prism3_function::{ArcSupplier, Supplier};
//!
//! struct Executor<E> {
//!     error_supplier: ArcSupplier<E>,
//! }
//!
//! impl<E> Executor<E> {
//!     fn execute(&self) -> Result<(), E> {
//!         // Can call directly in &self method!
//!         Err(self.error_supplier.get())
//!     }
//! }
//! ```
//!
//! ## 2. High-Concurrency Lock-Free Access
//!
//! ```rust
//! use prism3_function::{ArcSupplier, Supplier};
//! use std::thread;
//!
//! let factory = ArcSupplier::new(|| {
//!     String::from("Hello, World!")
//! });
//!
//! let handles: Vec<_> = (0..10)
//!     .map(|_| {
//!         let f = factory.clone();
//!         thread::spawn(move || f.get()) // Lock-free!
//!     })
//!     .collect();
//!
//! for h in handles {
//!     assert_eq!(h.join().unwrap(), "Hello, World!");
//! }
//! ```
//!
//! ## 3. Fixed Factories
//!
//! ```rust
//! use prism3_function::{BoxSupplier, Supplier};
//!
//! #[derive(Clone)]
//! struct Config {
//!     timeout: u64,
//! }
//!
//! let config_factory = BoxSupplier::new(|| Config {
//!     timeout: 30,
//! });
//!
//! assert_eq!(config_factory.get().timeout, 30);
//! assert_eq!(config_factory.get().timeout, 30);
//! ```
//!
//! # Performance Comparison
//!
//! For stateless scenarios in multi-threaded environments:
//!
//! - `ArcSupplier<T>`: Requires `Mutex`, lock contention on
//!   every `get()` call
//! - `ArcSupplier<T>`: Lock-free, can call `get()`
//!   concurrently without contention
//!
//! Benchmark results show `ArcSupplier` can be **10x
//! faster** than `ArcSupplier` in high-concurrency scenarios.
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

use crate::transformers::transformer::Transformer;

// ======================================================================
// Supplier Trait
// ======================================================================

/// Read-only supplier trait: generates values without modifying
/// state.
///
/// The core abstraction for stateless value generation. Unlike
/// `Supplier<T>`, it uses `&self` instead of `&mut self`, enabling
/// usage in read-only contexts and lock-free concurrent access.
///
/// # Key Characteristics
///
/// - **No input parameters**: Pure value generation
/// - **Read-only access**: Uses `&self`, doesn't modify state
/// - **Returns ownership**: Returns `T` (not `&T`) to avoid
///   lifetime issues
/// - **Lock-free concurrency**: `Arc` implementation doesn't need
///   `Mutex`
///
/// # Automatically Implemented for Closures
///
/// All `Fn() -> T` closures automatically implement this trait,
/// enabling seamless integration with both raw closures and
/// wrapped supplier types.
///
/// # Examples
///
/// ## Using with Generic Functions
///
/// ```rust
/// use prism3_function::{Supplier, BoxSupplier};
///
/// fn call_twice<S: Supplier<i32>>(supplier: &S)
///     -> (i32, i32)
/// {
///     (supplier.get(), supplier.get())
/// }
///
/// let s = BoxSupplier::new(|| 42);
/// assert_eq!(call_twice(&s), (42, 42));
///
/// let closure = || 100;
/// assert_eq!(call_twice(&closure), (100, 100));
/// ```
///
/// ## Stateless Factory
///
/// ```rust
/// use prism3_function::Supplier;
///
/// struct User {
///     name: String,
/// }
///
/// impl User {
///     fn new() -> Self {
///         User {
///             name: String::from("Default"),
///         }
///     }
/// }
///
/// let factory = || User::new();
/// let user1 = factory.get();
/// let user2 = factory.get();
/// // Each call creates a new User instance
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait Supplier<T> {
    /// Generates and returns a value.
    ///
    /// Executes the underlying function and returns the generated
    /// value. Uses `&self` because the supplier doesn't modify its
    /// own state.
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
    /// let supplier = BoxSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    fn get(&self) -> T;

    /// Converts to `BoxSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in a `BoxSupplier`. Custom implementations
    /// can override this method for optimization purposes.
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
    /// let boxed = closure.into_box();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn into_box(self) -> BoxSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplier::new(move || self.get())
    }

    /// Converts to `RcSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `RcSupplier`. Custom implementations
    /// can override this method for optimization purposes.
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
    /// let rc = closure.into_rc();
    /// assert_eq!(rc.get(), 42);
    /// ```
    fn into_rc(self) -> RcSupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcSupplier::new(move || self.get())
    }

    /// Converts to `ArcSupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `ArcSupplier`. Custom implementations
    /// can override this method for optimization purposes.
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
    /// let arc = closure.into_arc();
    /// assert_eq!(arc.get(), 42);
    /// ```
    fn into_arc(self) -> ArcSupplier<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + 'static,
    {
        ArcSupplier::new(move || self.get())
    }

    /// Converts to a closure implementing `FnMut() -> T`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in a closure. Custom implementations can override
    /// this method for optimization purposes.
    ///
    /// # Returns
    ///
    /// A closure implementing `FnMut() -> T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut fn_mut = closure.into_fn();
    /// assert_eq!(fn_mut(), 42);
    /// assert_eq!(fn_mut(), 42);
    /// ```
    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized,
    {
        move || self.get()
    }

    /// Converts to `BoxSupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in a
    /// `BoxSupplier`. Requires `Self: Clone`. Custom
    /// implementations can override this method for optimization.
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
    /// let boxed = closure.to_box();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_box()
    }

    /// Converts to `RcSupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in an
    /// `RcSupplier`. Requires `Self: Clone`. Custom
    /// implementations can override this method for optimization.
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
    /// let rc = closure.to_rc();
    /// assert_eq!(rc.get(), 42);
    /// ```
    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone().into_rc()
    }

    /// Converts to `ArcSupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in an
    /// `ArcSupplier`. Requires `Self: Clone + Send + Sync`.
    /// Custom implementations can override this method for
    /// optimization.
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
    /// let arc = closure.to_arc();
    /// assert_eq!(arc.get(), 42);
    /// ```
    fn to_arc(&self) -> ArcSupplier<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + 'static,
    {
        self.clone().into_arc()
    }

    /// Converts to a closure by cloning.
    ///
    /// This method clones the supplier and wraps it in a closure
    /// implementing `FnMut() -> T`. Requires `Self: Clone`. Custom
    /// implementations can override this method for optimization.
    ///
    /// # Returns
    ///
    /// A closure implementing `FnMut() -> T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Supplier;
    ///
    /// let closure = || 42;
    /// let mut fn_mut = closure.to_fn();
    /// assert_eq!(fn_mut(), 42);
    /// assert_eq!(fn_mut(), 42);
    /// ```
    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        self.clone().into_fn()
    }
}

// ======================================================================
// BoxSupplier - Single Ownership Implementation
// ======================================================================

/// Box-based single ownership read-only supplier.
///
/// Uses `Box<dyn Fn() -> T>` for single ownership scenarios. This
/// is the most lightweight read-only supplier with zero reference
/// counting overhead.
///
/// # Ownership Model
///
/// Methods consume `self` (move semantics) or borrow `&self` for
/// read-only operations. When you call methods like `map()`, the
/// original supplier is consumed and you get a new one:
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
/// ## Constant Factory
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let factory = BoxSupplier::new(|| 42);
/// assert_eq!(factory.get(), 42);
/// assert_eq!(factory.get(), 42);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BoxSupplier, Supplier};
///
/// let pipeline = BoxSupplier::new(|| 10)
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
    function: Box<dyn Fn() -> T>,
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
    /// let supplier = BoxSupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
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
    /// let constant = BoxSupplier::constant(42);
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
    /// * `mapper` - The transformer to apply to the output. Can be a
    ///   closure, function pointer, or any type implementing
    ///   `Transformer<T, U>`.
    ///
    /// # Returns
    ///
    /// A new mapped `BoxSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplier, Supplier};
    ///
    /// let mapped = BoxSupplier::new(|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    pub fn map<U, M>(self, mapper: M) -> BoxSupplier<U>
    where
        M: Transformer<T, U> + 'static,
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
    /// let filtered = BoxSupplier::new(|| 42)
    ///     .filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(self, predicate: P) -> BoxSupplier<Option<T>>
    where
        P: Fn(&T) -> bool + 'static,
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
    /// let zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    pub fn zip<U>(self, other: BoxSupplier<U>) -> BoxSupplier<(T, U)>
    where
        U: 'static,
    {
        BoxSupplier::new(move || (self.get(), other.get()))
    }
}

impl<T> Supplier<T> for BoxSupplier<T> {
    fn get(&self) -> T {
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

    // do NOT override BoxSupplier::to_arc() because BoxSupplier
    // is not Send + Sync and calling BoxSupplier::to_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut() -> T {
        move || (self.function)()
    }

    // Note: to_box, to_rc, to_arc, and to_fn cannot be implemented
    // for BoxSupplier because it does not implement Clone.
    // Box provides unique ownership and cannot be cloned unless
    // the inner type implements Clone, which dyn Fn() -> T does not.
    //
    // If you call these methods on BoxSupplier, the compiler
    // will fail with an error indicating that BoxSupplier<T>
    // does not implement Clone, which is required by the default
    // implementations of to_box, to_rc, to_arc, and to_fn.
}

// ======================================================================
// ArcSupplier - Thread-safe Shared Ownership Implementation
// ======================================================================

/// Thread-safe shared ownership read-only supplier.
///
/// Uses `Arc<dyn Fn() -> T + Send + Sync>` for thread-safe shared
/// ownership. **Lock-free** - no `Mutex` needed! Can be cloned and
/// sent across threads with excellent concurrent performance.
///
/// # Ownership Model
///
/// Methods borrow `&self` instead of consuming `self`. The
/// original supplier remains usable after method calls:
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
///
/// let source = ArcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Lock-Free Performance
///
/// Unlike `ArcSupplier`, this implementation doesn't need `Mutex`.
/// Multiple threads can call `get()` concurrently without lock
/// contention, making it ideal for high-concurrency scenarios.
///
/// # Examples
///
/// ## Thread-safe Factory
///
/// ```rust
/// use prism3_function::{ArcSupplier, Supplier};
/// use std::thread;
///
/// let factory = ArcSupplier::new(|| {
///     String::from("Hello")
/// });
///
/// let f1 = factory.clone();
/// let f2 = factory.clone();
///
/// let h1 = thread::spawn(move || f1.get());
/// let h2 = thread::spawn(move || f2.get());
///
/// assert_eq!(h1.join().unwrap(), "Hello");
/// assert_eq!(h2.join().unwrap(), "Hello");
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
/// assert_eq!(base.get(), 10);
/// assert_eq!(doubled.get(), 20);
/// assert_eq!(tripled.get(), 30);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcSupplier<T> {
    function: Arc<dyn Fn() -> T + Send + Sync>,
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
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        ArcSupplier {
            function: Arc::new(f),
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
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        ArcSupplier::new(move || value.clone())
    }

    /// Maps the output using a transformation function.
    ///
    /// Borrows `&self`, doesn't consume the original supplier.
    ///
    /// # Parameters
    ///
    /// * `mapper` - The transformer to apply to the output. Can be a
    ///   closure, function pointer, or any type implementing
    ///   `Transformer<T, U>`.
    ///
    /// # Returns
    ///
    /// A new mapped `ArcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcSupplier, Supplier};
    ///
    /// let source = ArcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// assert_eq!(mapped.get(), 20);
    /// ```
    pub fn map<U, M>(&self, mapper: M) -> ArcSupplier<U>
    where
        M: Transformer<T, U> + Send + Sync + 'static,
        U: Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let mapper = Arc::new(mapper);
        ArcSupplier {
            function: Arc::new(move || {
                let value = self_fn();
                mapper.apply(value)
            }),
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
    ///
    /// let source = ArcSupplier::new(|| 42);
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(&self, predicate: P) -> ArcSupplier<Option<T>>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let predicate = Arc::new(predicate);
        ArcSupplier {
            function: Arc::new(move || {
                let value = self_fn();
                if predicate(&value) {
                    Some(value)
                } else {
                    None
                }
            }),
        }
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. **Note:
    ///   Passed by reference, so the original supplier remains
    ///   usable.**
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
    /// assert_eq!(zipped.get(), (42, "hello"));
    ///
    /// // Both first and second still usable
    /// assert_eq!(first.get(), 42);
    /// assert_eq!(second.get(), "hello");
    /// ```
    pub fn zip<U>(&self, other: &ArcSupplier<U>) -> ArcSupplier<(T, U)>
    where
        U: Send + 'static,
    {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&other.function);
        ArcSupplier {
            function: Arc::new(move || (first(), second())),
        }
    }
}

impl<T> Supplier<T> for ArcSupplier<T> {
    fn get(&self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        BoxSupplier::new(move || (self.function)())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        RcSupplier::new(move || (self.function)())
    }

    fn into_arc(self) -> ArcSupplier<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut() -> T {
        move || (self.function)()
    }

    // Optimized implementations using Arc::clone instead of
    // wrapping in a closure

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxSupplier::new(move || self_fn())
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        RcSupplier::new(move || self_fn())
    }

    fn to_arc(&self) -> ArcSupplier<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + 'static,
    {
        self.clone()
    }

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        let self_fn = self.function.clone();
        move || self_fn()
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

// ======================================================================
// RcSupplier - Single-threaded Shared Ownership
// ======================================================================

/// Single-threaded shared ownership read-only supplier.
///
/// Uses `Rc<dyn Fn() -> T>` for single-threaded shared ownership.
/// Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcSupplier`, methods borrow `&self` instead of
/// consuming `self`:
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
/// ## Shared Factory
///
/// ```rust
/// use prism3_function::{RcSupplier, Supplier};
///
/// let factory = RcSupplier::new(|| {
///     String::from("Hello")
/// });
///
/// let f1 = factory.clone();
/// let f2 = factory.clone();
/// assert_eq!(f1.get(), "Hello");
/// assert_eq!(f2.get(), "Hello");
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
/// assert_eq!(base.get(), 10);
/// assert_eq!(doubled.get(), 20);
/// assert_eq!(tripled.get(), 30);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcSupplier<T> {
    function: Rc<dyn Fn() -> T>,
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
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        RcSupplier {
            function: Rc::new(f),
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
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
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
    /// * `mapper` - The transformer to apply to the output. Can be a
    ///   closure, function pointer, or any type implementing
    ///   `Transformer<T, U>`.
    ///
    /// # Returns
    ///
    /// A new mapped `RcSupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcSupplier, Supplier};
    ///
    /// let source = RcSupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// assert_eq!(mapped.get(), 20);
    /// ```
    pub fn map<U, M>(&self, mapper: M) -> RcSupplier<U>
    where
        M: Transformer<T, U> + 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let mapper = Rc::new(mapper);
        RcSupplier {
            function: Rc::new(move || {
                let value = self_fn();
                mapper.apply(value)
            }),
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
    ///
    /// let source = RcSupplier::new(|| 42);
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(&self, predicate: P) -> RcSupplier<Option<T>>
    where
        P: Fn(&T) -> bool + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let predicate = Rc::new(predicate);
        RcSupplier {
            function: Rc::new(move || {
                let value = self_fn();
                if predicate(&value) {
                    Some(value)
                } else {
                    None
                }
            }),
        }
    }

    /// Combines this supplier with another, producing a tuple.
    ///
    /// # Parameters
    ///
    /// * `other` - The other supplier to combine with. **Note:
    ///   Passed by reference, so the original supplier remains
    ///   usable.**
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
    /// assert_eq!(zipped.get(), (42, "hello"));
    ///
    /// // Both first and second still usable
    /// assert_eq!(first.get(), 42);
    /// assert_eq!(second.get(), "hello");
    /// ```
    pub fn zip<U>(&self, other: &RcSupplier<U>) -> RcSupplier<(T, U)>
    where
        U: 'static,
    {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&other.function);
        RcSupplier {
            function: Rc::new(move || (first(), second())),
        }
    }
}

impl<T> Supplier<T> for RcSupplier<T> {
    fn get(&self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxSupplier<T>
    where
        T: 'static,
    {
        BoxSupplier::new(move || (self.function)())
    }

    fn into_rc(self) -> RcSupplier<T>
    where
        T: 'static,
    {
        self
    }

    // do NOT override RcSupplier::to_arc() because RcSupplier
    // is not Send + Sync and calling RcSupplier::to_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut() -> T {
        move || (self.function)()
    }

    // Optimized implementations using Rc::clone instead of wrapping
    // in a closure

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        BoxSupplier::new(move || self_fn())
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone()
    }

    // Note: to_arc cannot be implemented for RcSupplier
    // because Rc is not Send + Sync, which is required for
    // ArcSupplier.
    //
    // If you call to_arc on RcSupplier, the compiler will
    // fail with an error indicating that RcSupplier<T> does
    // not satisfy the Send + Sync bounds required by the default
    // implementation of to_arc.

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        let self_fn = self.function.clone();
        move || self_fn()
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

// ======================================================================
// Implement Supplier for Closures
// ======================================================================

impl<T, F> Supplier<T> for F
where
    F: Fn() -> T,
{
    fn get(&self) -> T {
        self()
    }

    // Use optimized implementations for closures instead of the
    // default implementations. This avoids double wrapping by
    // directly creating the target type.

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
        Self: Sized + Send + Sync + 'static,
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

    // Optimized implementations for to_* methods

    fn to_box(&self) -> BoxSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.clone();
        BoxSupplier::new(self_fn)
    }

    fn to_rc(&self) -> RcSupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = self.clone();
        RcSupplier::new(self_fn)
    }

    fn to_arc(&self) -> ArcSupplier<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + 'static,
    {
        let self_fn = self.clone();
        ArcSupplier::new(self_fn)
    }

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        self.clone()
    }
}

// ======================================================================
// Note on Extension Traits for Closures
// ======================================================================
//
// We don't provide `FnSupplierOps` trait for `Fn() -> T` closures
// because:
//
// 1. All `Fn` closures also implement `FnMut`, so they can use `FnSupplierOps`
//    from the `supplier` module
// 2. Providing both would cause ambiguity errors due to overlapping trait impls
// 3. Rust doesn't support negative trait bounds to exclude `FnMut`
//
// Users of `Fn` closures should use `FnSupplierOps` from `supplier` module,
// or explicitly convert to `BoxSupplier` using `.into_box()` first.
