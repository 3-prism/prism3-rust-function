// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # MutatorOnce Tests
//!
//! Tests the complete functionality of MutatorOnce trait and its
//! implementations.

use qubit_function::BoxMutatorOnce;
use qubit_function::MutatorOnce;

// Test closures specialization and default behaviors

// Custom MutatorOnce using default into_box/into_fn/to_box/to_fn
struct MyMutatorOnce {
    data: Vec<i32>,
}

impl MutatorOnce<Vec<i32>> for MyMutatorOnce {
    fn apply(self, value: &mut Vec<i32>) {
        value.extend(self.data);
    }
}

// ============================================================================
// Tests for MutatorOnce trait default implementations
// ============================================================================

// ============================================================================
// Tests for BoxMutatorOnce
// ============================================================================

#[test]
fn test_box_mutator_once_noop() {
    // Test that noop() creates a mutator that does nothing
    let noop = BoxMutatorOnce::<i32>::noop();
    let mut value = 42;
    noop.apply(&mut value);
    assert_eq!(value, 42); // Value should remain unchanged

    // Test with Vec
    let noop_vec = BoxMutatorOnce::<Vec<i32>>::noop();
    let mut vec = vec![1, 2, 3];
    noop_vec.apply(&mut vec);
    assert_eq!(vec, vec![1, 2, 3]); // Vec should remain unchanged
}

#[test]
fn test_box_mutator_once_when() {
    // Test when() with condition that passes
    let data = vec![1, 2, 3];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data);
    });
    let conditional = mutator.when(|x: &Vec<i32>| !x.is_empty());

    let mut target = vec![0];
    conditional.apply(&mut target);
    assert_eq!(target, vec![0, 1, 2, 3]); // Should execute

    // Test when() with condition that fails
    let data2 = vec![4, 5];
    let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data2);
    });
    let conditional2 = mutator2.when(|x: &Vec<i32>| x.is_empty());

    let mut target2 = vec![0];
    conditional2.apply(&mut target2);
    assert_eq!(target2, vec![0]); // Should not execute
}

#[test]
fn test_box_mutator_once_and_then() {
    let chained = BoxMutatorOnce::new(|value: &mut i32| *value *= 2).and_then(|value: &mut i32| *value += 3);

    let mut value = 4;
    chained.apply(&mut value);
    assert_eq!(value, 11);
}

// ============================================================================
// Tests for BoxConditionalMutatorOnce
// ============================================================================

#[test]
fn test_box_conditional_mutator_once_mutate() {
    // Test mutate() when condition is true
    let data = vec![1, 2];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data);
    });
    let conditional = mutator.when(|x: &Vec<i32>| x.len() < 5);

    let mut target = vec![0];
    conditional.apply(&mut target);
    assert_eq!(target, vec![0, 1, 2]);

    // Test mutate() when condition is false
    let data2 = vec![3, 4];
    let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data2);
    });
    let conditional2 = mutator2.when(|x: &Vec<i32>| x.len() > 10);

    let mut target2 = vec![0];
    conditional2.apply(&mut target2);
    assert_eq!(target2, vec![0]); // Should remain unchanged
}

#[test]
fn test_box_conditional_mutator_once_and_then() {
    // Test and_then() to chain conditional mutators
    let data1 = vec![1, 2];
    let cond1 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| !x.is_empty());

    let data2 = vec![3, 4];
    let cond2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data2);
    })
    .when(|x: &Vec<i32>| x.len() < 10);

    let chained = cond1.and_then(cond2);

    let mut target = vec![0];
    chained.apply(&mut target);
    assert_eq!(target, vec![0, 1, 2, 3, 4]);

    // Test with one condition failing
    let data3 = vec![5, 6];
    let cond3 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data3);
    })
    .when(|x: &Vec<i32>| x.is_empty()); // This will fail

    let data4 = vec![7, 8];
    let cond4 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data4);
    })
    .when(|x: &Vec<i32>| x.len() < 10); // This will pass

    let chained2 = cond3.and_then(cond4);

    let mut target2 = vec![0];
    chained2.apply(&mut target2);
    assert_eq!(target2, vec![0, 7, 8]); // Only second mutator executes
}

#[test]
fn test_box_conditional_mutator_once_or_else() {
    // Test or_else() with condition true (when branch executes)
    let data1 = vec![1, 2, 3];
    let data2 = vec![99];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| !x.is_empty())
    .or_else(move |x: &mut Vec<i32>| {
        x.extend(data2);
    });

    let mut target = vec![0];
    mutator.apply(&mut target);
    assert_eq!(target, vec![0, 1, 2, 3]); // when branch executes

    // Test or_else() with condition false (or_else branch executes)
    let data3 = vec![4, 5];
    let data4 = vec![99];
    let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        x.extend(data3);
    })
    .when(|x: &Vec<i32>| x.is_empty())
    .or_else(move |x: &mut Vec<i32>| {
        x.extend(data4);
    });

    let mut target2 = vec![0];
    mutator2.apply(&mut target2);
    assert_eq!(target2, vec![0, 99]); // or_else branch executes
}

// ============================================================================
// Tests for closure implementations
// ============================================================================

// ============================================================================
// BoxConditionalMutatorOnce Debug/Display Tests
// ============================================================================

#[cfg(test)]
mod test_box_conditional_mutator_once_debug_display {
    use super::BoxMutatorOnce;
    use super::MutatorOnce;

    #[test]
    fn test_box_conditional_mutator_once_debug() {
        let data = vec![1, 2];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| x.extend(data));
        let conditional = mutator.when(|x: &Vec<i32>| x.len() < 5);

        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("BoxConditionalMutatorOnce"));
        assert!(debug_str.contains("BoxMutatorOnce"));
        assert!(debug_str.contains("BoxPredicate"));
    }

    #[test]
    fn test_box_conditional_mutator_once_display() {
        let data = vec![3, 4];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| x.extend(data));
        let conditional = mutator.when(|x: &Vec<i32>| !x.is_empty());

        let display_str = format!("{}", conditional);
        assert!(display_str.contains("BoxConditionalMutatorOnce"));
    }

    #[test]
    fn test_box_mutator_once_new_with_name() {
        let mutator = BoxMutatorOnce::new_with_name("test_mutator_once", |x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), Some("test_mutator_once"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_box_mutator_once_new_with_optional_name_some() {
        let mutator = BoxMutatorOnce::new_with_optional_name(|x: &mut i32| *x += 1, Some("optional_once".to_string()));
        assert_eq!(mutator.name(), Some("optional_once"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_box_mutator_once_new_with_optional_name_none() {
        let mutator = BoxMutatorOnce::new_with_optional_name(|x: &mut i32| *x += 1, None);
        assert_eq!(mutator.name(), None);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_box_mutator_once_name_and_set_name() {
        let mut mutator = BoxMutatorOnce::new(|x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), None);

        mutator.set_name("set_name_once");
        assert_eq!(mutator.name(), Some("set_name_once"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }
}
