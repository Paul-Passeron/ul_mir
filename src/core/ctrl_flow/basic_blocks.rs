use crate::core::ctrl_flow::{BlockId, LocalId, Operand, Place, RValue};

pub enum Terminator {
    Return(Option<Operand>),
    Goto(BlockId),
    Br {
        discriminant: Operand,
        then_dst: BlockId,
        else_dst: BlockId,
    },
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign(Place, RValue),
    StorageLive(LocalId),
    StorageDead(LocalId),
    Nop,
}
