// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulMutator types

use qubit_function::ArcStatefulMutator;
use qubit_function::BoxStatefulMutator;
use qubit_function::MutatorOnce;
use qubit_function::RcStatefulMutator;
use qubit_function::StatefulMutator;

// ============================================================================
// BoxStatefulMutator Tests
// ============================================================================

#[cfg(test)]
mod test_complex_scenarios {
    use super::ArcStatefulMutator;
    use super::BoxStatefulMutator;
    use super::RcStatefulMutator;
    use super::StatefulMutator;

    #[test]
    fn test_data_processing_pipeline() {
        let mut pipeline = BoxStatefulMutator::new(|x: &mut i32| {
            *x = (*x).clamp(0, 100);
        })
        .and_then(|x: &mut i32| *x /= 10)
        .and_then(|x: &mut i32| *x = *x * *x);

        let mut value1 = -50;
        pipeline.apply(&mut value1);
        assert_eq!(value1, 0);

        let mut value2 = 200;
        pipeline.apply(&mut value2);
        assert_eq!(value2, 100);

        let mut value3 = 30;
        pipeline.apply(&mut value3);
        assert_eq!(value3, 9);
    }

    #[test]
    fn test_string_processing() {
        let mut processor = BoxStatefulMutator::new(|s: &mut String| s.retain(|c| !c.is_whitespace()))
            .and_then(|s: &mut String| *s = s.to_lowercase())
            .and_then(|s: &mut String| s.push_str("!!!"));

        let mut text = String::from("Hello World");
        processor.apply(&mut text);
        assert_eq!(text, "helloworld!!!");
    }

    #[test]
    fn test_conditional_processing() {
        let cond1 = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let cond2 = BoxStatefulMutator::new(|x: &mut i32| *x = 100).when(|x: &i32| *x > 100);
        let mut processor = cond1.and_then(cond2);

        let mut small = 5;
        processor.apply(&mut small);
        assert_eq!(small, 10);

        let mut large = 60;
        processor.apply(&mut large);
        assert_eq!(large, 100);
    }

    #[test]
    fn test_mixed_operations() {
        let cond = BoxStatefulMutator::new(|x: &mut i32| *x -= 20).when(|x: &i32| *x > 50);
        let mut processor = BoxStatefulMutator::new(|x: &mut i32| *x += 10)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(cond);

        let mut value1 = 5;
        processor.apply(&mut value1);
        assert_eq!(value1, 30); // (5 + 10) * 2 = 30

        let mut value2 = 20;
        processor.apply(&mut value2);
        assert_eq!(value2, 40); // (20 + 10) * 2 = 60, 60 > 50 so 60 - 20 = 40
    }

    #[test]
    fn test_arc_mutator_reuse() {
        let double = ArcStatefulMutator::new(|x: &mut i32| *x *= 2);
        let add_ten = ArcStatefulMutator::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(add_ten.clone());
        let pipeline2 = add_ten.and_then(double.clone());

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.apply(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.apply(&mut value2);
        assert_eq!(value2, 30); // (5 + 10) * 2
    }

    #[test]
    fn test_rc_mutator_reuse() {
        let double = RcStatefulMutator::new(|x: &mut i32| *x *= 2);
        let add_ten = RcStatefulMutator::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(add_ten.clone());
        let pipeline2 = add_ten.and_then(double.clone());

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.apply(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.apply(&mut value2);
        assert_eq!(value2, 30); // (5 + 10) * 2
    }
}

// ============================================================================
// Custom Types Tests
// ============================================================================
