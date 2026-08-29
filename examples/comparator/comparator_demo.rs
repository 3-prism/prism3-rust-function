// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcComparator;
use qubit_function::BoxComparator;
use qubit_function::Comparator;
use qubit_function::RcComparator;

#[derive(Debug, Eq, PartialEq)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    println!("=== Comparator Demo ===\n");

    demo_box_comparator();
    demo_shared_comparators();
}

fn demo_box_comparator() {
    println!("--- BoxComparator sorting ---");

    let by_age = BoxComparator::new(|left: &User, right: &User| left.age.cmp(&right.age));
    let by_name = BoxComparator::new(|left: &User, right: &User| left.name.cmp(&right.name));
    let comparator = by_age.then_comparing(by_name);

    let mut users = vec![
        User {
            name: String::from("Charlie"),
            age: 30,
        },
        User {
            name: String::from("Alice"),
            age: 25,
        },
        User {
            name: String::from("Bob"),
            age: 25,
        },
    ];

    users.sort_by(|left, right| comparator.compare(left, right));
    println!("Sorted users: {users:?}");
    println!();
}

fn demo_shared_comparators() {
    println!("--- Shared comparators ---");

    let ascending = ArcComparator::new(|left: &i32, right: &i32| left.cmp(right));
    let descending = ascending.reversed();
    println!("ArcComparator descending 10 vs 3: {:?}", descending.compare(&10, &3));

    let by_length = RcComparator::new(|left: &String, right: &String| left.len().cmp(&right.len()));
    let alphabetic = RcComparator::new(|left: &String, right: &String| left.cmp(right));
    let combined = by_length.then_comparing(alphabetic.clone());
    println!(
        "RcComparator length/name 'aa' vs 'b': {:?}",
        combined.compare(&String::from("aa"), &String::from("b"))
    );
}
