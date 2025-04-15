use std::fmt::Display;

use crate::ast::Expression;
use crate::codegen::{CodegenContext, Type, TypedValue, ZERO_NAME};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMBuildGEP2, LLVMBuildLoad2, LLVMConstInt, LLVMIntTypeInContext};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Variable {
    pub name: String,
    pub fields: Vec<String>,
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        for field in &self.fields {
            write!(f, ".{}", field)?;
        }

        Ok(())
    }
}

impl Variable {
    pub fn codegen_gep(&self, cxt: &mut CodegenContext) -> anyhow::Result<TypedValue> {
        let var = cxt.vislayers.get_variable(&self.name);
        if var.is_none() {
            anyhow::bail!("Unknown variable {}", self.name);
        }
        let mut var = var.unwrap();
        let mut indices = vec![unsafe { LLVMConstInt(LLVMIntTypeInContext(cxt.cxt, 32), 0, 0) }];

        let orig_type = var.ty.clone();

        for field_name in &self.fields {
            if let Type::Custom(ty) = var.ty.as_ref() {
                let field = ty.fields.get(field_name);
                if field.is_none() {
                    anyhow::bail!(
                        "unknown field ({}) subscription of variable ({}) with type ({})",
                        field_name,
                        self.name,
                        var.ty
                    );
                }
                let field = field.unwrap();
                indices.push(unsafe {
                    LLVMConstInt(LLVMIntTypeInContext(cxt.cxt, 32), field.0 as u64, 0)
                });
                var.ty = field.1.clone();
            } else {
                anyhow::bail!(
                    "Field ({}) subscription of variable ({}) with primitive type ({})",
                    field_name,
                    self.name,
                    var.ty
                );
            }
        }

        /* Get ptr of field */
        let value = unsafe {
            LLVMBuildGEP2(
                cxt.builder,
                orig_type.llvm_type(cxt),
                var.value,
                indices.as_mut_ptr(),
                indices.len() as u32,
                ZERO_NAME,
            )
        };
        assert!(!value.is_null());

        Ok(TypedValue { value, ty: var.ty })
    }
}

impl Expression for Variable {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        let var = self.codegen_gep(cxt)?;

        /* Load field */
        let value =
            unsafe { LLVMBuildLoad2(cxt.builder, var.ty.llvm_type(cxt), var.value, ZERO_NAME) };

        Ok(TypedValue { value, ty: var.ty })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};

    #[test]
    fn simple() {
        check_ast!(ExprParser, "a", ast_node!(Variable, "a".into(), vec![]));

        check_ast!(ExprParser, "a12", ast_node!(Variable, "a12".into(), vec![]));

        check_ast!(
            ExprParser,
            "a12_lol",
            ast_node!(Variable, "a12_lol".into(), vec![])
        );
    }

    #[test]
    fn fields() {
        check_ast!(
            ExprParser,
            "a.b",
            ast_node!(Variable, "a".into(), vec!["b".into()])
        );

        check_ast!(
            ExprParser,
            "a.b.c.d",
            ast_node!(
                Variable,
                "a".into(),
                vec!["b".into(), "c".into(), "d".into()]
            )
        );

        check_ast!(
            ExprParser,
            "a12.s5",
            ast_node!(Variable, "a12".into(), vec!["s5".into()])
        );
    }
}
