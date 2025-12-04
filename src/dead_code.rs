use std::collections::HashSet;

use crate::core::ctrl_flow::{BlockId, Function, FunctionLinkage, Terminator};

impl Function<dyn FunctionLinkage> {
    pub fn get_reachable_blocks(&self) -> Option<HashSet<BlockId>> {
        let mut res = HashSet::new();
        let mut worklist = if let Some(entry) = self.linkage.get_linkage()?.entry_block {
            vec![entry]
        } else {
            return Some(res);
        };
        while let Some(bb) = worklist.pop() {
            if !res.insert(bb) {
                continue;
            }
            let block = &self.linkage.get_linkage()?.blocks[&bb];
            match &block.terminator {
                Terminator::Return(_) => (),
                Terminator::Goto(next) => {
                    worklist.push(*next);
                }
                Terminator::Iff {
                    then_dst, else_dst, ..
                } => {
                    worklist.push(*then_dst);
                    worklist.push(*else_dst);
                }
            }
        }
        Some(res)
    }
}
