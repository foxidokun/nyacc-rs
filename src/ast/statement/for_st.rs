use crate::ast::{Expression, Statement};
use crate::codegen::macros::c_str;
use crate::codegen::{CodegenContext, bool_from_value};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{
    LLVMAppendBasicBlockInContext, LLVMBuildBr, LLVMBuildCondBr, LLVMPositionBuilderAtEnd,
};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct For {
    pub start: Box<dyn Statement>,
    pub check: Box<dyn Expression>,
    pub step: Box<dyn Statement>,
    pub body: Vec<Box<dyn Statement>>,
}

impl Statement for For {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let loopst = Loop {
            start: Some(self.start.as_ref()),
            check: self.check.as_ref(),
            step: Some(self.step.as_ref()),
            body: &self.body,
        };

        codegen_loop(cxt, &loopst)
    }
}

pub(super) struct Loop<'a> {
    pub start: Option<&'a dyn Statement>,
    pub check: &'a dyn Expression,
    pub step: Option<&'a dyn Statement>,
    pub body: &'a Vec<Box<dyn Statement>>,
}

pub(super) fn codegen_loop(cxt: &mut CodegenContext, loopst: &Loop) -> anyhow::Result<()> {
    cxt.vislayers.enter_layer();
    let cur_func = cxt.vislayers.cur_fun().unwrap().0;

    let check_block = unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"check")) };
    let loop_block = unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"loop")) };
    let cont_block = unsafe { LLVMAppendBasicBlockInContext(cxt.cxt, cur_func, c_str!(c"cont")) };

    if let Some(start) = loopst.start {
        start.codegen(cxt)?;
    }

    unsafe {
        LLVMBuildBr(cxt.builder, check_block);
        LLVMPositionBuilderAtEnd(cxt.builder, check_block);
    }

    let check_expr = loopst.check.codegen(cxt)?;
    let check_expr = bool_from_value(cxt, &check_expr);

    unsafe {
        LLVMBuildCondBr(cxt.builder, check_expr.value, loop_block, cont_block);
        LLVMPositionBuilderAtEnd(cxt.builder, loop_block);
    }

    for st in loopst.body {
        st.codegen(cxt)?;
    }
    if let Some(step) = loopst.step {
        step.codegen(cxt)?;
    }

    unsafe {
        LLVMBuildBr(cxt.builder, check_block);
        LLVMPositionBuilderAtEnd(cxt.builder, cont_block);
    }

    cxt.vislayers.exit_layer();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn existing_val_empty() {
        check_ast!(
            StatementParser,
            "for (a = 3; a < 100; a = a * 2) {}",
            ast_node!(
                For,
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(Int, 3)
                ),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![]
            )
        );
    }

    #[test]
    fn existing_val_body() {
        check_ast!(
            StatementParser,
            "for (a = 3; a < 100; a = a * 2) {a = 3; a = 7;}",
            ast_node!(
                For,
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(Int, 3)
                ),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 3)
                    ),
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 7)
                    )
                ]
            )
        );
    }

    #[test]
    fn new_val() {
        check_ast!(
            StatementParser,
            "for (let a: u8 = 3; a < 100; a = a * 2) {a = 3; a = 7;}",
            ast_node!(
                For,
                ast_node!(Let, "a".into(), Some("u8".into()), ast_node!(Int, 3)),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 3)
                    ),
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 7)
                    )
                ]
            )
        );
    }
}
