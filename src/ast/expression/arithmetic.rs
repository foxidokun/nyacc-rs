use crate::ast::{Expression, OpType};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Arithmetic {
    pub lhs: Box<dyn Expression>,
    pub op: OpType,
    pub rhs: Box<dyn Expression>,
}

impl Expression for Arithmetic {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn int_as_expr() {
        check_ast!(ExprParser, "12", ast_node!(Int, 12));
    }

    #[test]
    fn simple_expressions() {
        check_ast!(
            ExprParser,
            "12 + 3",
            ast_node!(
                Arithmetic,
                ast_node!(Int, 12),
                OpType::Add,
                ast_node!(Int, 3)
            )
        );
    }

    #[test]
    fn brackets() {
        check_ast!(ExprParser, "(1)", ast_node!(Int, 1));

        check_ast!(
            ExprParser,
            "(1 + (a * 2) / 4)",
            ast_node!(
                Arithmetic,
                ast_node!(Int, 1),
                OpType::Add,
                ast_node!(
                    Arithmetic,
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    ),
                    OpType::Div,
                    ast_node!(Int, 4)
                )
            )
        );
    }

    #[test]
    fn order_of_eval() {
        check_ast!(
            ExprParser,
            "1 + 2 + 3",
            ast_node!(
                Arithmetic,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 1),
                    OpType::Add,
                    ast_node!(Int, 2)
                ),
                OpType::Add,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "1 + 2 * 3",
            ast_node!(
                Arithmetic,
                ast_node!(Int, 1),
                OpType::Add,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 2),
                    OpType::Mul,
                    ast_node!(Int, 3)
                )
            )
        );
    }
}
