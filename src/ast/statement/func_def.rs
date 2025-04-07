use crate::ast::{Statement, TypedArg};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FuncDef {
    pub name: String,
    pub args: Vec<TypedArg>,
    pub rettype: String,
}

impl Statement for FuncDef {
    fn codegen(&self, _: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        // No codegen needed, because it's just a definition
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::TypedArg;
    use crate::ast::macros::{ast_node, check_ast};
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
                ],
                "void".into()
            )
        );
    }

    #[test]
    fn empty() {
        check_ast!(
            ProgramBlockParser,
            "fn foo();",
            ast_node!(FuncDef, "foo".into(), vec![], "void".into())
        );
    }

    #[test]
    fn nonvoid_ret() {
        check_ast!(
            ProgramBlockParser,
            "fn foo() -> S;",
            ast_node!(FuncDef, "foo".into(), vec![], "S".into())
        );
    }
}
