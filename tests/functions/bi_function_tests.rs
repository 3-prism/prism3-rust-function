// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for BiFunction trait and its implementations

use qubit_function::ArcBiFunction;
use qubit_function::ArcBiPredicate;
use qubit_function::BiFunction;
use qubit_function::BiFunctionOnce;
use qubit_function::BoxBiFunction;
use qubit_function::RcBiFunction;
use qubit_function::RcBiPredicate;

// ============================================================================
// BiFunction Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_bi_function_trait_apply() {
    // Test that BiFunction trait's apply method works correctly
    let add = |x: &i32, y: &i32| *x + *y;
    assert_eq!(add.apply(&20, &22), 42);
    assert_eq!(add.apply(&0, &0), 0);
    assert_eq!(add.apply(&-10, &5), -5);
}

// ============================================================================
// Custom BiFunction Implementation Tests - Test Trait Default Methods
// ============================================================================

// ============================================================================
// BoxBiFunction Tests
// ============================================================================

#[test]
fn test_box_bi_function_new() {
    let add = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    assert_eq!(add.apply(&10, &15), 25);
}

#[test]
fn test_box_bi_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiFunction<&'a str, i32, usize> =
            BoxBiFunction::new(|x: &&'a str, y: &i32| x.len() + (*y as usize));
        func.apply(&value, &3)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_function_new_allows_non_static_u() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiFunction<i32, &'a str, usize> =
            BoxBiFunction::new(|x: &i32, y: &&'a str| (*x as usize) + y.len());
        func.apply(&3, &value)
    }

    let text = String::from("world");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_function_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: BoxBiFunction<&'a str, i32, &'a str> = BoxBiFunction::new(|x: &&'a str, _y: &i32| *x);
        func.apply(&value, &0)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_box_bi_function_constant() {
    let constant = BoxBiFunction::constant(42);
    assert_eq!(constant.apply(&1, &2), 42);
    assert_eq!(constant.apply(&100, &200), 42);
}

#[test]
fn test_box_bi_function_debug_display() {
    let func = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let debug_str = format!("{:?}", func);
    assert!(debug_str.contains("BoxBiFunction"));
    let display_str = format!("{}", func);
    assert!(display_str.starts_with("BoxBiFunction"));
}

#[test]
fn test_box_bi_function_and_then() {
    let add = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let chained = add.and_then(|value: &i32| value * 2);

    assert_eq!(chained.apply(&2, &3), 10);
}

// ============================================================================
// RcBiFunction Tests
// ============================================================================

#[test]
fn test_rc_bi_function_new() {
    let multiply = RcBiFunction::new(|x: &i32, y: &i32| *x * *y);
    assert_eq!(multiply.apply(&6, &7), 42);
}

#[test]
fn test_rc_bi_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: RcBiFunction<&'a str, i32, usize> = RcBiFunction::new(|x: &&'a str, y: &i32| x.len() + (*y as usize));
        func.apply(&value, &3)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_rc_bi_function_new_allows_non_static_u() {
    fn run<'a>(value: &'a str) -> usize {
        let func: RcBiFunction<i32, &'a str, usize> = RcBiFunction::new(|x: &i32, y: &&'a str| (*x as usize) + y.len());
        func.apply(&3, &value)
    }

    let text = String::from("world");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_rc_bi_function_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: RcBiFunction<&'a str, i32, &'a str> = RcBiFunction::new(|x: &&'a str, _y: &i32| *x);
        func.apply(&value, &0)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_rc_bi_function_constant() {
    let constant = RcBiFunction::constant(100);
    assert_eq!(constant.apply(&1, &2), 100);
}

#[test]
fn test_rc_bi_function_clone() {
    let original = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let cloned = original.clone();
    assert_eq!(original.apply(&10, &20), 30);
    assert_eq!(cloned.apply(&10, &20), 30);
}

#[test]
fn test_rc_bi_function_debug_display() {
    let func = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let debug_str = format!("{:?}", func);
    assert!(debug_str.contains("RcBiFunction"));
    let display_str = format!("{}", func);
    assert!(display_str.starts_with("RcBiFunction"));
}

#[test]
fn test_rc_bi_function_and_then() {
    use qubit_function::RcFunction;

    let add = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply_by_two = RcFunction::new(|x: &i32| *x * 2);

    let chained = add.and_then(multiply_by_two);
    assert_eq!(chained.apply(&2, &3), 10); // (2+3) * 2 = 10
}

// ============================================================================
// ArcBiFunction Tests
// ============================================================================

#[test]
fn test_arc_bi_function_new() {
    let divide = ArcBiFunction::new(|x: &i32, y: &i32| *x / *y);
    assert_eq!(divide.apply(&42, &2), 21);
}

#[test]
fn test_arc_bi_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: ArcBiFunction<&'a str, i32, usize> =
            ArcBiFunction::new(|x: &&'a str, y: &i32| x.len() + (*y as usize));
        func.apply(&value, &3)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_arc_bi_function_new_allows_non_static_u() {
    fn run<'a>(value: &'a str) -> usize {
        let func: ArcBiFunction<i32, &'a str, usize> =
            ArcBiFunction::new(|x: &i32, y: &&'a str| (*x as usize) + y.len());
        func.apply(&3, &value)
    }

    let text = String::from("world");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_arc_bi_function_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: ArcBiFunction<&'a str, i32, &'a str> = ArcBiFunction::new(|x: &&'a str, _y: &i32| *x);
        func.apply(&value, &0)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_arc_bi_function_constant() {
    let constant = ArcBiFunction::constant("hello".to_string());
    assert_eq!(constant.apply(&1, &2), "hello");
}

#[test]
fn test_arc_bi_function_clone() {
    let original = ArcBiFunction::new(|x: &i32, y: &i32| *x - *y);
    let cloned = original.clone();
    assert_eq!(original.apply(&50, &8), 42);
    assert_eq!(cloned.apply(&50, &8), 42);
}

#[test]
fn test_arc_bi_function_debug_display() {
    let func = ArcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let debug_str = format!("{:?}", func);
    assert!(debug_str.contains("ArcBiFunction"));
    let display_str = format!("{}", func);
    assert!(display_str.starts_with("ArcBiFunction"));
}

// ============================================================================
// Conversion Tests
// ============================================================================

// ============================================================================
// BiFunction Composition Tests
// ============================================================================

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_bi_function_with_complex_types() {
    let concat = |s1: &&str, s2: &&str| format!("{} {}", *s1, *s2);
    let boxed = BoxBiFunction::new(concat);

    assert_eq!(boxed.apply(&"Hello", &"World"), "Hello World");
}

#[test]
fn test_bi_function_with_option_types() {
    let combine_options = |opt1: &Option<i32>, opt2: &Option<i32>| match (opt1, opt2) {
        (Some(a), Some(b)) => Some(a + b),
        _ => None,
    };

    let func = RcBiFunction::new(combine_options);

    assert_eq!(func.apply(&Some(10), &Some(20)), Some(30));
    assert_eq!(func.apply(&Some(10), &None), None);
    assert_eq!(func.apply(&None, &Some(20)), None);
}

#[test]
fn test_bi_function_with_result_types() {
    let safe_divide = |a: &i32, b: &i32| {
        if *b == 0 { Err("Division by zero") } else { Ok(*a / *b) }
    };

    let func = ArcBiFunction::new(safe_divide);

    assert_eq!(func.apply(&10, &2), Ok(5));
    assert_eq!(func.apply(&10, &0), Err("Division by zero"));
}

// ============================================================================
// BoxBiFunction Extended Tests
// ============================================================================

#[test]
fn test_box_bi_function_new_with_name() {
    let func = BoxBiFunction::new_with_name("adder", |x: &i32, y: &i32| *x + *y);
    assert_eq!(func.name(), Some("adder"));
    assert_eq!(func.apply(&10, &20), 30);
}

#[test]
fn test_box_bi_function_new_with_optional_name() {
    let func1 = BoxBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x + *y, Some("named".to_string()));
    let func2 = BoxBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x + *y, None);

    assert_eq!(func1.name(), Some("named"));
    assert_eq!(func2.name(), None);
    assert_eq!(func1.apply(&5, &7), 12);
    assert_eq!(func2.apply(&5, &7), 12);
}

#[test]
fn test_box_bi_function_name_and_set_name() {
    let mut func = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    assert_eq!(func.name(), None);

    func.set_name("test_func");
    assert_eq!(func.name(), Some("test_func"));

    func.set_name("updated_name");
    assert_eq!(func.name(), Some("updated_name"));
}

// ============================================================================
// RcBiFunction Extended Tests
// ============================================================================

#[test]
fn test_rc_bi_function_new_with_name() {
    let func = RcBiFunction::new_with_name("multiplier", |x: &i32, y: &i32| *x * *y);
    assert_eq!(func.name(), Some("multiplier"));
    assert_eq!(func.apply(&6, &7), 42);
}

#[test]
fn test_rc_bi_function_new_with_optional_name() {
    let func1 = RcBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x * *y, Some("named".to_string()));
    let func2 = RcBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x * *y, None);

    assert_eq!(func1.name(), Some("named"));
    assert_eq!(func2.name(), None);
    assert_eq!(func1.apply(&6, &7), 42);
    assert_eq!(func2.apply(&6, &7), 42);
}

#[test]
fn test_rc_bi_function_name_and_set_name() {
    let mut func = RcBiFunction::new(|x: &i32, y: &i32| *x * *y);
    assert_eq!(func.name(), None);

    func.set_name("test_func");
    assert_eq!(func.name(), Some("test_func"));
}

// RcBiFunction cannot be converted to ArcBiFunction (not Send + Sync)

// RcBiFunction cannot be converted to ArcBiFunction (not Send + Sync)

// ============================================================================
// ArcBiFunction Extended Tests
// ============================================================================

#[test]
fn test_arc_bi_function_new_with_name() {
    let func = ArcBiFunction::new_with_name("divider", |x: &i32, y: &i32| *x / *y);
    assert_eq!(func.name(), Some("divider"));
    assert_eq!(func.apply(&42, &2), 21);
}

#[test]
fn test_arc_bi_function_new_with_optional_name() {
    let func1 = ArcBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x / *y, Some("named".to_string()));
    let func2 = ArcBiFunction::new_with_optional_name(|x: &i32, y: &i32| *x / *y, None);

    assert_eq!(func1.name(), Some("named"));
    assert_eq!(func2.name(), None);
    assert_eq!(func1.apply(&42, &2), 21);
    assert_eq!(func2.apply(&42, &2), 21);
}

#[test]
fn test_arc_bi_function_name_and_set_name() {
    let mut func = ArcBiFunction::new(|x: &i32, y: &i32| *x - *y);
    assert_eq!(func.name(), None);

    func.set_name("test_func");
    assert_eq!(func.name(), Some("test_func"));
}

// ============================================================================
// Conditional BiFunction Tests
// ============================================================================

#[test]
fn test_box_conditional_bi_function_when_or_else() {
    let add = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply = BoxBiFunction::new(|x: &i32, y: &i32| *x * *y);

    let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
    assert_eq!(conditional.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(conditional.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12
}

#[test]
fn test_rc_conditional_bi_function_when_or_else() {
    let add = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply = RcBiFunction::new(|x: &i32, y: &i32| *x * *y);

    let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
    assert_eq!(conditional.apply(&3, &4), 7); // when branch
    assert_eq!(conditional.apply(&-3, &4), -12); // or_else branch
}

#[test]
fn test_arc_conditional_bi_function_when_or_else() {
    let add = ArcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply = ArcBiFunction::new(|x: &i32, y: &i32| *x * *y);

    let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
    assert_eq!(conditional.apply(&3, &4), 7); // when branch
    assert_eq!(conditional.apply(&-3, &4), -12); // or_else branch
}

#[test]
fn test_conditional_bi_function_with_complex_predicates() {
    let add = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let subtract = BoxBiFunction::new(|x: &i32, y: &i32| *x - *y);

    let conditional = add.when(|x: &i32, y: &i32| *x >= *y).or_else(subtract);
    assert_eq!(conditional.apply(&5, &3), 8); // when branch: 5 >= 3, so 5 + 3 = 8
    assert_eq!(conditional.apply(&3, &5), -2); // or_else branch: 3 < 5, so 3 - 5 = -2
}

#[test]
fn test_conditional_bi_function_display_debug() {
    let add = BoxBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);

    let display = format!("{}", conditional);
    assert!(display.contains("BoxConditionalBiFunction"));

    let debug = format!("{:?}", conditional);
    assert!(debug.contains("BoxConditionalBiFunction"));
}
