use crate::visitor::Acceptor;
use std::fmt::Debug;

pub mod expression;
pub mod statement;

pub trait Expression: Acceptor + Debug {}

pub trait Statement: Acceptor + Debug {}

#[derive(Debug)]
pub struct TypedArg {
    name: String,
    tp: String,
}

impl TypedArg {
    pub fn new(name: String, tp: String) -> Self {
        Self { name, tp }
    }
}

#[derive(Debug)]
pub enum Comparator {
    LE,
    GE,
    LT,
    GT,
    EQ,
    NE,
}

#[derive(Debug)]
pub enum OpType {
    Mul,
    Div,
    Add,
    Sub,
}

#[cfg(test)]
mod macros {
    macro_rules! check_ast {
        ($parser:tt, $input:expr, $expected:expr) => {{
            use crate::utils::compare;

            let res = crate::grammar::$parser::new().parse($input);
            if let Err(e) = &res {
                eprintln!("Failed with err {:?}", e);
                assert!(false);
            }
            let res = res.unwrap();

            // We can tolerate 1 unused allocation for root node in tests
            #[allow(unused_allocation)]
            if !compare(res.as_ref(), $expected.as_ref()) {
                eprintln!("!! AST MISMATCH !!");
                eprintln!("Expected:\n\t{:?}", $expected);
                eprintln!("Got:\n\t{:?}", res.as_ref());

                assert!(false);
            }
        }};
    }

    macro_rules! ast_node {
        ($tp:ty, $($args:expr),+) => {
            Box::new(<$tp>::new($($args),+))
        }
    }

    pub(super) use {ast_node, check_ast};
}
