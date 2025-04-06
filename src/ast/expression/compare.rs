use crate::ast::{Comparator, Expression};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Compare {
    pub lhs: Box<dyn Expression>,
    pub cmp: Comparator,
    pub rhs: Box<dyn Expression>,
}

impl Expression for Compare {}

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
            "12 > 3",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::GT,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "12 < 3",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::LT,
                ast_node!(Int, 3)
            )
        );

        check_ast!(
            ExprParser,
            "12 == (3 + 4)",
            ast_node!(
                Compare,
                ast_node!(Int, 12),
                Comparator::EQ,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 3),
                    OpType::Add,
                    ast_node!(Int, 4)
                )
            )
        );
    }

    #[test]
    fn brackets() {
        check_ast!(
            ExprParser,
            "(12 < 4) == (3 + 4)",
            ast_node!(
                Compare,
                ast_node!(
                    Compare,
                    ast_node!(Int, 12),
                    Comparator::LT,
                    ast_node!(Int, 4)
                ),
                Comparator::EQ,
                ast_node!(
                    Arithmetic,
                    ast_node!(Int, 3),
                    OpType::Add,
                    ast_node!(Int, 4)
                )
            )
        );
    }
}
