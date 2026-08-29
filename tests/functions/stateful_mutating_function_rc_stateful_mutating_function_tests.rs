// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulMutatingFunction types (stateful FnMut(&mut T) ->
//! R)

use std::cell::RefCell;
use std::rc::Rc;

use qubit_function::ArcStatefulMutatingFunction;
use qubit_function::BoxStatefulMutatingFunction;
use qubit_function::MutatingFunctionOnce;
use qubit_function::RcStatefulMutatingFunction;
use qubit_function::StatefulMutatingFunction;

// ============================================================================
// StatefulMutatingFunction Default Implementation Tests
// ============================================================================

/// Test struct that implements StatefulMutatingFunction to test default methods
struct TestStatefulMutatingFunction {
    multiplier: i32,
}

impl TestStatefulMutatingFunction {
    fn new(multiplier: i32) -> Self {
        TestStatefulMutatingFunction { multiplier }
    }
}

impl StatefulMutatingFunction<i32, i32> for TestStatefulMutatingFunction {
    fn apply(&mut self, input: &mut i32) -> i32 {
        let old_value = *input;
        *input *= self.multiplier;
        old_value
    }
}

impl Clone for TestStatefulMutatingFunction {
    fn clone(&self) -> Self {
        TestStatefulMutatingFunction {
            multiplier: self.multiplier,
        }
    }
}

// ============================================================================
// BoxStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_rc_stateful_mutating_function {
    use super::RcStatefulMutatingFunction;
    use super::StatefulMutatingFunction;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_clone() {
        let counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut clone = counter.clone();

        let mut value1 = 5;
        assert_eq!(clone.apply(&mut value1), 1);
        assert_eq!(value1, 10);

        // Shared state
        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_identity() {
        let mut identity = RcStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.and_then::<String, _>(|count: &i32| format!("Call #{}", *count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }
}

// ============================================================================
// ArcStatefulMutatingFunction Tests
// ============================================================================
