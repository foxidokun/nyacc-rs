use crate::ast::Expression;
use crate::codegen::ZERO_NAME;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMBuildFNeg, LLVMBuildNeg};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct UnaryMinus {
    pub expr: Box<dyn Expression>,
}

impl Expression for UnaryMinus {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let mut expr = self.expr.codegen(cxt)?;

        expr.value = match expr.ty.as_ref() {
            crate::codegen::Type::Float(_) => unsafe {
                LLVMBuildFNeg(cxt.builder, expr.value, ZERO_NAME)
            },
            crate::codegen::Type::Int(_) => unsafe {
                LLVMBuildNeg(cxt.builder, expr.value, ZERO_NAME)
            },
            _ => anyhow::bail!("Unary minus on unsupported type {}", expr.ty),
        };

        assert!(!expr.value.is_null());

        Ok(expr)
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
        check_ast!(ExprParser, "-2", ast_node!(UnaryMinus, ast_node!(Int, 2)));

        check_ast!(
            ExprParser,
            "-(1 + 2)",
            ast_node!(
                UnaryMinus,
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
            "-2 + 3",
            ast_node!(
                Arithmetic,
                ast_node!(UnaryMinus, ast_node!(Int, 2)),
                OpType::Add,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "3 + -2",
            ast_node!(
                Arithmetic,
                ast_node!(Int, 3),
                OpType::Add,
                ast_node!(UnaryMinus, ast_node!(Int, 2))
            )
        );
    }
}
