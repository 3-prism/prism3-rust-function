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

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_box_mapper_identity() {
    let mut identity = BoxMapper::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_box_mapper_constant() {
    let mut constant = BoxMapper::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
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
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_box_mapper_compose() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    assert_eq!(composed.apply(10), 33); // (10 + 1) * 3
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

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

#[test]
fn test_box_mapper_into_box() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.apply(10), 11);
    assert_eq!(boxed.apply(10), 12);
}

#[test]
fn test_mapper_to_box_rc_arc_fn_non_consuming() {
    // Test non-consuming adapters `to_box`, `to_rc`, `to_arc`, `to_fn`
    // using a Cloneable custom mapper.
    #[derive(Clone)]
    struct CloneMapper {
        counter: i32,
    }

    impl Mapper<i32, i32> for CloneMapper {
        fn apply(&mut self, input: i32) -> i32 {
            self.counter += 1;
            input + self.counter
        }
    }

    let mapper = CloneMapper { counter: 0 };

    let mut b = mapper.to_box();
        assert_eq!(b.apply(10), 11);
        assert_eq!(b.apply(10), 12);

    let rc = mapper.to_rc();
    let mut r1 = rc.clone();
    let mut r2 = rc.clone();
        assert_eq!(r1.apply(10), 11);
        assert_eq!(r2.apply(10), 12);

    // to_arc requires Send+Sync. Make a trivially Send+Sync cloneable
    #[derive(Clone)]
    struct SCloneMapper {
        counter: i32,
    }

    impl Mapper<i32, i32> for SCloneMapper {
        fn apply(&mut self, input: i32) -> i32 {
            self.counter += 1;
            input * self.counter
        }
    }

    unsafe impl Send for SCloneMapper {}
    unsafe impl Sync for SCloneMapper {}

    let sm = SCloneMapper { counter: 0 };
    let mut a = sm.to_arc();
        assert_eq!(a.apply(3), 3);
        assert_eq!(a.apply(3), 6);

    let mut f = mapper.to_fn();
    assert_eq!(f(5), 6);
    assert_eq!(f(5), 7);
}

#[test]
fn test_box_mapper_into_rc() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.apply(10), 11);
    assert_eq!(rc_mapper.apply(10), 12);
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

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_arc_mapper_identity() {
    let mut identity = ArcMapper::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_arc_mapper_constant() {
    let mut constant = ArcMapper::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
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

    assert_eq!(mapper1.apply(10), 11);
    assert_eq!(mapper2.apply(10), 12);
    assert_eq!(mapper1.apply(10), 13);
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
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_arc_mapper_compose() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    assert_eq!(composed.apply(10), 33); // (10 + 1) * 3
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

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

#[test]
fn test_arc_mapper_into_box() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.apply(10), 11);
    assert_eq!(boxed.apply(10), 12);
}

#[test]
fn test_arc_mapper_into_arc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut arc_mapper = mapper.into_arc();
    assert_eq!(arc_mapper.apply(10), 11);
    assert_eq!(arc_mapper.apply(10), 12);
}

#[test]
fn test_arc_mapper_into_rc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.apply(10), 11);
    assert_eq!(rc_mapper.apply(10), 12);
}

#[test]
fn test_arc_mapper_to_box() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    // 非消费性转换：原始 mapper 仍然可用
    let mut boxed1 = mapper.to_box();
    assert_eq!(boxed1.apply(10), 11); // 10 + 1
    assert_eq!(boxed1.apply(10), 12); // 10 + 2

    // 可以多次调用 to_box()，每次都共享同一个底层状态
    let mut boxed2 = mapper.to_box();
    assert_eq!(boxed2.apply(10), 13); // 10 + 3 (共享状态)

    // 原始 mapper 也共享状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(10), 14); // 10 + 4
}

#[test]
fn test_arc_mapper_to_rc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    // 非消费性转换
    let rc_mapper = mapper.to_rc();
    let mut rc1 = rc_mapper.clone();
    let mut rc2 = rc_mapper.clone();

    // 共享状态
    assert_eq!(rc1.apply(10), 10); // 10 * 1
    assert_eq!(rc2.apply(10), 20); // 10 * 2
    assert_eq!(rc1.apply(10), 30); // 10 * 3

    // 原始 mapper 也共享同一个状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(10), 40); // 10 * 4
}

#[test]
fn test_arc_mapper_to_arc() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x - counter
    });

    // 非消费性转换
    let arc_mapper = mapper.to_arc();
    let mut arc1 = arc_mapper.clone();
    let mut arc2 = arc_mapper.clone();

    // 共享状态
    assert_eq!(arc1.apply(10), 9);  // 10 - 1
    assert_eq!(arc2.apply(10), 8);  // 10 - 2
    assert_eq!(arc1.apply(10), 7);  // 10 - 3

    // 原始 mapper 也共享同一个状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(10), 6); // 10 - 4
}

#[test]
fn test_arc_mapper_to_fn() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x + counter * 10
    });

    // 非消费性转换
    let mut closure1 = mapper.to_fn();
    assert_eq!(closure1(5), 15);  // 5 + 1 * 10
    assert_eq!(closure1(5), 25);  // 5 + 2 * 10

    // 可以多次调用 to_fn()，每次都共享同一个底层状态
    let mut closure2 = mapper.to_fn();
    assert_eq!(closure2(5), 35);  // 5 + 3 * 10 (共享状态)

    // 原始 mapper 也共享状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(5), 45); // 5 + 4 * 10
}

#[test]
fn test_arc_mapper_to_box_with_string() {
    let mut count = 0;
    let mapper = ArcMapper::new(move |x: String| {
        count += 1;
        format!("[{}] {}", count, x)
    });

    let mut boxed1 = mapper.to_box();
    assert_eq!(boxed1.apply("hello".to_string()), "[1] hello");
    assert_eq!(boxed1.apply("world".to_string()), "[2] world");

    let mut boxed2 = mapper.to_box();
    assert_eq!(boxed2.apply("rust".to_string()), "[3] rust"); // 共享状态
}

#[test]
fn test_arc_mapper_to_rc_shared_state() {
    let mut sum = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        sum += x;
        sum
    });

    let rc_mapper = mapper.to_rc();
    let mut rc1 = rc_mapper.clone();
    let mut rc2 = rc_mapper.clone();

    // 验证共享状态
    assert_eq!(rc1.apply(10), 10);
    assert_eq!(rc2.apply(20), 30); // 共享同一个 sum
    assert_eq!(rc1.apply(30), 60); // 继续累加

    // 原始 mapper 也共享状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(40), 100); // 60 + 40
}

#[test]
fn test_arc_mapper_to_arc_multiple_clones() {
    let mut product = 1;
    let mapper = ArcMapper::new(move |x: i32| {
        product *= x;
        product
    });

    let arc_mapper = mapper.to_arc();
    let mut arc1 = arc_mapper.clone();
    let mut arc2 = arc_mapper.clone();

    // 验证共享状态
    assert_eq!(arc1.apply(2), 2);   // 1 * 2
    assert_eq!(arc2.apply(3), 6);   // 2 * 3
    assert_eq!(arc1.apply(4), 24);  // 6 * 4

    // 原始 mapper 也共享状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(5), 120); // 24 * 5
}

#[test]
fn test_arc_mapper_to_fn_complex_type() {
    let mut history = Vec::new();
    let mapper = ArcMapper::new(move |x: i32| {
        history.push(x);
        (x, history.len())
    });

    let mut fn1 = mapper.to_fn();
    assert_eq!(fn1(10), (10, 1));
    assert_eq!(fn1(20), (20, 2));

    let mut fn2 = mapper.to_fn();
    assert_eq!(fn2(30), (30, 3)); // 共享状态

    // 原始 mapper 也共享状态
    let mut mapper_clone = mapper.clone();
    assert_eq!(mapper_clone.apply(40), (40, 4));
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

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_rc_mapper_identity() {
    let mut identity = RcMapper::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_rc_mapper_constant() {
    let mut constant = RcMapper::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
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

    assert_eq!(mapper1.apply(10), 11);
    assert_eq!(mapper2.apply(10), 12);
    assert_eq!(mapper1.apply(10), 13);
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
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_rc_mapper_compose() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
    assert_eq!(composed.apply(10), 33); // (10 + 1) * 3
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

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

#[test]
fn test_rc_mapper_into_box() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.apply(10), 11);
    assert_eq!(boxed.apply(10), 12);
}

#[test]
fn test_rc_mapper_into_rc() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.apply(10), 11);
    assert_eq!(rc_mapper.apply(10), 12);
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

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_closure_into_box() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut boxed = mapper.into_box();
    assert_eq!(boxed.apply(10), 11);
    assert_eq!(boxed.apply(10), 12);
}

#[test]
fn test_closure_into_rc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut rc_mapper = mapper.into_rc();
    assert_eq!(rc_mapper.apply(10), 11);
    assert_eq!(rc_mapper.apply(10), 12);
}

#[test]
fn test_closure_into_arc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    let mut arc_mapper = mapper.into_arc();
    assert_eq!(arc_mapper.apply(10), 11);
    assert_eq!(arc_mapper.apply(10), 12);
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
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
}

#[test]
fn test_fn_mapper_ops_compose() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x * counter
    };

    let mut composed = mapper.compose(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 22); // (10 + 1) * 2
}

#[test]
fn test_fn_mapper_ops_when() {
    let mut mapper = (|x: i32| x * 2).when(|x: &i32| *x > 0).or_else(|x: i32| -x);

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);
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

    assert_eq!(mapper.apply(15), 30);
    assert_eq!(mapper.apply(5), 6);
}

#[test]
fn test_arc_conditional_mapper_clone() {
    let conditional = ArcMapper::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the ArcConditionalMapper before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.apply(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.apply(-5), 95); // Condition not satisfied: -5 + 100
}

#[test]
fn test_rc_conditional_mapper_clone() {
    let conditional = RcMapper::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the RcConditionalMapper before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.apply(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.apply(-5), 95); // Condition not satisfied: -5 + 100
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

    assert_eq!(pipeline.apply(10), "Step1[1]: 10 -> Step2[1] -> Step3[1]");
    assert_eq!(pipeline.apply(20), "Step1[2]: 20 -> Step2[2] -> Step3[2]");
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
        sub_mapper.apply(x)
    });

    assert_eq!(mapper.apply(5), "Valid[1]: 10");
    assert_eq!(mapper.apply(-5), "Invalid[1]: 95");
    assert_eq!(mapper.apply(0), "Error[1]: 0");
    assert_eq!(mapper.apply(10), "Valid[2]: 20");
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

    assert_eq!(mapper.apply(100), (100, 1));
    assert_eq!(mapper.apply(200), (200, 2));
    assert_eq!(mapper.apply(300), (300, 3));
}

#[test]
fn test_stateful_accumulation() {
    let mut sum = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        sum += x;
        sum
    });

    assert_eq!(mapper.apply(10), 10);
    assert_eq!(mapper.apply(20), 30);
    assert_eq!(mapper.apply(30), 60);
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

    assert_eq!(mapper.apply(100), "Item #1: 100");
    assert_eq!(mapper.apply(200), "Item #2: 200");
}

#[test]
fn test_string_to_length() {
    let mut total_length = 0;
    let mut mapper = BoxMapper::new(move |s: String| {
        total_length += s.len();
        total_length
    });

    assert_eq!(mapper.apply("hello".to_string()), 5);
    assert_eq!(mapper.apply("world".to_string()), 10);
    assert_eq!(mapper.apply("!".to_string()), 11);
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

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);

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

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);

    // Predicate still usable
    assert!(predicate.test(&10));
    assert!(!predicate.test(&-10));
}

// ============================================================================
// Custom Mapper Default Implementation Tests
// ============================================================================

/// 自定义 Mapper 结构，用于测试默认的 into_xxx() 方法
struct CustomMapper {
    multiplier: i32,
}

impl Mapper<i32, i32> for CustomMapper {
    fn apply(&mut self, input: i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

/// 自定义线程安全的 Mapper 结构
struct CustomSendMapper {
    multiplier: i32,
}

impl Mapper<i32, i32> for CustomSendMapper {
    fn apply(&mut self, input: i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

// 为 CustomSendMapper 实现 Send，使其可以转换为 ArcMapper
unsafe impl Send for CustomSendMapper {}

#[test]
fn test_custom_mapper_into_box() {
    let mapper = CustomMapper { multiplier: 0 };
    let mut boxed = mapper.into_box();

    assert_eq!(boxed.apply(10), 10); // 10 * 1
    assert_eq!(boxed.apply(10), 20); // 10 * 2
    assert_eq!(boxed.apply(10), 30); // 10 * 3
}

#[test]
fn test_custom_mapper_into_rc() {
    let mapper = CustomMapper { multiplier: 0 };
    let mut rc_mapper = mapper.into_rc();

    assert_eq!(rc_mapper.apply(10), 10); // 10 * 1
    assert_eq!(rc_mapper.apply(10), 20); // 10 * 2
    assert_eq!(rc_mapper.apply(10), 30); // 10 * 3
}

#[test]
fn test_custom_mapper_into_rc_clone() {
    let mapper = CustomMapper { multiplier: 0 };
    let rc_mapper = mapper.into_rc();

    let mut mapper1 = rc_mapper.clone();
    let mut mapper2 = rc_mapper.clone();

    // 共享同一个状态
    assert_eq!(mapper1.apply(10), 10); // 10 * 1
    assert_eq!(mapper2.apply(10), 20); // 10 * 2
    assert_eq!(mapper1.apply(10), 30); // 10 * 3
}

#[test]
fn test_custom_send_mapper_into_arc() {
    let mapper = CustomSendMapper { multiplier: 0 };
    let mut arc_mapper = mapper.into_arc();

    assert_eq!(arc_mapper.apply(10), 10); // 10 * 1
    assert_eq!(arc_mapper.apply(10), 20); // 10 * 2
    assert_eq!(arc_mapper.apply(10), 30); // 10 * 3
}

#[test]
fn test_custom_send_mapper_into_arc_clone() {
    let mapper = CustomSendMapper { multiplier: 0 };
    let arc_mapper = mapper.into_arc();

    let mut mapper1 = arc_mapper.clone();
    let mut mapper2 = arc_mapper.clone();

    // 共享同一个状态
    assert_eq!(mapper1.apply(10), 10); // 10 * 1
    assert_eq!(mapper2.apply(10), 20); // 10 * 2
    assert_eq!(mapper1.apply(10), 30); // 10 * 3
}

#[test]
fn test_custom_mapper_composition() {
    let mapper1 = CustomMapper { multiplier: 0 };
    let boxed1 = mapper1.into_box();

    let mapper2 = CustomMapper { multiplier: 10 };
    let boxed2 = mapper2.into_box();

    let mut composed = boxed1.and_then(boxed2);

    // (10 * 1) = 10, 然后 10 * 11 = 110
    assert_eq!(composed.apply(10), 110);
    // (10 * 2) = 20, 然后 20 * 12 = 240
    assert_eq!(composed.apply(10), 240);
}

#[test]
fn test_custom_mapper_conditional() {
    let mapper1 = CustomMapper { multiplier: 1 };
    let boxed1 = mapper1.into_box();

    let mapper2 = CustomMapper { multiplier: 100 };
    let boxed2 = mapper2.into_box();

    let mut conditional = boxed1.when(|x: &i32| *x > 10).or_else(boxed2);

    // 15 > 10, 使用 mapper1: 15 * 2 = 30
    assert_eq!(conditional.apply(15), 30);
    // 5 <= 10, 使用 mapper2: 5 * 101 = 505
    assert_eq!(conditional.apply(5), 505);
    // 20 > 10, 使用 mapper1: 20 * 3 = 60
    assert_eq!(conditional.apply(20), 60);
}

/// 测试自定义 Mapper 与字符串类型
struct StringLengthMapper {
    total_length: usize,
}

impl Mapper<String, String> for StringLengthMapper {
    fn apply(&mut self, input: String) -> String {
        self.total_length += input.len();
        format!("[{}] {}", self.total_length, input)
    }
}

#[test]
fn test_custom_string_mapper_into_box() {
    let mapper = StringLengthMapper { total_length: 0 };
    let mut boxed = mapper.into_box();

    assert_eq!(boxed.apply("hello".to_string()), "[5] hello");
    assert_eq!(boxed.apply("world".to_string()), "[10] world");
    assert_eq!(boxed.apply("!".to_string()), "[11] !");
}

#[test]
fn test_custom_string_mapper_into_rc() {
    let mapper = StringLengthMapper { total_length: 0 };
    let mut rc_mapper = mapper.into_rc();

    assert_eq!(rc_mapper.apply("hello".to_string()), "[5] hello");
    assert_eq!(rc_mapper.apply("world".to_string()), "[10] world");
    assert_eq!(rc_mapper.apply("!".to_string()), "[11] !");
}

/// 测试带复杂状态的自定义 Mapper
struct StatefulMapper {
    count: i32,
    sum: i32,
    history: Vec<i32>,
}

impl Mapper<i32, (i32, i32, usize)> for StatefulMapper {
    fn apply(&mut self, input: i32) -> (i32, i32, usize) {
        self.count += 1;
        self.sum += input;
        self.history.push(input);
        (self.count, self.sum, self.history.len())
    }
}

#[test]
fn test_stateful_mapper_into_box() {
    let mapper = StatefulMapper {
        count: 0,
        sum: 0,
        history: Vec::new(),
    };
    let mut boxed = mapper.into_box();

    assert_eq!(boxed.apply(10), (1, 10, 1));
    assert_eq!(boxed.apply(20), (2, 30, 2));
    assert_eq!(boxed.apply(30), (3, 60, 3));
}

#[test]
fn test_stateful_mapper_into_rc() {
    let mapper = StatefulMapper {
        count: 0,
        sum: 0,
        history: Vec::new(),
    };
    let rc_mapper = mapper.into_rc();

    let mut mapper1 = rc_mapper.clone();
    let mut mapper2 = rc_mapper;

    // 共享同一个状态
    assert_eq!(mapper1.apply(10), (1, 10, 1));
    assert_eq!(mapper2.apply(20), (2, 30, 2));
    assert_eq!(mapper1.apply(30), (3, 60, 3));
}

// ============================================================================
// into_fn Tests
// ============================================================================

#[test]
fn test_box_mapper_into_fn() {
    let mut counter = 0;
    let mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut closure = mapper.into_fn();
    assert_eq!(closure(10), 11); // 10 + 1
    assert_eq!(closure(10), 12); // 10 + 2
    assert_eq!(closure(10), 13); // 10 + 3
}

#[test]
fn test_box_mapper_into_fn_identity() {
    let mapper = BoxMapper::<i32, i32>::identity();
    let mut closure = mapper.into_fn();

    assert_eq!(closure(42), 42);
    assert_eq!(closure(100), 100);
}

#[test]
fn test_box_mapper_into_fn_constant() {
    let mapper = BoxMapper::constant("hello");
    let mut closure = mapper.into_fn();

    assert_eq!(closure(1), "hello");
    assert_eq!(closure(2), "hello");
    assert_eq!(closure(3), "hello");
}

#[test]
fn test_arc_mapper_into_fn() {
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut closure = mapper.into_fn();
    assert_eq!(closure(10), 10); // 10 * 1
    assert_eq!(closure(10), 20); // 10 * 2
    assert_eq!(closure(10), 30); // 10 * 3
}

#[test]
fn test_arc_mapper_into_fn_identity() {
    let mapper = ArcMapper::<i32, i32>::identity();
    let mut closure = mapper.into_fn();

    assert_eq!(closure(42), 42);
    assert_eq!(closure(100), 100);
}

#[test]
fn test_arc_mapper_into_fn_constant() {
    let mapper = ArcMapper::constant("world");
    let mut closure = mapper.into_fn();

    assert_eq!(closure(1), "world");
    assert_eq!(closure(2), "world");
}

#[test]
fn test_rc_mapper_into_fn() {
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x - counter
    });

    let mut closure = mapper.into_fn();
    assert_eq!(closure(10), 9); // 10 - 1
    assert_eq!(closure(10), 8); // 10 - 2
    assert_eq!(closure(10), 7); // 10 - 3
}

#[test]
fn test_rc_mapper_into_fn_identity() {
    let mapper = RcMapper::<i32, i32>::identity();
    let mut closure = mapper.into_fn();

    assert_eq!(closure(42), 42);
    assert_eq!(closure(100), 100);
}

#[test]
fn test_rc_mapper_into_fn_constant() {
    let mapper = RcMapper::constant("rust");
    let mut closure = mapper.into_fn();

    assert_eq!(closure(1), "rust");
    assert_eq!(closure(2), "rust");
}

#[test]
fn test_closure_into_fn() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter * 10
    };

    let mut closure = mapper.into_fn();
    assert_eq!(closure(5), 15); // 5 + 1 * 10
    assert_eq!(closure(5), 25); // 5 + 2 * 10
    assert_eq!(closure(5), 35); // 5 + 3 * 10
}

#[test]
fn test_closure_into_fn_direct() {
    let mut counter = 0;
    let mut closure = (move |x: i32| {
        counter += 1;
        x * counter
    })
    .into_fn();

    assert_eq!(closure(10), 10); // 10 * 1
    assert_eq!(closure(10), 20); // 10 * 2
}

#[test]
fn test_custom_mapper_into_fn() {
    let mapper = CustomMapper { multiplier: 0 };
    let mut closure = mapper.into_fn();

    assert_eq!(closure(10), 10); // 10 * 1
    assert_eq!(closure(10), 20); // 10 * 2
    assert_eq!(closure(10), 30); // 10 * 3
}

#[test]
fn test_custom_string_mapper_into_fn() {
    let mapper = StringLengthMapper { total_length: 0 };
    let mut closure = mapper.into_fn();

    assert_eq!(closure("hello".to_string()), "[5] hello");
    assert_eq!(closure("world".to_string()), "[10] world");
    assert_eq!(closure("!".to_string()), "[11] !");
}

#[test]
fn test_stateful_mapper_into_fn() {
    let mapper = StatefulMapper {
        count: 0,
        sum: 0,
        history: Vec::new(),
    };
    let mut closure = mapper.into_fn();

    assert_eq!(closure(10), (1, 10, 1));
    assert_eq!(closure(20), (2, 30, 2));
    assert_eq!(closure(30), (3, 60, 3));
}

#[test]
fn test_into_fn_composition() {
    let mut counter1 = 0;
    let mapper1 = BoxMapper::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mapper2 = RcMapper::new(|x: i32| x * 2);

    let composed = mapper1.and_then(mapper2);
    let mut closure = composed.into_fn();

    assert_eq!(closure(10), 22); // (10 + 1) * 2
    assert_eq!(closure(10), 24); // (10 + 2) * 2
}

#[test]
fn test_into_fn_after_conversion() {
    let mut counter = 0;
    let original_closure = move |x: i32| {
        counter += 1;
        x + counter
    };

    // 先转换成 BoxMapper，再转回闭包
    let boxed = original_closure.into_box();
    let mut final_closure = boxed.into_fn();

    assert_eq!(final_closure(10), 11);
    assert_eq!(final_closure(10), 12);
}

#[test]
fn test_into_fn_with_string_return() {
    let mapper = BoxMapper::new(|x: i32| format!("Value: {}", x * 2));
    let mut closure = mapper.into_fn();

    assert_eq!(closure(5), "Value: 10");
    assert_eq!(closure(10), "Value: 20");
}

#[test]
fn test_into_fn_chained_usage() {
    // 测试链式调用：Mapper -> into_box -> and_then -> into_fn
    let mut counter = 0;
    let mapper1 = move |x: i32| {
        counter += 1;
        x + counter
    };

    let boxed = mapper1.into_box();
    let composed = boxed.and_then(|x: i32| x * 2);
    let mut closure = composed.into_fn();

    assert_eq!(closure(10), 22); // (10 + 1) * 2
    assert_eq!(closure(10), 24); // (10 + 2) * 2
}

// ============================================================================
// Closure to_xxx Non-Consuming Conversion Tests
// ============================================================================

#[test]
fn test_closure_to_box() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter
    };

    // 非消费性转换：原始闭包仍然可以使用
    let mut boxed = mapper.to_box();
    assert_eq!(boxed.apply(10), 11); // 10 + 1
    assert_eq!(boxed.apply(10), 12); // 10 + 2

    // 可以多次调用 to_box()
    let mut boxed2 = mapper.to_box();
    assert_eq!(boxed2.apply(20), 21); // 20 + 1 (新的状态)
    assert_eq!(boxed2.apply(20), 22); // 20 + 2

    // 原始的 boxed 仍然保持自己的状态
    assert_eq!(boxed.apply(10), 13); // 10 + 3
}

#[test]
fn test_closure_to_rc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x * counter
    };

    // 非消费性转换
    let rc_mapper = mapper.to_rc();
    let mut rc1 = rc_mapper.clone();
    let mut rc2 = rc_mapper.clone();

    // 共享状态
    assert_eq!(rc1.apply(10), 10); // 10 * 1
    assert_eq!(rc2.apply(10), 20); // 10 * 2
    assert_eq!(rc1.apply(10), 30); // 10 * 3

    // 可以再次调用 to_rc()，创建新的独立状态
    let rc_mapper2 = mapper.to_rc();
    let mut rc3 = rc_mapper2.clone();
    assert_eq!(rc3.apply(10), 10); // 10 * 1 (新的状态)
}

#[test]
fn test_closure_to_arc() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x - counter
    };

    // 非消费性转换（需要 Clone + Send + Sync）
    let arc_mapper = mapper.to_arc();
    let mut arc1 = arc_mapper.clone();
    let mut arc2 = arc_mapper.clone();

    // 共享状态
    assert_eq!(arc1.apply(10), 9); // 10 - 1
    assert_eq!(arc2.apply(10), 8); // 10 - 2
    assert_eq!(arc1.apply(10), 7); // 10 - 3

    // 可以再次调用 to_arc()，创建新的独立状态
    let arc_mapper2 = mapper.to_arc();
    let mut arc3 = arc_mapper2.clone();
    assert_eq!(arc3.apply(10), 9); // 10 - 1 (新的状态)
}

#[test]
fn test_closure_to_fn() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        x + counter * 10
    };

    // 非消费性转换
    let mut closure = mapper.to_fn();
    assert_eq!(closure(5), 15); // 5 + 1 * 10
    assert_eq!(closure(5), 25); // 5 + 2 * 10

    // 可以再次调用 to_fn()，创建新的独立状态
    let mut closure2 = mapper.to_fn();
    assert_eq!(closure2(5), 15); // 5 + 1 * 10 (新的状态)
    assert_eq!(closure2(5), 25); // 5 + 2 * 10

    // 原始的 closure 仍然保持自己的状态
    assert_eq!(closure(5), 35); // 5 + 3 * 10
}

#[test]
fn test_closure_to_box_with_string() {
    let mut count = 0;
    let mapper = move |x: String| {
        count += 1;
        format!("[{}] {}", count, x)
    };

    let mut boxed1 = mapper.to_box();
    assert_eq!(boxed1.apply("hello".to_string()), "[1] hello");
    assert_eq!(boxed1.apply("world".to_string()), "[2] world");

    let mut boxed2 = mapper.to_box();
    assert_eq!(boxed2.apply("rust".to_string()), "[1] rust"); // 新的状态
}

#[test]
fn test_closure_to_rc_shared_state() {
    let mut sum = 0;
    let mapper = move |x: i32| {
        sum += x;
        sum
    };

    let rc_mapper = mapper.to_rc();
    let mut rc1 = rc_mapper.clone();
    let mut rc2 = rc_mapper.clone();

    // 验证共享状态
    assert_eq!(rc1.apply(10), 10);
    assert_eq!(rc2.apply(20), 30); // 共享同一个 sum
    assert_eq!(rc1.apply(30), 60); // 继续累加
}

#[test]
fn test_closure_to_arc_thread_safe() {
    let mut product = 1;
    let mapper = move |x: i32| {
        product *= x;
        product
    };

    let arc_mapper = mapper.to_arc();
    let mut arc1 = arc_mapper.clone();
    let mut arc2 = arc_mapper.clone();

    // 验证共享状态（线程安全）
    assert_eq!(arc1.apply(2), 2);  // 1 * 2
    assert_eq!(arc2.apply(3), 6);  // 2 * 3
    assert_eq!(arc1.apply(4), 24); // 6 * 4
}

#[test]
fn test_closure_to_box_complex_type() {
    let mut history = Vec::new();
    let mapper = move |x: i32| {
        history.push(x);
        (x, history.len())
    };

    let mut boxed = mapper.to_box();
    assert_eq!(boxed.apply(10), (10, 1));
    assert_eq!(boxed.apply(20), (20, 2));
    assert_eq!(boxed.apply(30), (30, 3));
}

#[test]
fn test_closure_to_fn_multiple_calls() {
    let mut counter = 0;
    let mapper = move |x: i32| {
        counter += 1;
        (x, counter)
    };

    // 第一次调用 to_fn()
    let mut fn1 = mapper.to_fn();
    assert_eq!(fn1(100), (100, 1));
    assert_eq!(fn1(200), (200, 2));

    // 第二次调用 to_fn()，创建独立状态
    let mut fn2 = mapper.to_fn();
    assert_eq!(fn2(300), (300, 1)); // 新的计数器从1开始

    // 第一个闭包状态不受影响
    assert_eq!(fn1(400), (400, 3));
}

#[test]
fn test_closure_to_rc_multiple_conversions() {
    let mut value = 0;
    let mapper = move |x: i32| {
        value += x;
        value
    };

    // 第一个 RcMapper
    let rc1 = mapper.to_rc();
    let mut rc1_ref = rc1.clone();
    assert_eq!(rc1_ref.apply(10), 10);

    // 第二个 RcMapper（独立状态）
    let rc2 = mapper.to_rc();
    let mut rc2_ref = rc2.clone();
    assert_eq!(rc2_ref.apply(20), 20); // 独立的状态，从0开始

    // 第一个 RcMapper 的状态不受影响
    assert_eq!(rc1_ref.apply(30), 40); // 10 + 30
}

#[test]
fn test_closure_to_arc_multiple_conversions() {
    let mut count = 0;
    let mapper = move |x: i32| {
        count += 1;
        x * count
    };

    // 第一个 ArcMapper
    let arc1 = mapper.to_arc();
    let mut arc1_ref = arc1.clone();
    assert_eq!(arc1_ref.apply(10), 10); // 10 * 1

    // 第二个 ArcMapper（独立状态）
    let arc2 = mapper.to_arc();
    let mut arc2_ref = arc2.clone();
    assert_eq!(arc2_ref.apply(10), 10); // 10 * 1 (新的 count)

    // 第一个 ArcMapper 的状态不受影响
    assert_eq!(arc1_ref.apply(10), 20); // 10 * 2
}
