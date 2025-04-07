use std::fmt::Display;

use crate::ast::Expression;
use crate::codegen::{TypedValue, ZERO_NAME};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::LLVMBuildLoad2;
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

impl Expression for Variable {
    fn codegen(
        &self,
        cxt: &mut crate::codegen::CodegenContext,
    ) -> anyhow::Result<crate::codegen::TypedValue> {
        if !self.fields.is_empty() {
            anyhow::bail!("Custom types and therefore fields unsupported yet");
        }

        // Hack from prev version
        // Where we load local var into reg because of usage semantics
        // We probably have to fix this before custom structs

        let var = cxt.vislayers.get_variable(&self.name);
        if var.is_none() {
            anyhow::bail!("Unknown variable {}", self.name);
        }
        let var = var.unwrap();

        let value =
            unsafe { LLVMBuildLoad2(cxt.builder, var.ty.llvm_type(cxt), var.llvm_val, ZERO_NAME) };

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
