use crate::ast::{Expression, Statement, expression::Variable};
use crate::codegen::cast;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::LLVMBuildStore;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Assignment {
    pub var: Variable,
    pub expr: Box<dyn Expression>,
}

impl Statement for Assignment {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let var = self.var.codegen_gep(cxt)?;

        let expr = self.expr.codegen(cxt)?;
        let expr = cast(cxt, &expr.ty, &var.ty, expr.value);

        unsafe { LLVMBuildStore(cxt.builder, expr, var.value) };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Comparator;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn simple() {
        check_ast!(
            StatementParser,
            "a = 12;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Int, 12)
            )
        );

        check_ast!(
            StatementParser,
            "a = b;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Variable, "b".into(), vec![])
            )
        );
    }

    #[test]
    fn complex() {
        check_ast!(
            StatementParser,
            "a = b == c;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "b".into(), vec![]),
                    Comparator::EQ,
                    ast_node!(Variable, "c".into(), vec![])
                )
            )
        );
    }
}
