use std::ffi::CString;

use crate::ast::Expression;
use crate::codegen::{TypedValue, ZERO_NAME, cast};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMBuildCall2, LLVMGetNamedFunction};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Box<dyn Expression>>,
}

impl Expression for FunctionCall {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let func_type = cxt.definitions.get_func(&self.name);
        if func_type.is_none() {
            anyhow::bail!("Calling unknown function {}", self.name);
        }

        let func_type = func_type.unwrap().clone();

        let func_name = CString::new(self.name.clone()).unwrap();
        let func_object = unsafe { LLVMGetNamedFunction(cxt.module, func_name.as_ptr()) };
        assert!(!func_object.is_null()); // It exists because we founded it in cxt.definitions

        let mut computed_arg = Vec::with_capacity(func_type.0.len());
        for (i, arg) in self.args.iter().enumerate() {
            let argval = arg.codegen(cxt)?;
            let casted = cast(cxt, &argval.ty, &func_type.0[i], argval.value);
            computed_arg.push(casted);
        }

        let llvm_func_type = cxt.type_cache.get_func(&self.name).unwrap();

        let call = unsafe {
            LLVMBuildCall2(
                cxt.builder,
                *llvm_func_type,
                func_object,
                computed_arg.as_mut_ptr(),
                computed_arg.len() as u32,
                ZERO_NAME,
            )
        };
        assert!(!call.is_null());

        Ok(TypedValue {
            value: call,
            ty: func_type.1.clone(),
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
            "func()",
            ast_node!(FunctionCall, "func".into(), vec![])
        );

        check_ast!(
            ExprParser,
            "func(a, b, c)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(Variable, "a".into(), vec![]),
                    ast_node!(Variable, "b".into(), vec![]),
                    ast_node!(Variable, "c".into(), vec![])
                ]
            )
        );
    }

    #[test]
    fn complex_expr_in_args() {
        check_ast!(
            ExprParser,
            "func(a.field, b.field, c.delta.delta.strikh)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(Variable, "a".into(), vec!["field".into()]),
                    ast_node!(Variable, "b".into(), vec!["field".into()]),
                    ast_node!(
                        Variable,
                        "c".into(),
                        vec!["delta".into(), "delta".into(), "strikh".into()]
                    )
                ]
            )
        );

        check_ast!(
            ExprParser,
            "func(1 + 2, 3 * 4, 1 != 3)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 1),
                        OpType::Add,
                        ast_node!(Int, 2)
                    ),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 3),
                        OpType::Mul,
                        ast_node!(Int, 4)
                    ),
                    ast_node!(
                        Compare,
                        ast_node!(Int, 1),
                        crate::ast::Comparator::NE,
                        ast_node!(Int, 3)
                    )
                ]
            )
        );
    }
}
