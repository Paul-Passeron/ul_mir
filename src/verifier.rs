use std::collections::{HashMap, HashSet};

use crate::core::{
    Context,
    ctrl_flow::{
        BasicBlock, BlockId, Function, FunctionLinkage, LocalId, MirFunId, Place, Statement,
        Terminator,
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
        let lifetime_map = fun.build_lifetime_map();
        let linkage = if let Some(linkage) = fun.linkage.get_linkage() {
            linkage
        } else {
            return true;
        };
        for (bb_id, bb) in &linkage.blocks {
            if !seen.insert(*bb_id) {
                continue;
            }

            if !bb.check_lifetime(&lifetime_map[&bb_id]) {
                return false;
            }

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
        return true;
    }
}

impl Function<dyn FunctionLinkage> {
    pub fn build_lifetime_map(&self) -> HashMap<BlockId, HashSet<LocalId>> {
        let mut visited: HashSet<BlockId> = HashSet::new();
        let mut res = HashMap::new();
        let mut current_vars = HashSet::new();
        let linkage = if let Some(linkage) = self.linkage.get_linkage() {
            linkage
        } else {
            return res;
        };
        let mut worklist = linkage.entry_block.iter().copied().collect::<Vec<_>>();
        while let Some(block_id) = worklist.pop() {
            if !visited.insert(block_id) {
                continue;
            }
            res.insert(block_id, current_vars.clone());
            let block = &linkage.blocks[&block_id];
            block.get_surviving_vars(&mut current_vars);
            match &block.terminator {
                Terminator::Return(_) => (),
                Terminator::Goto(next) => worklist.push(*next),
                Terminator::Iff {
                    then_dst, else_dst, ..
                } => {
                    worklist.push(*then_dst);
                    worklist.push(*else_dst);
                }
            }
        }
        res
    }

    pub fn get_predecessors(&self) -> HashMap<BlockId, HashSet<BlockId>> {
        let linkage = if let Some(linkage) = self.linkage.get_linkage() {
            linkage
        } else {
            return HashMap::new();
        };
        let mut predecessors: HashMap<BlockId, HashSet<BlockId>> =
            HashMap::from_iter(linkage.blocks.iter().map(|x| (*x.0, HashSet::new())));
        for (block_id, block) in &linkage.blocks {
            match &block.terminator {
                Terminator::Return(_) => (),
                Terminator::Goto(id) => {
                    predecessors.get_mut(id).unwrap().insert(*block_id);
                }
                Terminator::Iff {
                    then_dst, else_dst, ..
                } => {
                    predecessors.get_mut(then_dst).unwrap().insert(*block_id);
                    predecessors.get_mut(else_dst).unwrap().insert(*block_id);
                }
            }
        }
        predecessors
    }
}

impl BasicBlock {
    pub fn check_lifetime(&self, live_locals: &HashSet<LocalId>) -> bool {
        let mut live_locals = live_locals.clone();
        for stmt in &self.statements {
            match stmt {
                Statement::Assign(place, _) => {
                    let local = place.get_unprojected_local();
                    if !live_locals.contains(&local) {
                        return false;
                    }
                }
                Statement::StorageLive(local) => {
                    if !live_locals.insert(*local) {
                        return false;
                    }
                }
                Statement::StorageDead(local) => {
                    if !live_locals.remove(local) {
                        return false;
                    }
                }
                Statement::Nop => (),
            }
        }
        return true;
    }

    pub fn get_surviving_vars(&self, set: &mut HashSet<LocalId>) {
        for stmt in &self.statements {
            match stmt {
                Statement::StorageLive(local) => {
                    set.insert(*local);
                }
                Statement::StorageDead(local) => {
                    set.remove(local);
                }
                Statement::Assign(_, _) => (),
                Statement::Nop => (),
            }
        }
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
