// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcStatefulConsumer` public type.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::ArcConditionalStatefulConsumer;
use crate::Predicate;
use crate::StatefulConsumer;
use crate::consumers::macros::impl_consumer_clone;
use crate::consumers::macros::impl_consumer_common_methods;
use crate::consumers::macros::impl_consumer_debug_display;
use crate::consumers::macros::impl_shared_consumer_methods;

/// The erased callback representation used by this implementation.
type ArcStatefulConsumerFn<T> = Arc<Mutex<dyn FnMut(&T) + Send>>;

// ============================================================================
// 4. ArcStatefulConsumer - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcStatefulConsumer struct
///
/// Consumer implementation based on `Arc<Mutex<dyn FnMut(&T) + Send>>` for
/// thread-safe shared ownership scenarios. This consumer can be safely cloned
/// and shared across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Arc`, allowing multiple owners
/// - **Thread Safety**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe mutable access
/// - **Non-Consuming API**: `and_then` borrows `&self`, original object remains
///   usable
/// - **Cross-Thread Sharing**: Can be sent to other threads and used
///
/// # Use Cases
///
/// Choose `ArcStatefulConsumer` when:
/// - Need to share consumers across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Using the same consumer in multiple places simultaneously
/// - Need thread safety (Send + Sync)
///
/// # Performance Considerations
///
/// `ArcStatefulConsumer` has some performance overhead compared to
/// `BoxStatefulConsumer`:
/// - **Reference Counting**: Atomic operations on clone/drop
/// - **Mutex Locking**: Each `accept` call requires lock acquisition
/// - **Lock Contention**: High concurrency may cause contention
///
/// These overheads are necessary for safe concurrent access. If thread safety
/// is not needed, consider using `RcStatefulConsumer` for less single-threaded
/// sharing overhead.
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Consumer, StatefulConsumer, ArcStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
///     l.lock().expect("mutex should not be poisoned").push(*x * 2);
/// });
/// let mut clone = consumer.clone();
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10]);
/// ```
///
/// # Locking and reentrancy
///
/// Each call acquires a `parking_lot::Mutex` and holds it while the user
/// callback runs. Synchronous re-entry through the same shared state
/// deadlocks. The mutex is not poisoned after a panic, and mutations completed
/// before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcStatefulConsumer<T> {
    /// The wrapped callback implementation.
    pub(super) function: ArcStatefulConsumerFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> ArcStatefulConsumer<T> {
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(ArcStatefulConsumer<T>, (FnMut(&T) + Send + 'static), |f| {
        Arc::new(Mutex::new(f))
    });

    // Generates: when() and and_then() methods that borrow &self (Arc can
    // clone)
    impl_shared_consumer_methods!(
        ArcStatefulConsumer<T>,
        ArcConditionalStatefulConsumer,
        ArcPredicate,
        StatefulConsumer,
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + 'static)
    );
}

impl<T> StatefulConsumer<T> for ArcStatefulConsumer<T> {
    #[inline]
    fn accept(&mut self, value: &T) {
        let mut function = self.function.lock();
        function(value)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(ArcStatefulConsumer<T>);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(ArcStatefulConsumer<T>);
