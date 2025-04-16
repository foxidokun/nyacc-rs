use crate::ast::{Expression, Statement};
use crate::codegen::{TypedValue, ZERO_NAME, cast};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{
    LLVMBuildAlloca, LLVMBuildStore, LLVMGetEntryBasicBlock, LLVMGetInsertBlock,
    LLVMPositionBuilderAtEnd,
};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Let {
    pub var: String,
    pub tp: Option<String>,
    pub expr: Box<dyn Expression>,
}

impl Statement for Let {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let cur_func = cxt.vislayers.cur_fun().unwrap().0;
        let entry_block = unsafe { LLVMGetEntryBasicBlock(cur_func) };
        assert!(!entry_block.is_null());
        let current_block = unsafe { LLVMGetInsertBlock(cxt.builder) };
        assert!(!current_block.is_null());

        // Codegen expr

        let mut expr = self.expr.codegen(cxt)?;
        if let Some(typename) = &self.tp {
            let ty = cxt.definitions.get_type(typename);
            if ty.is_none() {
                anyhow::bail!("Unknown type {} in let statement", typename);
            }
            let ty = ty.unwrap();
            expr.value = cast(cxt, &expr.ty, &ty, expr.value);
            expr.ty = ty;
        }

        // Codegen alloca in entry block
        unsafe { LLVMPositionBuilderAtEnd(cxt.builder, entry_block) };
        let alloca = unsafe { LLVMBuildAlloca(cxt.builder, expr.ty.llvm_type(cxt), ZERO_NAME) };
        assert!(!alloca.is_null());

        // Return into normal block
        unsafe { LLVMPositionBuilderAtEnd(cxt.builder, current_block) };
        unsafe { LLVMBuildStore(cxt.builder, expr.value, alloca) };

        // -- Remember var
        cxt.vislayers.add_variable(
            self.var.clone(),
            TypedValue {
                value: alloca,
                ty: expr.ty,
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn no_type() {
        check_ast!(
            StatementParser,
            "let a = 1;",
            ast_node!(Let, "a".into(), None, ast_node!(Int, 1))
        );
    }

    #[test]
    fn with_type() {
        check_ast!(
            StatementParser,
            "let a: u8 = 1;",
            ast_node!(Let, "a".into(), Some("u8".into()), ast_node!(Int, 1))
        );
    }
}
