// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxStatefulConsumer` public type.

use crate::BoxConditionalStatefulConsumer;
use crate::Predicate;
use crate::StatefulConsumer;
use crate::consumers::macros::impl_box_consumer_methods;
use crate::consumers::macros::impl_consumer_common_methods;
use crate::consumers::macros::impl_consumer_debug_display;

// ============================================================================
// 2. BoxStatefulConsumer - Single Ownership Implementation
// ============================================================================

/// BoxStatefulConsumer struct
///
/// Consumer implementation based on `Box<dyn FnMut(&T)>` for single ownership
/// scenarios. When sharing is not needed, this is the simplest and most
/// efficient consumer type.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership when used
/// - **Runtime cost**: One heap allocation and dynamic dispatch; no reference
///   counting or locking
/// - **Mutable State**: Can modify captured environment through `FnMut`
/// - **Builder Pattern**: Method chaining naturally consumes `self`
///
/// # Use Cases
///
/// Choose `BoxStatefulConsumer` when:
/// - Consumer is used only once or in a linear flow
/// - Building pipelines where ownership flows naturally
/// - No need to share consumers across contexts
/// - Performance critical and cannot accept sharing overhead
///
/// # Performance
///
/// `BoxStatefulConsumer` has the best performance among the three consumer
/// types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrowing checks
/// - Direct function calls through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Consumer, StatefulConsumer, BoxStatefulConsumer};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
///     l.lock().expect("mutex should not be poisoned").push(*x);
/// });
/// consumer.accept(&5);
/// assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxStatefulConsumer<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnMut(&T)>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxStatefulConsumer<T> {
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(BoxStatefulConsumer<T>, (FnMut(&T) + 'static), |f| Box::new(f));

    // Generates: when() and and_then() methods that consume self
    impl_box_consumer_methods!(BoxStatefulConsumer<T>, BoxConditionalStatefulConsumer, StatefulConsumer);
}

impl<T> StatefulConsumer<T> for BoxStatefulConsumer<T> {
    #[inline(always)]
    fn accept(&mut self, value: &T) {
        (self.function)(value)
    }
}

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(BoxStatefulConsumer<T>);
