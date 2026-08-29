// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_function::ArcTester;
use qubit_function::BoxTester;
use qubit_function::RcTester;
use qubit_function::Tester;

fn main() {
    println!("=== Tester Demo ===\n");

    demo_box_tester();
    demo_arc_tester();
    demo_rc_tester();
}

fn demo_box_tester() {
    println!("--- BoxTester logic ---");

    let healthy = Arc::new(AtomicBool::new(true));
    let requests = Arc::new(AtomicUsize::new(10));
    let max_requests = 100;

    let healthy_for_check = Arc::clone(&healthy);
    let requests_for_check = Arc::clone(&requests);
    let can_accept = BoxTester::new(move || healthy_for_check.load(Ordering::Relaxed))
        .and(move || requests_for_check.load(Ordering::Relaxed) < max_requests);

    println!("Can accept initially: {}", can_accept.test());
    requests.store(150, Ordering::Relaxed);
    println!("Can accept after overload: {}", can_accept.test());
    println!();
}

fn demo_arc_tester() {
    println!("--- ArcTester sharing ---");

    let enabled = Arc::new(AtomicBool::new(true));
    let enabled_for_check = Arc::clone(&enabled);
    let enabled_tester = ArcTester::new(move || enabled_for_check.load(Ordering::Relaxed));
    let disabled_tester = !&enabled_tester;

    println!("Enabled: {}", enabled_tester.test());
    println!("Disabled: {}", disabled_tester.test());
    enabled.store(false, Ordering::Relaxed);
    println!("Enabled after update: {}", enabled_tester.test());
    println!("Disabled after update: {}", disabled_tester.test());
    println!();
}

fn demo_rc_tester() {
    println!("--- RcTester single-threaded sharing ---");

    let value = Rc::new(Cell::new(0));
    let small_value = {
        let value = Rc::clone(&value);
        RcTester::new(move || value.get() < 3)
    };
    let even_value = {
        let value = Rc::clone(&value);
        RcTester::new(move || value.get() % 2 == 0)
    };
    let small_and_even = small_value.and(even_value.clone());

    println!("0 is small and even: {}", small_and_even.test());
    value.set(2);
    println!("2 is small and even: {}", small_and_even.test());
    value.set(3);
    println!("3 is small and even: {}", small_and_even.test());
}
