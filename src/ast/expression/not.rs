use crate::ast::Expression;
use crate::codegen::{ZERO_NAME, bool_from_llvm, bool_from_value};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Not {
    pub expr: Box<dyn Expression>,
}

impl Expression for Not {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let expr_tv = self.expr.codegen(cxt)?;
        let expr = bool_from_value(cxt, &expr_tv)?;

        let cmp_res = unsafe { llvm_sys::core::LLVMBuildNot(cxt.builder, expr.value, ZERO_NAME) };

        Ok(bool_from_llvm(cxt, cmp_res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn simple_expressions() {
        check_ast!(ExprParser, "!2", ast_node!(Not, ast_node!(Int, 2)));

        check_ast!(
            ExprParser,
            "!(1 + 2)",
            ast_node!(
                Not,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 1),
                    OpType::Add,
                    ast_node!(Int, 2)
                )
            )
        );
    }

    #[test]
    fn eval_order() {
        check_ast!(
            ExprParser,
            "!2 > 3",
            ast_node!(
                Compare,
                ast_node!(Not, ast_node!(Int, 2)),
                Comparator::GT,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "!2 + 3",
            ast_node!(
                Arithmetic,
                ast_node!(Not, ast_node!(Int, 2)),
                OpType::Add,
                ast_node!(Int, 3)
            )
        );
    }
}
