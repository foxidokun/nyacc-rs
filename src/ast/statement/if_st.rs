use crate::ast::{Expression, Statement};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct If {
    pub check: Box<dyn Expression>,
    pub true_body: Vec<Box<dyn Statement>>,
    pub else_body: Option<Vec<Box<dyn Statement>>>,
}

impl Statement for If {}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn only_if() {
        check_ast!(
            StatementParser,
            "if ((2 > 3) + 3) {1;}",
            ast_node!(
                If,
                ast_node!(
                    Arithmetic,
                    ast_node!(
                        Compare,
                        ast_node!(Int, 2),
                        Comparator::GT,
                        ast_node!(Int, 3)
                    ),
                    OpType::Add,
                    ast_node!(Int, 3)
                ),
                vec![ast_node!(ExprStatement, ast_node!(Int, 1))],
                None
            )
        );

        check_ast!(
            StatementParser,
            "if (1) {}",
            ast_node!(If, ast_node!(Int, 1), vec![], None)
        );
    }

    #[test]
    fn if_else() {
        check_ast!(
            StatementParser,
            "if (1) {} else {}",
            ast_node!(If, ast_node!(Int, 1), vec![], Some(vec![]))
        );
    }
}
