use std::collections::HashMap;
use std::ffi::{CString, c_int};

use crate::ast::{Statement, TypedArg};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMStructCreateNamed, LLVMStructSetBody};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<TypedArg>,
}

impl Statement for StructDef {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let mut llvm_types = Vec::new();
        let mut fields = HashMap::new();

        for (idx, field) in self.fields.iter().enumerate() {
            let ty = cxt.definitions.get_type(&field.tp);
            if ty.is_none() {
                anyhow::bail!("Unknown type as field {} in type {}", field.name, self.name);
            }
            let ty = ty.unwrap();
            let llvm_ty = ty.llvm_type(cxt);

            /* Register field */
            llvm_types.push(llvm_ty);
            fields.insert(field.name.clone(), (idx, ty));
        }

        /* Register in llvm
         * Should be after all checks
         */
        let own_name = CString::new(self.name.clone()).unwrap();
        let new_llvm_type = unsafe { LLVMStructCreateNamed(cxt.cxt, own_name.as_ptr()) };
        assert!(!new_llvm_type.is_null());
        unsafe {
            LLVMStructSetBody(
                new_llvm_type,
                llvm_types.as_mut_ptr(),
                llvm_types.len() as u32,
                false as c_int,
            )
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::TypedArg;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn empty_type() {
        check_ast!(
            ProgramBlockParser,
            "struct S {}",
            ast_node!(StructDef, "S".into(), vec![])
        )
    }

    #[test]
    fn normal() {
        check_ast!(
            ProgramBlockParser,
            "struct S {a : t1, b : t2}",
            ast_node!(
                StructDef,
                "S".into(),
                vec![
                    TypedArg::new("a".into(), "t1".into()),
                    TypedArg::new("b".into(), "t2".into())
                ]
            )
        )
    }

    #[test]
    fn trailing_comma() {
        check_ast!(
            ProgramBlockParser,
            "struct S {a : t1, b : t2 ,}",
            ast_node!(
                StructDef,
                "S".into(),
                vec![
                    TypedArg::new("a".into(), "t1".into()),
                    TypedArg::new("b".into(), "t2".into())
                ]
            )
        )
    }
}
