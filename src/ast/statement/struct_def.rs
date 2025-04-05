use crate::ast::{Statement, TypedArg};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<TypedArg>,
}

impl Statement for StructDef {}

#[cfg(test)]
mod tests {
    use crate::ast::TypedArg;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn empty_type() {
        check_ast!(
            ProgramBlockParser,
            "struct S {}",
            ast_node!(StructDef, "S".into(), vec![])
        )
    }

    #[test]
    fn normal() {
        check_ast!(
            ProgramBlockParser,
            "struct S {a : t1, b : t2}",
            ast_node!(
                StructDef,
                "S".into(),
                vec![
                    TypedArg::new("a".into(), "t1".into()),
                    TypedArg::new("b".into(), "t2".into())
                ]
            )
        )
    }

    #[test]
    fn trailing_comma() {
        check_ast!(
            ProgramBlockParser,
            "struct S {a : t1, b : t2 ,}",
            ast_node!(
                StructDef,
                "S".into(),
                vec![
                    TypedArg::new("a".into(), "t1".into()),
                    TypedArg::new("b".into(), "t2".into())
                ]
            )
        )
    }
}
