// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![allow(
    dead_code,
    unused_imports,
    reason = "split contract fixtures share support definitions"
)]
#![cfg(feature = "full")]
use std::cell::Cell;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

use qubit_function::ArcConsumer;
use qubit_function::ArcMutator;
use qubit_function::ArcPredicate;
use qubit_function::ArcStatefulBiConsumer;
use qubit_function::ArcStatefulBiTransformer;
use qubit_function::ArcStatefulConsumer;
use qubit_function::ArcStatefulFunction;
use qubit_function::ArcStatefulMutatingFunction;
use qubit_function::ArcStatefulMutator;
use qubit_function::ArcStatefulSupplier;
use qubit_function::ArcStatefulTransformer;
use qubit_function::ArcSupplier;
use qubit_function::ArcTransformer;
use qubit_function::BiConsumer;
use qubit_function::BiConsumerOnce;
use qubit_function::BiTransformer;
use qubit_function::BiTransformerOnce;
use qubit_function::BinaryOperator;
use qubit_function::BinaryOperatorOnce;
use qubit_function::BoxBiConsumer;
use qubit_function::BoxBiConsumerOnce;
use qubit_function::BoxConsumer;
use qubit_function::BoxConsumerOnce;
use qubit_function::BoxMutator;
use qubit_function::BoxMutatorOnce;
use qubit_function::BoxPredicate;
use qubit_function::BoxSupplier;
use qubit_function::BoxSupplierOnce;
use qubit_function::BoxTransformer;
use qubit_function::BoxTransformerOnce;
use qubit_function::Consumer;
use qubit_function::ConsumerOnce;
use qubit_function::Mutator;
use qubit_function::MutatorOnce;
use qubit_function::Predicate;
use qubit_function::RcConsumer;
use qubit_function::StatefulBiConsumer;
use qubit_function::StatefulBiTransformer;
use qubit_function::StatefulConsumer;
use qubit_function::StatefulFunction;
use qubit_function::StatefulMutatingFunction;
use qubit_function::StatefulMutator;
use qubit_function::StatefulSupplier;
use qubit_function::StatefulTransformer;
use qubit_function::Supplier;
use qubit_function::SupplierOnce;
use qubit_function::Transformer;
use qubit_function::TransformerOnce;
use qubit_function::UnaryOperator;
use qubit_function::UnaryOperatorOnce;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Borrowed<'a> {
    value: &'a i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BorrowedUnaryOp;

impl<'a> Transformer<Borrowed<'a>, Borrowed<'a>> for BorrowedUnaryOp {
    fn apply(&self, input: Borrowed<'a>) -> Borrowed<'a> {
        input
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BorrowedBinaryOp;

impl<'a> BiTransformer<Borrowed<'a>, Borrowed<'a>, Borrowed<'a>> for BorrowedBinaryOp {
    fn apply(&self, first: Borrowed<'a>, _second: Borrowed<'a>) -> Borrowed<'a> {
        first
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BorrowedUnaryOpOnce;

impl<'a> TransformerOnce<Borrowed<'a>, Borrowed<'a>> for BorrowedUnaryOpOnce {
    fn apply(self, input: Borrowed<'a>) -> Borrowed<'a> {
        input
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BorrowedBinaryOpOnce;

impl<'a> BiTransformerOnce<Borrowed<'a>, Borrowed<'a>, Borrowed<'a>> for BorrowedBinaryOpOnce {
    fn apply(self, first: Borrowed<'a>, _second: Borrowed<'a>) -> Borrowed<'a> {
        first
    }
}

fn make_box_supplier_with_lifetime(_: &i32) -> BoxSupplier<PhantomData<&i32>> {
    BoxSupplier::new(|| PhantomData)
}

fn make_box_supplier_once_with_lifetime(_: &i32) -> BoxSupplierOnce<PhantomData<&i32>> {
    BoxSupplierOnce::new(|| PhantomData)
}

fn make_arc_supplier_with_lifetime(_: &i32) -> ArcSupplier<PhantomData<&i32>> {
    ArcSupplier::new(|| PhantomData)
}

fn assert_stateful_function_impl<'a, F>(_: &'a i32, f: F)
where
    F: StatefulFunction<Borrowed<'a>, i32>,
{
    let _ = f;
}

fn assert_bi_transformer_impl<'a, F>(_: &'a i32, f: F)
where
    F: BiTransformer<Borrowed<'a>, Borrowed<'a>, i32>,
{
    let _ = f;
}

fn assert_unary_operator_impl<'a, F>(_: &'a i32, f: F)
where
    F: UnaryOperator<Borrowed<'a>>,
{
    let _ = f;
}

fn assert_binary_operator_impl<'a, F>(_: &'a i32, f: F)
where
    F: BinaryOperator<Borrowed<'a>>,
{
    let _ = f;
}

fn assert_unary_operator_once_impl<'a, F>(_: &'a i32, f: F)
where
    F: UnaryOperatorOnce<Borrowed<'a>>,
{
    let _ = f;
}

fn assert_binary_operator_once_impl<'a, F>(_: &'a i32, f: F)
where
    F: BinaryOperatorOnce<Borrowed<'a>>,
{
    let _ = f;
}

#[derive(Clone)]
struct SendNonSyncValue(Cell<i32>);

#[test]
fn test_consumers_allow_non_static_generic_on_new() {
    let n = 7;
    let input = Borrowed { value: &n };

    let box_sum = Rc::new(RefCell::new(0));
    let rc_sum = Rc::new(RefCell::new(0));
    let arc_sum = Arc::new(Mutex::new(0));

    let box_consumer = BoxConsumer::new({
        let box_sum = Rc::clone(&box_sum);
        move |item: &Borrowed<'_>| {
            *box_sum.borrow_mut() += *item.value;
        }
    });
    box_consumer.accept(&input);

    let rc_consumer = RcConsumer::new({
        let rc_sum = Rc::clone(&rc_sum);
        move |item: &Borrowed<'_>| {
            *rc_sum.borrow_mut() += *item.value;
        }
    });
    rc_consumer.accept(&input);

    let arc_consumer = ArcConsumer::new({
        let arc_sum = Arc::clone(&arc_sum);
        move |item: &Borrowed<'_>| {
            *arc_sum.lock().expect("lock should succeed") += *item.value;
        }
    });
    arc_consumer.accept(&input);

    assert_eq!(*box_sum.borrow(), 7);
    assert_eq!(*rc_sum.borrow(), 7);
    assert_eq!(*arc_sum.lock().expect("lock should succeed"), 7);
}

#[test]
fn test_bi_consumer_allow_non_static_generic_on_new() {
    let n = 5;
    let input = Borrowed { value: &n };
    let sink = Rc::new(RefCell::new(String::new()));

    let bi_consumer = BoxBiConsumer::new({
        let sink = Rc::clone(&sink);
        move |prefix: &&str, item: &Borrowed<'_>| {
            *sink.borrow_mut() = format!("{}-{}", *prefix, item.value);
        }
    });

    bi_consumer.accept(&"ok", &input);
    assert_eq!(&*sink.borrow(), "ok-5");
}

#[test]
fn test_consumer_once_allow_non_static_generic_on_new() {
    let n = 3;
    let input = Borrowed { value: &n };
    let sink = Rc::new(RefCell::new(0));

    let consumer_once = BoxConsumerOnce::new({
        let sink = Rc::clone(&sink);
        move |item: &Borrowed<'_>| {
            *sink.borrow_mut() = *item.value;
        }
    });
    consumer_once.accept(&input);

    let bi_sink = Rc::new(RefCell::new(0));
    let bi_consumer_once = BoxBiConsumerOnce::new({
        let bi_sink = Rc::clone(&bi_sink);
        move |left: &Borrowed<'_>, right: &Borrowed<'_>| {
            *bi_sink.borrow_mut() = *left.value + *right.value;
        }
    });
    bi_consumer_once.accept(&input, &input);

    assert_eq!(*sink.borrow(), 3);
    assert_eq!(*bi_sink.borrow(), 6);
}

#[test]
fn test_mutators_allow_non_static_generic_on_new() {
    let n = 11;
    let mut slot = Some(&n);

    let box_mutator = BoxMutator::new(|value: &mut Option<&i32>| {
        *value = None;
    });
    box_mutator.apply(&mut slot);
    assert_eq!(slot, None);

    let arc_mutator = ArcMutator::new(|value: &mut Option<&i32>| {
        if value.is_none() {
            *value = Some(&42);
        }
    });
    arc_mutator.apply(&mut slot);
    assert_eq!(slot, Some(&42));
}

#[test]
fn test_mutator_once_allow_non_static_generic_on_new() {
    let n = 9;
    let mut slot = Some(&n);

    let mutator_once = BoxMutatorOnce::new(|value: &mut Option<&i32>| {
        *value = None;
    });
    mutator_once.apply(&mut slot);

    assert_eq!(slot, None);
}

#[test]
fn test_predicate_and_transformer_allow_non_static_generic_on_new() {
    let n = 13;
    let value = Borrowed { value: &n };

    let predicate = BoxPredicate::new(|item: &Borrowed<'_>| *item.value > 10);
    assert!(predicate.test(&value));

    let arc_predicate = ArcPredicate::new(|item: &Borrowed<'_>| *item.value % 2 == 1);
    assert!(arc_predicate.test(&value));

    let transformer = BoxTransformer::new(|item: Borrowed<'_>| *item.value + 1);
    assert_eq!(transformer.apply(value), 14);

    let arc_transformer = ArcTransformer::new(|item: Borrowed<'_>| *item.value - 1);
    assert_eq!(arc_transformer.apply(value), 12);
}

#[test]
fn test_transformer_once_allow_non_static_generic_on_new() {
    let n = 8;
    let value = Borrowed { value: &n };

    let transformer_once = BoxTransformerOnce::new(|item: Borrowed<'_>| *item.value * 2);
    assert_eq!(transformer_once.apply(value), 16);
}

#[test]
fn test_suppliers_allow_non_static_generic_on_new() {
    let n = 21;

    let box_supplier: BoxSupplier<PhantomData<&i32>> = make_box_supplier_with_lifetime(&n);
    let box_supplier_once: BoxSupplierOnce<PhantomData<&i32>> = make_box_supplier_once_with_lifetime(&n);
    let arc_supplier: ArcSupplier<PhantomData<&i32>> = make_arc_supplier_with_lifetime(&n);

    assert_eq!(box_supplier.get(), PhantomData);
    assert_eq!(box_supplier_once.get(), PhantomData);
    assert_eq!(arc_supplier.get(), PhantomData);
}

#[test]
fn test_semantic_traits_allow_non_static_closure_implementations() {
    let a = 3;

    let stateful_function = |value: &Borrowed<'_>| *value.value;
    assert_stateful_function_impl(&a, stateful_function);

    let bi_transformer_with_borrow = |left: Borrowed<'_>, right: Borrowed<'_>| *left.value + *right.value;
    assert_bi_transformer_impl(&a, bi_transformer_with_borrow);

    assert_unary_operator_impl(&a, BorrowedUnaryOp);
    assert_binary_operator_impl(&a, BorrowedBinaryOp);
    assert_unary_operator_once_impl(&a, BorrowedUnaryOpOnce);
    assert_binary_operator_once_impl(&a, BorrowedBinaryOpOnce);
}

#[test]
fn test_stateful_arc_and_then_accepts_send_non_sync_callbacks() {
    let observed = Arc::new(AtomicI32::new(0));
    let observed_capture = Arc::clone(&observed);
    let state = Cell::new(0);
    let mut consumer = ArcStatefulConsumer::new(|_: &i32| {}).and_then(move |value: &i32| {
        state.set(*value);
        observed_capture.store(state.get(), Ordering::SeqCst);
    });
    consumer.accept(&3);
    assert_eq!(observed.load(Ordering::SeqCst), 3);

    let observed = Arc::new(AtomicI32::new(0));
    let observed_capture = Arc::clone(&observed);
    let state = Cell::new(0);
    let mut bi_consumer = ArcStatefulBiConsumer::new(|_: &i32, _: &i32| {}).and_then(move |left: &i32, right: &i32| {
        state.set(*left + *right);
        observed_capture.store(state.get(), Ordering::SeqCst);
    });
    bi_consumer.accept(&2, &4);
    assert_eq!(observed.load(Ordering::SeqCst), 6);

    let state = Cell::new(0);
    let mut function = ArcStatefulFunction::new(|value: &i32| *value + 1).and_then(move |value: &i32| {
        state.set(*value);
        state.get() * 2
    });
    assert_eq!(function.apply(&2), 6);

    let state = Cell::new(0);
    let mut mutating_function = ArcStatefulMutatingFunction::new(|value: &mut i32| {
        *value += 1;
        *value
    })
    .and_then(move |value: &i32| {
        state.set(*value);
        state.get() * 2
    });
    let mut input = 2;
    assert_eq!(mutating_function.apply(&mut input), 6);

    let state = Cell::new(0);
    let mut mutator = ArcStatefulMutator::new(|value: &mut i32| *value += 1).and_then(move |value: &mut i32| {
        state.set(*value);
        *value *= 2;
    });
    let mut input = 2;
    mutator.apply(&mut input);
    assert_eq!(input, 6);

    let state = Cell::new(0);
    let mut transformer = ArcStatefulTransformer::new(|value: i32| value + 1).and_then(move |value: i32| {
        state.set(value);
        state.get() * 2
    });
    assert_eq!(transformer.apply(2), 6);

    let state = Cell::new(0);
    let mut bi_transformer =
        ArcStatefulBiTransformer::new(|left: i32, right: i32| left + right).and_then(move |value: i32| {
            state.set(value);
            state.get() * 2
        });
    assert_eq!(bi_transformer.apply(2, 4), 12);
}
