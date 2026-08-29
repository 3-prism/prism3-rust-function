// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcStatefulConsumer` public type.

use std::cell::RefCell;
use std::rc::Rc;

use crate::Predicate;
use crate::RcConditionalStatefulConsumer;
use crate::StatefulConsumer;
use crate::consumers::macros::impl_consumer_clone;
use crate::consumers::macros::impl_consumer_common_methods;
use crate::consumers::macros::impl_consumer_debug_display;
use crate::consumers::macros::impl_shared_consumer_methods;

/// The erased callback representation used by this implementation.
type RcStatefulConsumerFn<T> = Rc<RefCell<dyn FnMut(&T)>>;

// ============================================================================
// 3. RcStatefulConsumer - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcStatefulConsumer struct
///
/// Consumer implementation based on `Rc<RefCell<dyn FnMut(&T)>>` for
/// single-threaded shared ownership scenarios. This consumer provides the
/// benefits of shared ownership without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allowing multiple owners
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrowing checks
/// - **No Lock Overhead**: More efficient than `ArcStatefulConsumer` for
///   single-threaded use
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains
///   usable
///
/// # Use Cases
///
/// Choose `RcStatefulConsumer` when:
/// - Need to share consumers within a single thread
/// - Thread safety is not needed
/// - Performance is important (avoid lock overhead)
/// - UI event handling in single-threaded frameworks
/// - Building complex single-threaded state machines
///
/// # Performance Considerations
///
/// `RcStatefulConsumer` performs better than `ArcStatefulConsumer` in
/// single-threaded scenarios:
/// - **Non-Atomic Counting**: clone/drop is cheaper than `Arc`
/// - **No Lock Overhead**: `RefCell` uses runtime checks, no locks
/// - **Better Cache Locality**: No atomic operations means better CPU cache
///   behavior
///
/// But still has slight overhead compared to `BoxStatefulConsumer`:
/// - **Reference Counting**: Non-atomic but still exists
/// - **Runtime Borrowing Checks**: `RefCell` checks at runtime
///
/// # Safety
///
/// `RcStatefulConsumer` is not thread-safe and does not implement `Send` or
/// `Sync`. Attempting to send it to another thread will result in a compilation
/// error. For thread-safe sharing, use `ArcStatefulConsumer` instead.
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Consumer, StatefulConsumer, RcStatefulConsumer};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let log = Rc::new(RefCell::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
///     l.borrow_mut().push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.borrow(), vec![10]);
/// ```
/// # Borrowing and reentrancy
///
/// Each call holds a mutable `RefCell` borrow while the user callback runs.
/// Synchronous re-entry through the same shared wrapper panics with a borrow
/// error. Mutations completed before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcStatefulConsumer<T> {
    /// The wrapped callback implementation.
    pub(super) function: RcStatefulConsumerFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcStatefulConsumer<T> {
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(RcStatefulConsumer<T>, (FnMut(&T) + 'static), |f| Rc::new(RefCell::new(
        f
    )));

    // Generates: when() and and_then() methods that borrow &self (Rc can clone)
    impl_shared_consumer_methods!(
        RcStatefulConsumer<T>,
        RcConditionalStatefulConsumer,
        RcPredicate,
        StatefulConsumer,
        predicate_bounds = ('static),
        chained_bounds = ('static)
    );
}

impl<T> StatefulConsumer<T> for RcStatefulConsumer<T> {
    #[inline]
    fn accept(&mut self, value: &T) {
        let mut function = self.function.borrow_mut();
        function(value)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(RcStatefulConsumer<T>);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(RcStatefulConsumer<T>);
