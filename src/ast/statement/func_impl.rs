use crate::ast::{Statement, TypedArg};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FuncImpl {
    name: String,
    args: Vec<TypedArg>,
    body: Vec<Box<dyn Statement>>,
}

impl Statement for FuncImpl {}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType, TypedArg};
    use crate::utils::nodes::*;

    #[test]
    fn with_args() {
        check_ast!(
            ProgramBlockParser,
            "fn foo(a: type1, b:type2) {a;}",
            ast_node!(
                FuncImpl,
                "foo".into(),
                vec![
                    TypedArg::new("a".into(), "type1".into()),
                    TypedArg::new("b".into(), "type2".into()),
                ],
                vec![ast_node!(
                    ExprStatement,
                    ast_node!(Variable, "a".into(), vec![])
                )]
            )
        );
    }

    #[test]
    fn empty() {
        check_ast!(
            ProgramBlockParser,
            "fn foo() {}",
            ast_node!(FuncImpl, "foo".into(), vec![], vec![])
        );
    }
}
