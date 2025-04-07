use crate::ast::{Expression, Statement};
use crate::codegen::cast;
use crate::codegen::macros::c_str;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{
    LLVMAppendBasicBlockInContext, LLVMBuildRet, LLVMBuildRetVoid, LLVMPositionBuilderAtEnd,
};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Return {
    pub expr: Option<Box<dyn Expression>>,
}

impl Statement for Return {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let cur_func = cxt.vislayers.cur_fun().unwrap().0;

        if self.expr.is_none() {
            unsafe { LLVMBuildRetVoid(cxt.builder) };
            return Ok(());
        }

        let rettype = cxt.vislayers.cur_fun().unwrap().1.clone();
        let expr = self.expr.as_ref().unwrap().codegen(cxt)?;
        let expr = cast(cxt, &expr.ty, &rettype, expr.value);

        unsafe { LLVMBuildRet(cxt.builder, expr) };

        unsafe {
            let unreach_block =
                LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"unreachable"));
            LLVMPositionBuilderAtEnd(cxt.builder, unreach_block);
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn retval() {
        check_ast!(
            StatementParser,
            "return 1;",
            ast_node!(Return, Some(ast_node!(Int, 1)))
        );
    }

    #[test]
    fn retvoid() {
        check_ast!(StatementParser, "return ;", ast_node!(Return, None));
    }
}
