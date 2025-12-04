use std::collections::HashSet;

use crate::core::ctrl_flow::{BlockId, Function, Terminator};

impl Function {
    pub fn get_reachable_blocks(&self) -> Option<HashSet<BlockId>> {
        let mut res = HashSet::new();
        let mut worklist = if let Some(entry) = self.get_function_data().map(|x| x.entry_block) {
            vec![entry]
        } else {
            return Some(res);
        };
        while let Some(bb) = worklist.pop() {
            if !res.insert(bb) {
                continue;
            }
            let block = &self.get_function_data_unchecked().blocks[&bb];
            if block.is_none() {
                continue;
            }
            let block = block.as_ref().unwrap();
            match &block.terminator {
                Terminator::Return(_) => (),
                Terminator::Goto(next) => {
                    worklist.push(*next);
                }
                Terminator::Br {
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
