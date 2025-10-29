/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates how Predicate's into_fn/to_fn methods can be used in scenarios requiring FnMut

use prism3_function::predicate::{
    ArcPredicate,
    BoxPredicate,
    Predicate,
    RcPredicate,
};

fn main() {
    println!("=== Demonstrating Predicate compatibility with FnMut ===\n");

    demo_with_iterator_filter();
    demo_with_vec_retain();
    demo_with_generic_function();
    demo_thread_safe();
}

/// Demonstrates usage with Iterator::filter (filter requires FnMut)
fn demo_with_iterator_filter() {
    println!("1. Using Iterator::filter");

    let pred = BoxPredicate::new(|x: &i32| *x > 0);
    let numbers = vec![-2, -1, 0, 1, 2, 3];
    let positives: Vec<_> = numbers.iter().copied().filter(pred.into_fn()).collect();
    println!("   Original data: {:?}", numbers);
    println!("   Filtered result: {:?}", positives);
    assert_eq!(positives, vec![1, 2, 3]);
    println!("   ✓ BoxPredicate::into_fn() can be used in filter\n");
}

/// Demonstrates usage with Vec::retain (retain requires FnMut)
fn demo_with_vec_retain() {
    println!("2. Using Vec::retain");

    // RcPredicate example
    let pred = RcPredicate::new(|x: &i32| *x % 2 == 0);
    let mut numbers = vec![1, 2, 3, 4, 5, 6];
    println!("   Original data: {:?}", numbers);
    numbers.retain(pred.to_fn());
    println!("   Retained even numbers: {:?}", numbers);
    assert_eq!(numbers, vec![2, 4, 6]);

    // Original predicate is still available
    assert!(pred.test(&10));
    println!("   ✓ RcPredicate::to_fn() can be used in retain");
    println!("   ✓ Original predicate is still available\n");
}

/// Demonstrates usage with generic functions that require FnMut
fn demo_with_generic_function() {
    println!("3. Using generic functions (requires FnMut)");

    fn count_matching<F>(items: &[i32], mut predicate: F) -> usize
    where
        F: FnMut(&i32) -> bool,
    {
        items.iter().filter(|x| predicate(x)).count()
    }

    let pred = RcPredicate::new(|x: &i32| *x > 10);
    let count1 = count_matching(&[5, 15, 8, 20], pred.to_fn());
    println!("   First call: count = {}", count1);
    assert_eq!(count1, 2);

    // Original predicate can be reused
    let count2 = count_matching(&[12, 3, 18], pred.to_fn());
    println!("   Second call: count = {}", count2);
    assert_eq!(count2, 2);

    println!("   ✓ RcPredicate::to_fn() can be passed to generic functions requiring FnMut");
    println!("   ✓ Original predicate can be converted and used multiple times\n");
}

/// Demonstrates thread-safe usage
fn demo_thread_safe() {
    println!("4. Thread-safe usage");

    let pred = ArcPredicate::new(|x: &i32| *x > 0);
    // clone and convert into a 'static closure so it can be moved to another thread
    let closure = pred.clone().into_fn();

    // Closure can be passed between threads
    let handle = std::thread::spawn(move || {
        let numbers = [-2, -1, 0, 1, 2, 3];
        numbers.iter().copied().filter(closure).count()
    });

    let count = handle.join().unwrap();
    println!("   Filtered result count in thread: {}", count);
    assert_eq!(count, 3);

    // Original predicate is still available
    assert!(pred.test(&5));
    println!("   ✓ ArcPredicate::to_fn() returns a thread-safe closure");
    println!("   ✓ Original predicate is still available in main thread\n");
}
