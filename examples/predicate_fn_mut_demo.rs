/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 Predicate 的 into_fn/to_fn 方法如何在需要 FnMut 的场景中使用

use prism3_function::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

fn main() {
    println!("=== 演示 Predicate 与 FnMut 的兼容性 ===\n");

    demo_with_iterator_filter();
    demo_with_vec_retain();
    demo_with_generic_function();
    demo_thread_safe();
}

/// 演示与 Iterator::filter 一起使用（filter 需要 FnMut）
fn demo_with_iterator_filter() {
    println!("1. 使用 Iterator::filter");

    let pred = BoxPredicate::new(|x: &i32| *x > 0);
    let numbers = vec![-2, -1, 0, 1, 2, 3];
    let positives: Vec<_> = numbers.iter().copied().filter(pred.into_fn()).collect();
    println!("   原始数据: {:?}", numbers);
    println!("   筛选结果: {:?}", positives);
    assert_eq!(positives, vec![1, 2, 3]);
    println!("   ✓ BoxPredicate::into_fn() 可以用在 filter 中\n");
}

/// 演示与 Vec::retain 一起使用（retain 需要 FnMut）
fn demo_with_vec_retain() {
    println!("2. 使用 Vec::retain");

    // RcPredicate 示例
    let pred = RcPredicate::new(|x: &i32| *x % 2 == 0);
    let mut numbers = vec![1, 2, 3, 4, 5, 6];
    println!("   原始数据: {:?}", numbers);
    numbers.retain(pred.to_fn());
    println!("   保留偶数: {:?}", numbers);
    assert_eq!(numbers, vec![2, 4, 6]);

    // 原始 predicate 仍然可用
    assert!(pred.test(&10));
    println!("   ✓ RcPredicate::to_fn() 可以用在 retain 中");
    println!("   ✓ 原始 predicate 仍然可用\n");
}

/// 演示与需要 FnMut 的泛型函数一起使用
fn demo_with_generic_function() {
    println!("3. 使用泛型函数（需要 FnMut）");

    fn count_matching<F>(items: &[i32], mut predicate: F) -> usize
    where
        F: FnMut(&i32) -> bool,
    {
        items.iter().filter(|x| predicate(x)).count()
    }

    let pred = RcPredicate::new(|x: &i32| *x > 10);
    let count1 = count_matching(&[5, 15, 8, 20], pred.to_fn());
    println!("   第一次调用: count = {}", count1);
    assert_eq!(count1, 2);

    // 原始 predicate 可以重复使用
    let count2 = count_matching(&[12, 3, 18], pred.to_fn());
    println!("   第二次调用: count = {}", count2);
    assert_eq!(count2, 2);

    println!("   ✓ RcPredicate::to_fn() 可以传递给需要 FnMut 的泛型函数");
    println!("   ✓ 原始 predicate 可以多次转换使用\n");
}

/// 演示线程安全的使用
fn demo_thread_safe() {
    println!("4. 线程安全使用");

    let pred = ArcPredicate::new(|x: &i32| *x > 0);
    let closure = pred.to_fn();

    // 闭包可以在线程间传递
    let handle = std::thread::spawn(move || {
        let numbers = [-2, -1, 0, 1, 2, 3];
        numbers.iter().copied().filter(closure).count()
    });

    let count = handle.join().unwrap();
    println!("   线程中筛选结果数量: {}", count);
    assert_eq!(count, 3);

    // 原始 predicate 仍然可用
    assert!(pred.test(&5));
    println!("   ✓ ArcPredicate::to_fn() 返回的闭包是线程安全的");
    println!("   ✓ 原始 predicate 在主线程中仍然可用\n");
}
