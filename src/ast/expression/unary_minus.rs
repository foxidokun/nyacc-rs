use crate::ast::Expression;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct UnaryMinus {
    pub expr: Box<dyn Expression>,
}

impl Expression for UnaryMinus {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::OpType;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn simple_expressions() {
        check_ast!(ExprParser, "-2", ast_node!(UnaryMinus, ast_node!(Int, 2)));

        check_ast!(
            ExprParser,
            "-(1 + 2)",
            ast_node!(
                UnaryMinus,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 1),
                    OpType::Add,
                    ast_node!(Int, 2)
                )
            )
        );
    }

    #[test]
    fn eval_order() {
        check_ast!(
            ExprParser,
            "-2 + 3",
            ast_node!(
                Arithmetic,
                ast_node!(UnaryMinus, ast_node!(Int, 2)),
                OpType::Add,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "3 + -2",
            ast_node!(
                Arithmetic,
                ast_node!(Int, 3),
                OpType::Add,
                ast_node!(UnaryMinus, ast_node!(Int, 2))
            )
        );
    }
}
