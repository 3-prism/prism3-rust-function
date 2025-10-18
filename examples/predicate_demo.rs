/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Comprehensive demonstration of the Predicate abstraction.
//!
//! This example shows:
//! - Basic predicate usage with closures
//! - BoxPredicate for single-ownership scenarios
//! - RcPredicate for single-threaded reuse
//! - ArcPredicate for multi-threaded scenarios
//! - Logical composition (AND, OR, NOT)
//! - Interior mutability patterns
//! - Type conversions

use prism3_function::predicate::{
    ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== Predicate 使用示例 ===\n");

    basic_closure_predicates();
    println!();

    box_predicate_examples();
    println!();

    rc_predicate_examples();
    println!();

    arc_predicate_examples();
    println!();

    logical_composition_examples();
    println!();

    interior_mutability_examples();
    println!();

    practical_use_cases();
}

/// Basic closure predicate usage
fn basic_closure_predicates() {
    println!("--- 1. 闭包谓词基础用法 ---");

    // Simple closure predicate
    let is_positive = |x: &i32| *x > 0;
    println!("5 是正数? {}", is_positive.test(&5));
    println!("-3 是正数? {}", is_positive.test(&-3));

    // Combining closures
    let is_even = |x: &i32| x % 2 == 0;
    let is_positive_and_even = is_positive.and(is_even);
    println!("4 是正偶数? {}", is_positive_and_even.test(&4));
    println!("5 是正偶数? {}", is_positive_and_even.test(&5));

    // Using predicates with iterators
    let numbers = [-2, -1, 0, 1, 2, 3, 4, 5];
    let positives: Vec<_> = numbers
        .iter()
        .filter(|x| is_positive.test(x))
        .copied()
        .collect();
    println!("正数: {:?}", positives);
}

/// BoxPredicate examples - single ownership
fn box_predicate_examples() {
    println!("--- 2. BoxPredicate 示例（单一所有权）---");

    // Basic BoxPredicate
    let pred = BoxPredicate::new(|x: &i32| *x > 0);
    println!("BoxPredicate 测试 5: {}", pred.test(&5));

    // Named predicate for better debugging
    let named_pred =
        BoxPredicate::new_with_name("is_positive_even", |x: &i32| *x > 0 && x % 2 == 0);
    println!("谓词名称: {:?}", named_pred.name());
    println!("测试 4: {}", named_pred.test(&4));

    // Method chaining - consumes self
    let positive = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let even = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let combined = positive.and(even);
    println!("组合谓词名称: {:?}", combined.name());
    println!("测试 4: {}", combined.test(&4));
}

/// RcPredicate examples - single-threaded reuse
fn rc_predicate_examples() {
    println!("--- 3. RcPredicate 示例（单线程复用）---");

    let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

    // Multiple compositions without consuming the original
    let positive_and_even = is_positive.and(is_even.clone());
    let positive_or_even = is_positive.or(is_even.clone());

    println!("原始谓词仍可用:");
    println!("  is_positive.test(&5) = {}", is_positive.test(&5));
    println!("  is_even.test(&4) = {}", is_even.test(&4));

    println!("组合谓词:");
    println!(
        "  positive_and_even.test(&4) = {}",
        positive_and_even.test(&4)
    );
    println!(
        "  positive_or_even.test(&5) = {}",
        positive_or_even.test(&5)
    );

    // Cloning
    let cloned = is_positive.clone();
    println!("克隆的谓词: {}", cloned.test(&10));
}

/// ArcPredicate examples - multi-threaded scenarios
fn arc_predicate_examples() {
    println!("--- 4. ArcPredicate 示例（多线程场景）---");

    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

    // Create combined predicate
    let combined = is_positive.and(is_even);

    // Use in multiple threads
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let pred = combined.clone();
            std::thread::spawn(move || {
                let value = i * 2;
                println!("  线程 {} 测试 {}: {}", i, value, pred.test(&value));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Original predicates still usable
    println!("主线程中原始谓词仍可用:");
    println!("  is_positive.test(&5) = {}", is_positive.test(&5));
}

/// Logical composition examples
fn logical_composition_examples() {
    println!("--- 5. 逻辑组合示例 ---");

    let positive = RcPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let even = RcPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let less_than_ten = RcPredicate::new_with_name("less_than_ten", |x: &i32| *x < 10);

    // AND composition
    let positive_and_even = positive.and(even.clone());
    println!("positive AND even: name={:?}", positive_and_even.name());
    println!("  测试 4: {}", positive_and_even.test(&4));
    println!("  测试 5: {}", positive_and_even.test(&5));

    // OR composition
    let positive_or_even = positive.or(even.clone());
    println!("positive OR even: name={:?}", positive_or_even.name());
    println!("  测试 -2: {}", positive_or_even.test(&-2));
    println!("  测试 5: {}", positive_or_even.test(&5));

    // NOT composition
    let not_positive = positive.not();
    println!("NOT positive: name={:?}", not_positive.name());
    println!("  测试 5: {}", not_positive.test(&5));
    println!("  测试 -3: {}", not_positive.test(&-3));

    // NAND composition
    let nand = positive.nand(even.clone());
    println!("positive NAND even: name={:?}", nand.name());
    println!("  测试 3: {}", nand.test(&3)); // true NAND false = true
    println!("  测试 4: {}", nand.test(&4)); // true NAND true = false

    // XOR composition
    let xor = positive.xor(even.clone());
    println!("positive XOR even: name={:?}", xor.name());
    println!("  测试 3: {}", xor.test(&3)); // true XOR false = true
    println!("  测试 4: {}", xor.test(&4)); // true XOR true = false
    println!("  测试 -2: {}", xor.test(&-2)); // false XOR true = true

    // NOR composition
    let nor = positive.nor(even.clone());
    println!("positive NOR even: name={:?}", nor.name());
    println!("  测试 -3: {}", nor.test(&-3)); // false NOR false = true
    println!("  测试 3: {}", nor.test(&3)); // true NOR false = false
    println!("  测试 -2: {}", nor.test(&-2)); // false NOR true = false
    println!("  测试 4: {}", nor.test(&4)); // true NOR true = false

    // Complex composition
    let complex = positive.and(even.clone()).and(less_than_ten.clone());
    println!("Complex composition: name={:?}", complex.name());
    println!("  测试 4: {}", complex.test(&4));
    println!("  测试 12: {}", complex.test(&12));
}

/// Interior mutability examples
fn interior_mutability_examples() {
    println!("--- 6. 内部可变性示例 ---");

    // BoxPredicate with counter (RefCell)
    println!("BoxPredicate 带计数器:");
    let count = RefCell::new(0);
    let pred = BoxPredicate::new(move |x: &i32| {
        *count.borrow_mut() += 1;
        *x > 0
    });
    println!("  测试 5: {}", pred.test(&5));
    println!("  测试 -3: {}", pred.test(&-3));
    println!("  测试 10: {}", pred.test(&10));
    // Note: count is moved into the closure, so we can't access it here

    // RcPredicate with cache (RefCell + HashMap)
    println!("\nRcPredicate 带缓存:");
    let cache: RefCell<HashMap<i32, bool>> = RefCell::new(HashMap::new());
    let expensive_pred = RcPredicate::new(move |x: &i32| {
        let mut c = cache.borrow_mut();
        *c.entry(*x).or_insert_with(|| {
            println!("    计算 {} 的结果（昂贵操作）", x);
            *x > 0 && x % 2 == 0
        })
    });

    println!("  首次测试 4:");
    println!("    结果: {}", expensive_pred.test(&4));
    println!("  再次测试 4（使用缓存）:");
    println!("    结果: {}", expensive_pred.test(&4));
    println!("  测试 3:");
    println!("    结果: {}", expensive_pred.test(&3));

    // ArcPredicate with thread-safe counter (Mutex)
    println!("\nArcPredicate 带线程安全计数器:");
    let counter = Arc::new(Mutex::new(0));
    let pred = ArcPredicate::new({
        let counter = Arc::clone(&counter);
        move |x: &i32| {
            let mut c = counter.lock().unwrap();
            *c += 1;
            *x > 0
        }
    });

    let pred_clone = pred.clone();
    let counter_clone = Arc::clone(&counter);

    let handle = std::thread::spawn(move || {
        pred_clone.test(&5);
        pred_clone.test(&10);
    });

    pred.test(&3);
    handle.join().unwrap();

    println!("  总共调用次数: {}", counter_clone.lock().unwrap());
}

/// Practical use cases
fn practical_use_cases() {
    println!("--- 7. 实际应用场景 ---");

    // Validation rules
    println!("场景 1: 表单验证");
    struct User {
        name: String,
        age: i32,
        email: String,
    }

    let name_valid =
        RcPredicate::new_with_name("name_not_empty", |user: &User| !user.name.is_empty());

    let age_valid = RcPredicate::new_with_name("age_between_18_120", |user: &User| {
        user.age >= 18 && user.age <= 120
    });

    let email_valid =
        RcPredicate::new_with_name("email_contains_at", |user: &User| user.email.contains('@'));

    let all_valid = name_valid.and(age_valid.clone()).and(email_valid.clone());

    let user1 = User {
        name: "Alice".to_string(),
        age: 25,
        email: "alice@example.com".to_string(),
    };

    let user2 = User {
        name: "".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    };

    println!("  user1 验证: {}", all_valid.test(&user1));
    println!("  user2 验证: {}", all_valid.test(&user2));

    // Filter pipeline
    println!("\n场景 2: 数据过滤流水线");
    let numbers: Vec<i32> = (-10..=10).collect();

    let positive = |x: &i32| *x > 0;
    let even = |x: &i32| x % 2 == 0;
    let less_than_eight = |x: &i32| *x < 8;

    let filtered: Vec<i32> = numbers
        .iter()
        .filter(|x| positive.test(x))
        .filter(|x| even.test(x))
        .filter(|x| less_than_eight.test(x))
        .copied()
        .collect();

    println!("  过滤后的数字: {:?}", filtered);

    // Strategy pattern
    println!("\n场景 3: 策略模式");
    let mut strategies: HashMap<&str, RcPredicate<i32>> = HashMap::new();
    strategies.insert("positive", RcPredicate::new(|x: &i32| *x > 0));
    strategies.insert("negative", RcPredicate::new(|x: &i32| *x < 0));
    strategies.insert("even", RcPredicate::new(|x: &i32| x % 2 == 0));

    let test_value = 4;
    for (name, pred) in strategies.iter() {
        println!(
            "  {} 策略测试 {}: {}",
            name,
            test_value,
            pred.test(&test_value)
        );
    }
}
