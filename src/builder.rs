use crate::core::{
    Context,
    ctrl_flow::{
        BasicBlock, BinOp, BlockId, Function, FunctionBody, LocalId, MirFunId, Operand, Place,
        RValue, Statement, Terminator,
    },
    types::MirType,
};

pub struct Builder {
    current_block: BlockId,
    statements: Vec<Statement>,
    fun: MirFunId,
}

#[derive(Debug)]
pub enum BuildError {
    NoBlockInFun(BlockId, MirFunId),
    InvalidInstructionIndex(usize),
    MismatchedBinOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
}

impl Builder {
    pub fn new(fun: MirFunId, current_block: BlockId) -> Self {
        Self {
            current_block,
            statements: vec![],
            fun,
        }
    }

    pub fn current_block(&self) -> BlockId {
        self.current_block
    }

    pub fn current_fun(&self) -> MirFunId {
        self.fun
    }

    fn finish(mut self, terminator: Terminator, ctx: &mut Context) -> BlockId {
        let statements = self.statements;
        self.statements = vec![];
        let bl = BasicBlock {
            statements,
            terminator,
        };
        match &mut ctx.functions.get_mut(&self.fun).unwrap().body {
            FunctionBody::External => unreachable!(),
            FunctionBody::Defined(function_data) => function_data,
        }
        .blocks
        .insert(self.current_block, Some(bl));
        self.current_block
    }

    fn check_destination(&self, destination: BlockId, ctx: &Context) -> Result<(), BuildError> {
        if !ctx.functions[&self.fun]
            .get_function_data_unchecked()
            .blocks
            .contains_key(&destination)
        {
            Err(BuildError::NoBlockInFun(self.current_block, self.fun))
        } else {
            Ok(())
        }
    }

    pub fn goto(self, destination: BlockId, ctx: &mut Context) -> Result<BlockId, BuildError> {
        self.check_destination(destination, ctx)?;
        Ok(self.finish(Terminator::Goto(destination), ctx))
    }

    pub fn br(
        self,
        discriminant: Operand,
        then_dst: BlockId,
        else_dst: BlockId,
        ctx: &mut Context,
    ) -> Result<BlockId, BuildError> {
        self.check_destination(then_dst, ctx)?;
        self.check_destination(else_dst, ctx)?;
        Ok(self.finish(
            Terminator::Br {
                discriminant,
                then_dst,
                else_dst,
            },
            ctx,
        ))
    }

    pub fn ret(self, op: Option<Operand>, ctx: &mut Context) -> BlockId {
        self.finish(Terminator::Return(op), ctx)
    }

    pub fn stmt(&mut self, stmt: Statement) -> &mut Self {
        self.statements.push(stmt);
        self
    }

    pub fn assign(&mut self, place: Place, rvalue: RValue, ctx: &mut Context) -> Option<&mut Self> {
        let place_ty = place.get_type(&ctx.functions[&self.fun], ctx)?;
        let rvalue_ty = rvalue.get_type(&ctx.functions[&self.fun], ctx)?;
        if place_ty != rvalue_ty {
            return None;
        }
        Some(self.stmt(Statement::Assign(place, rvalue)))
    }

    pub fn binop(
        &mut self,
        binop: BinOp,
        lhs: Operand,
        rhs: Operand,
        place: Place,
        ctx: &mut Context,
    ) -> Result<&mut Self, BuildError> {
        let rval = RValue::BinOp(binop, lhs.clone(), rhs.clone());
        self.assign(place, rval, ctx)
            .ok_or_else(|| BuildError::MismatchedBinOp {
                op: binop,
                lhs: lhs,
                rhs: rhs,
            })
    }
}

impl Context {
    pub fn new_local(&mut self, fun: MirFunId, ty: MirType) -> Option<LocalId> {
        dbg!(fun);
        dbg!(ty);
        todo!()
    }
}

impl Function {
    pub fn reserve_new_block(&mut self) -> Option<BlockId> {
        let id: BlockId = self.get_function_data()?.blocks.len() as BlockId;
        self.get_function_data_mut()?.blocks.insert(id, None);
        Some(id)
    }
}
