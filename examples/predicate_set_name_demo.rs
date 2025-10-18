/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates the set_name and new_with_name methods of Predicate

use prism3_function::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

fn main() {
    println!("=== Predicate Naming Functionality Demo ===\n");

    demo_box_predicate();
    demo_rc_predicate();
    demo_arc_predicate();
}

/// Demonstrates the naming functionality of BoxPredicate
fn demo_box_predicate() {
    println!("1. BoxPredicate Naming Functionality");

    // Create a predicate with name using new_with_name
    let pred1 = BoxPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
    println!("   Created with new_with_name:");
    println!("     Name: {:?}", pred1.name());
    println!("     Test 5: {}", pred1.test(&5));

    // Set name for an existing predicate using set_name
    let mut pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
    println!("\n   Created with new then set_name:");
    println!("     Initial name: {:?}", pred2.name());
    pred2.set_name("is_even");
    println!("     Name after setting: {:?}", pred2.name());
    println!("     Test 4: {}", pred2.test(&4));

    // Combined predicates automatically generate new names
    let pred3 = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let pred4 = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let combined = pred3.and(pred4);
    println!("\n   Combined predicate name:");
    println!("     Auto-generated name: {:?}", combined.name());
    println!("     Test 4: {}\n", combined.test(&4));
}

/// Demonstrates the naming functionality of RcPredicate
fn demo_rc_predicate() {
    println!("2. RcPredicate Naming Functionality");

    // Using new_with_name
    let pred1 = RcPredicate::new_with_name("greater_than_10", |x: &i32| *x > 10);
    println!("   Using new_with_name:");
    println!("     Name: {:?}", pred1.name());
    println!("     Test 15: {}", pred1.test(&15));

    // Using set_name
    let mut pred2 = RcPredicate::new(|x: &i32| *x < 100);
    println!("\n   Using set_name:");
    println!("     Initial name: {:?}", pred2.name());
    pred2.set_name("less_than_100");
    println!("     Name after setting: {:?}", pred2.name());
    println!("     Test 50: {}", pred2.test(&50));

    // Name is preserved after cloning
    let pred3 = pred2.clone();
    println!("\n   Name preserved after cloning:");
    println!("     Cloned name: {:?}", pred3.name());
    println!("     Test 80: {}\n", pred3.test(&80));
}

/// Demonstrates the naming functionality of ArcPredicate
fn demo_arc_predicate() {
    println!("3. ArcPredicate Naming Functionality (Thread-Safe)");

    // Using new_with_name
    let pred1 = ArcPredicate::new_with_name("is_uppercase", |s: &String| {
        s.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
    });
    println!("   Using new_with_name:");
    println!("     Name: {:?}", pred1.name());
    println!("     Test 'HELLO': {}", pred1.test(&"HELLO".to_string()));

    // Using set_name
    let mut pred2 = ArcPredicate::new(|s: &String| s.len() > 5);
    println!("\n   Using set_name:");
    println!("     Initial name: {:?}", pred2.name());
    pred2.set_name("longer_than_5");
    println!("     Name after setting: {:?}", pred2.name());
    println!(
        "     Test 'Hello World': {}",
        pred2.test(&"Hello World".to_string())
    );

    // Name is preserved when sharing between threads
    let pred3 = pred2.clone();
    let handle = std::thread::spawn(move || {
        let name = pred3.name().map(|s| s.to_string());
        let result = pred3.test(&"Threading".to_string());
        (name, result)
    });

    let (name, result) = handle.join().unwrap();
    println!("\n   Accessing from thread:");
    println!("     Name in thread: {:?}", name);
    println!("     Test 'Threading' in thread: {}", result);

    // Original predicate is still available
    println!("\n   Original predicate still available:");
    println!("     Original name: {:?}", pred2.name());
    println!("     Test 'Rust': {}\n", pred2.test(&"Rust".to_string()));
}
