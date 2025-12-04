use std::collections::HashMap;

use crate::core::{
    ctrl_flow::function::{Function, MirFunId},
    types::{
        int::IntType,
        struct_ty::{Struct, StructId},
    },
};

pub mod builder;
pub mod ctrl_flow;
pub mod types;

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

pub struct Context {
    pub functions: HashMap<MirFunId, Function>,
    pub structs: HashMap<StructId, Struct>,
    pub entry_point: Option<MirFunId>,
    pub ptr_size: PtrSize,
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
