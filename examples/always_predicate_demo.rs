/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use prism3_function::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

fn main() {
    println!("=== BoxPredicate always_true/always_false 演示 ===\n");

    // BoxPredicate::always_true
    let always_true: BoxPredicate<i32> = BoxPredicate::always_true();
    println!("BoxPredicate::always_true():");
    println!("  test(&42): {}", always_true.test(&42));
    println!("  test(&-1): {}", always_true.test(&-1));
    println!("  test(&0): {}", always_true.test(&0));
    println!("  name: {:?}", always_true.name());

    // BoxPredicate::always_false
    let always_false: BoxPredicate<i32> = BoxPredicate::always_false();
    println!("\nBoxPredicate::always_false():");
    println!("  test(&42): {}", always_false.test(&42));
    println!("  test(&-1): {}", always_false.test(&-1));
    println!("  test(&0): {}", always_false.test(&0));
    println!("  name: {:?}", always_false.name());

    println!("\n=== RcPredicate always_true/always_false 演示 ===\n");

    // RcPredicate::always_true
    let rc_always_true: RcPredicate<String> = RcPredicate::always_true();
    println!("RcPredicate::always_true():");
    println!(
        "  test(&\"hello\"): {}",
        rc_always_true.test(&"hello".to_string())
    );
    println!(
        "  test(&\"world\"): {}",
        rc_always_true.test(&"world".to_string())
    );
    println!("  name: {:?}", rc_always_true.name());

    // RcPredicate::always_false
    let rc_always_false: RcPredicate<String> = RcPredicate::always_false();
    println!("\nRcPredicate::always_false():");
    println!(
        "  test(&\"hello\"): {}",
        rc_always_false.test(&"hello".to_string())
    );
    println!(
        "  test(&\"world\"): {}",
        rc_always_false.test(&"world".to_string())
    );
    println!("  name: {:?}", rc_always_false.name());

    // 可以克隆和重用
    let rc_clone = rc_always_true.clone();
    println!("\n克隆后仍可使用:");
    println!(
        "  原始: test(&\"test\"): {}",
        rc_always_true.test(&"test".to_string())
    );
    println!(
        "  克隆: test(&\"test\"): {}",
        rc_clone.test(&"test".to_string())
    );

    println!("\n=== ArcPredicate always_true/always_false 演示 ===\n");

    // ArcPredicate::always_true
    let arc_always_true: ArcPredicate<i32> = ArcPredicate::always_true();
    println!("ArcPredicate::always_true():");
    println!("  test(&100): {}", arc_always_true.test(&100));
    println!("  test(&-100): {}", arc_always_true.test(&-100));
    println!("  name: {:?}", arc_always_true.name());

    // ArcPredicate::always_false
    let arc_always_false: ArcPredicate<i32> = ArcPredicate::always_false();
    println!("\nArcPredicate::always_false():");
    println!("  test(&100): {}", arc_always_false.test(&100));
    println!("  test(&-100): {}", arc_always_false.test(&-100));
    println!("  name: {:?}", arc_always_false.name());

    println!("\n=== 与其他 predicate 组合使用 ===\n");

    // 与 always_true 组合（AND）
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let combined_and_true = is_positive.and(BoxPredicate::always_true());
    println!("is_positive AND always_true:");
    println!(
        "  test(&5): {} (相当于 is_positive)",
        combined_and_true.test(&5)
    );
    println!(
        "  test(&-3): {} (相当于 is_positive)",
        combined_and_true.test(&-3)
    );

    // 与 always_false 组合（AND）
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let combined_and_false = is_positive.and(BoxPredicate::always_false());
    println!("\nis_positive AND always_false:");
    println!("  test(&5): {} (总是 false)", combined_and_false.test(&5));
    println!("  test(&-3): {} (总是 false)", combined_and_false.test(&-3));

    // 与 always_true 组合（OR）
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let combined_or_true = is_positive.or(BoxPredicate::always_true());
    println!("\nis_positive OR always_true:");
    println!("  test(&5): {} (总是 true)", combined_or_true.test(&5));
    println!("  test(&-3): {} (总是 true)", combined_or_true.test(&-3));

    // 与 always_false 组合（OR）
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let combined_or_false = is_positive.or(BoxPredicate::always_false());
    println!("\nis_positive OR always_false:");
    println!(
        "  test(&5): {} (相当于 is_positive)",
        combined_or_false.test(&5)
    );
    println!(
        "  test(&-3): {} (相当于 is_positive)",
        combined_or_false.test(&-3)
    );

    println!("\n=== 实用场景：默认通过/拒绝过滤器 ===\n");

    // 场景1：默认全部通过的过滤器
    let numbers = vec![1, 2, 3, 4, 5];
    let pass_all = BoxPredicate::<i32>::always_true();
    let filtered: Vec<_> = numbers.iter().copied().filter(pass_all.into_fn()).collect();
    println!("默认通过所有元素: {:?} -> {:?}", numbers, filtered);

    // 场景2：默认全部拒绝的过滤器
    let numbers = vec![1, 2, 3, 4, 5];
    let reject_all = BoxPredicate::<i32>::always_false();
    let filtered: Vec<_> = numbers
        .iter()
        .copied()
        .filter(reject_all.into_fn())
        .collect();
    println!("默认拒绝所有元素: {:?} -> {:?}", numbers, filtered);

    // 场景3：可配置的过滤器
    fn configurable_filter(enable_filter: bool) -> BoxPredicate<i32> {
        if enable_filter {
            BoxPredicate::new(|x: &i32| *x > 3)
        } else {
            BoxPredicate::always_true()
        }
    }

    let numbers = vec![1, 2, 3, 4, 5];

    let filter_enabled = configurable_filter(true);
    let filtered: Vec<_> = numbers
        .iter()
        .copied()
        .filter(filter_enabled.into_fn())
        .collect();
    println!("\n过滤器启用: {:?} -> {:?}", numbers, filtered);

    let filter_disabled = configurable_filter(false);
    let filtered: Vec<_> = numbers
        .iter()
        .copied()
        .filter(filter_disabled.into_fn())
        .collect();
    println!("过滤器禁用: {:?} -> {:?}", numbers, filtered);
}
