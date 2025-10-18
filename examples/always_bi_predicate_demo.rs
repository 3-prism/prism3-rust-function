/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use prism3_function::bi_predicate::{ArcBiPredicate, BiPredicate, BoxBiPredicate, RcBiPredicate};

fn main() {
    println!("=== BoxBiPredicate always_true/always_false Demo ===\n");

    // BoxBiPredicate::always_true
    let always_true: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_true();
    println!("BoxBiPredicate::always_true():");
    println!("  test(&42, &10): {}", always_true.test(&42, &10));
    println!("  test(&-1, &5): {}", always_true.test(&-1, &5));
    println!("  test(&0, &0): {}", always_true.test(&0, &0));
    println!("  name: {:?}", always_true.name());

    // BoxBiPredicate::always_false
    let always_false: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_false();
    println!("\nBoxBiPredicate::always_false():");
    println!("  test(&42, &10): {}", always_false.test(&42, &10));
    println!("  test(&-1, &5): {}", always_false.test(&-1, &5));
    println!("  test(&0, &0): {}", always_false.test(&0, &0));
    println!("  name: {:?}", always_false.name());

    println!("\n=== RcBiPredicate always_true/always_false Demo ===\n");

    // RcBiPredicate::always_true
    let rc_always_true: RcBiPredicate<String, i32> = RcBiPredicate::always_true();
    println!("RcBiPredicate::always_true():");
    println!(
        "  test(&\"hello\", &5): {}",
        rc_always_true.test(&"hello".to_string(), &5)
    );
    println!(
        "  test(&\"world\", &-3): {}",
        rc_always_true.test(&"world".to_string(), &-3)
    );
    println!("  name: {:?}", rc_always_true.name());

    // RcBiPredicate::always_false
    let rc_always_false: RcBiPredicate<String, i32> = RcBiPredicate::always_false();
    println!("\nRcBiPredicate::always_false():");
    println!(
        "  test(&\"hello\", &5): {}",
        rc_always_false.test(&"hello".to_string(), &5)
    );
    println!(
        "  test(&\"world\", &-3): {}",
        rc_always_false.test(&"world".to_string(), &-3)
    );
    println!("  name: {:?}", rc_always_false.name());

    // Can be cloned and reused
    let rc_clone = rc_always_true.clone();
    println!("\nAfter cloning, still usable:");
    println!(
        "  Original: test(&\"test\", &1): {}",
        rc_always_true.test(&"test".to_string(), &1)
    );
    println!(
        "  Clone: test(&\"test\", &2): {}",
        rc_clone.test(&"test".to_string(), &2)
    );

    println!("\n=== ArcBiPredicate always_true/always_false Demo ===\n");

    // ArcBiPredicate::always_true
    let arc_always_true: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_true();
    println!("ArcBiPredicate::always_true():");
    println!("  test(&100, &50): {}", arc_always_true.test(&100, &50));
    println!("  test(&-100, &25): {}", arc_always_true.test(&-100, &25));
    println!("  name: {:?}", arc_always_true.name());

    // ArcBiPredicate::always_false
    let arc_always_false: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_false();
    println!("\nArcBiPredicate::always_false():");
    println!("  test(&100, &50): {}", arc_always_false.test(&100, &50));
    println!("  test(&-100, &25): {}", arc_always_false.test(&-100, &25));
    println!("  name: {:?}", arc_always_false.name());

    println!("\n=== Combining with other bi-predicates ===\n");

    // Combining with always_true (AND)
    let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
    let combined_and_true = sum_positive.and(BoxBiPredicate::always_true());
    println!("sum_positive AND always_true:");
    println!(
        "  test(&5, &3): {} (equivalent to sum_positive)",
        combined_and_true.test(&5, &3)
    );
    println!(
        "  test(&-3, &-5): {} (equivalent to sum_positive)",
        combined_and_true.test(&-3, &-5)
    );

    // Combining with always_false (AND)
    let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
    let combined_and_false = sum_positive.and(BoxBiPredicate::always_false());
    println!("\nsum_positive AND always_false:");
    println!(
        "  test(&5, &3): {} (always false)",
        combined_and_false.test(&5, &3)
    );
    println!(
        "  test(&-3, &-5): {} (always false)",
        combined_and_false.test(&-3, &-5)
    );

    // Combining with always_true (OR)
    let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
    let combined_or_true = sum_positive.or(BoxBiPredicate::always_true());
    println!("\nsum_positive OR always_true:");
    println!(
        "  test(&5, &3): {} (always true)",
        combined_or_true.test(&5, &3)
    );
    println!(
        "  test(&-3, &-5): {} (always true)",
        combined_or_true.test(&-3, &-5)
    );

    // Combining with always_false (OR)
    let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
    let combined_or_false = sum_positive.or(BoxBiPredicate::always_false());
    println!("\nsum_positive OR always_false:");
    println!(
        "  test(&5, &3): {} (equivalent to sum_positive)",
        combined_or_false.test(&5, &3)
    );
    println!(
        "  test(&-3, &-5): {} (equivalent to sum_positive)",
        combined_or_false.test(&-3, &-5)
    );

    println!("\n=== Practical scenarios: Default pass/reject filters ===\n");

    // Scenario 1: Default pass-all filter
    let pairs = vec![(1, 2), (3, 4), (5, 6)];
    let pass_all = BoxBiPredicate::<i32, i32>::always_true();
    let closure = pass_all.into_fn();
    let filtered: Vec<_> = pairs.iter().filter(|(x, y)| closure(x, y)).collect();
    println!("Default pass all elements: {:?} -> {:?}", pairs, filtered);

    // Scenario 2: Default reject-all filter
    let pairs = vec![(1, 2), (3, 4), (5, 6)];
    let reject_all = BoxBiPredicate::<i32, i32>::always_false();
    let closure = reject_all.into_fn();
    let filtered: Vec<_> = pairs.iter().filter(|(x, y)| closure(x, y)).collect();
    println!("Default reject all elements: {:?} -> {:?}", pairs, filtered);

    // Scenario 3: Configurable filter
    fn configurable_filter(enable_filter: bool) -> BoxBiPredicate<i32, i32> {
        if enable_filter {
            BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 5)
        } else {
            BoxBiPredicate::always_true()
        }
    }

    let pairs = vec![(1, 2), (3, 4), (5, 6)];

    let filter_enabled = configurable_filter(true);
    let closure = filter_enabled.into_fn();
    let filtered: Vec<_> = pairs.iter().filter(|(x, y)| closure(x, y)).collect();
    println!("\nFilter enabled: {:?} -> {:?}", pairs, filtered);

    let filter_disabled = configurable_filter(false);
    let closure = filter_disabled.into_fn();
    let filtered: Vec<_> = pairs.iter().filter(|(x, y)| closure(x, y)).collect();
    println!("Filter disabled: {:?} -> {:?}", pairs, filtered);
}
