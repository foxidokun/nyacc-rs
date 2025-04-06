pub mod nodes {
    pub use crate::ast::{
        expression::{
            Arithmetic, Compare, Float, FunctionCall, Int, Not, StructCtor, UnaryMinus, Variable,
        },
        statement::{
            Assignment, ExprStatement, For, FuncDef, FuncImpl, If, Let, Program, Return, StructDef,
            While,
        },
    };
}

use std::fmt::Debug;

// It's used in tests
#[allow(dead_code)]
pub fn compare(lhs: &dyn Debug, rhs: &dyn Debug) -> bool {
    let lhs_fmt = format!("{:?}", lhs);
    let rhs_fmt = format!("{:?}", rhs);

    lhs_fmt == rhs_fmt
}
