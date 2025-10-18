/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ConsumerOnce Types
//!
//! Provides implementations of one-time consumer interfaces for executing one-time operations
//! that accept a single input parameter but return no result.
//!
//! This module provides a unified `ConsumerOnce` trait and one concrete implementation:
//!
//! - **`BoxConsumerOnce<T>`**: Box-based single ownership implementation
//!
//! # Why No Arc/Rc Variants?
//!
//! Unlike `Consumer` and `ReadonlyConsumer`, this module does **not** provide `ArcConsumerOnce`
//! or `RcConsumerOnce` implementations. This is a design decision based on the fact that
//! `FnOnce` semantics are fundamentally incompatible with shared ownership. See design docs for details.
//!
//! # Design Philosophy
//!
//! ConsumerOnce uses `FnOnce(&T)` semantics for truly one-time consumption operations.
//! Unlike Consumer, ConsumerOnce consumes itself on first call. Suitable for initialization
//! callbacks, cleanup callbacks, and similar scenarios.
//!
//! # Author
//!
//! Hu Haixing

use std::fmt;

// ============================================================================
// 1. ConsumerOnce Trait - Unified ConsumerOnce Interface
// ============================================================================

/// ConsumerOnce trait - Unified one-time consumer interface
///
/// Defines the core behavior of all one-time consumer types. Similar to consumers
/// implementing `FnOnce(&T)`, executes operations that accept a value reference but
/// return no result (only side effects), consuming itself in the process.
///
/// # Automatic Implementation
///
/// - All closures implementing `FnOnce(&T)`
/// - `BoxConsumerOnce<T>`
///
/// # Features
///
/// - **Unified Interface**: All consumer types share the same `accept` method signature
/// - **Automatic Implementation**: Closures automatically implement this trait with zero overhead
/// - **Type Conversion**: Can be converted to BoxConsumerOnce
/// - **Generic Programming**: Write functions that work with any one-time consumer type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: ConsumerOnce<i32>>(consumer: C, value: &i32) {
///     consumer.accept(value);
/// }
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let box_con = BoxConsumerOnce::new(move |x: &i32| {
///     l.lock().unwrap().push(*x);
/// });
/// apply_consumer(box_con, &5);
/// assert_eq!(*log.lock().unwrap(), vec![5]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait ConsumerOnce<T> {
    /// Execute one-time consumption operation
    ///
    /// Executes an operation on the given reference. The operation typically reads
    /// the input value or produces side effects, but does not modify the input
    /// value itself. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `value` - Reference to the value to be consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let consumer = BoxConsumerOnce::new(|x: &i32| println!("{}", x));
    /// consumer.accept(&5);
    /// ```
    fn accept(self, value: &T);

    /// Convert to BoxConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ConsumerOnce;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32| {
    ///     l.lock().unwrap().push(*x);
    /// };
    /// let box_consumer = closure.into_box();
    /// box_consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    fn into_box(self) -> BoxConsumerOnce<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to closure
    ///
    /// **⚠️ Consumes `self`**: The original consumer will be unavailable after calling this method.
    ///
    /// Converts a one-time consumer to a closure that can be used directly in places
    /// where the standard library requires `FnOnce`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnOnce(&T)`
    fn into_fn(self) -> impl FnOnce(&T)
    where
        Self: Sized + 'static,
        T: 'static;
}

// ============================================================================
// 2. BoxConsumerOnce - Single Ownership Implementation
// ============================================================================

/// BoxConsumerOnce struct
///
/// One-time consumer implementation based on `Box<dyn FnOnce(&T)>` for single ownership scenarios.
/// This is the simplest consumer type for truly one-time use.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership on use
/// - **Zero Overhead**: No reference counting or lock overhead
/// - **One-time Use**: Consumes self on first call
/// - **Builder Pattern**: Method chaining naturally consumes `self`
///
/// # Use Cases
///
/// Choose `BoxConsumerOnce` when:
/// - Consumer is truly used only once
/// - Building pipelines where ownership flows naturally
/// - Consumer captures values that should be consumed
/// - Performance critical and cannot accept shared overhead
///
/// # Performance
///
/// `BoxConsumerOnce` has the best performance:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
///
/// let consumer = BoxConsumerOnce::new(|x: &i32| {
///     println!("Value: {}", x);
/// });
/// consumer.accept(&5);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct BoxConsumerOnce<T> {
    function: Box<dyn FnOnce(&T)>,
    name: Option<String>,
}

impl<T> BoxConsumerOnce<T>
where
    T: 'static,
{
    /// Create a new BoxConsumerOnce
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type
    ///
    /// # Parameters
    ///
    /// * `f` - Closure to be wrapped
    ///
    /// # Returns
    ///
    /// Returns a new `BoxConsumerOnce<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxConsumerOnce::new(move |x: &i32| {
    ///     l.lock().unwrap().push(*x + 1);
    /// });
    /// consumer.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![6]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce {
            function: Box::new(f),
            name: None,
        }
    }

    /// Get the consumer's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the consumer's name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sequentially chain another one-time consumer
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
    /// # Returns
    ///
    /// Returns a new combined `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = BoxConsumerOnce::new(move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// });
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: ConsumerOnce<T> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxConsumerOnce::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    /// Create a no-op consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let noop = BoxConsumerOnce::<i32>::noop();
    /// noop.accept(&42);
    /// // Value unchanged
    /// ```
    pub fn noop() -> Self {
        BoxConsumerOnce::new(|_| {})
    }

    /// Create a print consumer
    ///
    /// Returns a consumer that prints the input value.
    ///
    /// # Returns
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let print = BoxConsumerOnce::<i32>::print();
    /// print.accept(&42); // Prints: 42
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        BoxConsumerOnce::new(|t| {
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
    /// # Returns
    ///
    /// Returns a print consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    ///
    /// let print = BoxConsumerOnce::<i32>::print_with("Value: ");
    /// print.accept(&42); // Prints: Value: 42
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxConsumerOnce::new(move |t| {
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
    /// # Returns
    ///
    /// Returns a conditional consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let conditional = BoxConsumerOnce::if_then(
    ///     |x: &i32| *x > 0,
    ///     move |x: &i32| {
    ///         l.lock().unwrap().push(*x);
    ///     },
    /// );
    ///
    /// conditional.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![5]);
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnOnce(&T) -> bool + 'static,
        C: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce::new(move |t| {
            if predicate(t) {
                consumer(t);
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
    /// * `C1` - Then consumer type
    /// * `C2` - Else consumer type
    ///
    /// # Parameters
    ///
    /// * `predicate` - Predicate function
    /// * `then_consumer` - Consumer to execute when predicate is true
    /// * `else_consumer` - Consumer to execute when predicate is false
    ///
    /// # Returns
    ///
    /// Returns a conditional branch consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, BoxConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let conditional = BoxConsumerOnce::if_then_else(
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
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnOnce(&T) -> bool + 'static,
        C1: FnOnce(&T) + 'static,
        C2: FnOnce(&T) + 'static,
    {
        BoxConsumerOnce::new(move |t| {
            if predicate(t) {
                then_consumer(t);
            } else {
                else_consumer(t);
            }
        })
    }
}

impl<T> ConsumerOnce<T> for BoxConsumerOnce<T> {
    fn accept(self, value: &T) {
        (self.function)(value)
    }

    fn into_box(self) -> BoxConsumerOnce<T>
    where
        T: 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnOnce(&T)
    where
        T: 'static,
    {
        self.function
    }
}

impl<T> fmt::Debug for BoxConsumerOnce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxConsumerOnce")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T> fmt::Display for BoxConsumerOnce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxConsumerOnce({})", name),
            None => write!(f, "BoxConsumerOnce"),
        }
    }
}

// ============================================================================
// 3. Implement ConsumerOnce trait for closures
// ============================================================================

/// Implement ConsumerOnce for all FnOnce(&T)
impl<T, F> ConsumerOnce<T> for F
where
    F: FnOnce(&T),
{
    fn accept(self, value: &T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumerOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumerOnce::new(self)
    }

    fn into_fn(self) -> impl FnOnce(&T)
    where
        Self: Sized + 'static,
        T: 'static,
    {
        self
    }
}

// ============================================================================
// 4. Extension methods for closures
// ============================================================================

/// Extension trait providing one-time consumer composition methods for closures
///
/// Provides `and_then` and other composition methods for all closures implementing `FnOnce(&T)`,
/// allowing closures to chain methods directly without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxConsumerOnce**: Composed results can continue chaining
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnOnce(&T)` closures automatically get these methods
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ConsumerOnce, FnConsumerOnceOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let chained = (move |x: &i32| {
///     l1.lock().unwrap().push(*x * 2);
/// }).and_then(move |x: &i32| {
///     l2.lock().unwrap().push(*x + 10);
/// });
/// chained.accept(&5);
/// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnConsumerOnceOps<T>: FnOnce(&T) + Sized {
    /// Sequentially chain another one-time consumer
    ///
    /// Returns a new consumer that executes the current operation first, then the next operation.
    /// Consumes the current closure and returns `BoxConsumerOnce<T>`.
    ///
    /// # Type Parameters
    ///
    /// * `C` - Type of the next consumer
    ///
    /// # Parameters
    ///
    /// * `next` - Consumer to execute after the current operation
    ///
    /// # Returns
    ///
    /// Returns a combined `BoxConsumerOnce<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ConsumerOnce, FnConsumerOnceOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = (move |x: &i32| {
    ///     l1.lock().unwrap().push(*x * 2);
    /// }).and_then(move |x: &i32| {
    ///     l2.lock().unwrap().push(*x + 10);
    /// }).and_then(|x: &i32| println!("Result: {}", x));
    ///
    /// chained.accept(&5);
    /// assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxConsumerOnce<T>
    where
        Self: 'static,
        C: ConsumerOnce<T> + 'static,
        T: 'static,
    {
        let first = self;
        let second = next;
        BoxConsumerOnce::new(move |t| {
            first(t);
            second.accept(t);
        })
    }
}

/// Implement FnConsumerOnceOps for all closure types
impl<T, F> FnConsumerOnceOps<T> for F where F: FnOnce(&T) {}
