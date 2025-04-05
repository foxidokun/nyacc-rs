use crate::ast::Expression;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct FunctionCall {
    name: String,
    args: Vec<Box<dyn Expression>>,
}

impl Expression for FunctionCall {}

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
            "func()",
            ast_node!(FunctionCall, "func".into(), vec![])
        );

        check_ast!(
            ExprParser,
            "func(a, b, c)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(Variable, "a".into(), vec![]),
                    ast_node!(Variable, "b".into(), vec![]),
                    ast_node!(Variable, "c".into(), vec![])
                ]
            )
        );
    }

    #[test]
    fn complex_expr_in_args() {
        check_ast!(
            ExprParser,
            "func(a.field, b.field, c.delta.delta.strikh)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(Variable, "a".into(), vec!["field".into()]),
                    ast_node!(Variable, "b".into(), vec!["field".into()]),
                    ast_node!(
                        Variable,
                        "c".into(),
                        vec!["delta".into(), "delta".into(), "strikh".into()]
                    )
                ]
            )
        );

        check_ast!(
            ExprParser,
            "func(1 + 2, 3 * 4, 1 != 3)",
            ast_node!(
                FunctionCall,
                "func".into(),
                vec![
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 1),
                        OpType::Add,
                        ast_node!(Int, 2)
                    ),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Int, 3),
                        OpType::Mul,
                        ast_node!(Int, 4)
                    ),
                    ast_node!(
                        Compare,
                        ast_node!(Int, 1),
                        crate::ast::Comparator::NE,
                        ast_node!(Int, 3)
                    )
                ]
            )
        );
    }
}
