use crate::ast::{Comparator, Expression};
use crate::codegen::{Type, ZERO_NAME, bool_from_llvm, cast};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Compare {
    pub lhs: Box<dyn Expression>,
    pub cmp: Comparator,
    pub rhs: Box<dyn Expression>,
}

impl Expression for Compare {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let lhs_tv = self.lhs.codegen(cxt)?;
        let rhs_tv = self.rhs.codegen(cxt)?;

        let common_type = Type::common_type(&lhs_tv.ty, &rhs_tv.ty)?;

        let lhs = cast(cxt, &lhs_tv.ty, &common_type, lhs_tv.value);
        let rhs = cast(cxt, &rhs_tv.ty, &common_type, rhs_tv.value);

        macro_rules! dispatch_binop {
            ($([$op:tt, $float_pred:tt, $int_pred:tt ]),+) => {
                match self.cmp {
                $(
                    Comparator::$op => {
                        let res = match common_type.as_ref() {
                            Type::Int(_) => {
                                unsafe {llvm_sys::core::LLVMBuildICmp(
                                    cxt.builder,
                                    llvm_sys::LLVMIntPredicate::$int_pred,
                                    lhs,
                                    rhs,
                                    ZERO_NAME
                                )}
                            },
                            Type::Float(_) => {
                                unsafe {llvm_sys::core::LLVMBuildFCmp(
                                    cxt.builder,
                                    llvm_sys::LLVMRealPredicate::$float_pred,
                                    lhs,
                                    rhs,
                                    ZERO_NAME
                                )}
                            },
                            _ => { panic!("This kind of errors should be catched during Type::common_type call") }
                        };
                        assert!(!res.is_null(), "Failed to compare (cmp: {}) args types: {} {}", Comparator::$op, lhs_tv.ty, rhs_tv.ty);
                        res
                    },
                )+
                }
            };
        }

        let cmp_res = dispatch_binop!(
            [LE, LLVMRealOLE, LLVMIntSLE],
            [GE, LLVMRealOGE, LLVMIntSGE],
            [LT, LLVMRealOLT, LLVMIntSLT],
            [GT, LLVMRealOGT, LLVMIntSGT],
            [EQ, LLVMRealOEQ, LLVMIntEQ],
            [NE, LLVMRealONE, LLVMIntNE]
        );

        Ok(bool_from_llvm(cxt, cmp_res))
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
            "12 > 3",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::GT,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "12 < 3",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::LT,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "12 == (3 + 4)",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::EQ,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 3),
                    OpType::Add,
                    ast_node!(Int, 4)
                )
            )
        );
    }

    #[test]
    fn brackets() {
        check_ast!(
            ExprParser,
            "(12 < 4) == (3 + 4)",
            ast_node!(
                Compare,
                ast_node!(
                    Compare,
                    ast_node!(Int, 12),
                    Comparator::LT,
                    ast_node!(Int, 4)
                ),
                Comparator::EQ,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 3),
                    OpType::Add,
                    ast_node!(Int, 4)
                )
            )
        );
    }
}
