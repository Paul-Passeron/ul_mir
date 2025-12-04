use std::collections::{HashMap, HashSet};

use crate::{
    builder::Builder,
    core::{
        Context,
        ctrl_flow::{FunStatic, Function, FunctionLinkage},
        types::{MirType, function::FunctionType},
    },
};

#[test]
fn test_builder_new() {
    let mut ctx = Context::new(crate::core::PtrSize::_64Bit);
    let fun: Function<dyn FunctionLinkage> = Function {
        name: "my_function".to_string(),
        ty: FunctionType::new(vec![], MirType::Void, false),
        linkage: Box::new(FunStatic {
            params: vec![],
            blocks: HashMap::new(),
            entry_block: None,
            reserved: HashSet::new(),
            last_reserved: 0,
            locals: vec![],
        }),
    };
    ctx.functions.insert(0, fun);

    let entry_block = ctx.reserve_new_block(0).unwrap();

    let _ = Builder::new(0, entry_block, 0, &mut ctx);
}
