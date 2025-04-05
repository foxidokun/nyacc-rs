use crate::ast::Statement;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Program {
    blocks: Vec<Box<dyn Statement>>,
}
