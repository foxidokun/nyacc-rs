use crate::ast::Statement;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Program {
    pub blocks: Vec<Box<dyn Statement>>,
}

impl Statement for Program {
    fn codegen(&self, cxt: &mut crate::codegen::CodegenContext) -> anyhow::Result<()> {
        for block in &self.blocks {
            block.codegen(cxt)?;
        }

        Ok(())
    }
}
