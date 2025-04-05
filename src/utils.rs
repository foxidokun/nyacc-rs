pub mod nodes {
    pub use crate::ast::{
        expression::{Arithmetic, Compare, Float, FunctionCall, Int, Not, UnaryMinus, Variable},
        statement::{
            Assignment, ExprStatement, For, FuncDef, FuncImpl, If, Let, Program, StructDef, While,
        },
    };
}

use std::fmt::Debug;

pub fn compare(lhs: &dyn Debug, rhs: &dyn Debug) -> bool {
    let lhs_fmt = format!("{:?}", lhs);
    let rhs_fmt = format!("{:?}", rhs);

    lhs_fmt == rhs_fmt
}
