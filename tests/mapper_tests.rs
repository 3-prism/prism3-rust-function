/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcMapper, ArcPredicate, BoxMapper, BoxPredicate, FnMapperOps, Mapper, Predicate, RcMapper,
    RcPredicate,
};

// ============================================================================
// BoxMapper Tests
// ============================================================================

#[test]
fn test_box_mapper_new() {
    let mut counter = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.map(10), 11);
    assert_eq!(mapper.map(10), 12);
    assert_eq!(mapper.map(10), 13);
}

#[test]
fn test_box_mapper_identity() {
    let mut identity = BoxMapper::<i32, i32>::identity();
    assert_eq!(identity.map(42), 42);
    assert_eq!(identity.map(100), 100);
}

#[test]
fn test_box_mapper_constant() {
    let mut constant = BoxMapper::constant("hello");
    assert_eq!(constant.map(1), "hello");
    assert_eq!(constant.map(2), "hello");
    assert_eq!(constant.map(3), "hello");
}

#[test]
fn test_box_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = BoxMapper::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = BoxMapper::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 24); // (10 + 2) * 2
    assert_eq!(composed.map(10), 39); // (10 + 3) * 3
}

#[test]
fn test_box_mapper_compose() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    assert_eq!(composed.map(10), 33); // (10 + 1) * 3
}

#[test]
fn test_box_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = BoxMapper::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.map(15), "High[1]: 30");
    assert_eq!(mapper.map(5), "Low[1]: 6");
    assert_eq!(mapper.map(20), "High[2]: 40");
    assert_eq!(mapper.map(3), "Low[2]: 4");
}

#[test]
fn test_box_mapper_into_box() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.map(10), 11);
    assert_eq!(boxed.map(10), 12);
}

#[test]
fn test_box_mapper_into_rc() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.map(10), 11);
    assert_eq!(rc_mapper.map(10), 12);
}

// ============================================================================
// ArcMapper Tests
// ============================================================================

#[test]
fn test_arc_mapper_new() {
    let mut counter = 0;
    let mut mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.map(10), 11);
    assert_eq!(mapper.map(10), 12);
    assert_eq!(mapper.map(10), 13);
}

#[test]
fn test_arc_mapper_identity() {
    let mut identity = ArcMapper::<i32, i32>::identity();
    assert_eq!(identity.map(42), 42);
    assert_eq!(identity.map(100), 100);
}

#[test]
fn test_arc_mapper_constant() {
    let mut constant = ArcMapper::constant("hello");
    assert_eq!(constant.map(1), "hello");
    assert_eq!(constant.map(2), "hello");
    assert_eq!(constant.map(3), "hello");
}

#[test]
fn test_arc_mapper_clone() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    assert_eq!(mapper1.map(10), 11);
    assert_eq!(mapper2.map(10), 12);
    assert_eq!(mapper1.map(10), 13);
}

#[test]
fn test_arc_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = ArcMapper::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = ArcMapper::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 24); // (10 + 2) * 2
    assert_eq!(composed.map(10), 39); // (10 + 3) * 3
}

#[test]
fn test_arc_mapper_compose() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    assert_eq!(composed.map(10), 33); // (10 + 1) * 3
}

#[test]
fn test_arc_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = ArcMapper::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.map(15), "High[1]: 30");
    assert_eq!(mapper.map(5), "Low[1]: 6");
    assert_eq!(mapper.map(20), "High[2]: 40");
    assert_eq!(mapper.map(3), "Low[2]: 4");
}

#[test]
fn test_arc_mapper_into_box() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.map(10), 11);
    assert_eq!(boxed.map(10), 12);
}

#[test]
fn test_arc_mapper_into_arc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut arc_mapper = mapper.into_arc();
    assert_eq!(arc_mapper.map(10), 11);
    assert_eq!(arc_mapper.map(10), 12);
}

#[test]
fn test_arc_mapper_into_rc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.map(10), 11);
    assert_eq!(rc_mapper.map(10), 12);
}

// ============================================================================
// RcMapper Tests
// ============================================================================

#[test]
fn test_rc_mapper_new() {
    let mut counter = 0;
    let mut mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.map(10), 11);
    assert_eq!(mapper.map(10), 12);
    assert_eq!(mapper.map(10), 13);
}

#[test]
fn test_rc_mapper_identity() {
    let mut identity = RcMapper::<i32, i32>::identity();
    assert_eq!(identity.map(42), 42);
    assert_eq!(identity.map(100), 100);
}

#[test]
fn test_rc_mapper_constant() {
    let mut constant = RcMapper::constant("hello");
    assert_eq!(constant.map(1), "hello");
    assert_eq!(constant.map(2), "hello");
    assert_eq!(constant.map(3), "hello");
}

#[test]
fn test_rc_mapper_clone() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    assert_eq!(mapper1.map(10), 11);
    assert_eq!(mapper2.map(10), 12);
    assert_eq!(mapper1.map(10), 13);
}

#[test]
fn test_rc_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = RcMapper::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = RcMapper::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 24); // (10 + 2) * 2
    assert_eq!(composed.map(10), 39); // (10 + 3) * 3
}

#[test]
fn test_rc_mapper_compose() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 22); // (10 + 1) * 2
    assert_eq!(composed.map(10), 33); // (10 + 1) * 3
}

#[test]
fn test_rc_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = RcMapper::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.map(15), "High[1]: 30");
    assert_eq!(mapper.map(5), "Low[1]: 6");
    assert_eq!(mapper.map(20), "High[2]: 40");
    assert_eq!(mapper.map(3), "Low[2]: 4");
}

#[test]
fn test_rc_mapper_into_box() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.map(10), 11);
    assert_eq!(boxed.map(10), 12);
}

#[test]
fn test_rc_mapper_into_rc() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.map(10), 11);
    assert_eq!(rc_mapper.map(10), 12);
}

// ============================================================================
// Closure Mapper Tests
// ============================================================================

#[test]
fn test_closure_as_mapper() {
    let mut counter = 0;
    let mut mapper = |x: i32| {
        counter += 1;
        x + counter
    };

    assert_eq!(mapper.map(10), 11);
    assert_eq!(mapper.map(10), 12);
    assert_eq!(mapper.map(10), 13);
}

#[test]
fn test_closure_into_box() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.map(10), 11);
    assert_eq!(boxed.map(10), 12);
}

#[test]
fn test_closure_into_rc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.map(10), 11);
    assert_eq!(rc_mapper.map(10), 12);
}

#[test]
fn test_closure_into_arc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut arc_mapper = mapper.into_arc();
    assert_eq!(arc_mapper.map(10), 11);
    assert_eq!(arc_mapper.map(10), 12);
}

// ============================================================================
// FnMapperOps Tests
// ============================================================================

#[test]
fn test_fn_mapper_ops_and_then() {
    let mut counter1 = 0;
    let mapper1 = move |x: i32| {
        counter1 += 1;
        x + counter1
    };

    let mut counter2 = 0;
    let mapper2 = move |x: i32| {
        counter2 += 1;
        x * counter2
    };

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 24); // (10 + 2) * 2
}

#[test]
fn test_fn_mapper_ops_compose() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x * counter
    };

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.map(10), 11); // (10 + 1) * 1
    assert_eq!(composed.map(10), 22); // (10 + 1) * 2
}

#[test]
fn test_fn_mapper_ops_when() {
    let mut mapper = (|x: i32| x * 2).when(|x: &i32| *x > 0).or_else(|x: i32| -x);

    assert_eq!(mapper.map(5), 10);
    assert_eq!(mapper.map(-5), 5);
}

// ============================================================================
// Conditional Mapper Tests
// ============================================================================

#[test]
fn test_box_conditional_mapper_with_predicate() {
    let predicate = BoxPredicate::new(|x: &i32| *x >= 10);

    let mut mapper = BoxMapper::new(|x: i32| x * 2)
        .when(predicate)
        .or_else(|x| x + 1);

    assert_eq!(mapper.map(15), 30);
    assert_eq!(mapper.map(5), 6);
}

#[test]
fn test_arc_conditional_mapper_clone() {
    let conditional = ArcMapper::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the ArcConditionalMapper before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.map(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.map(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.map(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.map(-5), 95); // Condition not satisfied: -5 + 100
}

#[test]
fn test_rc_conditional_mapper_clone() {
    let conditional = RcMapper::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the RcConditionalMapper before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.map(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.map(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.map(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.map(-5), 95); // Condition not satisfied: -5 + 100
}

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[test]
fn test_complex_pipeline() {
    let mut counter1 = 0;
    let step1 = BoxMapper::new(move |x: i32| {
        counter1 += 1;
        format!("Step1[{}]: {}", counter1, x)
    });

    let mut counter2 = 0;
    let step2 = BoxMapper::new(move |s: String| {
        counter2 += 1;
        format!("{} -> Step2[{}]", s, counter2)
    });

    let mut counter3 = 0;
    let step3 = BoxMapper::new(move |s: String| {
        counter3 += 1;
        format!("{} -> Step3[{}]", s, counter3)
    });

    let mut pipeline = step1.and_then(step2).and_then(step3);

    assert_eq!(pipeline.map(10), "Step1[1]: 10 -> Step2[1] -> Step3[1]");
    assert_eq!(pipeline.map(20), "Step1[2]: 20 -> Step2[2] -> Step3[2]");
}

#[test]
fn test_nested_conditional() {
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut error_count = 0;

    let mut mapper = BoxMapper::new(move |x: i32| {
        valid_count += 1;
        format!("Valid[{}]: {}", valid_count, x * 2)
    })
    .when(|x: &i32| *x > 0)
    .or_else(move |x: i32| {
        let mut sub_mapper = BoxMapper::new(move |x: i32| {
            invalid_count += 1;
            format!("Invalid[{}]: {}", invalid_count, x + 100)
        })
        .when(move |x: &i32| *x < 0)
        .or_else(move |x: i32| {
            error_count += 1;
            format!("Error[{}]: {}", error_count, x)
        });
        sub_mapper.map(x)
    });

    assert_eq!(mapper.map(5), "Valid[1]: 10");
    assert_eq!(mapper.map(-5), "Invalid[1]: 95");
    assert_eq!(mapper.map(0), "Error[1]: 0");
    assert_eq!(mapper.map(10), "Valid[2]: 20");
}

// ============================================================================
// State Modification Tests
// ============================================================================

#[test]
fn test_stateful_counting() {
    let mut count = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        count += 1;
        (x, count)
    });

    assert_eq!(mapper.map(100), (100, 1));
    assert_eq!(mapper.map(200), (200, 2));
    assert_eq!(mapper.map(300), (300, 3));
}

#[test]
fn test_stateful_accumulation() {
    let mut sum = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        sum += x;
        sum
    });

    assert_eq!(mapper.map(10), 10);
    assert_eq!(mapper.map(20), 30);
    assert_eq!(mapper.map(30), 60);
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[test]
fn test_different_types() {
    let mut counter = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        format!("Item #{}: {}", counter, x)
    });

    assert_eq!(mapper.map(100), "Item #1: 100");
    assert_eq!(mapper.map(200), "Item #2: 200");
}

#[test]
fn test_string_to_length() {
    let mut total_length = 0;
    let mut mapper = BoxMapper::new(move |s: String| {
        total_length += s.len();
        total_length
    });

    assert_eq!(mapper.map("hello".to_string()), 5);
    assert_eq!(mapper.map("world".to_string()), 10);
    assert_eq!(mapper.map("!".to_string()), 11);
}

// ============================================================================
// Predicate Integration Tests
// ============================================================================

#[test]
fn test_with_arc_predicate() {
    let predicate = ArcPredicate::new(|x: &i32| *x > 0);

    let mut mapper = ArcMapper::new(|x: i32| x * 2)
        .when(predicate.clone())
        .or_else(|x: i32| -x);

    assert_eq!(mapper.map(5), 10);
    assert_eq!(mapper.map(-5), 5);

    // Predicate still usable
    assert!(predicate.test(&10));
    assert!(!predicate.test(&-10));
}

#[test]
fn test_with_rc_predicate() {
    let predicate = RcPredicate::new(|x: &i32| *x > 0);

    let mut mapper = RcMapper::new(|x: i32| x * 2)
        .when(predicate.clone())
        .or_else(|x: i32| -x);

    assert_eq!(mapper.map(5), 10);
    assert_eq!(mapper.map(-5), 5);

    // Predicate still usable
    assert!(predicate.test(&10));
    assert!(!predicate.test(&-10));
}
