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
pub struct FunctionType {
    args: Vec<MirType>,
    ret_ty: Box<MirType>,
    variadic: bool,
}

impl FunctionType {
    pub fn into_mir(self) -> MirType {
        MirType::Function(self)
    }

    pub fn new(args: Vec<MirType>, ret_ty: MirType, variadic: bool) -> Self {
        Self {
            args,
            ret_ty: Box::new(ret_ty),
            variadic,
        }
    }

    pub fn params(&self) -> &[MirType] {
        &self.args
    }

    pub fn ret_ty(&self) -> &MirType {
        &self.ret_ty
    }

    pub fn variadic(&self) -> bool {
        self.variadic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
}

impl IntType {
    pub fn into_mir(self) -> MirType {
        MirType::Int(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    ty: Box<MirType>,
    length: u32,
}

impl ArrayType {
    pub fn into_mir(self) -> MirType {
        MirType::Array(self)
    }

    pub fn new(ty: MirType, length: u32) -> Self {
        Self {
            ty: Box::new(ty),
            length,
        }
    }

    pub fn element(&self) -> &MirType {
        &self.ty
    }

    pub fn length(&self) -> u32 {
        self.length
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleType {
    tys: Vec<MirType>,
}

impl TupleType {
    pub fn new(tys: Vec<MirType>) -> Self {
        Self { tys }
    }

    pub fn into_mir(self) -> MirType {
        MirType::Tuple(self)
    }

    pub fn fields(&self) -> &Vec<MirType> {
        &self.tys
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtrType {
    pointee: Box<MirType>,
}

impl PtrType {
    pub fn new(pointee: MirType) -> Self {
        Self {
            pointee: Box::new(pointee),
        }
    }

    pub fn into_mir(self) -> MirType {
        MirType::Ptr(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirType {
    Void,
    Int(IntType),
    Ptr(PtrType),
    Array(ArrayType),
    Function(FunctionType),
    Tuple(TupleType),
    Struct(StructId),
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
