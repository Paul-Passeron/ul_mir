use std::collections::{HashMap, HashSet};

use crate::core::{
    Context,
    ctrl_flow::{
        BlockId, LocalId, Place,
        basic_blocks::{Statement, Terminator},
        function::{Function, MirFunId},
    },
};

impl Context {
    pub fn verifiy(&self) -> bool {
        for (fun, _) in &self.functions {
            if !self.verify_fun(*fun) {
                return false;
            }
        }
        return true;
    }

    pub fn verify_fun(&self, fun_id: MirFunId) -> bool {
        let mut seen: HashSet<BlockId> = HashSet::new();
        let fun = &self.functions[&fun_id];
        let data = if let Some(data) = fun.get_function_data() {
            data
        } else {
            return true;
        };
        for (bb_id, bb) in &data.blocks {
            if !seen.insert(*bb_id) {
                continue;
            }
            if let Some(bb) = bb {
                for stmt in &bb.statements {
                    match stmt {
                        Statement::Assign(place, rvalue) => {
                            let rval_ty = rvalue.get_type(fun, self);
                            let pl_ty = place.get_type(fun, self);
                            if rval_ty.is_none() || pl_ty.is_none() {
                                return false;
                            }
                            if rval_ty.unwrap() != pl_ty.unwrap() {
                                return false;
                            }
                        }
                        _ => (),
                    }
                }
                todo!("Check terminator")
            }
        }
        return true;
    }
}

impl Function {
    pub fn get_predecessors(&self) -> HashMap<BlockId, HashSet<BlockId>> {
        let data = if let Some(data) = self.get_function_data() {
            data
        } else {
            return HashMap::new();
        };
        let mut predecessors: HashMap<BlockId, HashSet<BlockId>> =
            HashMap::from_iter(data.blocks.iter().map(|x| (*x.0, HashSet::new())));
        for (block_id, block) in &data.blocks {
            if let Some(block) = block {
                match &block.terminator {
                    Terminator::Return(_) => (),
                    Terminator::Goto(id) => {
                        predecessors.get_mut(id).unwrap().insert(*block_id);
                    }
                    Terminator::Br {
                        then_dst, else_dst, ..
                    } => {
                        predecessors.get_mut(then_dst).unwrap().insert(*block_id);
                        predecessors.get_mut(else_dst).unwrap().insert(*block_id);
                    }
                }
            }
        }
        predecessors
    }
}

impl Place {
    pub fn get_unprojected_local(&self) -> LocalId {
        let mut proj = self;
        while let Place::Projection(place, _) = proj {
            proj = place.as_ref();
        }

        match proj {
            Place::Local(id) => *id,
            Place::Projection(_, _) => unreachable!(),
        }
    }
}
