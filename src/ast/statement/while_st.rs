use crate::{
    ast::{Expression, Statement},
    visitor::{Acceptor, Visitor},
};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct While {
    cond: Box<dyn Expression>,
    body: Vec<Box<dyn Statement>>,
}

impl Statement for While {}

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
