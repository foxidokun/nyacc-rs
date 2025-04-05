use crate::ast::{Statement, TypedArg};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FuncDef {
    name: String,
    args: Vec<TypedArg>,
}

impl Statement for FuncDef {}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType, TypedArg};
    use crate::utils::nodes::*;

    #[test]
    fn with_args() {
        check_ast!(
            ProgramBlockParser,
            "fn foo(a: type1, b:type2);",
            ast_node!(
                FuncDef,
                "foo".into(),
                vec![
                    TypedArg::new("a".into(), "type1".into()),
                    TypedArg::new("b".into(), "type2".into()),
                ]
            )
        );
    }

    #[test]
    fn empty() {
        check_ast!(
            ProgramBlockParser,
            "fn foo();",
            ast_node!(FuncDef, "foo".into(), vec![])
        );
    }
}
