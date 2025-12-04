use crate::core::{
    ctrl_flow::function::MirFunId,
    types::{MirType, int::IntType},
};

pub mod basic_blocks;
pub mod function;

// SSA value
pub type LocalId = u32;

// Basic block ID
pub type BlockId = u32;

// Result of computation
pub enum RValue {
    Use(Operand),
    BinOp(BinOp, Operand, Operand),
    UnOp(UnOp, Operand),
    Cast(CastKind, Operand, MirType),
    Ref(Place),
    Len(Place),
    Aggregate(Vec<Operand>),
}

pub enum CastKind {
    PtrToInt,
    IntToPtr,
    Reinterpret,
}

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    ShiftRight,
    ShiftLeft,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Leq,
}

pub enum UnOp {
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    Local(LocalId),
    Projection(Box<Place>, ProjectionKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    Deref,
    Field(u32),
    Index(LocalId),
}

#[derive(Clone, Debug)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
    Call { func: MirFunId, args: Vec<Operand> },
}

#[derive(Clone, Debug)]
pub enum Constant {
    Int(i64, IntType),
    Bool(bool),
    Str(String),
    Null,
}
