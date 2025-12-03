use std::collections::HashSet;

use crate::core::{BlockId, Function, FunctionLinkage, Terminator};

impl Function<dyn FunctionLinkage> {
    pub fn get_reachable_blocks(&self) -> Option<HashSet<BlockId>> {
        let mut res = HashSet::new();
        let mut worklist = if let Some(entry) = self.linkage.get_linkage()?.entry_block {
            res.insert(entry);
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
                    res.insert(*next);
                    worklist.push(*next);
                }
                Terminator::Iff {
                    then_dst, else_dst, ..
                } => {
                    res.insert(*then_dst);
                    worklist.push(*then_dst);
                    res.insert(*else_dst);
                    worklist.push(*else_dst);
                }
            }
        }
        Some(res)
    }
}
