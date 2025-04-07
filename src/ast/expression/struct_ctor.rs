use crate::ast::Expression;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct StructCtor {
    pub name: String,
    pub args: Vec<Box<dyn Expression>>,
}

impl Expression for StructCtor {
    fn codegen(
        &self,
        _: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::OpType;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn simple_expressions() {
        check_ast!(
            ExprParser,
            "S { 1, 2, 3 }",
            ast_node!(
                StructCtor,
                "S".into(),
                vec![ast_node!(Int, 1), ast_node!(Int, 2), ast_node!(Int, 3)]
            )
        )
    }

    #[test]
    fn complex_args() {
        check_ast!(
            ExprParser,
            "S { 1, G {2}, 3 + 2 / (1) }",
            ast_node!(
                StructCtor,
                "S".into(),
                vec![
                    ast_node!(Int, 1),
                    ast_node!(StructCtor, "G".into(), vec![ast_node!(Int, 2)]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 3),
                        OpType::Add,
                        ast_node!(
                            Arithmetic,
                            ast_node!(Int, 2),
                            OpType::Div,
                            ast_node!(Int, 1)
                        )
                    )
                ]
            )
        )
    }
}
