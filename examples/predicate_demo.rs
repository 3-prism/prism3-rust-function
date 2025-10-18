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
    println!("=== Predicate Usage Examples ===\n");

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
    println!("--- 1. Basic Closure Predicate Usage ---");

    // Simple closure predicate
    let is_positive = |x: &i32| *x > 0;
    println!("Is 5 positive? {}", is_positive.test(&5));
    println!("Is -3 positive? {}", is_positive.test(&-3));

    // Combining closures
    let is_even = |x: &i32| x % 2 == 0;
    let is_positive_and_even = is_positive.and(is_even);
    println!("Is 4 positive and even? {}", is_positive_and_even.test(&4));
    println!("Is 5 positive and even? {}", is_positive_and_even.test(&5));

    // Using predicates with iterators
    let numbers = [-2, -1, 0, 1, 2, 3, 4, 5];
    let positives: Vec<_> = numbers
        .iter()
        .filter(|x| is_positive.test(x))
        .copied()
        .collect();
    println!("Positive numbers: {:?}", positives);
}

/// BoxPredicate examples - single ownership
fn box_predicate_examples() {
    println!("--- 2. BoxPredicate Examples (Single Ownership) ---");

    // Basic BoxPredicate
    let pred = BoxPredicate::new(|x: &i32| *x > 0);
    println!("BoxPredicate test 5: {}", pred.test(&5));

    // Named predicate for better debugging
    let named_pred =
        BoxPredicate::new_with_name("is_positive_even", |x: &i32| *x > 0 && x % 2 == 0);
    println!("Predicate name: {:?}", named_pred.name());
    println!("Test 4: {}", named_pred.test(&4));

    // Method chaining - consumes self
    let positive = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let even = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let combined = positive.and(even);
    println!("Combined predicate name: {:?}", combined.name());
    println!("Test 4: {}", combined.test(&4));
}

/// RcPredicate examples - single-threaded reuse
fn rc_predicate_examples() {
    println!("--- 3. RcPredicate Examples (Single-threaded Reuse) ---");

    let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

    // Multiple compositions without consuming the original
    let positive_and_even = is_positive.and(is_even.clone());
    let positive_or_even = is_positive.or(is_even.clone());

    println!("Original predicates still available:");
    println!("  is_positive.test(&5) = {}", is_positive.test(&5));
    println!("  is_even.test(&4) = {}", is_even.test(&4));

    println!("Combined predicates:");
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
    println!("Cloned predicate: {}", cloned.test(&10));
}

/// ArcPredicate examples - multi-threaded scenarios
fn arc_predicate_examples() {
    println!("--- 4. ArcPredicate Examples (Multi-threaded Scenarios) ---");

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
                println!("  Thread {} testing {}: {}", i, value, pred.test(&value));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Original predicates still usable
    println!("Original predicates still available in main thread:");
    println!("  is_positive.test(&5) = {}", is_positive.test(&5));
}

/// Logical composition examples
fn logical_composition_examples() {
    println!("--- 5. Logical Composition Examples ---");

    let positive = RcPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let even = RcPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let less_than_ten = RcPredicate::new_with_name("less_than_ten", |x: &i32| *x < 10);

    // AND composition
    let positive_and_even = positive.and(even.clone());
    println!("positive AND even: name={:?}", positive_and_even.name());
    println!("  Test 4: {}", positive_and_even.test(&4));
    println!("  Test 5: {}", positive_and_even.test(&5));

    // OR composition
    let positive_or_even = positive.or(even.clone());
    println!("positive OR even: name={:?}", positive_or_even.name());
    println!("  Test -2: {}", positive_or_even.test(&-2));
    println!("  Test 5: {}", positive_or_even.test(&5));

    // NOT composition
    let not_positive = positive.not();
    println!("NOT positive: name={:?}", not_positive.name());
    println!("  Test 5: {}", not_positive.test(&5));
    println!("  Test -3: {}", not_positive.test(&-3));

    // NAND composition
    let nand = positive.nand(even.clone());
    println!("positive NAND even: name={:?}", nand.name());
    println!("  Test 3: {}", nand.test(&3)); // true NAND false = true
    println!("  Test 4: {}", nand.test(&4)); // true NAND true = false

    // XOR composition
    let xor = positive.xor(even.clone());
    println!("positive XOR even: name={:?}", xor.name());
    println!("  Test 3: {}", xor.test(&3)); // true XOR false = true
    println!("  Test 4: {}", xor.test(&4)); // true XOR true = false
    println!("  Test -2: {}", xor.test(&-2)); // false XOR true = true

    // NOR composition
    let nor = positive.nor(even.clone());
    println!("positive NOR even: name={:?}", nor.name());
    println!("  Test -3: {}", nor.test(&-3)); // false NOR false = true
    println!("  Test 3: {}", nor.test(&3)); // true NOR false = false
    println!("  Test -2: {}", nor.test(&-2)); // false NOR true = false
    println!("  Test 4: {}", nor.test(&4)); // true NOR true = false

    // Complex composition
    let complex = positive.and(even.clone()).and(less_than_ten.clone());
    println!("Complex composition: name={:?}", complex.name());
    println!("  Test 4: {}", complex.test(&4));
    println!("  Test 12: {}", complex.test(&12));
}

/// Interior mutability examples
fn interior_mutability_examples() {
    println!("--- 6. Interior Mutability Examples ---");

    // BoxPredicate with counter (RefCell)
    println!("BoxPredicate with counter:");
    let count = RefCell::new(0);
    let pred = BoxPredicate::new(move |x: &i32| {
        *count.borrow_mut() += 1;
        *x > 0
    });
    println!("  Test 5: {}", pred.test(&5));
    println!("  Test -3: {}", pred.test(&-3));
    println!("  Test 10: {}", pred.test(&10));
    // Note: count is moved into the closure, so we can't access it here

    // RcPredicate with cache (RefCell + HashMap)
    println!("\nRcPredicate with cache:");
    let cache: RefCell<HashMap<i32, bool>> = RefCell::new(HashMap::new());
    let expensive_pred = RcPredicate::new(move |x: &i32| {
        let mut c = cache.borrow_mut();
        *c.entry(*x).or_insert_with(|| {
            println!("    Computing result for {} (expensive operation)", x);
            *x > 0 && x % 2 == 0
        })
    });

    println!("  First test 4:");
    println!("    Result: {}", expensive_pred.test(&4));
    println!("  Test 4 again (using cache):");
    println!("    Result: {}", expensive_pred.test(&4));
    println!("  Test 3:");
    println!("    Result: {}", expensive_pred.test(&3));

    // ArcPredicate with thread-safe counter (Mutex)
    println!("\nArcPredicate with thread-safe counter:");
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

    println!("  Total call count: {}", counter_clone.lock().unwrap());
}

/// Practical use cases
fn practical_use_cases() {
    println!("--- 7. Practical Use Cases ---");

    // Validation rules
    println!("Scenario 1: Form Validation");
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

    println!("  user1 validation: {}", all_valid.test(&user1));
    println!("  user2 validation: {}", all_valid.test(&user2));

    // Filter pipeline
    println!("\nScenario 2: Data Filtering Pipeline");
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

    println!("  Filtered numbers: {:?}", filtered);

    // Strategy pattern
    println!("\nScenario 3: Strategy Pattern");
    let mut strategies: HashMap<&str, RcPredicate<i32>> = HashMap::new();
    strategies.insert("positive", RcPredicate::new(|x: &i32| *x > 0));
    strategies.insert("negative", RcPredicate::new(|x: &i32| *x < 0));
    strategies.insert("even", RcPredicate::new(|x: &i32| x % 2 == 0));

    let test_value = 4;
    for (name, pred) in strategies.iter() {
        println!(
            "  {} strategy test {}: {}",
            name,
            test_value,
            pred.test(&test_value)
        );
    }
}
