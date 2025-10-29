/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Transformers Module
//!
//! This module provides transformer-related functional programming
//! abstractions for converting values from one type to another, including
//! single-parameter transformers, bi-transformers, and their stateful
//! variants.
//!
//! # Author
//!
//! Haixing Hu

pub mod bi_transformer;
pub mod bi_transformer_once;
pub mod stateful_bi_transformer;
pub mod stateful_transformer;
pub mod transformer;
pub mod transformer_once;

pub use bi_transformer::{
    ArcBiTransformer,
    ArcBinaryOperator,
    BiTransformer,
    BinaryOperator,
    BoxBiTransformer,
    BoxBinaryOperator,
    FnBiTransformerOps,
    RcBiTransformer,
    RcBinaryOperator,
};
pub use bi_transformer_once::{
    BiTransformerOnce,
    BinaryOperatorOnce,
    BoxBiTransformerOnce,
    BoxBinaryOperatorOnce,
    FnBiTransformerOnceOps,
};
pub use stateful_bi_transformer::{
    ArcConditionalStatefulBiTransformer,
    ArcStatefulBiTransformer,
    BoxConditionalStatefulBiTransformer,
    BoxStatefulBiTransformer,
    FnStatefulBiTransformerOps,
    RcConditionalStatefulBiTransformer,
    RcStatefulBiTransformer,
    StatefulBiTransformer,
};
pub use stateful_transformer::{
    ArcConditionalStatefulTransformer,
    ArcStatefulTransformer,
    BoxConditionalStatefulTransformer,
    BoxStatefulTransformer,
    FnStatefulTransformerOps,
    RcConditionalStatefulTransformer,
    RcStatefulTransformer,
    StatefulTransformer,
};
pub use transformer::{
    ArcConditionalTransformer,
    ArcTransformer,
    ArcUnaryOperator,
    BoxConditionalTransformer,
    BoxTransformer,
    BoxUnaryOperator,
    FnTransformerOps,
    RcConditionalTransformer,
    RcTransformer,
    RcUnaryOperator,
    Transformer,
    UnaryOperator,
};
pub use transformer_once::{
    BoxConditionalTransformerOnce,
    BoxTransformerOnce,
    BoxUnaryOperatorOnce,
    FnTransformerOnceOps,
    TransformerOnce,
    UnaryOperatorOnce,
};
