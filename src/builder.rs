use crate::core::{
    BasicBlock, BinOp, BlockId, Context, Function, LocalId, MirFunId, MirType, Operand, Place,
    RValue, Statement, Terminator,
};

pub struct Builder<'a> {
    current_block: BlockId,
    current_instruction: usize,
    statements: Vec<Statement>,
    fun: MirFunId,
    ctx: &'a mut Context,
}

pub enum BuildError {
    NoBlockInFun(BlockId, MirFunId),
    InvalidInstructionIndex(usize),
    MismatchedBinOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
}

impl Function {
    pub fn reserve_new_block(&mut self) -> BlockId {
        let res = self.last_reserved;
        self.last_reserved += 1;
        self.reserved.insert(res);
        res
    }

    pub fn new_local(&mut self, ty: MirType) -> LocalId {
        let min = self
            .locals
            .iter()
            .map(|x| x.0)
            .max_by(u32::cmp)
            .unwrap_or(0);
        let res = min + 1;
        self.locals.push((res, ty));
        res
    }
}

impl Context {
    pub fn reserve_new_block(&mut self, fun: MirFunId) -> Option<BlockId> {
        let func = self.functions.get_mut(&fun)?;
        Some(func.reserve_new_block())
    }

    pub fn new_local(&mut self, fun: MirFunId, ty: MirType) -> Option<LocalId> {
        let func = self.functions.get_mut(&fun)?;
        Some(func.new_local(ty))
    }
}

impl<'a> Builder<'a> {
    pub fn new(
        fun: MirFunId,
        current_block: BlockId,
        current_instruction: usize,
        ctx: &'a mut Context,
    ) -> Self {
        Self {
            current_block,
            current_instruction,
            statements: vec![],
            fun,
            ctx,
        }
    }

    pub fn current_block(&self) -> BlockId {
        self.current_block
    }

    pub fn current_fun(&self) -> MirFunId {
        self.fun
    }

    fn build(mut self, terminator: Terminator) -> BlockId {
        let statements = self.statements;
        self.statements = vec![];
        let bl = BasicBlock {
            statements,
            terminator,
        };
        self.ctx
            .functions
            .get_mut(&self.fun)
            .unwrap()
            .blocks
            .insert(self.current_block, bl);
        self.current_block
    }

    fn check_destination(&self, destination: BlockId) -> Result<(), BuildError> {
        if !self.ctx.functions[&self.fun]
            .reserved
            .contains(&destination)
        {
            Err(BuildError::NoBlockInFun(self.current_block, self.fun))
        } else {
            Ok(())
        }
    }

    pub fn build_goto(self, destination: BlockId) -> Result<(), BuildError> {
        self.check_destination(destination)?;
        self.build(Terminator::Goto(destination));
        Ok(())
    }

    pub fn build_iff(
        self,
        discriminant: Operand,
        then_dst: BlockId,
        else_dst: BlockId,
    ) -> Result<(), BuildError> {
        self.check_destination(then_dst)?;
        self.check_destination(else_dst)?;
        self.build(Terminator::Iff {
            discriminant,
            then_dst,
            else_dst,
        });
        Ok(())
    }

    pub fn build_return(self, op: Option<Operand>) {
        self.build(Terminator::Return(op));
    }

    pub fn position_at(&mut self, instr: usize) -> Result<(), BuildError> {
        if instr >= self.statements.len() {
            Err(BuildError::InvalidInstructionIndex(instr))
        } else {
            self.current_instruction = instr;
            Ok(())
        }
    }

    pub fn push_stmt(&mut self, stmt: Statement) {
        self.statements.insert(self.current_instruction, stmt);
        self.current_instruction += 1;
    }

    pub fn build_rval(&mut self, rval: RValue, dest: Option<Place>) -> Option<Place> {
        let fun = &self.ctx.functions[&self.fun];
        let ty = rval.get_type(fun, self.ctx)?;
        let place = dest.unwrap_or_else(|| Place::Local(self.ctx.new_local(self.fun, ty).unwrap()));
        let stmt = Statement::Assign(place.clone(), rval);
        self.push_stmt(stmt);
        Some(place)
    }

    pub fn build_binop(
        &mut self,
        binop: BinOp,
        lhs: Operand,
        rhs: Operand,
        dest: Option<Place>,
    ) -> Result<Place, BuildError> {
        let rval = RValue::BinOp(binop, lhs.clone(), rhs.clone());
        self.build_rval(rval, dest)
            .ok_or_else(|| BuildError::MismatchedBinOp {
                op: binop,
                lhs: lhs,
                rhs: rhs,
            })
    }
}
