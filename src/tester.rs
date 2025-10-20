/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Tester Type
//!
//! Provides tester implementations that test conditions or states and return
//! boolean values, without accepting input parameters.
//!
//! # Overview
//!
//! **Tester** is a functional abstraction for testing conditions or states
//! without accepting input. It can check system status, wait for conditions,
//! or perform health checks.
//!
//! This module implements **Option 3** from the design document: a unified
//! `Tester` trait with multiple concrete implementations optimized for
//! different ownership and concurrency scenarios.
//!
//! # Core Design Principles
//!
//! 1. **Returns boolean**: `Tester` returns `bool` to indicate test results
//! 2. **Uses `&self`**: Tester is only responsible for "judgment", not
//!    "state management"
//! 3. **No TesterOnce**: Very limited use cases, lacks practical examples
//! 4. **State management is caller's responsibility**: Tester only reads
//!    state, does not modify state
//!
//! # Three Implementations
//!
//! - **`BoxTester`**: Single ownership using `Box<dyn Fn() -> bool>`.
//!   Zero overhead, cannot be cloned. Best for one-time use and builder
//!   patterns.
//!
//! - **`ArcTester`**: Thread-safe shared ownership using
//!   `Arc<dyn Fn() -> bool + Send + Sync>`. Can be cloned and sent across
//!   threads. Lock-free overhead.
//!
//! - **`RcTester`**: Single-threaded shared ownership using
//!   `Rc<dyn Fn() -> bool>`. Can be cloned but cannot be sent across
//!   threads. Lower overhead than `ArcTester`.
//!
//! # Comparison with Other Functional Abstractions
//!
//! | Type      | Input | Output | self      | Modify? | Use Cases   |
//! |-----------|-------|--------|-----------|---------|-------------|
//! | Tester    | None  | `bool` | `&self`   | No      | State Check |
//! | Predicate | `&T`  | `bool` | `&self`   | No      | Filter      |
//! | Supplier  | None  | `T`    | `&mut`    | Yes     | Factory     |
//!
//! # Examples
//!
//! ## Basic State Checking
//!
//! ```rust
//! use prism3_function::{BoxTester, Tester};
//! use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
//!
//! // State managed externally
//! let count = Arc::new(AtomicUsize::new(0));
//! let count_clone = Arc::clone(&count);
//!
//! let tester = BoxTester::new(move || {
//!     count_clone.load(Ordering::Relaxed) <= 3
//! });
//!
//! assert!(tester.test());  // true (0)
//! count.fetch_add(1, Ordering::Relaxed);
//! assert!(tester.test());  // true (1)
//! count.fetch_add(1, Ordering::Relaxed);
//! assert!(tester.test());  // true (2)
//! count.fetch_add(1, Ordering::Relaxed);
//! assert!(tester.test());  // true (3)
//! count.fetch_add(1, Ordering::Relaxed);
//! assert!(!tester.test()); // false (4)
//! ```
//!
//! ## Logical Combination
//!
//! ```rust
//! use prism3_function::{BoxTester, Tester};
//! use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
//!
//! // Simulate microservice health check scenario
//! let cpu_usage = Arc::new(AtomicUsize::new(0));
//! let memory_usage = Arc::new(AtomicUsize::new(0));
//! let is_healthy = Arc::new(AtomicBool::new(true));
//! let is_ready = Arc::new(AtomicBool::new(false));
//! let max_cpu = 80;
//! let max_memory = 90;
//!
//! let cpu_clone = Arc::clone(&cpu_usage);
//! let memory_clone = Arc::clone(&memory_usage);
//! let health_clone = Arc::clone(&is_healthy);
//! let ready_clone = Arc::clone(&is_ready);
//!
//! // System resource check: CPU and memory within safe limits
//! let resources_ok = BoxTester::new(move || {
//!     cpu_clone.load(Ordering::Relaxed) < max_cpu
//! })
//! .and(move || {
//!     memory_clone.load(Ordering::Relaxed) < max_memory
//! });
//!
//! // Service status check: healthy or ready
//! let service_ok = BoxTester::new(move || {
//!     health_clone.load(Ordering::Relaxed)
//! })
//! .or(move || {
//!     ready_clone.load(Ordering::Relaxed)
//! });
//!
//! // Combined condition: resources normal and service available
//! let can_accept_traffic = resources_ok.and(service_ok);
//!
//! // Test different state combinations
//! // Initial state: resources normal and service healthy
//! cpu_usage.store(50, Ordering::Relaxed);
//! memory_usage.store(60, Ordering::Relaxed);
//! assert!(can_accept_traffic.test()); // resources normal and service healthy
//!
//! // Service unhealthy but ready
//! is_healthy.store(false, Ordering::Relaxed);
//! is_ready.store(true, Ordering::Relaxed);
//! assert!(can_accept_traffic.test()); // resources normal and service ready
//!
//! // CPU usage too high
//! cpu_usage.store(95, Ordering::Relaxed);
//! assert!(!can_accept_traffic.test()); // resources exceeded
//!
//! // Service unhealthy but ready
//! is_healthy.store(false, Ordering::Relaxed);
//! cpu_usage.store(50, Ordering::Relaxed);
//! assert!(can_accept_traffic.test()); // still ready
//! ```
//!
//! ## Thread-Safe Sharing
//!
//! ```rust
//! use prism3_function::{ArcTester, Tester};
//! use std::thread;
//!
//! let shared = ArcTester::new(|| true);
//! let clone = shared.clone();
//!
//! let handle = thread::spawn(move || {
//!     clone.test()
//! });
//!
//! assert!(handle.join().unwrap());
//! ```
//!
//! # Author
//!
//! Hu Haixing

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Core Tester Trait
// ============================================================================

/// Tests whether a state or condition holds
///
/// Tester is a functional abstraction for testing states or conditions. It
/// accepts no parameters and returns a boolean value indicating the test
/// result of some state or condition.
///
/// # Core Characteristics
///
/// - **No input parameters**: Captures context through closures
/// - **Returns boolean**: Indicates test results
/// - **Uses `&self`**: Does not modify its own state, only reads external
///   state
/// - **Repeatable calls**: The same Tester can call `test()` multiple times
///
/// # Use Cases
///
/// - **State checking**: Check system or service status
/// - **Condition waiting**: Repeatedly check until conditions are met
/// - **Health monitoring**: Check system health status
/// - **Precondition validation**: Verify conditions before operations
///
/// # Design Philosophy
///
/// Tester's responsibility is "test judgment", not "state management".
/// State management is the caller's responsibility. Tester only reads state
/// and returns judgment results.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxTester, Tester};
/// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
///
/// // State managed externally
/// let ready = Arc::new(AtomicBool::new(false));
/// let ready_clone = Arc::clone(&ready);
///
/// // Tester only responsible for reading state
/// let tester = BoxTester::new(move || {
///     ready_clone.load(Ordering::Acquire)
/// });
///
/// // Can be called multiple times
/// assert!(!tester.test());
/// ready.store(true, Ordering::Release);
/// assert!(tester.test());
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait Tester {
    /// Executes the test and returns the test result
    ///
    /// This method can be called multiple times without modifying the Tester's
    /// own state.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the condition holds, otherwise returns `false`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    ///
    /// let tester = BoxTester::new(|| true);
    /// assert!(tester.test());
    /// ```
    fn test(&self) -> bool;

    /// Converts this tester to `BoxTester`
    ///
    /// # Return Value
    ///
    /// A `BoxTester` that wraps this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, BoxTester};
    ///
    /// let closure = || true;
    /// let boxed: BoxTester = closure.into_box();
    /// ```
    fn into_box(self) -> BoxTester
    where
        Self: Sized + 'static,
    {
        BoxTester {
            function: Box::new(move || self.test()),
        }
    }

    /// Converts this tester to `RcTester`
    ///
    /// # Return Value
    ///
    /// A `RcTester` that wraps this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, RcTester};
    ///
    /// let closure = || true;
    /// let rc: RcTester = closure.into_rc();
    /// ```
    fn into_rc(self) -> RcTester
    where
        Self: Sized + 'static,
    {
        RcTester {
            function: Rc::new(move || self.test()),
        }
    }

    /// Converts this tester to `ArcTester`
    ///
    /// # Return Value
    ///
    /// An `ArcTester` that wraps this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, ArcTester};
    ///
    /// let closure = || true;
    /// let arc: ArcTester = closure.into_arc();
    /// ```
    fn into_arc(self) -> ArcTester
    where
        Self: Sized + Send + Sync + 'static,
    {
        ArcTester {
            function: Arc::new(move || self.test()),
        }
    }

    /// Converts this tester to a boxed function pointer
    ///
    /// # Return Value
    ///
    /// A `Box<dyn Fn() -> bool>` that wraps this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::Tester;
    ///
    /// let closure = || true;
    /// let func = closure.into_fn();
    /// assert!(func());
    /// ```
    fn into_fn(self) -> impl Fn() -> bool
    where
        Self: Sized + 'static,
    {
        Box::new(move || self.test())
    }

    /// Clones and converts this tester to `BoxTester`
    ///
    /// # Return Value
    ///
    /// A `BoxTester` that wraps a clone of this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, BoxTester, ArcTester};
    ///
    /// let arc = ArcTester::new(|| true);
    /// let boxed: BoxTester = arc.to_box();
    /// // arc is still available
    /// ```
    fn to_box(&self) -> BoxTester
    where
        Self: Clone + 'static,
    {
        self.clone().into_box()
    }

    /// Clones and converts this tester to `RcTester`
    ///
    /// # Return Value
    ///
    /// A `RcTester` that wraps a clone of this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, RcTester, ArcTester};
    ///
    /// let arc = ArcTester::new(|| true);
    /// let rc: RcTester = arc.to_rc();
    /// // arc is still available
    /// ```
    fn to_rc(&self) -> RcTester
    where
        Self: Clone + 'static,
    {
        self.clone().into_rc()
    }

    /// Clones and converts this tester to `ArcTester`
    ///
    /// # Return Value
    ///
    /// An `ArcTester` that wraps a clone of this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, ArcTester, RcTester};
    ///
    /// let rc = RcTester::new(|| true);
    /// // Note: This will panic for RcTester as it's not Send + Sync
    /// // let arc: ArcTester = rc.to_arc();
    /// ```
    fn to_arc(&self) -> ArcTester
    where
        Self: Clone + Send + Sync + 'static,
    {
        self.clone().into_arc()
    }

    /// Clones and converts this tester to a boxed function pointer
    ///
    /// # Return Value
    ///
    /// A `Box<dyn Fn() -> bool>` that wraps a clone of this tester
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{Tester, ArcTester};
    ///
    /// let arc = ArcTester::new(|| true);
    /// let func = arc.to_fn();
    /// // arc is still available
    /// assert!(func());
    /// ```
    fn to_fn(&self) -> impl Fn() -> bool
    where
        Self: Clone + 'static,
    {
        self.clone().into_fn()
    }
}

// ============================================================================
// BoxTester: Single Ownership Implementation
// ============================================================================

/// Single ownership Tester implemented using `Box`
///
/// `BoxTester` wraps a closure in `Box<dyn Fn() -> bool>`, providing single
/// ownership semantics with no additional allocation overhead beyond the
/// initial boxing.
///
/// # Characteristics
///
/// - **Single ownership**: Cannot be cloned
/// - **Zero overhead**: Single heap allocation
/// - **Consuming combination**: `and()`/`or()`/`not()` consume `self`
/// - **Type flexibility**: Accepts any `Tester` implementation
///
/// # Use Cases
///
/// - One-time testing scenarios
/// - Builder patterns requiring ownership transfer
/// - Simple state checking without sharing
/// - Chained calls with ownership transfer
///
/// # Examples
///
/// ```rust
/// use prism3_function::{BoxTester, Tester};
/// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
///
/// // State managed externally
/// let count = Arc::new(AtomicUsize::new(0));
/// let count_clone = Arc::clone(&count);
///
/// let tester = BoxTester::new(move || {
///     count_clone.load(Ordering::Relaxed) < 3
/// });
///
/// assert!(tester.test());
/// count.fetch_add(1, Ordering::Relaxed);
/// assert!(tester.test());
/// count.fetch_add(1, Ordering::Relaxed);
/// assert!(tester.test());
/// count.fetch_add(1, Ordering::Relaxed);
/// assert!(!tester.test());
///
/// // Logical combination
/// let combined = BoxTester::new(|| true)
///     .and(|| false)
///     .or(|| true);
/// assert!(combined.test());
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct BoxTester {
    function: Box<dyn Fn() -> bool>,
}

impl BoxTester {
    /// Creates a new `BoxTester` from a closure
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type implementing `Fn() -> bool`
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::BoxTester;
    ///
    /// let tester = BoxTester::new(|| true);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        BoxTester { function: Box::new(f) }
    }

    /// Combines this tester with another tester using logical AND
    ///
    /// Returns a new `BoxTester` that returns `true` only when both tests
    /// pass. Short-circuit evaluation: if the first test fails, the second
    /// will not be executed.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing `Tester`
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical AND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
    ///
    /// // Simulate service status
    /// let request_count = Arc::new(AtomicUsize::new(0));
    /// let is_available = Arc::new(AtomicBool::new(true));
    /// let max_requests = 1000;
    ///
    /// let count_clone = Arc::clone(&request_count);
    /// let available_clone = Arc::clone(&is_available);
    ///
    /// // Service available and request count not exceeded
    /// let service_ok = BoxTester::new(move || {
    ///     available_clone.load(Ordering::Relaxed)
    /// })
    /// .and(move || {
    ///     count_clone.load(Ordering::Relaxed) < max_requests
    /// });
    ///
    /// // Initial state: available and request count 0
    /// assert!(service_ok.test());
    ///
    /// // Simulate request increase
    /// request_count.store(500, Ordering::Relaxed);
    /// assert!(service_ok.test());
    ///
    /// // Request count exceeded
    /// request_count.store(1500, Ordering::Relaxed);
    /// assert!(!service_ok.test());
    ///
    /// // Service unavailable
    /// is_available.store(false, Ordering::Relaxed);
    /// assert!(!service_ok.test());
    /// ```
    pub fn and<T>(self, next: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        let self_fn = self.function;
        let next_tester = next;
        BoxTester::new(move || self_fn() && next_tester.test())
    }

    /// Combines this tester with another tester using logical OR
    ///
    /// Returns a new `BoxTester` that returns `true` if either test passes.
    /// Short-circuit evaluation: if the first test passes, the second will
    /// not be executed.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing `Tester`
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical OR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
    ///
    /// // Simulate service status
    /// let request_count = Arc::new(AtomicUsize::new(0));
    /// let is_healthy = Arc::new(AtomicBool::new(true));
    /// let max_requests = 100;
    ///
    /// let count_clone = Arc::clone(&request_count);
    /// let health_clone = Arc::clone(&is_healthy);
    ///
    /// // Service healthy or low request count
    /// let can_serve = BoxTester::new(move || {
    ///     health_clone.load(Ordering::Relaxed)
    /// })
    /// .or(move || {
    ///     count_clone.load(Ordering::Relaxed) < max_requests
    /// });
    ///
    /// // Initial state: healthy and request count 0
    /// assert!(can_serve.test());
    ///
    /// // Request count increased but within limit
    /// request_count.store(50, Ordering::Relaxed);
    /// assert!(can_serve.test());
    ///
    /// // Request count exceeded but service healthy
    /// request_count.store(150, Ordering::Relaxed);
    /// assert!(can_serve.test()); // still healthy
    ///
    /// // Service unhealthy but low request count
    /// is_healthy.store(false, Ordering::Relaxed);
    /// request_count.store(50, Ordering::Relaxed);
    /// assert!(can_serve.test()); // low request count
    ///
    /// // Unhealthy and high request count
    /// request_count.store(150, Ordering::Relaxed);
    /// assert!(!can_serve.test());
    /// ```
    pub fn or<T>(self, next: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        let self_fn = self.function;
        let next_tester = next;
        BoxTester::new(move || self_fn() || next_tester.test())
    }

    /// Negates the result of this tester
    ///
    /// Returns a new `BoxTester` that returns the opposite value of the
    /// original test result.
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical NOT
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    ///
    /// // Simulate resource usage
    /// let memory_usage = Arc::new(AtomicUsize::new(0));
    /// let max_memory = 1024; // MB
    ///
    /// let memory_clone = Arc::clone(&memory_usage);
    ///
    /// // Memory usage not exceeded
    /// let memory_ok = BoxTester::new(move || {
    ///     memory_clone.load(Ordering::Relaxed) <= max_memory
    /// });
    ///
    /// // Initial state: normal memory usage
    /// memory_usage.store(512, Ordering::Relaxed);
    /// assert!(memory_ok.test());
    ///
    /// // Memory usage exceeded (negated)
    /// let memory_critical = memory_ok.not();
    /// assert!(!memory_critical.test());
    ///
    /// // Memory usage exceeded
    /// memory_usage.store(2048, Ordering::Relaxed);
    /// assert!(memory_critical.test());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> BoxTester {
        let self_fn = self.function;
        BoxTester::new(move || !self_fn())
    }

    /// Combines this tester with another tester using logical NAND
    ///
    /// Returns a new `BoxTester` that returns `true` unless both tests pass.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing `Tester`
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical NAND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    ///
    /// let flag1 = Arc::new(AtomicBool::new(true));
    /// let flag2 = Arc::new(AtomicBool::new(true));
    ///
    /// let flag1_clone = Arc::clone(&flag1);
    /// let flag2_clone = Arc::clone(&flag2);
    ///
    /// let nand = BoxTester::new(move || {
    ///     flag1_clone.load(Ordering::Relaxed)
    /// })
    /// .nand(move || {
    ///     flag2_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// // Both true returns false
    /// assert!(!nand.test());
    ///
    /// // At least one false returns true
    /// flag1.store(false, Ordering::Relaxed);
    /// assert!(nand.test());
    /// ```
    pub fn nand<T>(self, next: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        let self_fn = self.function;
        let next_tester = next;
        BoxTester::new(move || !(self_fn() && next_tester.test()))
    }

    /// Combines this tester with another tester using logical XOR
    ///
    /// Returns a new `BoxTester` that returns `true` if exactly one test
    /// passes.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing `Tester`
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical XOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    ///
    /// let flag1 = Arc::new(AtomicBool::new(true));
    /// let flag2 = Arc::new(AtomicBool::new(false));
    ///
    /// let flag1_clone1 = Arc::clone(&flag1);
    /// let flag2_clone1 = Arc::clone(&flag2);
    ///
    /// let xor = BoxTester::new(move || {
    ///     flag1_clone1.load(Ordering::Relaxed)
    /// })
    /// .xor(move || {
    ///     flag2_clone1.load(Ordering::Relaxed)
    /// });
    ///
    /// // One true one false returns true
    /// assert!(xor.test());
    ///
    /// // Both true returns false
    /// flag2.store(true, Ordering::Relaxed);
    /// assert!(!xor.test());
    ///
    /// // Both false returns false
    /// flag1.store(false, Ordering::Relaxed);
    /// flag2.store(false, Ordering::Relaxed);
    /// assert!(!xor.test());
    /// ```
    pub fn xor<T>(self, next: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        let self_fn = self.function;
        let next_tester = next;
        BoxTester::new(move || self_fn() ^ next_tester.test())
    }

    /// Combines this tester with another tester using logical NOR
    ///
    /// Returns a new `BoxTester` that returns `true` only when both tests
    /// fail. Equivalent to `!(self OR other)`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing `Tester`
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `BoxTester` representing logical NOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    ///
    /// let flag1 = Arc::new(AtomicBool::new(false));
    /// let flag2 = Arc::new(AtomicBool::new(false));
    ///
    /// let flag1_clone = Arc::clone(&flag1);
    /// let flag2_clone = Arc::clone(&flag2);
    ///
    /// let nor = BoxTester::new(move || {
    ///     flag1_clone.load(Ordering::Relaxed)
    /// })
    /// .nor(move || {
    ///     flag2_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// // Both false returns true
    /// assert!(nor.test());
    ///
    /// // At least one true returns false
    /// flag1.store(true, Ordering::Relaxed);
    /// assert!(!nor.test());
    /// ```
    pub fn nor<T>(self, next: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        let self_fn = self.function;
        let next_tester = next;
        BoxTester::new(move || !(self_fn() || next_tester.test()))
    }
}

impl Tester for BoxTester {
    fn test(&self) -> bool {
        (self.function)()
    }

    fn into_box(self) -> BoxTester {
        self
    }

    fn into_rc(self) -> RcTester {
        let func = self.function;
        RcTester {
            function: Rc::new(func),
        }
    }

    // Note: BoxTester does not implement Send + Sync, so into_arc()
    // cannot be implemented. Calling into_arc() on BoxTester will result
    // in a compile error due to the Send + Sync trait bounds not being
    // satisfied. The default Tester trait implementation will be used.

    fn into_fn(self) -> impl Fn() -> bool {
        move || (self.function)()
    }

    // Note: BoxTester does not implement Clone, so to_box(), to_rc(),
    // to_arc(), and to_fn() cannot be implemented. Calling these methods
    // on BoxTester will result in a compile error due to the Clone trait
    // bound not being satisfied. The default Tester trait implementations
    // will be used.
}

// ============================================================================
// ArcTester: Thread-Safe Shared Ownership Implementation
// ============================================================================

/// Thread-safe shared ownership Tester implemented using `Arc`
///
/// `ArcTester` wraps a closure in `Arc<dyn Fn() -> bool + Send + Sync>`,
/// allowing the tester to be cloned and safely shared across threads.
///
/// # Characteristics
///
/// - **Shared ownership**: Can be cloned
/// - **Thread-safe**: Can be sent across threads
/// - **Lock-free overhead**: Uses `Fn` without needing `Mutex`
/// - **Borrowing combination**: `and()`/`or()`/`not()` borrow `&self`
///
/// # Use Cases
///
/// - Multi-threaded testing scenarios
/// - Health checks shared across threads
/// - Test states requiring concurrent access
/// - Background monitoring tasks
///
/// # Examples
///
/// ```rust
/// use prism3_function::{ArcTester, Tester};
/// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
/// use std::thread;
///
/// // Shared atomic counter
/// let counter = Arc::new(AtomicUsize::new(0));
/// let counter_clone = Arc::clone(&counter);
///
/// let shared = ArcTester::new(move || {
///     counter_clone.load(Ordering::Relaxed) <= 5
/// });
///
/// let clone = shared.clone();
/// let handle = thread::spawn(move || {
///     clone.test()
/// });
///
/// assert!(handle.join().unwrap());
/// counter.fetch_add(1, Ordering::Relaxed);
/// assert!(shared.test());
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct ArcTester {
    function: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl ArcTester {
    /// Creates a new `ArcTester` from a closure
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type implementing `Fn() -> bool + Send + Sync`
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::ArcTester;
    ///
    /// let tester = ArcTester::new(|| true);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        ArcTester { function: Arc::new(f) }
    }

    /// Combines this tester with another tester using logical AND
    ///
    /// Returns a new `ArcTester` that returns `true` only when both tests
    /// pass. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical AND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
    /// use std::thread;
    ///
    /// // Simulate database connection pool status
    /// let active_connections = Arc::new(AtomicUsize::new(0));
    /// let is_pool_healthy = Arc::new(AtomicBool::new(true));
    /// let max_connections = 50;
    ///
    /// let conn_clone = Arc::clone(&active_connections);
    /// let health_clone = Arc::clone(&is_pool_healthy);
    ///
    /// // Connection pool health check
    /// let pool_healthy = ArcTester::new(move || {
    ///     health_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// // Connection count check
    /// let conn_ok = ArcTester::new(move || {
    ///     conn_clone.load(Ordering::Relaxed) < max_connections
    /// });
    ///
    /// // Combined check: pool healthy and connection count not exceeded
    /// let pool_ready = pool_healthy.and(&conn_ok);
    ///
    /// // Multi-threaded test
    /// let pool_ready_clone = pool_ready.clone();
    /// let handle = thread::spawn(move || {
    ///     pool_ready_clone.test()
    /// });
    ///
    /// // Initial state should pass
    /// assert!(handle.join().unwrap());
    /// assert!(pool_ready.test());
    ///
    /// // Connection count exceeded
    /// active_connections.store(60, Ordering::Relaxed);
    /// assert!(!pool_ready.test());
    ///
    /// // Connection pool unhealthy
    /// is_pool_healthy.store(false, Ordering::Relaxed);
    /// assert!(!pool_ready.test());
    /// ```
    pub fn and(&self, next: &ArcTester) -> ArcTester {
        let self_fn = Arc::clone(&self.function);
        let next_fn = Arc::clone(&next.function);
        ArcTester {
            function: Arc::new(move || self_fn() && next_fn()),
        }
    }

    /// Combines this tester with another tester using logical OR
    ///
    /// Returns a new `ArcTester` that returns `true` if either test passes.
    /// Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical OR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
    /// use std::thread;
    ///
    /// // Simulate load balancer status
    /// let server_load = Arc::new(AtomicUsize::new(0));
    /// let is_server_healthy = Arc::new(AtomicBool::new(true));
    /// let max_load = 80;
    /// let emergency_mode = Arc::new(AtomicBool::new(false));
    ///
    /// let load_clone = Arc::clone(&server_load);
    /// let health_clone = Arc::clone(&is_server_healthy);
    /// let emergency_clone = Arc::clone(&emergency_mode);
    ///
    /// // Server low load
    /// let low_load = ArcTester::new(move || {
    ///     load_clone.load(Ordering::Relaxed) < max_load
    /// });
    ///
    /// // Emergency mode check
    /// let emergency_check = ArcTester::new(move || {
    ///     emergency_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// // Server health check
    /// let server_healthy = ArcTester::new(move || {
    ///     health_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// // Emergency mode or server healthy
    /// let can_handle_requests = emergency_check.or(&server_healthy);
    ///
    /// // Combined condition: low load or can handle requests
    /// let should_route_here = low_load.or(&can_handle_requests);
    ///
    /// // Multi-threaded test
    /// let router_clone = should_route_here.clone();
    /// let handle = thread::spawn(move || {
    ///     router_clone.test()
    /// });
    ///
    /// // Initial state: low load and healthy
    /// assert!(handle.join().unwrap());
    /// assert!(should_route_here.test());
    ///
    /// // High load but server healthy
    /// server_load.store(90, Ordering::Relaxed);
    /// assert!(should_route_here.test()); // still healthy
    ///
    /// // Server unhealthy but emergency mode
    /// is_server_healthy.store(false, Ordering::Relaxed);
    /// emergency_mode.store(true, Ordering::Relaxed);
    /// assert!(should_route_here.test()); // emergency mode
    ///
    /// // Unhealthy and not emergency mode
    /// emergency_mode.store(false, Ordering::Relaxed);
    /// assert!(!should_route_here.test());
    /// ```
    pub fn or(&self, next: &ArcTester) -> ArcTester {
        let self_fn = Arc::clone(&self.function);
        let next_fn = Arc::clone(&next.function);
        ArcTester {
            function: Arc::new(move || self_fn() || next_fn()),
        }
    }

    /// Negates the result of this tester
    ///
    /// Returns a new `ArcTester` that returns the opposite value of the
    /// original test result. Borrows `&self`, so the original tester remains
    /// available.
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical NOT
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    /// use std::thread;
    ///
    /// // Simulate task queue status
    /// let pending_tasks = Arc::new(AtomicUsize::new(0));
    /// let max_queue_size = 100;
    ///
    /// let tasks_clone = Arc::clone(&pending_tasks);
    ///
    /// // Queue not full
    /// let queue_available = ArcTester::new(move || {
    ///     tasks_clone.load(Ordering::Relaxed) < max_queue_size
    /// });
    ///
    /// // Queue full (negated)
    /// let queue_full = queue_available.not();
    ///
    /// // Multi-threaded test
    /// let queue_full_clone = queue_full.clone();
    /// let handle = thread::spawn(move || {
    ///     queue_full_clone.test()
    /// });
    ///
    /// // Initial state: queue not full
    /// pending_tasks.store(50, Ordering::Relaxed);
    /// assert!(queue_available.test());
    /// assert!(!handle.join().unwrap());
    /// assert!(!queue_full.test());
    ///
    /// // Queue near full
    /// pending_tasks.store(95, Ordering::Relaxed);
    /// assert!(queue_available.test());
    /// assert!(!queue_full.test());
    ///
    /// // Queue full
    /// pending_tasks.store(120, Ordering::Relaxed);
    /// assert!(!queue_available.test());
    /// assert!(queue_full.test());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> ArcTester {
        let func = Arc::clone(&self.function);
        ArcTester {
            function: Arc::new(move || !func()),
        }
    }

    /// Combines this tester with another tester using logical NAND
    ///
    /// Returns a new `ArcTester` that returns `true` unless both tests pass.
    /// Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical NAND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    /// use std::thread;
    ///
    /// let flag1 = Arc::new(AtomicBool::new(true));
    /// let flag2 = Arc::new(AtomicBool::new(true));
    ///
    /// let flag1_clone = Arc::clone(&flag1);
    /// let flag2_clone = Arc::clone(&flag2);
    ///
    /// let tester1 = ArcTester::new(move || {
    ///     flag1_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let tester2 = ArcTester::new(move || {
    ///     flag2_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let nand = tester1.nand(&tester2);
    ///
    /// // Both true returns false
    /// assert!(!nand.test());
    ///
    /// // At least one false returns true
    /// flag1.store(false, Ordering::Relaxed);
    /// assert!(nand.test());
    ///
    /// // Original tester still available
    /// assert!(!tester1.test());
    /// assert!(tester2.test());
    /// ```
    pub fn nand(&self, next: &ArcTester) -> ArcTester {
        let self_fn = Arc::clone(&self.function);
        let next_fn = Arc::clone(&next.function);
        ArcTester {
            function: Arc::new(move || !(self_fn() && next_fn())),
        }
    }

    /// Combines this tester with another tester using logical XOR
    ///
    /// Returns a new `ArcTester` that returns `true` if exactly one test
    /// passes. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical XOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    /// use std::thread;
    ///
    /// let flag1 = Arc::new(AtomicBool::new(true));
    /// let flag2 = Arc::new(AtomicBool::new(false));
    ///
    /// let flag1_clone = Arc::clone(&flag1);
    /// let flag2_clone = Arc::clone(&flag2);
    ///
    /// let tester1 = ArcTester::new(move || {
    ///     flag1_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let tester2 = ArcTester::new(move || {
    ///     flag2_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let xor = tester1.xor(&tester2);
    ///
    /// // One true one false returns true
    /// assert!(xor.test());
    ///
    /// // Both true returns false
    /// flag2.store(true, Ordering::Relaxed);
    /// assert!(!xor.test());
    ///
    /// // Both false returns false
    /// flag1.store(false, Ordering::Relaxed);
    /// flag2.store(false, Ordering::Relaxed);
    /// assert!(!xor.test());
    ///
    /// // Original tester still available
    /// assert!(!tester1.test());
    /// assert!(!tester2.test());
    /// ```
    pub fn xor(&self, next: &ArcTester) -> ArcTester {
        let self_fn = Arc::clone(&self.function);
        let next_fn = Arc::clone(&next.function);
        ArcTester {
            function: Arc::new(move || self_fn() ^ next_fn()),
        }
    }

    /// Combines this tester with another tester using logical NOR
    ///
    /// Returns a new `ArcTester` that returns `true` only when both tests
    /// fail. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `ArcTester` representing logical NOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{ArcTester, Tester};
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    /// use std::thread;
    ///
    /// let flag1 = Arc::new(AtomicBool::new(false));
    /// let flag2 = Arc::new(AtomicBool::new(false));
    ///
    /// let flag1_clone = Arc::clone(&flag1);
    /// let flag2_clone = Arc::clone(&flag2);
    ///
    /// let tester1 = ArcTester::new(move || {
    ///     flag1_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let tester2 = ArcTester::new(move || {
    ///     flag2_clone.load(Ordering::Relaxed)
    /// });
    ///
    /// let nor = tester1.nor(&tester2);
    ///
    /// // Both false returns true
    /// assert!(nor.test());
    ///
    /// // At least one true returns false
    /// flag1.store(true, Ordering::Relaxed);
    /// assert!(!nor.test());
    ///
    /// // Original tester still available
    /// assert!(tester1.test());
    /// assert!(!tester2.test());
    /// ```
    pub fn nor(&self, next: &ArcTester) -> ArcTester {
        let self_fn = Arc::clone(&self.function);
        let next_fn = Arc::clone(&next.function);
        ArcTester {
            function: Arc::new(move || !(self_fn() || next_fn())),
        }
    }
}

impl Tester for ArcTester {
    fn test(&self) -> bool {
        (self.function)()
    }

    fn into_box(self) -> BoxTester {
        let func = self.function;
        BoxTester {
            function: Box::new(move || func()),
        }
    }

    fn into_rc(self) -> RcTester {
        let func = self.function;
        RcTester {
            function: Rc::new(move || func()),
        }
    }

    fn into_arc(self) -> ArcTester {
        self
    }

    fn into_fn(self) -> impl Fn() -> bool {
        move || (self.function)()
    }

    fn to_box(&self) -> BoxTester {
        let self_fn = self.function.clone();
        BoxTester {
            function: Box::new(move || self_fn()),
        }
    }

    fn to_rc(&self) -> RcTester {
        let self_fn = self.function.clone();
        RcTester {
            function: Rc::new(move || self_fn()),
        }
    }

    fn to_arc(&self) -> ArcTester {
        self.clone()
    }

    fn to_fn(&self) -> impl Fn() -> bool {
        let self_fn = self.function.clone();
        move || self_fn()
    }
}

impl Clone for ArcTester {
    /// Creates a clone of this `ArcTester`.
    ///
    /// The cloned instance shares the same underlying function with
    /// the original, allowing multiple references to the same test
    /// logic.
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
        }
    }
}

// ============================================================================
// RcTester: Single-Threaded Shared Ownership Implementation
// ============================================================================

/// Single-threaded shared ownership Tester implemented using `Rc`
///
/// `RcTester` wraps a closure in `Rc<dyn Fn() -> bool>`, allowing the tester
/// to be cloned and shared within a single thread. Since it doesn't use atomic
/// operations, it has lower overhead than `ArcTester`.
///
/// # Characteristics
///
/// - **Shared ownership**: Can be cloned
/// - **Single-threaded**: Cannot be sent across threads
/// - **Low overhead**: Uses `Fn` without needing `RefCell`
/// - **Borrowing combination**: `and()`/`or()`/`not()` borrow `&self`
///
/// # Use Cases
///
/// - Single-threaded testing scenarios requiring sharing
/// - Event-driven systems (single-threaded)
/// - Callback-intensive code requiring cloneable tests
/// - Performance-sensitive single-threaded code
///
/// # Examples
///
/// ```rust
/// use prism3_function::{RcTester, Tester};
///
/// let shared = RcTester::new(|| true);
///
/// // Clone for multiple uses
/// let clone1 = shared.clone();
/// let clone2 = shared.clone();
///
/// // Non-consuming combination
/// let combined = shared.and(&clone1);
/// ```
///
/// # Author
///
/// Hu Haixing
pub struct RcTester {
    function: Rc<dyn Fn() -> bool>,
}

impl RcTester {
    /// Creates a new `RcTester` from a closure
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type implementing `Fn() -> bool`
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Return Value
    ///
    /// A new `RcTester` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::RcTester;
    ///
    /// let tester = RcTester::new(|| true);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        RcTester { function: Rc::new(f) }
    }

    /// Combines this tester with another tester using logical AND
    ///
    /// Returns a new `RcTester` that returns `true` only when both tests
    /// pass. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical AND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let first = RcTester::new(|| true);
    /// let second = RcTester::new(|| true);
    /// let combined = first.and(&second);
    /// // first and second are still available
    /// ```
    pub fn and(&self, next: &RcTester) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        let next_fn = Rc::clone(&next.function);
        RcTester {
            function: Rc::new(move || self_fn() && next_fn()),
        }
    }

    /// Combines this tester with another tester using logical OR
    ///
    /// Returns a new `RcTester` that returns `true` if either test passes.
    /// Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical OR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let first = RcTester::new(|| false);
    /// let second = RcTester::new(|| true);
    /// let combined = first.or(&second);
    /// // first and second are still available
    /// ```
    pub fn or(&self, next: &RcTester) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        let next_fn = Rc::clone(&next.function);
        RcTester {
            function: Rc::new(move || self_fn() || next_fn()),
        }
    }

    /// Negates the result of this tester
    ///
    /// Returns a new `RcTester` that returns the opposite value of the
    /// original test result. Borrows `&self`, so the original tester remains
    /// available.
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical NOT
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let original = RcTester::new(|| true);
    /// let negated = original.not();
    /// // original is still available
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(&self) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        RcTester {
            function: Rc::new(move || !self_fn()),
        }
    }

    /// Combines this tester with another tester using logical NAND
    ///
    /// Returns a new `RcTester` that returns `true` unless both tests pass.
    /// Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical NAND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let first = RcTester::new(|| true);
    /// let second = RcTester::new(|| true);
    /// let nand = first.nand(&second);
    ///
    /// // Both true returns false
    /// assert!(!nand.test());
    ///
    /// // first and second still available
    /// assert!(first.test());
    /// assert!(second.test());
    /// ```
    pub fn nand(&self, next: &RcTester) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        let next_fn = Rc::clone(&next.function);
        RcTester {
            function: Rc::new(move || !(self_fn() && next_fn())),
        }
    }

    /// Combines this tester with another tester using logical XOR
    ///
    /// Returns a new `RcTester` that returns `true` if exactly one test
    /// passes. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical XOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let first = RcTester::new(|| true);
    /// let second = RcTester::new(|| false);
    /// let xor = first.xor(&second);
    ///
    /// // One true one false returns true
    /// assert!(xor.test());
    ///
    /// // first and second still available
    /// assert!(first.test());
    /// assert!(!second.test());
    /// ```
    pub fn xor(&self, next: &RcTester) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        let next_fn = Rc::clone(&next.function);
        RcTester {
            function: Rc::new(move || self_fn() ^ next_fn()),
        }
    }

    /// Combines this tester with another tester using logical NOR
    ///
    /// Returns a new `RcTester` that returns `true` only when both tests
    /// fail. Borrows `&self`, so the original tester remains available.
    ///
    /// # Parameters
    ///
    /// * `next` - The tester to combine with
    ///
    /// # Return Value
    ///
    /// A new `RcTester` representing logical NOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{RcTester, Tester};
    ///
    /// let first = RcTester::new(|| false);
    /// let second = RcTester::new(|| false);
    /// let nor = first.nor(&second);
    ///
    /// // Both false returns true
    /// assert!(nor.test());
    ///
    /// // first and second still available
    /// assert!(!first.test());
    /// assert!(!second.test());
    /// ```
    pub fn nor(&self, next: &RcTester) -> RcTester {
        let self_fn = Rc::clone(&self.function);
        let next_fn = Rc::clone(&next.function);
        RcTester {
            function: Rc::new(move || !(self_fn() || next_fn())),
        }
    }
}

impl Tester for RcTester {
    fn test(&self) -> bool {
        (self.function)()
    }

    fn into_box(self) -> BoxTester {
        BoxTester {
            function: Box::new(move || (self.function)()),
        }
    }

    fn into_rc(self) -> RcTester {
        self
    }

    // Note: RcTester is not Send + Sync, so into_arc() cannot be
    // implemented. Calling into_arc() on RcTester will result in a
    // compile error due to the Send + Sync trait bounds not being
    // satisfied. The default Tester trait implementation will be used.

    fn into_fn(self) -> impl Fn() -> bool {
        move || (self.function)()
    }

    fn to_box(&self) -> BoxTester {
        let self_fn = self.function.clone();
        BoxTester {
            function: Box::new(move || self_fn()),
        }
    }

    fn to_rc(&self) -> RcTester {
        self.clone()
    }

    // Note: RcTester is not Send + Sync, so to_arc() cannot be
    // implemented. Calling to_arc() on RcTester will result in a compile
    // error due to the Send + Sync trait bounds not being satisfied. The
    // default Tester trait implementation will be used.

    fn to_fn(&self) -> impl Fn() -> bool {
        let self_fn = self.function.clone();
        move || self_fn()
    }
}

impl Clone for RcTester {
    /// Creates a clone of this `RcTester`.
    ///
    /// The cloned instance shares the same underlying function with
    /// the original, allowing multiple references to the same test
    /// logic.
    fn clone(&self) -> Self {
        Self {
            function: Rc::clone(&self.function),
        }
    }
}

// ============================================================================
// Tester Implementation for Closures
// ============================================================================

impl<F> Tester for F
where
    F: Fn() -> bool,
{
    fn test(&self) -> bool {
        self()
    }

    fn into_box(self) -> BoxTester
    where
        Self: Sized + 'static,
    {
        BoxTester::new(self)
    }

    fn into_rc(self) -> RcTester
    where
        Self: Sized + 'static,
    {
        RcTester::new(self)
    }

    fn into_arc(self) -> ArcTester
    where
        Self: Sized + Send + Sync + 'static,
    {
        ArcTester::new(self)
    }

    fn into_fn(self) -> impl Fn() -> bool
    where
        Self: Sized + 'static,
    {
        self
    }

    // Note: Closures generally do not implement Clone unless they are
    // empty or only capture Copy types. Therefore, to_box(), to_rc(),
    // to_arc(), and to_fn() will use the default trait implementation,
    // which will result in a compile error for most closures due to the
    // Clone trait bound not being satisfied.
}

// ============================================================================
// Extension Trait for Convenient Closure Conversion
// ============================================================================

/// Extension trait providing logical composition methods for closures
///
/// This trait is automatically implemented for all closures and function
/// pointers that match `Fn() -> bool`, enabling method chaining starting
/// from a closure.
///
/// # Examples
///
/// ```rust
/// use prism3_function::{FnTesterOps, Tester};
///
/// let is_ready = || true;
/// let is_available = || true;
///
/// // Combine testers using extension methods
/// let combined = is_ready.and(is_available);
/// assert!(combined.test());
/// ```
///
/// # Author
///
/// Hu Haixing
pub trait FnTesterOps: Sized + Fn() -> bool + 'static {
    /// Returns a tester that represents the logical AND of this tester
    /// and another
    ///
    /// # Parameters
    ///
    /// * `other` - The other tester to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original tester, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure
    ///   - A function pointer
    ///   - A `BoxTester`, `RcTester`, or `ArcTester`
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical AND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || true;
    /// let is_available = || true;
    ///
    /// let combined = is_ready.and(is_available);
    /// assert!(combined.test());
    /// ```
    fn and<T>(self, other: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        BoxTester::new(move || self.test() && other.test())
    }

    /// Returns a tester that represents the logical OR of this tester
    /// and another
    ///
    /// # Parameters
    ///
    /// * `other` - The other tester to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original tester, clone it first (if it implements
    ///   `Clone`). Can be:
    ///   - Another closure
    ///   - A function pointer
    ///   - A `BoxTester`, `RcTester`, or `ArcTester`
    ///   - Any type implementing `Tester`
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical OR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || false;
    /// let is_fallback = || true;
    ///
    /// let combined = is_ready.or(is_fallback);
    /// assert!(combined.test());
    /// ```
    fn or<T>(self, other: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        BoxTester::new(move || self.test() || other.test())
    }

    /// Returns a tester that represents the logical negation of this tester
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical negation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || false;
    /// let not_ready = is_ready.not();
    /// assert!(not_ready.test());
    /// ```
    fn not(self) -> BoxTester {
        BoxTester::new(move || !self.test())
    }

    /// Returns a tester that represents the logical NAND (NOT AND) of this
    /// tester and another
    ///
    /// NAND returns `true` unless both testers are `true`.
    /// Equivalent to `!(self AND other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other tester to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original tester, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Tester` implementation.
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical NAND
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || true;
    /// let is_available = || true;
    ///
    /// let nand = is_ready.nand(is_available);
    /// assert!(!nand.test());  // !(true && true) = false
    /// ```
    fn nand<T>(self, other: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        BoxTester::new(move || !(self.test() && other.test()))
    }

    /// Returns a tester that represents the logical XOR (exclusive OR) of
    /// this tester and another
    ///
    /// XOR returns `true` if exactly one of the testers is `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other tester to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original tester, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Tester` implementation.
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical XOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || true;
    /// let is_available = || false;
    ///
    /// let xor = is_ready.xor(is_available);
    /// assert!(xor.test());  // true ^ false = true
    /// ```
    fn xor<T>(self, other: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        BoxTester::new(move || self.test() ^ other.test())
    }

    /// Returns a tester that represents the logical NOR (NOT OR) of this
    /// tester and another
    ///
    /// NOR returns `true` only when both testers are `false`. Equivalent
    /// to `!(self OR other)`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other tester to combine with. **Note: This parameter
    ///   is passed by value and will transfer ownership.** If you need to
    ///   preserve the original tester, clone it first (if it implements
    ///   `Clone`). Accepts closures, function pointers, or any
    ///   `Tester` implementation.
    ///
    /// # Return Value
    ///
    /// A `BoxTester` representing the logical NOR
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{FnTesterOps, Tester};
    ///
    /// let is_ready = || false;
    /// let is_available = || false;
    ///
    /// let nor = is_ready.nor(is_available);
    /// assert!(nor.test());  // !(false || false) = true
    /// ```
    fn nor<T>(self, other: T) -> BoxTester
    where
        T: Tester + 'static,
    {
        BoxTester::new(move || !(self.test() || other.test()))
    }
}

// Blanket implementation for all closures
impl<F> FnTesterOps for F where F: Fn() -> bool + 'static {}
