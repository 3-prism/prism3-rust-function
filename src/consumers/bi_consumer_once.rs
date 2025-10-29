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

use crate::predicates::bi_predicate::{
    BiPredicate,
    BoxBiPredicate,
};

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
///     consumer.accept_once(a, b);
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
    /// consumer.accept_once(&5, &3);
    /// ```
    fn accept_once(self, first: &T, second: &U);

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
    /// let box_consumer = closure.into_box_once();
    /// box_consumer.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn into_box_once(self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumerOnce::new(move |t, u| self.accept_once(t, u))
    }

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
    fn into_fn_once(self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        move |t, u| self.accept_once(t, u)
    }

    /// Convert to BoxBiConsumerOnce without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current bi-consumer and then converts the clone
    /// to a `BoxBiConsumerOnce`.
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
    /// let box_consumer = closure.to_box_once();
    /// box_consumer.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn to_box_once(&self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_box_once()
    }

    /// Convert to closure without consuming self
    ///
    /// **⚠️ Requires Clone**: This method requires `Self` to implement
    /// `Clone`. Clones the current bi-consumer and then converts the clone
    /// to a closure.
    ///
    /// # Returns
    ///
    /// Returns a closure implementing `FnOnce(&T, &U)`
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
    /// let func = closure.to_fn_once();
    /// func(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    fn to_fn_once(&self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone().into_fn_once()
    }
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
/// consumer.accept_once(&5, &3);
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxBiConsumerOnce<T, U> {
    function: Box<BiConsumerOnceFn<T, U>>,
    name: Option<String>,
}

// Methods that don't require T: 'static or U: 'static
impl<T, U> BoxBiConsumerOnce<T, U> {
    /// Gets the name of the consumer
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of the consumer
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }
}

// Methods that require T: 'static and U: 'static
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
    /// consumer.accept_once(&5, &3);
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

    /// Creates a new BoxBiConsumerOnce with a name
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the consumer
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumerOnce<T, U>` instance with the specified name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l = log.clone();
    /// let consumer = BoxBiConsumerOnce::new_with_name("sum_logger", move |x: &i32, y: &i32| {
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// assert_eq!(consumer.name(), Some("sum_logger"));
    /// consumer.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn new_with_name<F>(name: &str, f: F) -> Self
    where
        F: FnOnce(&T, &U) + 'static,
    {
        BoxBiConsumerOnce {
            function: Box::new(f),
            name: Some(name.to_string()),
        }
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
    /// noop.accept_once(&42, &10);
    /// // Values unchanged
    /// ```
    pub fn noop() -> Self {
        BoxBiConsumerOnce::new(|_, _| {})
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
    /// * `next` - The consumer to execute after the current operation. **Note:
    ///   This parameter is passed by value and will transfer ownership.** Since
    ///   `BoxBiConsumerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumerOnce<T, U>`
    ///   - Any type implementing `BiConsumerOnce<T, U>`
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
    /// let first = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// });
    /// let second = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // Both first and second are moved and consumed
    /// let chained = first.and_then(second);
    /// chained.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// // first.accept(&2, &3); // Would not compile - moved
    /// // second.accept(&2, &3); // Would not compile - moved
    /// ```
    pub fn and_then<C>(self, next: C) -> Self
    where
        C: BiConsumerOnce<T, U> + 'static,
    {
        let first = self.function;
        let second = next;
        BoxBiConsumerOnce::new(move |t, u| {
            first(t, u);
            second.accept_once(t, u);
        })
    }

    /// Creates a conditional bi-consumer
    ///
    /// Returns a bi-consumer that only executes when a predicate is satisfied.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The predicate type
    ///
    /// # Parameters
    ///
    /// * `predicate` - The condition to check. **Note: This parameter is passed
    ///   by value and will transfer ownership.** If you need to preserve the
    ///   original bi-predicate, clone it first (if it implements `Clone`). Can be:
    ///   - A closure: `|x: &T, y: &U| -> bool`
    ///   - A function pointer: `fn(&T, &U) -> bool`
    ///   - A `BoxBiPredicate<T, U>`
    ///   - Any type implementing `BiPredicate<T, U>`
    ///
    /// # Returns
    ///
    /// Returns `BoxConditionalBiConsumerOnce<T, U>`
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
    ///     l.lock().unwrap().push(*x + *y);
    /// });
    /// let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    ///
    /// conditional.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]);
    /// ```
    pub fn when<P>(self, predicate: P) -> BoxConditionalBiConsumerOnce<T, U>
    where
        P: BiPredicate<T, U> + 'static,
    {
        BoxConditionalBiConsumerOnce {
            consumer: self,
            predicate: predicate.into_box(),
        }
    }
}

impl<T, U> BiConsumerOnce<T, U> for BoxBiConsumerOnce<T, U> {
    fn accept_once(self, first: &T, second: &U) {
        (self.function)(first, second)
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U>
    where
        T: 'static,
        U: 'static,
    {
        self
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U)
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
// 3. BoxConditionalBiConsumerOnce - Box-based Conditional BiConsumerOnce
// =======================================================================

/// BoxConditionalBiConsumerOnce struct
///
/// A conditional one-time bi-consumer that only executes when a predicate is satisfied.
/// Uses `BoxBiConsumerOnce` and `BoxBiPredicate` for single ownership semantics.
///
/// This type is typically created by calling `BoxBiConsumerOnce::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements BiConsumerOnce**: Can be used anywhere a `BiConsumerOnce` is expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
///     l.lock().unwrap().push(*x + *y);
/// });
/// let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
///
/// conditional.accept_once(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]); // Executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
///     l1.lock().unwrap().push(*x + *y);
/// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
///   .or_else(move |x: &i32, y: &i32| {
///     l2.lock().unwrap().push(*x * *y);
/// });
///
/// consumer.accept_once(&5, &3);
/// assert_eq!(*log.lock().unwrap(), vec![8]); // when branch executed
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxConditionalBiConsumerOnce<T, U> {
    consumer: BoxBiConsumerOnce<T, U>,
    predicate: BoxBiPredicate<T, U>,
}

impl<T, U> BiConsumerOnce<T, U> for BoxConditionalBiConsumerOnce<T, U>
where
    T: 'static,
    U: 'static,
{
    fn accept_once(self, first: &T, second: &U) {
        if self.predicate.test(first, second) {
            self.consumer.accept_once(first, second);
        }
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U> {
        let pred = self.predicate;
        let consumer = self.consumer;
        BoxBiConsumerOnce::new(move |t, u| {
            if pred.test(t, u) {
                consumer.accept_once(t, u);
            }
        })
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U) {
        let pred = self.predicate;
        let consumer = self.consumer;
        move |t: &T, u: &U| {
            if pred.test(t, u) {
                consumer.accept_once(t, u);
            }
        }
    }
}

impl<T, U> BoxConditionalBiConsumerOnce<T, U>
where
    T: 'static,
    U: 'static,
{
    /// Chains another consumer in sequence
    ///
    /// Combines the current conditional consumer with another consumer into a new
    /// consumer. The current conditional consumer executes first, followed by the
    /// next consumer.
    ///
    /// # Parameters
    ///
    /// * `next` - The next consumer to execute. **Note: This parameter is passed
    ///   by value and will transfer ownership.** Since `BoxBiConsumerOnce`
    ///   cannot be cloned, the parameter will be consumed. Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumerOnce<T, U>`
    ///   - Any type implementing `BiConsumerOnce<T, U>`
    ///
    /// # Returns
    ///
    /// Returns a new `BoxBiConsumerOnce<T, U>`
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
    /// let cond = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    /// let second = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// // Both cond and second are moved and consumed
    /// let chained = cond.and_then(second);
    /// chained.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    /// // cond.accept(&2, &3); // Would not compile - moved
    /// // second.accept(&2, &3); // Would not compile - moved
    /// ```
    pub fn and_then<C>(self, next: C) -> BoxBiConsumerOnce<T, U>
    where
        C: BiConsumerOnce<T, U> + 'static,
    {
        let first = self;
        let second = next;
        BoxBiConsumerOnce::new(move |t, u| {
            first.accept_once(t, u);
            second.accept_once(t, u);
        })
    }

    /// Adds an else branch
    ///
    /// Executes the original consumer when the condition is satisfied, otherwise
    /// executes else_consumer.
    ///
    /// # Parameters
    ///
    /// * `else_consumer` - The consumer for the else branch. **Note: This parameter
    ///   is passed by value and will transfer ownership.** Since
    ///   `BoxBiConsumerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumerOnce<T, U>`
    ///   - Any type implementing `BiConsumerOnce<T, U>`
    ///
    /// # Returns
    ///
    /// Returns the composed `BoxBiConsumerOnce<T, U>`
    ///
    /// # Examples
    ///
    /// ## Using a closure (recommended)
    ///
    /// ```rust
    /// use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::new()));
    /// let l1 = log.clone();
    /// let l2 = log.clone();
    /// let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
    ///     l1.lock().unwrap().push(*x + *y);
    /// }).when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    ///   .or_else(move |x: &i32, y: &i32| {
    ///     l2.lock().unwrap().push(*x * *y);
    /// });
    ///
    /// consumer.accept_once(&5, &3);
    /// assert_eq!(*log.lock().unwrap(), vec![8]); // Condition satisfied
    /// ```
    pub fn or_else<C>(self, else_consumer: C) -> BoxBiConsumerOnce<T, U>
    where
        C: BiConsumerOnce<T, U> + 'static,
    {
        let pred = self.predicate;
        let then_cons = self.consumer;
        let else_cons = else_consumer;
        BoxBiConsumerOnce::new(move |t, u| {
            if pred.test(t, u) {
                then_cons.accept_once(t, u);
            } else {
                else_cons.accept_once(t, u);
            }
        })
    }
}

// =======================================================================
// 5. Implement BiConsumerOnce trait for closures
// =======================================================================

/// Implements BiConsumerOnce for all FnOnce(&T, &U)
impl<T, U, F> BiConsumerOnce<T, U> for F
where
    F: FnOnce(&T, &U),
{
    fn accept_once(self, first: &T, second: &U) {
        self(first, second)
    }

    fn into_box_once(self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumerOnce::new(self)
    }

    fn into_fn_once(self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + 'static,
        T: 'static,
        U: 'static,
    {
        self
    }

    fn to_box_once(&self) -> BoxBiConsumerOnce<T, U>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        BoxBiConsumerOnce::new(self.clone())
    }

    fn to_fn_once(&self) -> impl FnOnce(&T, &U)
    where
        Self: Sized + Clone + 'static,
        T: 'static,
        U: 'static,
    {
        self.clone()
    }
}

// =======================================================================
// 6. Provide extension methods for closures
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
/// chained.accept_once(&5, &3);
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
    /// * `next` - The consumer to execute after the current operation. **Note:
    ///   This parameter is passed by value and will transfer ownership.** Since
    ///   `BoxBiConsumerOnce` cannot be cloned, the parameter will be consumed.
    ///   Can be:
    ///   - A closure: `|x: &T, y: &U|`
    ///   - A `BoxBiConsumerOnce<T, U>`
    ///   - Any type implementing `BiConsumerOnce<T, U>`
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
    /// chained.accept_once(&5, &3);
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
            second.accept_once(t, u);
        })
    }
}

/// Implements FnBiConsumerOnceOps for all closure types
impl<T, U, F> FnBiConsumerOnceOps<T, U> for F where F: FnOnce(&T, &U) {}
