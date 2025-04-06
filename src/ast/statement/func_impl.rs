use std::ffi::CString;

use crate::ast::{Statement, TypedArg};
use crate::codegen::macros::c_str;
use crate::codegen::{Value, ZERO_NAME};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{
    LLVMAppendBasicBlock, LLVMBuildAlloca, LLVMBuildStore, LLVMGetNamedFunction, LLVMGetParam,
    LLVMPositionBuilderAtEnd,
};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FuncImpl {
    pub name: String,
    pub args: Vec<TypedArg>,
    pub rettype: String,
    pub body: Vec<Box<dyn Statement>>,
}

impl Statement for FuncImpl {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let rettype = cxt.definitions.get_type(&self.rettype);
        if rettype.is_none() {
            anyhow::bail!("Unknown rettype {} in function {}", self.rettype, self.name);
        };
        let rettype = rettype.unwrap();

        // -- Enter function
        cxt.vislayers.set_rettype(rettype);
        cxt.vislayers.enter_layer();

        // -- Get function object
        // Note: Definitions should be generated before codegen by compile functions

        let func_name = CString::new(self.name.clone()).unwrap();
        let func = unsafe { LLVMGetNamedFunction(cxt.module, func_name.as_ptr() as *const _) };

        assert!(
            !func.is_null(),
            "Definitions should be generated before codegen by compile functions"
        );

        // -- Create entry block
        let block = unsafe { LLVMAppendBasicBlock(func, c_str!(c"entry")) };
        unsafe {
            LLVMPositionBuilderAtEnd(cxt.builder, block);
        }

        // -- Allocate args

        for (i, arg) in self.args.iter().enumerate() {
            let argtype = cxt.definitions.get_type(&arg.tp);
            if argtype.is_none() {
                anyhow::bail!(
                    "Unknown type {} in func {} argument {}",
                    arg.tp,
                    self.name,
                    i
                );
            }
            let argtype = argtype.unwrap();

            let llvm_ty = argtype.llvm_type(cxt);
            let alloca = unsafe { LLVMBuildAlloca(cxt.builder, llvm_ty, ZERO_NAME) };
            unsafe { LLVMBuildStore(cxt.builder, LLVMGetParam(func, i as u32), alloca) };

            // -- Remember arg
            cxt.vislayers.add_variable(
                arg.name.clone(),
                Value {
                    llvm_val: alloca,
                    ty: argtype,
                },
            );
        }

        // -- Codegen body
        for st in &self.body {
            st.codegen(cxt)?;
        }

        cxt.vislayers.exit_layer();
        cxt.vislayers.clear_rettype();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::TypedArg;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn with_args() {
        check_ast!(
            ProgramBlockParser,
            "fn foo(a: type1, b:type2) {a;}",
            ast_node!(
                FuncImpl,
                "foo".into(),
                vec![
                    TypedArg::new("a".into(), "type1".into()),
                    TypedArg::new("b".into(), "type2".into()),
                ],
                "void".into(),
                vec![ast_node!(
                    ExprStatement,
                    ast_node!(Variable, "a".into(), vec![])
                )]
            )
        );
    }

    #[test]
    fn empty() {
        check_ast!(
            ProgramBlockParser,
            "fn foo() {}",
            ast_node!(FuncImpl, "foo".into(), vec![], "void".into(), vec![])
        );
    }

    #[test]
    fn nonvoid_ret() {
        check_ast!(
            ProgramBlockParser,
            "fn foo() -> S {}",
            ast_node!(FuncImpl, "foo".into(), vec![], "S".into(), vec![])
        );
    }
}
