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
//! A **ReadonlySupplier** is a functional abstraction that
//! generates values without accepting input or modifying its own
//! state. Unlike `Supplier`, it uses `&self` instead of `&mut
//! self`, enabling usage in read-only contexts and lock-free
//! concurrent access.
//!
//! # Key Differences from Supplier
//!
//! | Aspect | Supplier | ReadonlySupplier |
//! |--------|----------|------------------|
//! | self signature | `&mut self` | `&self` |
//! | Closure type | `FnMut() -> T` | `Fn() -> T` |
//! | Can modify state | Yes | No |
//! | Arc implementation | `Arc<Mutex<FnMut>>` | `Arc<Fn>` (lock-free!) |
//! | Use cases | Counter, generator | Factory, constant, high concurrency |
//!
//! # Three Implementations
//!
//! - **`BoxReadonlySupplier<T>`**: Single ownership using `Box<dyn
//!   Fn() -> T>`. Zero overhead, cannot be cloned. Best for
//!   one-time use in read-only contexts.
//!
//! - **`ArcReadonlySupplier<T>`**: Thread-safe shared ownership
//!   using `Arc<dyn Fn() -> T + Send + Sync>`. **Lock-free** - no
//!   Mutex needed! Can be cloned and sent across threads with
//!   excellent performance.
//!
//! - **`RcReadonlySupplier<T>`**: Single-threaded shared ownership
//!   using `Rc<dyn Fn() -> T>`. Can be cloned but not sent across
//!   threads. Lightweight alternative to `ArcReadonlySupplier`.
//!
//! # Use Cases
//!
//! ## 1. Calling in `&self` Methods
//!
//! ```rust
//! use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
//!
//! struct Executor<E> {
//!     error_supplier: ArcReadonlySupplier<E>,
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
//! use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
//! use std::thread;
//!
//! let factory = ArcReadonlySupplier::new(|| {
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
//! use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
//!
//! #[derive(Clone)]
//! struct Config {
//!     timeout: u64,
//! }
//!
//! let config_factory = BoxReadonlySupplier::new(|| Config {
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
//! - `ArcReadonlySupplier<T>`: Lock-free, can call `get()`
//!   concurrently without contention
//!
//! Benchmark results show `ArcReadonlySupplier` can be **10x
//! faster** than `ArcSupplier` in high-concurrency scenarios.
//!
//! # Author
//!
//! Haixing Hu

use std::rc::Rc;
use std::sync::Arc;

use crate::transformer::Transformer;

// ======================================================================
// ReadonlySupplier Trait
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
/// use prism3_function::{ReadonlySupplier, BoxReadonlySupplier};
///
/// fn call_twice<S: ReadonlySupplier<i32>>(supplier: &S)
///     -> (i32, i32)
/// {
///     (supplier.get(), supplier.get())
/// }
///
/// let s = BoxReadonlySupplier::new(|| 42);
/// assert_eq!(call_twice(&s), (42, 42));
///
/// let closure = || 100;
/// assert_eq!(call_twice(&closure), (100, 100));
/// ```
///
/// ## Stateless Factory
///
/// ```rust
/// use prism3_function::ReadonlySupplier;
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
pub trait ReadonlySupplier<T> {
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
    /// use prism3_function::{ReadonlySupplier, BoxReadonlySupplier};
    ///
    /// let supplier = BoxReadonlySupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    fn get(&self) -> T;

    /// Converts to `BoxReadonlySupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in a `BoxReadonlySupplier`. Custom implementations
    /// can override this method for optimization purposes.
    ///
    /// # Returns
    ///
    /// A new `BoxReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let boxed = closure.into_box();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn into_box(self) -> BoxReadonlySupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxReadonlySupplier::new(move || self.get())
    }

    /// Converts to `RcReadonlySupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `RcReadonlySupplier`. Custom implementations
    /// can override this method for optimization purposes.
    ///
    /// # Returns
    ///
    /// A new `RcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let rc = closure.into_rc();
    /// assert_eq!(rc.get(), 42);
    /// ```
    fn into_rc(self) -> RcReadonlySupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcReadonlySupplier::new(move || self.get())
    }

    /// Converts to `ArcReadonlySupplier`.
    ///
    /// This method has a default implementation that wraps the
    /// supplier in an `ArcReadonlySupplier`. Custom implementations
    /// can override this method for optimization purposes.
    ///
    /// # Returns
    ///
    /// A new `ArcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let arc = closure.into_arc();
    /// assert_eq!(arc.get(), 42);
    /// ```
    fn into_arc(self) -> ArcReadonlySupplier<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + 'static,
    {
        ArcReadonlySupplier::new(move || self.get())
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
    /// use prism3_function::ReadonlySupplier;
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

    /// Converts to `BoxReadonlySupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in a
    /// `BoxReadonlySupplier`. Requires `Self: Clone`. Custom
    /// implementations can override this method for optimization.
    ///
    /// # Returns
    ///
    /// A new `BoxReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let boxed = closure.to_box();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn to_box(&self) -> BoxReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        BoxReadonlySupplier::new(move || cloned.get())
    }

    /// Converts to `RcReadonlySupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in an
    /// `RcReadonlySupplier`. Requires `Self: Clone`. Custom
    /// implementations can override this method for optimization.
    ///
    /// # Returns
    ///
    /// A new `RcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let rc = closure.to_rc();
    /// assert_eq!(rc.get(), 42);
    /// ```
    fn to_rc(&self) -> RcReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        RcReadonlySupplier::new(move || cloned.get())
    }

    /// Converts to `ArcReadonlySupplier` by cloning.
    ///
    /// This method clones the supplier and wraps it in an
    /// `ArcReadonlySupplier`. Requires `Self: Clone + Send + Sync`.
    /// Custom implementations can override this method for
    /// optimization.
    ///
    /// # Returns
    ///
    /// A new `ArcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ReadonlySupplier;
    ///
    /// let closure = || 42;
    /// let arc = closure.to_arc();
    /// assert_eq!(arc.get(), 42);
    /// ```
    fn to_arc(&self) -> ArcReadonlySupplier<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + 'static,
    {
        let cloned = self.clone();
        ArcReadonlySupplier::new(move || cloned.get())
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
    /// use prism3_function::ReadonlySupplier;
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
        let cloned = self.clone();
        move || cloned.get()
    }
}

// ======================================================================
// BoxReadonlySupplier - Single Ownership Implementation
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
/// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
///
/// let supplier = BoxReadonlySupplier::new(|| 10);
/// let mapped = supplier.map(|x| x * 2);
/// // supplier is no longer usable here
/// ```
///
/// # Examples
///
/// ## Constant Factory
///
/// ```rust
/// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
///
/// let factory = BoxReadonlySupplier::new(|| 42);
/// assert_eq!(factory.get(), 42);
/// assert_eq!(factory.get(), 42);
/// ```
///
/// ## Method Chaining
///
/// ```rust
/// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
///
/// let pipeline = BoxReadonlySupplier::new(|| 10)
///     .map(|x| x * 2)
///     .map(|x| x + 5);
///
/// assert_eq!(pipeline.get(), 25);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxReadonlySupplier<T> {
    function: Box<dyn Fn() -> T>,
}

impl<T> BoxReadonlySupplier<T>
where
    T: 'static,
{
    /// Creates a new `BoxReadonlySupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
    ///
    /// let supplier = BoxReadonlySupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        BoxReadonlySupplier {
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
    /// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
    ///
    /// let constant = BoxReadonlySupplier::constant(42);
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        BoxReadonlySupplier::new(move || value.clone())
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
    /// A new mapped `BoxReadonlySupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
    ///
    /// let mapped = BoxReadonlySupplier::new(|| 10)
    ///     .map(|x| x * 2)
    ///     .map(|x| x + 5);
    /// assert_eq!(mapped.get(), 25);
    /// ```
    pub fn map<U, M>(self, mapper: M) -> BoxReadonlySupplier<U>
    where
        M: Transformer<T, U> + 'static,
        U: 'static,
    {
        BoxReadonlySupplier::new(move || mapper.apply(self.get()))
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
    /// A new filtered `BoxReadonlySupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
    ///
    /// let filtered = BoxReadonlySupplier::new(|| 42)
    ///     .filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(self, predicate: P) -> BoxReadonlySupplier<Option<T>>
    where
        P: Fn(&T) -> bool + 'static,
    {
        BoxReadonlySupplier::new(move || {
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
    /// A new `BoxReadonlySupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxReadonlySupplier, ReadonlySupplier};
    ///
    /// let first = BoxReadonlySupplier::new(|| 42);
    /// let second = BoxReadonlySupplier::new(|| "hello");
    /// let zipped = first.zip(second);
    ///
    /// assert_eq!(zipped.get(), (42, "hello"));
    /// ```
    pub fn zip<U>(self, other: BoxReadonlySupplier<U>) -> BoxReadonlySupplier<(T, U)>
    where
        U: 'static,
    {
        BoxReadonlySupplier::new(move || (self.get(), other.get()))
    }
}

impl<T> ReadonlySupplier<T> for BoxReadonlySupplier<T> {
    fn get(&self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxReadonlySupplier<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcReadonlySupplier<T>
    where
        T: 'static,
    {
        RcReadonlySupplier::new(self.function)
    }

    // do NOT override BoxReadonlySupplier::to_arc() because BoxReadonlySupplier
    // is not Send + Sync and calling BoxReadonlySupplier::to_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function()
    }

    // Note: to_box, to_rc, to_arc, and to_fn cannot be implemented
    // for BoxReadonlySupplier because it does not implement Clone.
    // Box provides unique ownership and cannot be cloned unless
    // the inner type implements Clone, which dyn Fn() -> T does not.
    //
    // If you call these methods on BoxReadonlySupplier, the compiler
    // will fail with an error indicating that BoxReadonlySupplier<T>
    // does not implement Clone, which is required by the default
    // implementations of to_box, to_rc, to_arc, and to_fn.
}

// ======================================================================
// ArcReadonlySupplier - Thread-safe Shared Ownership Implementation
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
/// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
///
/// let source = ArcReadonlySupplier::new(|| 10);
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
/// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
/// use std::thread;
///
/// let factory = ArcReadonlySupplier::new(|| {
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
/// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
///
/// let base = ArcReadonlySupplier::new(|| 10);
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
pub struct ArcReadonlySupplier<T> {
    function: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T> ArcReadonlySupplier<T>
where
    T: Send + 'static,
{
    /// Creates a new `ArcReadonlySupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `ArcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
    ///
    /// let supplier = ArcReadonlySupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        ArcReadonlySupplier {
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
    /// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
    ///
    /// let constant = ArcReadonlySupplier::constant(42);
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        ArcReadonlySupplier::new(move || value.clone())
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
    /// A new mapped `ArcReadonlySupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
    ///
    /// let source = ArcReadonlySupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// assert_eq!(mapped.get(), 20);
    /// ```
    pub fn map<U, M>(&self, mapper: M) -> ArcReadonlySupplier<U>
    where
        M: Transformer<T, U> + Send + Sync + 'static,
        U: Send + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let mapper = Arc::new(mapper);
        ArcReadonlySupplier {
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
    /// A new filtered `ArcReadonlySupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
    ///
    /// let source = ArcReadonlySupplier::new(|| 42);
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(&self, predicate: P) -> ArcReadonlySupplier<Option<T>>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        let self_fn = Arc::clone(&self.function);
        let predicate = Arc::new(predicate);
        ArcReadonlySupplier {
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
    /// A new `ArcReadonlySupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcReadonlySupplier, ReadonlySupplier};
    ///
    /// let first = ArcReadonlySupplier::new(|| 42);
    /// let second = ArcReadonlySupplier::new(|| "hello");
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
    pub fn zip<U>(&self, other: &ArcReadonlySupplier<U>) -> ArcReadonlySupplier<(T, U)>
    where
        U: Send + 'static,
    {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&other.function);
        ArcReadonlySupplier {
            function: Arc::new(move || (first(), second())),
        }
    }
}

impl<T> ReadonlySupplier<T> for ArcReadonlySupplier<T> {
    fn get(&self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxReadonlySupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxReadonlySupplier::new(move || self_fn())
    }

    fn into_rc(self) -> RcReadonlySupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        RcReadonlySupplier::new(move || self_fn())
    }

    fn into_arc(self) -> ArcReadonlySupplier<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function()
    }

    // Optimized implementations using Arc::clone instead of
    // wrapping in a closure

    fn to_box(&self) -> BoxReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = Arc::clone(&self.function);
        BoxReadonlySupplier::new(move || self_fn())
    }

    fn to_rc(&self) -> RcReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = Arc::clone(&self.function);
        RcReadonlySupplier::new(move || self_fn())
    }

    fn to_arc(&self) -> ArcReadonlySupplier<T>
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
        let self_fn = Arc::clone(&self.function);
        move || self_fn()
    }
}

impl<T> Clone for ArcReadonlySupplier<T> {
    /// Clones the `ArcReadonlySupplier`.
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
// RcReadonlySupplier - Single-threaded Shared Ownership
// ======================================================================

/// Single-threaded shared ownership read-only supplier.
///
/// Uses `Rc<dyn Fn() -> T>` for single-threaded shared ownership.
/// Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcReadonlySupplier`, methods borrow `&self` instead of
/// consuming `self`:
///
/// ```rust
/// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
///
/// let source = RcReadonlySupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// ```
///
/// # Examples
///
/// ## Shared Factory
///
/// ```rust
/// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
///
/// let factory = RcReadonlySupplier::new(|| {
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
/// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
///
/// let base = RcReadonlySupplier::new(|| 10);
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
pub struct RcReadonlySupplier<T> {
    function: Rc<dyn Fn() -> T>,
}

impl<T> RcReadonlySupplier<T>
where
    T: 'static,
{
    /// Creates a new `RcReadonlySupplier`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `RcReadonlySupplier<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
    ///
    /// let supplier = RcReadonlySupplier::new(|| 42);
    /// assert_eq!(supplier.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        RcReadonlySupplier {
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
    /// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
    ///
    /// let constant = RcReadonlySupplier::constant(42);
    /// assert_eq!(constant.get(), 42);
    /// assert_eq!(constant.get(), 42);
    /// ```
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        RcReadonlySupplier::new(move || value.clone())
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
    /// A new mapped `RcReadonlySupplier<U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
    ///
    /// let source = RcReadonlySupplier::new(|| 10);
    /// let mapped = source.map(|x| x * 2);
    /// // source is still usable
    /// assert_eq!(mapped.get(), 20);
    /// ```
    pub fn map<U, M>(&self, mapper: M) -> RcReadonlySupplier<U>
    where
        M: Transformer<T, U> + 'static,
        U: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let mapper = Rc::new(mapper);
        RcReadonlySupplier {
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
    /// A new filtered `RcReadonlySupplier<Option<T>>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
    ///
    /// let source = RcReadonlySupplier::new(|| 42);
    /// let filtered = source.filter(|x| x % 2 == 0);
    ///
    /// assert_eq!(filtered.get(), Some(42));
    /// ```
    pub fn filter<P>(&self, predicate: P) -> RcReadonlySupplier<Option<T>>
    where
        P: Fn(&T) -> bool + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let predicate = Rc::new(predicate);
        RcReadonlySupplier {
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
    /// A new `RcReadonlySupplier<(T, U)>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcReadonlySupplier, ReadonlySupplier};
    ///
    /// let first = RcReadonlySupplier::new(|| 42);
    /// let second = RcReadonlySupplier::new(|| "hello");
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
    pub fn zip<U>(&self, other: &RcReadonlySupplier<U>) -> RcReadonlySupplier<(T, U)>
    where
        U: 'static,
    {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&other.function);
        RcReadonlySupplier {
            function: Rc::new(move || (first(), second())),
        }
    }
}

impl<T> ReadonlySupplier<T> for RcReadonlySupplier<T> {
    fn get(&self) -> T {
        (self.function)()
    }

    fn into_box(self) -> BoxReadonlySupplier<T>
    where
        T: 'static,
    {
        let self_fn = self.function;
        BoxReadonlySupplier::new(move || self_fn())
    }

    fn into_rc(self) -> RcReadonlySupplier<T>
    where
        T: 'static,
    {
        self
    }

    // do NOT override RcReadonlySupplier::to_arc() because RcReadonlySupplier
    // is not Send + Sync and calling RcReadonlySupplier::to_arc() will cause a compile error

    fn into_fn(self) -> impl FnMut() -> T {
        let function = self.function;
        move || function()
    }

    // Optimized implementations using Rc::clone instead of wrapping
    // in a closure

    fn to_box(&self) -> BoxReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let self_fn = Rc::clone(&self.function);
        BoxReadonlySupplier::new(move || self_fn())
    }

    fn to_rc(&self) -> RcReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        self.clone()
    }

    // Note: to_arc cannot be implemented for RcReadonlySupplier
    // because Rc is not Send + Sync, which is required for
    // ArcReadonlySupplier.
    //
    // If you call to_arc on RcReadonlySupplier, the compiler will
    // fail with an error indicating that RcReadonlySupplier<T> does
    // not satisfy the Send + Sync bounds required by the default
    // implementation of to_arc.

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        let self_fn = Rc::clone(&self.function);
        move || self_fn()
    }
}

impl<T> Clone for RcReadonlySupplier<T> {
    /// Clones the `RcReadonlySupplier`.
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
// Implement ReadonlySupplier for Closures
// ======================================================================

impl<T, F> ReadonlySupplier<T> for F
where
    F: Fn() -> T,
{
    fn get(&self) -> T {
        self()
    }

    // Use optimized implementations for closures instead of the
    // default implementations. This avoids double wrapping by
    // directly creating the target type.

    fn into_box(self) -> BoxReadonlySupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxReadonlySupplier::new(self)
    }

    fn into_rc(self) -> RcReadonlySupplier<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcReadonlySupplier::new(self)
    }

    fn into_arc(self) -> ArcReadonlySupplier<T>
    where
        Self: Sized + Send + Sync + 'static,
        T: Send + 'static,
    {
        ArcReadonlySupplier::new(self)
    }

    fn into_fn(self) -> impl FnMut() -> T
    where
        Self: Sized,
    {
        // For closures, we can directly return a FnMut closure
        // that captures self
        move || self()
    }

    // Optimized implementations for to_* methods

    fn to_box(&self) -> BoxReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        BoxReadonlySupplier::new(cloned)
    }

    fn to_rc(&self) -> RcReadonlySupplier<T>
    where
        Self: Clone + 'static,
        T: 'static,
    {
        let cloned = self.clone();
        RcReadonlySupplier::new(cloned)
    }

    fn to_arc(&self) -> ArcReadonlySupplier<T>
    where
        Self: Clone + Send + Sync + 'static,
        T: Send + 'static,
    {
        let cloned = self.clone();
        ArcReadonlySupplier::new(cloned)
    }

    fn to_fn(&self) -> impl FnMut() -> T
    where
        Self: Clone,
    {
        let cloned = self.clone();
        move || cloned()
    }
}

// ======================================================================
// Note on Extension Traits for Closures
// ======================================================================
//
// We don't provide `FnReadonlySupplierOps` trait for `Fn() -> T` closures
// because:
//
// 1. All `Fn` closures also implement `FnMut`, so they can use `FnSupplierOps`
//    from the `supplier` module
// 2. Providing both would cause ambiguity errors due to overlapping trait impls
// 3. Rust doesn't support negative trait bounds to exclude `FnMut`
//
// Users of `Fn` closures should use `FnSupplierOps` from `supplier` module,
// or explicitly convert to `BoxReadonlySupplier` using `.into_box()` first.
