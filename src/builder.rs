use crate::core::{
    BasicBlock, BlockId, Context, Function, MirFunId, Operand, Statement, Terminator,
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
}

impl Function {
    pub fn reserve_new_block(&mut self) -> BlockId {
        let res = self.last_reserved;
        self.last_reserved += 1;
        self.reserved.insert(res);
        res
    }
}

impl Context {
    pub fn reserve_new_block(&mut self, fun: MirFunId) -> Option<BlockId> {
        let func = self.functions.get_mut(&fun)?;
        Some(func.reserve_new_block())
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
}
