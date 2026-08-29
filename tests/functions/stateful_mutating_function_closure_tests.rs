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
mod test_closure {
    use super::MutatingFunctionOnce;
    use super::Rc;
    use super::RefCell;
    use super::StatefulMutatingFunction;

    #[test]
    fn test_closure_implements_trait() {
        // Use Rc<RefCell<>> to properly test stateful behavior
        let count = Rc::new(RefCell::new(0));
        let count_clone = Rc::clone(&count);
        let closure = move |x: &mut i32| {
            let mut current = count_clone.borrow_mut();
            *current += 1;
            *x *= 2;
            *current
        };

        // Test direct closure calls
        let mut value = 5;
        let direct_result1 = closure(&mut value);
        assert_eq!(direct_result1, 1);
        assert_eq!(value, 10);

        let direct_result2 = closure(&mut value);
        assert_eq!(direct_result2, 2);
        assert_eq!(value, 20);

        // Test with trait
        let count2 = Rc::new(RefCell::new(0));
        let count2_clone = Rc::clone(&count2);
        let mut closure2 = move |x: &mut i32| {
            let mut current = count2_clone.borrow_mut();
            *current += 1;
            *x *= 2;
            *current
        };

        // Test that closure implements StatefulMutatingFunction trait
        let _trait_check: &mut dyn StatefulMutatingFunction<i32, i32> = &mut closure2;

        let mut value2 = 5;
        let result1 = StatefulMutatingFunction::apply(&mut closure2, &mut value2);
        assert_eq!(result1, 1);
        assert_eq!(value2, 10);

        let result2 = StatefulMutatingFunction::apply(&mut closure2, &mut value2);
        assert_eq!(result2, 2);
        assert_eq!(value2, 20);
    }
}

// ============================================================================
// StatefulMutatingFunction Debug and Display Tests
// ============================================================================
