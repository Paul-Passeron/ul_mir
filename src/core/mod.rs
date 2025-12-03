use std::collections::{HashMap, HashSet};

use crate::core::types::{
    MirType,
    function::FunctionType,
    int::IntType,
    struct_ty::{Struct, StructId},
};

pub mod types;
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
    Iff {
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

pub struct FunExtern;
pub struct FunStatic {
    pub params: Vec<LocalId>,
    pub blocks: HashMap<BlockId, BasicBlock>,
    pub entry_block: Option<BlockId>,
    pub reserved: HashSet<BlockId>,
    pub last_reserved: BlockId,
    pub locals: Vec<(LocalId, MirType)>,
}

pub trait FunctionLinkage {
    fn get_linkage(&self) -> Option<&FunStatic>;
    fn get_linkage_mut(&mut self) -> Option<&mut FunStatic>;
}

impl FunctionLinkage for FunExtern {
    fn get_linkage(&self) -> Option<&FunStatic> {
        None
    }

    fn get_linkage_mut(&mut self) -> Option<&mut FunStatic> {
        None
    }
}

impl Function<dyn FunctionLinkage> {
    pub fn reserve_new_block(&mut self) -> Option<BlockId> {
        let linkage = self.linkage.get_linkage_mut()?;
        let res = linkage.last_reserved;
        linkage.last_reserved += 1;
        linkage.reserved.insert(res);
        Some(res)
    }

    pub fn new_local(&mut self, ty: MirType) -> Option<LocalId> {
        let min = self
            .linkage
            .get_linkage()?
            .locals
            .iter()
            .map(|x| x.0)
            .max_by(u32::cmp)
            .unwrap_or(0);
        let res = min + 1;
        self.linkage.get_linkage_mut()?.locals.push((res, ty));
        Some(res)
    }
}

impl FunctionLinkage for FunStatic {
    fn get_linkage(&self) -> Option<&FunStatic> {
        Some(self)
    }

    fn get_linkage_mut(&mut self) -> Option<&mut FunStatic> {
        Some(self)
    }
}

pub struct Function<T>
where
    T: FunctionLinkage + ?Sized,
{
    pub name: String,
    pub ty: FunctionType,
    pub linkage: Box<T>,
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub struct Context {
    pub functions: HashMap<MirFunId, Function<dyn FunctionLinkage>>,
    pub structs: HashMap<StructId, Struct>,
    pub entry_point: Option<MirFunId>,
    pub ptr_size: PtrSize,
}

#[derive(Clone, Copy, Debug)]
pub enum PtrSize {
    _8Bit,
    _16Bit,
    _32Bit,
    _64Bit,
}

impl PtrSize {
    pub fn corresponding_int(&self) -> IntType {
        match self {
            PtrSize::_8Bit => IntType::U8,
            PtrSize::_16Bit => IntType::U16,
            PtrSize::_32Bit => IntType::U32,
            PtrSize::_64Bit => IntType::U64,
        }
    }
}

impl Context {
    pub fn new(ptr_size: PtrSize) -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            entry_point: None,
            ptr_size,
        }
    }
}
