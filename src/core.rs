use std::collections::{HashMap, HashSet};

// SSA value
pub type LocalId = u32;

// Basic block ID
pub type BlockId = u32;

// Function ID
pub type MirFunId = u32;

// Struct ID
pub type StructId = u32;

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

pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, MirType)>,
    pub packed: bool,
    pub align: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirType {
    Void,
    I8,
    I16,
    I32,
    I64, // Signed primitive types
    U8,
    U16,
    U32,
    U64, // Unsigned primitive types
    Bool,
    Ptr(Box<MirType>),
    Struct(StructId),
    Tuple(Vec<MirType>),
    Array(Box<MirType>, u32),
}

pub enum CastKind {
    PtrToInt,
    IntToPtr,
    Reinterpret,
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
pub enum Place {
    Local(LocalId),
    Projection(Box<Place>, ProjectionKind),
}

#[derive(Clone, Copy)]
pub enum ProjectionKind {
    Deref,
    Field(u32),
    Index(LocalId),
}

#[derive(Clone)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
    Call { func: MirFunId, args: Vec<Operand> },
}

#[derive(Clone)]
pub enum Constant {
    Int(i64, MirType),
    Bool(bool),
    Str(String),
    Null,
}

pub struct Function {
    pub name: String,
    pub params: Vec<(LocalId, MirType)>,
    pub variadic: bool,
    pub return_ty: MirType,
    pub locals: Vec<(LocalId, MirType)>,
    pub blocks: HashMap<BlockId, BasicBlock>,
    pub entry_block: Option<BlockId>,
    pub reserved: HashSet<BlockId>,
    pub last_reserved: BlockId,
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub struct Context {
    pub functions: HashMap<MirFunId, Function>,
    pub structs: HashMap<StructId, Struct>,
    pub entry_point: Option<MirFunId>,
    pub ptr_size: MirType,
}

impl Context {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            entry_point: None,
            ptr_size: MirType::U64,
        }
    }
}
