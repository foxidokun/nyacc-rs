use crate::ast::{Expression, Statement};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct ExprStatement {
    pub expr: Box<dyn Expression>,
}

impl Statement for ExprStatement {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        self.expr.codegen(cxt)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::OpType;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn variable() {
        check_ast!(
            StatementParser,
            "a;",
            ast_node!(ExprStatement, ast_node!(Variable, "a".into(), vec![]))
        );
    }

    #[test]
    fn arithmetic() {
        check_ast!(
            StatementParser,
            "1 / 2;",
            ast_node!(
                ExprStatement,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 1),
                    OpType::Div,
                    ast_node!(Int, 2)
                )
            )
        );
    }
}
