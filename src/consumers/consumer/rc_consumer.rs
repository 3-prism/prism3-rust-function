// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcConsumer` public type.

use std::rc::Rc;

use crate::Consumer;
use crate::Predicate;
use crate::RcConditionalConsumer;
use crate::consumers::macros::impl_consumer_clone;
use crate::consumers::macros::impl_consumer_common_methods;
use crate::consumers::macros::impl_consumer_debug_display;
use crate::consumers::macros::impl_shared_consumer_methods;

// ============================================================================
// 3. RcConsumer - Single-threaded Shared Ownership Implementation
// ============================================================================

/// RcConsumer struct
///
/// Non-mutating consumer implementation based on `Rc<dyn Fn(&T)>` for
/// single-threaded shared ownership scenarios. The wrapper does not need
/// `RefCell` because it only invokes a shared `Fn`.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable through `Rc`, allows multiple owners
/// - **Single-threaded**: Not thread-safe, cannot be sent across threads
/// - **No Wrapper Interior Mutability Overhead**: No RefCell needed by the
///   wrapper
/// - **Non-consuming API**: `and_then` borrows `&self`, original object remains
///   usable
///
/// # Use Cases
///
/// Choose `RcConsumer` when:
/// - Need to share non-mutating consumer within a single thread
/// - Observation operations requiring single-threaded shared ownership
/// - Event handling in single-threaded UI frameworks
///
/// # Ownership Cost
///
/// `Rc` uses non-atomic reference counting and is single-threaded; `Arc` uses
/// atomic reference counting and supports thread-safe sharing. Neither wrapper
/// adds a mutex around callback invocation. Choose based on thread-safety
/// requirements and benchmark the real workload when performance matters.
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Consumer, RcConsumer};
///
/// let consumer = RcConsumer::new(|x: &i32| {
///     println!("Observed: {}", x);
/// });
/// let clone = consumer.clone();
///
/// consumer.accept(&5);
/// clone.accept(&10);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcConsumer<T> {
    /// The wrapped callback implementation.
    pub(super) function: Rc<dyn Fn(&T)>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcConsumer<T> {
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(RcConsumer<T>, (Fn(&T) + 'static), |f| { Rc::new(f) });

    // Generates: when() and and_then() methods that borrow &self (Rc can clone)
    impl_shared_consumer_methods!(
        RcConsumer<T>,
        RcConditionalConsumer,
        RcPredicate,
        Consumer,
        predicate_bounds = ('static),
        chained_bounds = ('static)
    );
}

impl<T> Consumer<T> for RcConsumer<T> {
    #[inline(always)]
    fn accept(&self, value: &T) {
        (self.function)(value)
    }
}

// Use macro to generate Clone implementation
impl_consumer_clone!(RcConsumer<T>);

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(RcConsumer<T>);
