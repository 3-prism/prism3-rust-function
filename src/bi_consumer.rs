/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiConsumer Types
//!
//! Provides bi-consumer interface implementations for operations accepting
//! two input parameters without returning a result.
//!
//! This module provides a unified `BiConsumer` trait and three concrete
//! implementations based on different ownership models:
//!
//! - **`BoxBiConsumer<T, U>`**: Box-based single ownership for one-time use
//! - **`ArcBiConsumer<T, U>`**: Arc<Mutex<>>-based thread-safe shared
//!   ownership
//! - **`RcBiConsumer<T, U>`**: Rc<RefCell<>>-based single-threaded shared
//!   ownership
//!
//! # Design Philosophy
//!
//! BiConsumer uses `FnMut(&T, &U)` semantics: can modify its own state but
//! does NOT modify input values. Suitable for statistics, accumulation, and
//! event processing scenarios involving two parameters.
//!
//! # Author
//!
//! Haixing Hu

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Type alias for bi-consumer function to simplify complex types.
///
/// Represents a mutable function taking two references and returning
/// nothing. Used to reduce type complexity in struct definitions.
type BiConsumerFn<T, U> = dyn FnMut(&T, &U);

/// Type alias for thread-safe bi-consumer function.
///
/// Represents a mutable function with Send bound for thread-safe usage.
type SendBiConsumerFn<T, U> = dyn FnMut(&T, &U) + Send;

// =======================================================================
// 1. BiConsumer Trait - Unified BiConsumer Interface
// =======================================================================

/// BiConsumer trait - Unified bi-consumer interface
///
/// Defines core behavior for all bi-consumer types. Similar to Java's
/// `BiConsumer<T, U>` interface, performs operations accepting two values
/// but returning no result (side effects only).
///
/// BiConsumer can modify its own state (e.g., accumulate, count) but
/// should NOT modify the consumed values themselves.
///
/// # Automatic Implementations
///
/// - All closures implementing `FnMut(&T, &U)`
/// - `BoxBiConsumer<T, U>`, `ArcBiConsumer<T, U>`, `RcBiConsumer<T, U>`
///
/// # Features
///
/// - **Unified Interface**: All bi-consumer types share the same `accept`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Easy conversion between ownership models
/// - **Generic Programming**: Write functions accepting any bi-consumer
///   type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer, ArcBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_bi_consumer<C: BiConsumer<i32, i32>>(
///     consumer: &mut C,
///     a: &i32,
///     b: &i32
/// ) {
///     consumer.accept(a, b);
/// }
///
/// // Works with any bi-consumer type
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut box_con = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// apply_bi_consumer(&mut box_con, &5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait BiConsumer<T, U> {
    /// Performs the consumption operation
    ///
    /// Executes an operation on the given two references. The operation
    /// typically reads input values or produces side effects, but does not
    /// modify the input values themselves. Can modify the consumer's own
    /// state.
    ///
    /// # Parameters
    ///
    /// * `first` - Reference to the first value to consume
    /// * `second` - Reference to the second value to consume
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn accept(&mut self, first: &T, second: &U);

    /// Converts to BoxBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the current bi-consumer to `BoxBiConsumer<T, U>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling, the original consumer is no longer available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcBiConsumer`],
    /// [`RcBiConsumer`]), call `.clone()` first if you need to keep the
    /// original.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumer;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// };
    /// let mut box_consumer = closure.into_box();
    /// box_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;

    /// Converts to RcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `RcBiConsumer<T, U>`
    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;

    /// Converts to ArcBiConsumer
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `ArcBiConsumer<T, U>`
    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static;

    /// Converts bi-consumer to a closure
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the bi-consumer to a closure usable with standard library
    /// methods requiring `FnMut`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let mut func = consumer.into_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;
}

// =======================================================================
// 2. BoxBiConsumer - Single Ownership Implementation
// =======================================================================

/// BoxBiConsumer struct
///
/// A bi-consumer implementation based on `Box<dyn FnMut(&T, &U)>` for
/// single ownership scenarios. This is the simplest and most efficient
/// bi-consumer type when sharing is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Builder Pattern**: Method chaining consumes `self` naturally
///
/// # Use Cases
///
/// Choose `BoxBiConsumer` when:
/// - The bi-consumer is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the consumer across contexts
/// - Performance is critical and sharing overhead is unacceptable
///
/// # Performance
///
/// `BoxBiConsumer` has the best performance among the three bi-consumer
/// types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, BoxBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxBiConsumer<T, U> {
    function: Box<BiConsumerFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new BoxBiConsumer
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
    /// Returns a new `BoxBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        BoxBiConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = BoxBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).and_then(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumer<T, U> + 'static,
    {
        let mut first = self.function;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }

    /// Creates a no-op bi-consumer
    ///
    /// Returns a bi-consumer that performs no operation.
    ///
    /// # Returns
    ///
    /// Returns a no-op bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut noop = BoxBiConsumer::<i32, i32>::noop();
    /// noop.accept(&42, &10);
    /// // Values unchanged
    /// ```
    pub fn noop() -> Self {
        BoxBiConsumer::new(|_, _| {})
    }

    /// Creates a print bi-consumer
    ///
    /// Returns a bi-consumer that prints the input values.
    ///
    /// # Returns
    ///
    /// Returns a print bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut print = BoxBiConsumer::<i32, i32>::print();
    /// print.accept(&42, &10); // Prints: (42, 10)
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        BoxBiConsumer::new(|t, u| {
            println!("({:?}, {:?})", t, u);
        })
    }

    /// Creates a print bi-consumer with prefix
    ///
    /// Returns a bi-consumer that prints the input values with a prefix.
    ///
    /// # Parameters
    ///
    /// * `prefix` - The prefix string
    ///
    /// # Returns
    ///
    /// Returns a print bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    ///
    /// let mut print = BoxBiConsumer::<i32, i32>::print_with("Values: ");
    /// print.accept(&42, &10); // Prints: Values: (42, 10)
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxBiConsumer::new(move |t, u| {
            println!("{}{:?}, {:?}", prefix, t, u);
        })
    }

    /// Creates a conditional bi-consumer
    ///
    /// Returns a bi-consumer that only executes when the predicate is
    /// true.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    /// * `C` - The consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `consumer` - The consumer to execute
    ///
    /// # Returns
    ///
    /// Returns a conditional bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut conditional = BoxBiConsumer::if_then(
    ///     |x: &i32, y: &i32| *x > 0 && *y > 0,
    ///     move |x: &i32, y: &i32| {
    ///         l.lock().unwrap().push(*x + *y);
    ///     },
    /// );
    ///
    /// conditional.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    ///
    /// conditional.accept(&-5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T, &U) -> bool + 'static,
        C: FnMut(&T, &U) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred(t, u) {
                cons(t, u);
            }
        })
    }

    /// Creates a conditional branch bi-consumer
    ///
    /// Returns a bi-consumer that executes different operations based on
    /// the predicate.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    /// * `C1` - The then consumer type
    /// * `C2` - The else consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate function
    /// * `then_consumer` - The consumer to execute when predicate is true
    /// * `else_consumer` - The consumer to execute when predicate is false
    ///
    /// # Returns
    ///
    /// Returns a conditional branch bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, BoxBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut conditional = BoxBiConsumer::if_then_else(
    ///     |x: &i32, y: &i32| *x > *y,
    ///     move |x: &i32, _y: &i32| {
    ///         l1.lock().unwrap().push(*x);
    ///     },
    ///     move |_x: &i32, y: &i32| {
    ///         l2.lock().unwrap().push(*y);
    ///     },
    /// );
    ///
    /// conditional.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// conditional.accept(&2, &5);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 5]);
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnMut(&T, &U) -> bool + 'static,
        C1: FnMut(&T, &U) + 'static,
        C2: FnMut(&T, &U) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        BoxBiConsumer::new(move |t, u| {
            if pred(t, u) {
                then_cons(t, u);
            } else {
                else_cons(t, u);
            }
        })
    }
}

impl<T, U> BiConsumer<T, U> for BoxBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let mut func = self.function;
        RcBiConsumer::new(move |t, u| func(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        panic!(
            "Cannot convert BoxBiConsumer to ArcBiConsumer: inner \
                function may not be Send"
        )
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }
}

impl<T, U> fmt::Debug for BoxBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for BoxBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxBiConsumer({})", name),
            None => write!(f, "BoxBiConsumer"),
        }
    }
}

// =======================================================================
// 3. ArcBiConsumer - Thread-Safe Shared Ownership Implementation
// =======================================================================

/// ArcBiConsumer struct
///
/// A bi-consumer implementation based on
/// `Arc<Mutex<dyn FnMut(&T, &U) + Send>>` for thread-safe shared
/// ownership scenarios. This consumer can be safely cloned and shared
/// across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
/// - **Cross-Thread Sharing**: Can be sent to and used by other threads
///
/// # Use Cases
///
/// Choose `ArcBiConsumer` when:
/// - Need to share bi-consumer across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Thread safety (Send + Sync) is required
///
/// # Performance Considerations
///
/// `ArcBiConsumer` has some overhead compared to `BoxBiConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread
/// safety is not needed, consider using `RcBiConsumer` for lower
/// overhead in single-threaded sharing.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, ArcBiConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct ArcBiConsumer<T, U> {
    function: Arc<Mutex<SendBiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> ArcBiConsumer<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    /// Creates a new ArcBiConsumer
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
    /// Returns a new `ArcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + Send + 'static,
    {
        ArcBiConsumer {
            function: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Converts to a closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function via Arc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = Arc::clone(&self.function);
        move |t: &T, u: &U| {
            func.lock().unwrap()(t, u);
        }
    }

    /// Chains another ArcBiConsumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `ArcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, ArcBiConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// });
    /// let second = ArcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    pub fn and_then(&self, next: &ArcBiConsumer<T, U>) -> ArcBiConsumer<T, U> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcBiConsumer {
            function: Arc::new(Mutex::new(move |t: &T, u: &U| {
                first.lock().unwrap()(t, u);
                second.lock().unwrap()(t, u);
            })),
            name: None,
        }
    }
}

impl<T, U> BiConsumer<T, U> for ArcBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.lock().unwrap())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.function;
        BoxBiConsumer::new(move |t, u| func.lock().unwrap()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.function;
        RcBiConsumer::new(move |t, u| func.lock().unwrap()(t, u))
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = self.function;
        move |t: &T, u: &U| {
            func.lock().unwrap()(t, u);
        }
    }
}

impl<T, U> Clone for ArcBiConsumer<T, U> {
    /// Clones the ArcBiConsumer
    ///
    /// Creates a new ArcBiConsumer sharing the underlying function with
    /// the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for ArcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for ArcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcBiConsumer({})", name),
            None => write!(f, "ArcBiConsumer"),
        }
    }
}

// =======================================================================
// 4. RcBiConsumer - Single-Threaded Shared Ownership Implementation
// =======================================================================

/// RcBiConsumer struct
///
/// A bi-consumer implementation based on `Rc<RefCell<dyn FnMut(&T, &U)>>`
/// for single-threaded shared ownership scenarios. This consumer provides
/// the benefits of shared ownership without the overhead of thread
/// safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot send across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **No Lock Overhead**: More efficient than `ArcBiConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original remains
///   usable
///
/// # Use Cases
///
/// Choose `RcBiConsumer` when:
/// - Need to share bi-consumer within a single thread
/// - Thread safety is not needed
/// - Performance matters (avoiding lock overhead)
/// - Single-threaded UI framework event handling
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcBiConsumer` performs better than `ArcBiConsumer` in single-threaded
/// scenarios:
/// - **Non-Atomic Counting**: clone/drop cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checking, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU
///   cache behavior
///
/// But still has slight overhead compared to `BoxBiConsumer`:
/// - **Reference Counting**: Though non-atomic, still exists
/// - **Runtime Borrow Checking**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcBiConsumer` is not thread-safe and does not implement `Send` or
/// `Sync`. Attempting to send it to another thread will result in a
/// compile error. For thread-safe sharing, use `ArcBiConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, RcBiConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
///     l.borrow_mut().push(*x + *y);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5, &3);
/// assert_eq!(*log.borrow(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RcBiConsumer<T, U> {
    function: Rc<RefCell<BiConsumerFn<T, U>>>,
    name: Option<String>,
}

impl<T, U> RcBiConsumer<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new RcBiConsumer
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
    /// Returns a new `RcBiConsumer<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T, &U) + 'static,
    {
        RcBiConsumer {
            function: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// Gets the name of the consumer
    ///
    /// # Returns
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Converts to a closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function via Rc.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnMut(&T, &U)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l.borrow_mut().push(*x + *y);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T, u: &U| {
            func.borrow_mut()(t, u);
        }
    }

    /// Chains another RcBiConsumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Borrows &self, does not consume the original
    /// consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a new composed `RcBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, RcBiConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l1.borrow_mut().push(*x + *y);
    /// });
    /// let second = RcBiConsumer::new(move |x: &i32, y: &i32| {
    ///     l2.borrow_mut().push(*x * *y);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second still usable after chaining
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.borrow(), vec![8, 15]);
    /// ```
    pub fn and_then(&self, next: &RcBiConsumer<T, U>) -> RcBiConsumer<T, U> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcBiConsumer {
            function: Rc::new(RefCell::new(move |t: &T, u: &U| {
                first.borrow_mut()(t, u);
                second.borrow_mut()(t, u);
            })),
            name: None,
        }
    }
}

impl<T, U> BiConsumer<T, U> for RcBiConsumer<T, U> {
    fn accept(&mut self, first: &T, second: &U) {
        (self.function.borrow_mut())(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        let func = self.function;
        BoxBiConsumer::new(move |t, u| func.borrow_mut()(t, u))
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        T: Send + 'static,
        U: Send + 'static,
    {
        panic!("Cannot convert RcBiConsumer to ArcBiConsumer (not Send)")
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        let func = self.function;
        move |t: &T, u: &U| {
            func.borrow_mut()(t, u);
        }
    }
}

impl<T, U> Clone for RcBiConsumer<T, U> {
    /// Clones the RcBiConsumer
    ///
    /// Creates a new RcBiConsumer sharing the underlying function with the
    /// original instance.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T, U> fmt::Debug for RcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcBiConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for RcBiConsumer<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcBiConsumer({})", name),
            None => write!(f, "RcBiConsumer"),
        }
    }
}

// =======================================================================
// 5. Implement BiConsumer trait for closures
// =======================================================================

/// Implements BiConsumer for all FnMut(&T, &U)
impl<T, U, F> BiConsumer<T, U> for F
where
    F: FnMut(&T, &U),
{
    fn accept(&mut self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box(self) -> BoxBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumer::new(self)
    }

    fn into_rc(self) -> RcBiConsumer<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        RcBiConsumer::new(self)
    }

    fn into_arc(self) -> ArcBiConsumer<T, U>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        ArcBiConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }
}

// =======================================================================
// 6. Provide extension methods for closures
// =======================================================================

/// Extension trait providing bi-consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `FnMut(&T, &U)`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Design Rationale
///
/// This trait allows closures to be composed naturally using method
/// syntax, similar to iterator combinators. Composition methods consume
/// the closure and return `BoxBiConsumer<T, U>`, which can be further
/// chained.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxBiConsumer**: Composition results are
///   `BoxBiConsumer<T, U>` for continued chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&T, &U)` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumer, FnBiConsumerOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut chained = (move |x: &i32, y: &i32| {
///     l1.lock().unwrap().push(*x + *y);
/// }).and_then(move |x: &i32, y: &i32| {
///     l2.lock().unwrap().push(*x * *y);
/// });
/// chained.accept(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait FnBiConsumerOps<T, U>: FnMut(&T, &U) + Sized {
    /// Chains another consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes the current closure and returns
    /// `BoxBiConsumer<T, U>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - The consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumer<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumer, FnBiConsumerOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = (move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).and_then(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// }).and_then(|x: &i32, y: &i32| println!("Result: {}, {}", x, y));
    ///
    /// chained.accept(&5, &3); // Prints: Result: 5, 3
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxBiConsumer<T, U>
    where
        Self: 'static,
        C: BiConsumer<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxBiConsumer::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnBiConsumerOps for all closure types
impl<T, U, F> FnBiConsumerOps<T, U> for F where F: FnMut(&T, &U) {}
