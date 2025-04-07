use crate::{
    ast::{Expression, Statement},
    visitor::{Acceptor, Visitor},
};
use derive_new::new;
use nyacc_proc::Acceptor;

use super::for_st::{Loop, codegen_loop};

#[derive(new, Acceptor, Debug)]
pub struct While {
    pub cond: Box<dyn Expression>,
    pub body: Vec<Box<dyn Statement>>,
}

impl Statement for While {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        let loopst = Loop {
            start: None,
            check: self.cond.as_ref(),
            step: None,
            body: &self.body,
        };

        codegen_loop(cxt, &loopst)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn simple() {
        check_ast!(
            StatementParser,
            "while ((1 + 2) > 3) {1;}",
            ast_node!(
                While,
                ast_node!(
                    Compare,
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 1),
                        OpType::Add,
                        ast_node!(Int, 2)
                    ),
                    Comparator::GT,
                    ast_node!(Int, 3)
                ),
                vec![ast_node!(ExprStatement, ast_node!(Int, 1))]
            )
        )
    }
}
