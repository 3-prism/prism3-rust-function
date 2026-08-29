// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cmp::Ordering;

use qubit_function::BoxComparator;
use qubit_function::Comparator;

struct BoxNaturalComparator;

impl Comparator<i32> for BoxNaturalComparator {
    fn compare(&self, left: &i32, right: &i32) -> Ordering {
        left.cmp(right)
    }
}

#[test]
fn test_box_comparator_observable_behavior() {
    let type_name = std::any::type_name::<BoxComparator<i32>>();
    assert!(type_name.contains("BoxComparator"), "{type_name}");
}

/// Verifies the complete diagnostic metadata contract of `BoxComparator`.
#[test]
fn test_box_comparator_name_and_diagnostics() {
    let mut comparator =
        BoxComparator::new_with_optional_name(|left: &i32, right: &i32| left.cmp(right), Some("ascending".to_owned()));

    assert_eq!(comparator.name(), Some("ascending"));
    assert_eq!(comparator.compare(&1, &2), Ordering::Less);
    assert_eq!(format!("{comparator:?}"), "BoxComparator { name: Some(\"ascending\") }");
    assert_eq!(format!("{comparator}"), "BoxComparator(ascending)");

    comparator.set_name("natural");
    assert_eq!(comparator.name(), Some("natural"));
    comparator.clear_name();
    assert_eq!(comparator.name(), None);
    assert_eq!(format!("{comparator}"), "BoxComparator");
}

/// Verifies chainable naming for a boxed comparator.
#[test]
fn test_box_comparator_with_name() {
    let comparator =
        BoxComparator::new_with_name("original", |left: &i32, right: &i32| left.cmp(right)).with_name("ascending");

    assert_eq!(comparator.name(), Some("ascending"));
    assert_eq!(comparator.compare(&2, &1), Ordering::Greater);
}

/// Verifies that Box composition accepts any Comparator implementation.
#[test]
fn test_box_comparator_then_comparing_semantic_trait() {
    let comparator = BoxComparator::new(|_: &i32, _: &i32| Ordering::Equal).then_comparing(BoxNaturalComparator);

    assert_eq!(comparator.compare(&1, &2), Ordering::Less);
}
