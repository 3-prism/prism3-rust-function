/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Types
//!
//! Provides implementations of consumer interfaces for executing operations that accept a single input parameter but return no result.
//!
//! This module provides a unified `Consumer` trait and three concrete implementations based on different ownership models:
//!
//! - **`BoxConsumer<T>`**: Box-based single ownership implementation for one-time use scenarios
//! - **`ArcConsumer<T>`**: Thread-safe shared ownership implementation based on Arc<Mutex<>>
//! - **`RcConsumer<T>`**: Single-threaded shared ownership implementation based on Rc<RefCell<>>
//!
//! # Design Philosophy
//!
//! Consumer uses `FnMut(&T)` semantics, allowing modification of its own state but not the input value. Suitable for
//! statistics, accumulation, event handling, and other scenarios.
//!
//! # Author
//!
//! Hu Haixing

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Type alias for consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and returns nothing.
/// It is used to reduce type complexity in struct definitions.
type ConsumerFn<T> = dyn FnMut(&T);

/// Type alias for thread-safe consumer function to simplify complex types.
///
/// This type alias represents a mutable function that takes a reference and returns nothing,
/// with Send bound for thread-safe usage. It is used to reduce type complexity
/// in Arc-based struct definitions.
type SendConsumerFn<T> = dyn FnMut(&T) + Send;

// ============================================================================
// 1. Consumer Trait - Unified Consumer Interface
// ============================================================================

/// Consumer trait - Unified consumer interface
///
/// Defines the core behavior of all consumer types. Similar to Java's `Consumer<T>` interface,
/// executes operations that accept a value but return no result (side effects only).
///
/// Consumer can modify its own state (such as accumulation, counting), but should not modify the consumed value itself.
///
/// # Automatic Implementation
///
/// - All closures implementing `FnMut(&T)`
/// - `BoxConsumer<T>`, `ArcConsumer<T>`, `RcConsumer<T>`
///
/// # Features
///
/// - **Unified Interface**: All consumer types share the same `accept` method signature
/// - **Automatic Implementation**: Closures automatically implement this trait with zero overhead
/// - **Type Conversion**: Easy conversion between different ownership models
/// - **Generic Programming**: Write functions that work with any consumer type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer, ArcConsumer};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// // Works with any consumer type
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut box_con = BoxConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// apply_consumer(&mut box_con, &5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait Consumer<T> {
    /// Execute consumption operation
    ///
    /// Performs an operation on the given reference. The operation typically reads the input value or produces side effects,
    /// but does not modify the input value itself. Can modify the consumer's own state.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut consumer = BoxConsumer::new(|x: &i32| println!("{}", x));
    /// let value = 5;
    /// consumer.accept(&value);
    /// ```
    fn accept(&mut self, value: &T);

    /// Convert to BoxConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts the current consumer to `BoxConsumer<T>`.
    ///
    /// # Ownership
    ///
    /// This method **consumes** the consumer (takes ownership of `self`).
    /// After calling this method, the original consumer is no longer available.
    ///
    /// **Tip**: For cloneable consumers ([`ArcConsumer`], [`RcConsumer`]),
    /// if you need to preserve the original object, you can call `.clone()` first.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Consumer;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// };
    /// let mut box_consumer = closure.into_box();
    /// box_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to RcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `RcConsumer<T>`
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to ArcConsumer
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Return Value
    ///
    /// Returns the wrapped `ArcConsumer<T>`
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts the consumer to a closure that can be used directly in standard library functions requiring `FnMut`.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnMut(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    /// let mut func = consumer.into_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_fn(self) -> impl FnMut(&T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxConsumer - Single Ownership Implementation
// ============================================================================

/// BoxConsumer struct
///
/// Consumer implementation based on `Box<dyn FnMut(&T)>` for single ownership scenarios.
/// When sharing is not needed, this is the simplest and most efficient consumer type.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership when used
/// - **Zero Overhead**: No reference counting or lock overhead
/// - **Mutable State**: Can modify captured environment through `FnMut`
/// - **Builder Pattern**: Method chaining naturally consumes `self`
///
/// # Use Cases
///
/// Choose `BoxConsumer` when:
/// - Consumer is used only once or in a linear flow
/// - Building pipelines where ownership flows naturally
/// - No need to share consumers across contexts
/// - Performance critical and cannot accept sharing overhead
///
/// # Performance
///
/// `BoxConsumer` has the best performance among the three consumer types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrowing checks
/// - Direct function calls through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, BoxConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct BoxConsumer<T> {
    function: Box<dyn FnMut(&T)>,
    name: Option<String>,
}

impl<T> BoxConsumer<T>
where
    T: 'static,
{
    /// Create a new BoxConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `BoxConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxConsumer {
            function: Box::new(f),
            name: None,
        }
    }

    /// Create a new named BoxConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `BoxConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = BoxConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        BoxConsumer {
            function: Box::new(f),
            name: Some(name.into()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut noop = BoxConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation. Consumes self.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Return Value
    ///
    /// Returns a new combined `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = BoxConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: Consumer<T> + 'static,
    {
        let mut first = self.function;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// Create a print consumer
    ///
    /// Returns a consumer that prints the input value.
    ///
    /// # Return Value
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print();
    /// print.accept(&42); // Prints: 42
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        BoxConsumer::new(|t| {
            println!("{:?}", t);
        })
    }

    /// Create a print consumer with prefix
    ///
    /// Returns a consumer that prints the input value with the specified prefix.
    ///
    /// # Parameters
    ///
    /// * `prefix` - Prefix string
    ///
    /// # Return Value
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    ///
    /// let mut print = BoxConsumer::<i32>::print_with("Value: ");
    /// print.accept(&42); // Prints: Value: 42
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxConsumer::new(move |t| {
            println!("{}{:?}", prefix, t);
        })
    }

    /// Create a conditional consumer
    ///
    /// Returns a consumer that executes the operation only when the predicate is true.
    ///
    /// # Type Parameters
    ///
    /// * `P` - Predicate type
    /// * `C` - Consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - Predicate function
    /// * `consumer` - Consumer to execute
    ///
    /// # Return Value
    ///
    /// Returns a conditional consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut conditional = BoxConsumer::if_then(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l.lock().unwrap().push(*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// conditional.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]); // Unchanged
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }

    /// Create a conditional branch consumer
    ///
    /// Returns a consumer that executes different operations based on the predicate.
    ///
    /// # Type Parameters
    ///
    /// * `P` - Predicate type
    /// * `C1` - then consumer type
    /// * `C2` - else consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - Predicate function
    /// * `then_consumer` - Consumer to execute when predicate is true
    /// * `else_consumer` - Consumer to execute when predicate is false
    ///
    /// # Return Value
    ///
    /// Returns a conditional branch consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, BoxConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut conditional = BoxConsumer::if_then_else(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l1.lock().unwrap().push(*x);
    ///     },
    ///     move |x: &i32| {
    ///         l2.lock().unwrap().push(-*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    ///
    /// conditional.accept(&-5);
    /// assert_eq!(*log.lock().unwrap(), vec![5, 5]); // -(-5) = 5
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&T) + 'static,
        C2: FnMut(&T) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        BoxConsumer::new(move |t| {
            if pred(t) {
                then_cons(t);
            } else {
                else_cons(t);
            }
        })
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let mut func = self.function;
        RcConsumer::new(move |t| func(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        // Note: BoxConsumer's function is not necessarily Send, so it cannot be safely converted to ArcConsumer
        // We panic here because this conversion is unsafe at the type system level
        panic!("Cannot convert BoxConsumer to ArcConsumer: BoxConsumer's inner function may not be Send")
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        self.function
    }
}

impl<T> fmt::Debug for BoxConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxConsumer({})", name),
            None => write!(f, "BoxConsumer"),
        }
    }
}

// ============================================================================
// 3. ArcConsumer - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcConsumer struct
///
/// Consumer implementation based on `Arc<Mutex<dyn FnMut(&T) + Send>>` for thread-safe shared ownership scenarios.
/// This consumer can be safely cloned and shared across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allowing multiple owners
/// - **Thread Safety**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains usable
/// - **Cross-Thread Sharing**: Can be sent to other threads and used
///
/// # Use Cases
///
/// Choose `ArcConsumer` when:
/// - Need to share consumers across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Need thread safety (Send + Sync)
///
/// # Performance Considerations
///
/// `ArcConsumer` has some performance overhead compared to `BoxConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread safety is not needed,
/// consider using `RcConsumer` for less single-threaded sharing overhead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, ArcConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcConsumer::new(move |x: &i32| {
///     l.lock().unwrap().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct ArcConsumer<T> {
    function: Arc<Mutex<SendConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> ArcConsumer<T>
where
    T: Send + 'static,
{
    /// Create a new ArcConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `ArcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcConsumer {
            function: Arc::new(Mutex::new(f)),
            name: None,
        }
    }

    /// Create a new named ArcConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `ArcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = ArcConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnMut(&T) + Send + 'static,
    {
        ArcConsumer {
            function: Arc::new(Mutex::new(f)),
            name: Some(name.into()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    ///
    /// let mut noop = ArcConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        ArcConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another ArcConsumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Borrows &self, does not consume the original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Return Value
    ///
    /// Returns a new combined `ArcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = ArcConsumer::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// });
    /// let second = ArcConsumer::new(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]); // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T> {
        let first = Arc::clone(&self.function);
        let second = Arc::clone(&next.function);
        ArcConsumer {
            function: Arc::new(Mutex::new(move |t: &T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
            name: None,
        }
    }

    /// Convert to closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function through Arc.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnMut(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, ArcConsumer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = ArcConsumer::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = Arc::clone(&self.function);
        move |t: &T| {
            func.lock().unwrap()(t);
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        RcConsumer::new(move |t| func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func.lock().unwrap()(t);
        }
    }
}

impl<T> Clone for ArcConsumer<T> {
    /// Clone ArcConsumer
    ///
    /// Creates a new ArcConsumer that shares the underlying function with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for ArcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for ArcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "ArcConsumer({})", name),
            None => write!(f, "ArcConsumer"),
        }
    }
}

// ============================================================================
// 4. RcConsumer - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcConsumer struct
///
/// Consumer implementation based on `Rc<RefCell<dyn FnMut(&T)>>` for single-threaded shared ownership scenarios.
/// This consumer provides the benefits of shared ownership without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allowing multiple owners
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrowing checks
/// - **No Lock Overhead**: More efficient than `ArcConsumer` for single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains usable
///
/// # Use Cases
///
/// Choose `RcConsumer` when:
/// - Need to share consumers within a single thread
/// - Thread safety is not needed
/// - Performance is important (avoid lock overhead)
/// - UI event handling in single-threaded frameworks
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcConsumer` performs better than `ArcConsumer` in single-threaded scenarios:
/// - **Non-Atomic Counting**: clone/drop is cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checks, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU cache behavior
///
/// But still has slight overhead compared to `BoxConsumer`:
/// - **Reference Counting**: Non-atomic but still exists
/// - **Runtime Borrowing Checks**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcConsumer` is not thread-safe and does not implement `Send` or `Sync`.
/// Attempting to send it to another thread will result in a compilation error.
/// For thread-safe sharing, use `ArcConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, RcConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcConsumer::new(move |x: &i32| {
///     l.borrow_mut().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.borrow(), vec![10]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct RcConsumer<T> {
    function: Rc<RefCell<ConsumerFn<T>>>,
    name: Option<String>,
}

impl<T> RcConsumer<T>
where
    T: 'static,
{
    /// Create a new RcConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `RcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcConsumer {
            function: Rc::new(RefCell::new(f)),
            name: None,
        }
    }

    /// Create a new named RcConsumer
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the consumer
    /// * `f` - Closure to wrap
    ///
    /// # Return Value
    ///
    /// Returns a new `RcConsumer<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let mut consumer = RcConsumer::new_with_name("my_consumer", move |x: &i32| {
    ///     l.borrow_mut().push(*x + 1);
    /// });
    /// assert_eq!(consumer.name(), Some("my_consumer"));
    /// consumer.accept(&5);
    /// assert_eq!(*log.borrow(), vec![6]);
    /// ```
    pub fn new_with_name<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        RcConsumer {
            function: Rc::new(RefCell::new(f)),
            name: Some(name.into()),
        }
    }

    /// Create a no-op consumer
    ///
    /// Returns a consumer that performs no operation.
    ///
    /// # Return Value
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    ///
    /// let mut noop = RcConsumer::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        RcConsumer::new(|_| {})
    }

    /// Get the consumer's name
    ///
    /// # Return Value
    ///
    /// Returns the consumer's name, or `None` if not set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    ///
    /// # Parameters
    ///
    /// * `name` - Name to set
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another RcConsumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Borrows &self, does not consume the original consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Return Value
    ///
    /// Returns a new combined `RcConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let first = RcConsumer::new(move |x: &i32| {
    ///     l1.borrow_mut().push(*x * 2);
    /// });
    /// let second = RcConsumer::new(move |x: &i32| {
    ///     l2.borrow_mut().push(*x + 10);
    /// });
    ///
    /// let mut chained = first.and_then(&second);
    ///
    /// // first and second remain usable after chaining
    /// chained.accept(&5);
    /// assert_eq!(*log.borrow(), vec![10, 15]); // (5 * 2), (5 + 10)
    /// ```
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T> {
        let first = Rc::clone(&self.function);
        let second = Rc::clone(&next.function);
        RcConsumer {
            function: Rc::new(RefCell::new(move |t: &T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
            name: None,
        }
    }

    /// Convert to closure (without consuming self)
    ///
    /// Creates a new closure that calls the underlying function through Rc.
    ///
    /// # Return Value
    ///
    /// Returns a closure implementing `FnMut(&T)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, RcConsumer};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let log = Rc::new(RefCell::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = RcConsumer::new(move |x: &i32| {
    ///     l.borrow_mut().push(*x);
    /// });
    ///
    /// let mut func = consumer.to_fn();
    /// func(&5);
    /// assert_eq!(*log.borrow(), vec![5]);
    /// ```
    pub fn to_fn(&self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = Rc::clone(&self.function);
        move |t: &T| {
            func.borrow_mut()(t);
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &T) {
        (self.function.borrow_mut())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        let func = self.function;
        BoxConsumer::new(move |t| func.borrow_mut()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcConsumer to ArcConsumer (not Send)")
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        T: 'static,
    {
        let func = self.function;
        move |t: &T| {
            func.borrow_mut()(t);
        }
    }
}

impl<T> Clone for RcConsumer<T> {
    /// Clone RcConsumer
    ///
    /// Creates a new RcConsumer that shares the underlying function with the original instance.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for RcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcConsumer")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for RcConsumer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "RcConsumer({})", name),
            None => write!(f, "RcConsumer"),
        }
    }
}

// ============================================================================
// 5. Implement Consumer trait for closures
// ============================================================================

/// Implement Consumer for all FnMut(&T)
impl<T, F> Consumer<T> for F
where
    F: FnMut(&T),
{
    fn accept(&mut self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(self)
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcConsumer::new(self)
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcConsumer::new(self)
    }

    fn into_fn(self) -> impl FnMut(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 6. Extension methods for closures
// ============================================================================

/// Extension trait providing consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures implementing `FnMut(&T)`,
/// allowing direct method chaining on closures without explicit wrapper types.
///
/// # Design Philosophy
///
/// This trait allows closures to be naturally composed using method syntax, similar to iterator combinators.
/// Composition methods consume the closure and return `BoxConsumer<T>`, which can continue chaining.
///
/// # Features
///
/// - **Natural Syntax**: Direct method chaining on closures
/// - **Returns BoxConsumer**: Composition results in `BoxConsumer<T>`, can continue chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnMut(&T)` closures automatically get these methods
///
/// # Examples
///
/// ```rust
/// use prism3_function::{Consumer, FnConsumerOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let mut chained = (move |x: &i32| {
///     l1.lock().unwrap().push(*x * 2);
/// }).and_then(move |x: &i32| {
///     l2.lock().unwrap().push(*x + 10);
/// });
/// chained.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10, 15]); // (5 * 2), (5 + 10)
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnConsumerOps<T>: FnMut(&T) + Sized {
    /// Sequentially chain another consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Consumes the current closure and returns `BoxConsumer<T>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Return Value
    ///
    /// Returns a combined `BoxConsumer<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Consumer, FnConsumerOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let mut chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// }).and_then(|x: &i32| println!("Result: {}", x));
    ///
    /// chained.accept(&5); // Prints: Result: 5
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// Implement FnConsumerOps for all closure types
impl<T, F> FnConsumerOps<T> for F where F: FnMut(&T) {}
