use std::collections::HashMap;

use crate::core::{
    ctrl_flow::{BlockId, LocalId, basic_blocks::BasicBlock},
    types::{MirType, function::FunctionType},
};

pub type MirFunId = u32;

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

    pub fn reserve_new_block(&mut self) -> Option<BlockId> {
        let id: BlockId = self.get_function_data()?.blocks.len() as BlockId;
        self.get_function_data_mut()?.blocks.insert(id, None);
        Some(id)
    }
}
