use nyacc_proc::acceptor_func;

use crate::utils::nodes::*;

pub trait Visitor {
    acceptor_func!(Assignment);
    acceptor_func!(For);
    acceptor_func!(FuncDef);
    acceptor_func!(FuncImpl);
    acceptor_func!(If);
    acceptor_func!(Let);
    acceptor_func!(Program);
    acceptor_func!(StructDef);
    acceptor_func!(While);
    acceptor_func!(Arithmetic);
    acceptor_func!(Compare);
    acceptor_func!(Int);
    acceptor_func!(Float);
    acceptor_func!(FunctionCall);
    acceptor_func!(Not);
    acceptor_func!(UnaryMinus);
    acceptor_func!(Variable);
    acceptor_func!(ExprStatement);
}

pub trait Acceptor {
    fn accept(&self, visitor: &mut dyn Visitor);
}
