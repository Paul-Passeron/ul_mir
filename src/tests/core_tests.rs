use std::collections::{HashMap, HashSet};

use crate::core::{
    ctrl_flow::{FunStatic, Function, FunctionLinkage},
    types::{MirType, function::FunctionType},
};

#[test]
fn test_function_reserve_new_block() {
    let mut fun: Function<dyn FunctionLinkage> = Function {
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

    for i in 0..1500 {
        let reserved = fun.reserve_new_block().unwrap();
        assert_eq!(i, reserved);
    }
}
