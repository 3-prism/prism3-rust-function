/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiConsumerOnce Types
//!
//! Provides one-time bi-consumer interface implementations for operations
//! accepting two input parameters without returning a result.
//!
//! This module provides a unified `BiConsumerOnce` trait and one concrete
//! implementation:
//!
//! - **`BoxBiConsumerOnce<T, U>`**: Box-based single ownership
//!   implementation
//!
//! # Why No Arc/Rc Variants?
//!
//! Unlike `BiConsumer` and `ReadonlyBiConsumer`, this module does **not**
//! provide `ArcBiConsumerOnce` or `RcBiConsumerOnce` implementations. This
//! is a design decision based on the fundamental incompatibility between
//! `FnOnce` semantics and shared ownership. See the design documentation
//! for details.
//!
//! # Design Philosophy
//!
//! BiConsumerOnce uses `FnOnce(&T, &U)` semantics: for truly one-time
//! consumption operations. Unlike BiConsumer, BiConsumerOnce consumes
//! itself on first call. Suitable for initialization callbacks, cleanup
//! callbacks, etc.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;

// ==========================================================================
// Type Aliases
// ==========================================================================

/// Type alias for bi-consumer once function signature.
type BiConsumerOnceFn<T, U> = dyn FnOnce(&T, &U);

// =======================================================================
// 1. BiConsumerOnce Trait - Unified Interface
// =======================================================================

/// BiConsumerOnce trait - Unified one-time bi-consumer interface
///
/// Defines core behavior for all one-time bi-consumer types. Similar to a
/// bi-consumer implementing `FnOnce(&T, &U)`, performs operations
/// accepting two value references but returning no result (side effects
/// only), consuming itself in the process.
///
/// # Automatic Implementations
///
/// - All closures implementing `FnOnce(&T, &U)`
/// - `BoxBiConsumerOnce<T, U>`
///
/// # Features
///
/// - **Unified Interface**: All bi-consumer types share the same `accept`
///   method signature
/// - **Automatic Implementation**: Closures automatically implement this
///   trait with zero overhead
/// - **Type Conversions**: Can convert to BoxBiConsumerOnce
/// - **Generic Programming**: Write functions accepting any one-time
///   bi-consumer type
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// fn apply_consumer<C: BiConsumerOnce<i32, i32>>(
///     consumer: C,
///     a: &i32,
///     b: &i32
/// ) {
///     consumer.accept(a, b);
/// }
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let box_con = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// apply_consumer(box_con, &5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]);
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait BiConsumerOnce<T, U> {
    /// Performs the one-time consumption operation
    ///
    /// Executes an operation on the given two references. The operation
    /// typically reads input values or produces side effects, but does not
    /// modify the input values themselves. Consumes self.
    ///
    /// # Parameters
    ///
    /// * `first` - Reference to the first value to consume
    /// * `second` - Reference to the second value to consume
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    ///
    /// let consumer = BoxBiConsumerOnce::new(|x: &i32, y: &i32| {
    ///     println!("Sum: {}", x + y);
    /// });
    /// consumer.accept(&5, &3);
    /// ```
    fn accept(self, first: &T, second: &U);

    /// Converts to BoxBiConsumerOnce
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// # Returns
    ///
    /// Returns the wrapped `BoxBiConsumerOnce<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BiConsumerOnce;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let closure = move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// };
    /// let box_consumer = closure.into_box();
    /// box_consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_box(self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;

    /// Converts to a closure
    ///
    /// **⚠️ Consumes `self`**: Original consumer becomes unavailable after
    /// calling this method.
    ///
    /// Converts the one-time bi-consumer to a closure usable with standard
    /// library methods requiring `FnOnce`.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnOnce(&T, &U)`
    fn into_fn(self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static;
}

// =======================================================================
// 2. BoxBiConsumerOnce - Single Ownership Implementation
// =======================================================================

/// BoxBiConsumerOnce struct
///
/// A one-time bi-consumer implementation based on
/// `Box<dyn FnOnce(&T, &U)>` for single ownership scenarios. This is the
/// simplest one-time bi-consumer type for truly one-time use.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Zero Overhead**: No reference counting or locking
/// - **One-Time Use**: Consumes self on first call
/// - **Builder Pattern**: Method chaining consumes `self` naturally
///
/// # Use Cases
///
/// Choose `BoxBiConsumerOnce` when:
/// - The bi-consumer is truly used only once
/// - Building pipelines where ownership naturally flows
/// - The consumer captures values that should be consumed
/// - Performance is critical and sharing overhead is unacceptable
///
/// # Performance
///
/// `BoxBiConsumerOnce` has the best performance:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
///
/// let consumer = BoxBiConsumerOnce::new(|x: &i32, y: &i32| {
///     println!("Sum: {}", x + y);
/// });
/// consumer.accept(&5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxBiConsumerOnce<T, U> {
    function: Box<BiConsumerOnceFn<T, U>>,
    name: Option<String>,
}

impl<T, U> BoxBiConsumerOnce<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Creates a new BoxBiConsumerOnce
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
    /// Returns a new `BoxBiConsumerOnce<T, U>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x * 2 + *y);
    /// });
    /// consumer.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![13]);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&T, &U) + 'static,
    {
        BoxBiConsumerOnce {
            function: Box::new(f),
            name: None,
        }
    }

    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Chains another one-time bi-consumer in sequence
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
    /// Returns a new composed `BoxBiConsumerOnce<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).and_then(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumerOnce<T, U> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxBiConsumerOnce::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }

    /// Creates a no-op bi-consumer
    ///
    /// # Returns
    ///
    /// Returns a no-op bi-consumer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    ///
    /// let noop = BoxBiConsumerOnce::<i32, i32>::noop();
    /// noop.accept(&42, &10);
    /// // Values unchanged
    /// ```
    pub fn noop() -> Self {
        BoxBiConsumerOnce::new(|_, _| {})
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
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    ///
    /// let print = BoxBiConsumerOnce::<i32, i32>::print();
    /// print.accept(&42, &10); // Prints: (42, 10)
    /// ```
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        BoxBiConsumerOnce::new(|t, u| {
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
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    ///
    /// let print = BoxBiConsumerOnce::<i32, i32>::print_with("Values: ");
    /// print.accept(&42, &10); // Prints: Values: (42, 10)
    /// ```
    pub fn print_with(prefix: &str) -> Self
    where
        T: std::fmt::Debug,
        U: std::fmt::Debug,
    {
        let prefix = prefix.to_string();
        BoxBiConsumerOnce::new(move |t, u| {
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
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let conditional = BoxBiConsumerOnce::if_then(
    ///     |x: &i32, y: &i32| *x > 0 && *y > 0,
    ///     move |x: &i32, y: &i32| {
    ///         l.lock().unwrap().push(*x + *y);
    ///     },
    /// );
    ///
    /// conditional.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnOnce(&T, &U) -> bool + 'static,
        C: FnOnce(&T, &U) + 'static,
    {
        BoxBiConsumerOnce::new(move |t, u| {
            if predicate(t, u) {
                consumer(t, u);
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
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let conditional = BoxBiConsumerOnce::if_then_else(
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
    /// ```
    pub fn if_then_else<P, C1, C2>(predicate: P, then_consumer: C1, else_consumer: C2) -> Self
    where
        P: FnOnce(&T, &U) -> bool + 'static,
        C1: FnOnce(&T, &U) + 'static,
        C2: FnOnce(&T, &U) + 'static,
    {
        BoxBiConsumerOnce::new(move |t, u| {
            if predicate(t, u) {
                then_consumer(t, u);
            } else {
                else_consumer(t, u);
            }
        })
    }
}

impl<T, U> BiConsumerOnce<T, U> for BoxBiConsumerOnce<T, U> {
    fn accept(self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box(self) -> BoxBiConsumerOnce<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_fn(self) -> impl FnOnce(&T, &U)
    where
        T: 'static,
        U: 'static,
    {
        self.function
    }
}

impl<T, U> fmt::Debug for BoxBiConsumerOnce<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxBiConsumerOnce")
            .field("name", &self.name)
            .field("function", &"<function>")
            .finish()
    }
}

impl<T, U> fmt::Display for BoxBiConsumerOnce<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "BoxBiConsumerOnce({})", name),
            None => write!(f, "BoxBiConsumerOnce"),
        }
    }
}

// =======================================================================
// 3. Implement BiConsumerOnce trait for closures
// =======================================================================

/// Implements BiConsumerOnce for all FnOnce(&T, &U)
impl<T, U, F> BiConsumerOnce<T, U> for F
where
    F: FnOnce(&T, &U),
{
    fn accept(self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box(self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumerOnce::new(self)
    }

    fn into_fn(self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }
}

// =======================================================================
// 4. Provide extension methods for closures
// =======================================================================

/// Extension trait providing one-time bi-consumer composition methods for
/// closures
///
/// Provides `and_then` and other composition methods for all closures
/// implementing `FnOnce(&T, &U)`, enabling direct method chaining on
/// closures without explicit wrapper types.
///
/// # Features
///
/// - **Natural Syntax**: Chain operations directly on closures
/// - **Returns BoxBiConsumerOnce**: Composition results can be further
///   chained
/// - **Zero Cost**: No overhead when composing closures
/// - **Automatic Implementation**: All `FnOnce(&T, &U)` closures get
///   these methods automatically
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BiConsumerOnce, FnBiConsumerOnceOps};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let chained = (move |x: &i32, y: &i32| {
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
pub trait FnBiConsumerOnceOps<T, U>: FnOnce(&T, &U) + Sized {
    /// Chains another one-time bi-consumer in sequence
    ///
    /// Returns a new consumer executing the current operation first, then
    /// the next operation. Consumes the current closure and returns
    /// `BoxBiConsumerOnce<T, U>`.
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
    /// Returns the composed `BoxBiConsumerOnce<T, U>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, FnBiConsumerOnceOps};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let chained = (move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).and_then(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// }).and_then(|x: &i32, y: &i32| {
    ///     println!("Result: {}, {}", x, y);
    /// });
    ///
    /// chained.accept(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// ```
    fn and_then<C>(self, next: C) -> BoxBiConsumerOnce<T, U>
    where
        Self: 'static,
        C: BiConsumerOnce<T, U> + 'static,
        T: 'static,
        U: 'static,
    {
        let first = self;
        let second = next;
        BoxBiConsumerOnce::new(move |t, u| {
            first(t, u);
            second.accept(t, u);
        })
    }
}

/// Implements FnBiConsumerOnceOps for all closure types
impl<T, U, F> FnBiConsumerOnceOps<T, U> for F where F: FnOnce(&T, &U) {}
