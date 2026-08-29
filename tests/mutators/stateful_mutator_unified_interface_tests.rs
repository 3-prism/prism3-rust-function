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
mod test_unified_interface {
    use super::ArcStatefulMutator;
    use super::BoxStatefulMutator;
    use super::RcStatefulMutator;
    use super::StatefulMutator;

    fn apply_mutator<C: StatefulMutator<i32>>(mutator: &mut C, value: i32) -> i32 {
        let mut val = value;
        mutator.apply(&mut val);
        val
    }

    #[test]
    fn test_with_box_consumer() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_arc_consumer() {
        let mut mutator = ArcStatefulMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_rc_consumer() {
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_closure() {
        let mut closure = |x: &mut i32| *x *= 2;
        assert_eq!(apply_mutator(&mut closure, 5), 10);
    }
}

// ============================================================================
// Complex Scenarios Tests
// ============================================================================
