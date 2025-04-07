use crate::ast::{Expression, Statement};
use crate::codegen::bool_from_value;
use crate::codegen::macros::c_str;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{
    LLVMAppendBasicBlockInContext, LLVMBuildBr, LLVMBuildCondBr, LLVMPositionBuilderAtEnd,
};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct If {
    pub check: Box<dyn Expression>,
    pub true_body: Vec<Box<dyn Statement>>,
    pub else_body: Option<Vec<Box<dyn Statement>>>,
}

impl Statement for If {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let cur_func = cxt.vislayers.cur_fun().unwrap().0;

        let check_block =
            unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"check")) };
        let true_block =
            unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"true_block")) };
        let not_true_block = unsafe {
            LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"false_cont_block"))
        };

        unsafe {
            LLVMBuildBr(cxt.builder, check_block);
            LLVMPositionBuilderAtEnd(cxt.builder, check_block);
        }

        let check_expr = self.check.codegen(cxt)?;
        let check_expr = bool_from_value(cxt, &check_expr);

        unsafe {
            LLVMBuildCondBr(cxt.builder, check_expr.value, true_block, not_true_block);
            LLVMPositionBuilderAtEnd(cxt.builder, true_block);
        }

        cxt.vislayers.enter_layer();
        // -- Codegen true branch
        for st in &self.true_body {
            st.codegen(cxt)?;
        }
        cxt.vislayers.exit_layer();

        // -- Codegen false branch if it exists
        if let Some(false_body) = &self.else_body {
            let cont_block =
                unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"cont_block")) };

            // -- Br true -> cont
            unsafe { LLVMBuildBr(cxt.builder, cont_block) };

            // -- Codegen else branch
            unsafe { LLVMPositionBuilderAtEnd(cxt.builder, not_true_block) };

            cxt.vislayers.enter_layer();
            for st in false_body {
                st.codegen(cxt)?;
            }
            cxt.vislayers.exit_layer();

            // -- Br false -> cont
            unsafe { LLVMBuildBr(cxt.builder, cont_block) };

            // -- Set cursor to cont
            unsafe { LLVMPositionBuilderAtEnd(cxt.builder, cont_block) };
        } else {
            // -- Br true -> cont
            unsafe { LLVMBuildBr(cxt.builder, not_true_block) };

            // -- Set cursor to cont
            unsafe { LLVMPositionBuilderAtEnd(cxt.builder, not_true_block) };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn only_if() {
        check_ast!(
            StatementParser,
            "if ((2 > 3) + 3) {1;}",
            ast_node!(
                If,
                ast_node!(
                    Arithmetic,
                    ast_node!(
                        Compare,
                        ast_node!(Int, 2),
                        Comparator::GT,
                        ast_node!(Int, 3)
                    ),
                    OpType::Add,
                    ast_node!(Int, 3)
                ),
                vec![ast_node!(ExprStatement, ast_node!(Int, 1))],
                None
            )
        );

        check_ast!(
            StatementParser,
            "if (1) {}",
            ast_node!(If, ast_node!(Int, 1), vec![], None)
        );
    }

    #[test]
    fn if_else() {
        check_ast!(
            StatementParser,
            "if (1) {} else {}",
            ast_node!(If, ast_node!(Int, 1), vec![], Some(vec![]))
        );
    }
}
