use crate::core::{
    Context,
    builder::block_builder::BlockBuilder,
    ctrl_flow::{
        BlockId, LocalId,
        function::{Function, FunctionBody, FunctionData, MirFunId},
    },
    types::{MirType, function::FunctionType},
};
use indexmap::IndexMap;
use std::iter::once;

pub struct FunctionBuilder {
    ret_ty: MirType,
    params: Vec<MirType>,
    name: String,
    variadic: bool,
}

impl FunctionBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ret_ty: MirType::Void,
            params: vec![],
            variadic: false,
        }
    }

    pub fn variadic(mut self, v: bool) -> Self {
        self.variadic = v;
        self
    }

    pub fn param(mut self, ty: MirType) -> Self {
        self.params.push(ty);
        self
    }

    pub fn ret_ty(mut self, ty: MirType) -> Self {
        self.ret_ty = ty;
        self
    }

    pub fn build(self, ctx: &mut Context) -> MirFunId {
        let sig = FunctionType::new(self.params, self.ret_ty, self.variadic);
        let fun = Function {
            name: self.name,
            ty: sig,
            body: FunctionBody::External,
        };
        ctx.add_function(fun)
    }

    pub fn define(self, ctx: &mut Context) -> DefinedFunctionBuilder {
        let sig = FunctionType::new(self.params, self.ret_ty, self.variadic);
        let data = FunctionData {
            params: sig
                .params()
                .iter()
                .enumerate()
                .map(|x| x.0 as LocalId)
                .collect(),
            locals: sig
                .params()
                .iter()
                .enumerate()
                .map(|x| (x.0 as LocalId, x.1.clone()))
                .collect(),
            blocks: IndexMap::from_iter(once((0, None))),
            entry_block: 0,
        };
        let fun = Function {
            name: self.name,
            ty: sig,
            body: FunctionBody::Defined(data),
        };
        let id = ctx.add_function(fun);
        DefinedFunctionBuilder::new(id)
    }
}

pub struct DefinedFunctionBuilder {
    id: MirFunId,
}

impl DefinedFunctionBuilder {
    pub(super) fn new(id: MirFunId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> MirFunId {
        self.id
    }

    fn data_mut<'a>(&self, ctx: &'a mut Context) -> &'a mut FunctionData {
        let fun = ctx.functions.get_mut(&self.id).unwrap();
        fun.get_function_data_mut_unchecked()
    }

    fn data<'a>(&self, ctx: &'a Context) -> &'a FunctionData {
        ctx.functions[&self.id].get_function_data_unchecked()
    }

    pub fn add_local(&self, ty: MirType, ctx: &mut Context) -> LocalId {
        let data = self.data_mut(ctx);
        let next_id = data.locals.len() as LocalId;
        data.locals.push((next_id, ty));
        next_id
    }

    pub fn reserve_block(&self, ctx: &mut Context) -> BlockId {
        let data = self.data_mut(ctx);
        let next_id = data.blocks.len() as BlockId;
        data.blocks.insert(next_id, None);
        next_id
    }

    pub fn build_block(&mut self, block_id: BlockId) -> BlockBuilder {
        BlockBuilder::new(self.id, block_id)
    }

    pub fn entry_block(&self, ctx: &Context) -> BlockId {
        self.data(ctx).entry_block
    }

    pub fn finish(self) -> MirFunId {
        self.id
    }

    pub fn param_count(&self, ctx: &Context) -> usize {
        self.data(ctx).params.len()
    }

    pub fn param(&self, i: usize, ctx: &Context) -> Option<LocalId> {
        self.data(ctx).params.get(i).copied()
    }
}
