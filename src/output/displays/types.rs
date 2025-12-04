use std::fmt::Display;

use crate::core::{
    Context,
    types::{
        MirType, array::ArrayType, function::FunctionType, int::IntType, ptr::PtrType,
        tuple::TupleType,
    },
};

impl Display for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntType::I8 => write!(f, "i8"),
            IntType::I16 => write!(f, "i16"),
            IntType::I32 => write!(f, "i32"),
            IntType::I64 => write!(f, "i64"),
            IntType::U8 => write!(f, "u8"),
            IntType::U16 => write!(f, "u16"),
            IntType::U32 => write!(f, "u32"),
            IntType::U64 => write!(f, "u64"),
            IntType::Bool => write!(f, "bool"),
        }
    }
}

pub struct TupleTypeDisplayer<'a> {
    ty: &'a TupleType,
    ctx: &'a Context,
}

impl<'a> TupleTypeDisplayer<'a> {
    pub fn new(ty: &'a TupleType, ctx: &'a Context) -> Self {
        TupleTypeDisplayer { ty, ctx }
    }
}

impl<'a> Display for TupleTypeDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.ty
                .fields()
                .iter()
                .map(|x| format!("{}", x.display(self.ctx)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl TupleType {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> TupleTypeDisplayer<'a> {
        TupleTypeDisplayer { ty: self, ctx }
    }
}

pub struct PtrTypeDisplayer<'a> {
    ty: &'a PtrType,
    ctx: &'a Context,
}

impl<'a> PtrTypeDisplayer<'a> {
    pub fn new(ty: &'a PtrType, ctx: &'a Context) -> Self {
        PtrTypeDisplayer { ty, ctx }
    }
}

impl<'a> Display for PtrTypeDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{}", self.ty.pointee().display(self.ctx))
    }
}

impl PtrType {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> PtrTypeDisplayer<'a> {
        PtrTypeDisplayer { ty: self, ctx }
    }
}

pub struct ArrayTypeDisplayer<'a> {
    ty: &'a ArrayType,
    ctx: &'a Context,
}

impl<'a> ArrayTypeDisplayer<'a> {
    pub fn new(ty: &'a ArrayType, ctx: &'a Context) -> Self {
        Self { ty, ctx }
    }
}

impl<'a> Display for ArrayTypeDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}; {}]",
            self.ty.pointee().display(self.ctx),
            self.ty.length()
        )
    }
}

impl ArrayType {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> ArrayTypeDisplayer<'a> {
        ArrayTypeDisplayer { ty: self, ctx }
    }
}

pub struct FunTypeDisplayer<'a> {
    ty: &'a FunctionType,
    ctx: &'a Context,
}

impl<'a> FunTypeDisplayer<'a> {
    pub fn new(ty: &'a FunctionType, ctx: &'a Context) -> Self {
        Self { ty, ctx }
    }
}

impl<'a> Display for FunTypeDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) -> {}",
            self.ty
                .params()
                .iter()
                .map(|x| format!("{}", x.display(self.ctx)))
                .collect::<Vec<_>>()
                .join(", "),
            self.ty.ret_ty().display(self.ctx)
        )
    }
}

impl FunctionType {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> FunTypeDisplayer<'a> {
        FunTypeDisplayer { ty: self, ctx }
    }
}

pub struct TypeDisplayer<'a> {
    ty: &'a MirType,
    ctx: &'a Context,
}

impl MirType {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> TypeDisplayer<'a> {
        TypeDisplayer { ty: self, ctx }
    }
}

impl<'a> Display for TypeDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ty {
            MirType::Void => write!(f, "void"),
            MirType::Int(ty) => write!(f, "{}", ty),
            MirType::Ptr(ty) => write!(f, "{}", ty.display(self.ctx)),
            MirType::Array(ty) => write!(f, "{}", ty.display(self.ctx)),
            MirType::Function(ty) => write!(f, "{}", ty.display(self.ctx)),
            MirType::Tuple(ty) => write!(f, "{}", ty.display(self.ctx)),
            MirType::Struct(id) => {
                let name = &self.ctx.structs[&id].name;
                write!(f, "struct {}", name)
            }
        }
    }
}
