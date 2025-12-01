use std::collections::HashSet;

use crate::core::{BlockId, Function, Terminator};

impl Function {
    pub fn get_reachable_blocks(&self) -> HashSet<BlockId> {
        let mut res = HashSet::new();
        let mut worklist = if let Some(entry) = self.entry_block {
            res.insert(entry);
            vec![entry]
        } else {
            return res;
        };
        while let Some(bb) = worklist.pop() {
            if !res.insert(bb) {
                continue;
            }
            let block = &self.blocks[&bb];
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
        res
    }
}
