use std::collections::HashMap;

use crate::core::types::{MirType, function::FunctionType};

// SSA value
pub type LocalId = u32;

// Basic block ID
pub type BlockId = u32;

// Function ID
pub type MirFunId = u32;

pub enum Statement {
    Assign(Place, RValue),
    StorageLive(LocalId),
    StorageDead(LocalId),
    Nop,
}

pub enum Terminator {
    Return(Option<Operand>),
    Goto(BlockId),
    Br {
        discriminant: Operand,
        then_dst: BlockId,
        else_dst: BlockId,
    },
}

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
    Int(i64, MirType),
    Bool(bool),
    Str(String),
    Null,
}

pub enum FunctionBody {
    External,
    Defined(FunctionData),
}

pub struct FunctionData {
    pub params: Vec<LocalId>,
    pub locals: Vec<(LocalId, MirType)>,
    pub blocks: HashMap<BlockId, Option<BasicBlock>>,
    pub entry_block: BlockId,
}

pub struct Function {
    pub name: String,
    pub ty: FunctionType,
    pub body: FunctionBody,
}

impl Function {
    pub fn get_function_data(&self) -> Option<&FunctionData> {
        match &self.body {
            FunctionBody::Defined(data) => Some(data),
            FunctionBody::External => None,
        }
    }

    pub fn get_function_data_unchecked(&self) -> &FunctionData {
        match &self.body {
            FunctionBody::Defined(data) => data,
            FunctionBody::External => panic!("Cannot get function data for external function"),
        }
    }

    pub fn get_function_data_mut(&mut self) -> Option<&mut FunctionData> {
        match &mut self.body {
            FunctionBody::Defined(data) => Some(data),
            FunctionBody::External => None,
        }
    }

    pub fn get_function_data_mut_unchecked(&mut self) -> &mut FunctionData {
        match &mut self.body {
            FunctionBody::Defined(data) => data,
            FunctionBody::External => panic!("Cannot get function data for external function"),
        }
    }
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}
