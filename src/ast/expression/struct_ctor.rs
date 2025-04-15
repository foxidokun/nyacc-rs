use crate::ast::Expression;
use crate::codegen::{TypedValue, ZERO_NAME};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMBuildAlloca, LLVMBuildLoad2, LLVMBuildMemSet, LLVMConstInt, LLVMGetEntryBasicBlock, LLVMGetInsertBlock, LLVMIntTypeInContext, LLVMPositionBuilderAtEnd, LLVMSizeOf};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct StructCtor {
    pub name: String
}

impl Expression for StructCtor {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let cur_func = cxt.vislayers.cur_fun().unwrap().0;
        let entry_block = unsafe { LLVMGetEntryBasicBlock(cur_func) };
        assert!(!entry_block.is_null());
        let current_block = unsafe { LLVMGetInsertBlock(cxt.builder) };
        assert!(!current_block.is_null());

        let ty = cxt.definitions.get_type(&self.name);
        if ty.is_none() {
            anyhow::bail!("Ctor for unknown type {}", self.name);
        }
        let ty = ty.unwrap();
        let llvm_ty = ty.llvm_type(cxt);

        /* Create alloca in entry block */
        unsafe { LLVMPositionBuilderAtEnd(cxt.builder, entry_block) };
        let alloca = unsafe { LLVMBuildAlloca(cxt.builder, llvm_ty, ZERO_NAME) };
        assert!(!alloca.is_null());
        unsafe { LLVMPositionBuilderAtEnd(cxt.builder, current_block) };

        /* Zero init variable */
        unsafe {
            let res = LLVMBuildMemSet(
                cxt.builder,
                alloca,
                LLVMConstInt(LLVMIntTypeInContext(cxt.cxt, 64), 0, 0),
                LLVMSizeOf(llvm_ty),
                1
            );
            assert!(!res.is_null());
        }

        /* Load because of expression semantics */
        let value = unsafe { LLVMBuildLoad2(cxt.builder, llvm_ty, alloca, ZERO_NAME) };
        assert!(!value.is_null());

        Ok(TypedValue {
            value,
            ty
        })
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
            "S {}",
            ast_node!(
                StructCtor,
                "S".into()
            )
        )
    }

    #[test]
    fn complex_args() {
        check_ast!(
            ExprParser,
            "S {}",
            ast_node!(
                StructCtor,
                "S".into()
            )
        )
    }
}
