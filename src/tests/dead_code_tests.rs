use std::collections::{HashMap, HashSet};

use crate::core::{
    BasicBlock, Constant, FunStatic, Function, FunctionLinkage, Operand, Terminator,
    types::{MirType, function::FunctionType, int::IntType},
};

fn create_test_function_with_blocks(
    blocks: HashMap<u32, BasicBlock>,
    entry_block: Option<u32>,
) -> Function<dyn FunctionLinkage> {
    let reserved: HashSet<u32> = blocks.keys().copied().collect();
    let last_reserved = reserved.iter().max().map(|x| x + 1).unwrap_or(0);

    Function {
        name: "test_func".to_string(),
        ty: FunctionType::new(vec![], MirType::Void, false),
        linkage: Box::new(FunStatic {
            params: vec![],
            blocks,
            entry_block,
            reserved,
            last_reserved,
            locals: vec![],
        }),
    }
}

// No entry block tests

#[test]
fn test_get_reachable_blocks_no_entry() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, None);
    let reachable = fun.get_reachable_blocks().unwrap();

    assert!(reachable.is_empty());
}

// Single block tests

#[test]
fn test_get_reachable_blocks_single_block() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 1);
    assert!(reachable.contains(&0));
}

// Linear chain tests

#[test]
fn test_get_reachable_blocks_linear_chain() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(2),
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
    assert!(reachable.contains(&2));
}

#[test]
fn test_get_reachable_blocks_long_chain() {
    let mut blocks = HashMap::new();
    for i in 0..100 {
        let terminator = if i == 99 {
            Terminator::Return(None)
        } else {
            Terminator::Goto(i + 1)
        };
        blocks.insert(
            i,
            BasicBlock {
                statements: vec![],
                terminator,
            },
        );
    }

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 100);
    for i in 0..100 {
        assert!(reachable.contains(&i));
    }
}

// Branch tests

#[test]
fn test_get_reachable_blocks_with_branches() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
    assert!(reachable.contains(&2));
}

#[test]
fn test_get_reachable_blocks_nested_branches() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(false)),
                then_dst: 3,
                else_dst: 4,
            },
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(5),
        },
    );
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    blocks.insert(
        4,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    blocks.insert(
        5,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 6);
    for i in 0..6 {
        assert!(reachable.contains(&i));
    }
}

// Unreachable block tests

#[test]
fn test_get_reachable_blocks_unreachable_branch() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    // Block 2 is unreachable
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
    assert!(!reachable.contains(&2));
}

#[test]
fn test_get_reachable_blocks_multiple_unreachable() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    // Blocks 1-5 are all unreachable
    for i in 1..=5 {
        blocks.insert(
            i,
            BasicBlock {
                statements: vec![],
                terminator: Terminator::Return(None),
            },
        );
    }

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 1);
    assert!(reachable.contains(&0));
    for i in 1..=5 {
        assert!(!reachable.contains(&i));
    }
}

#[test]
fn test_get_reachable_blocks_unreachable_after_branch() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    // Block 3 is unreachable
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(4),
        },
    );
    // Block 4 is unreachable
    blocks.insert(
        4,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
    assert!(reachable.contains(&2));
    assert!(!reachable.contains(&3));
    assert!(!reachable.contains(&4));
}

// Loop tests

#[test]
fn test_get_reachable_blocks_loop() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 2,
                else_dst: 3,
            },
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1), // Back edge
        },
    );
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 4);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
    assert!(reachable.contains(&2));
    assert!(reachable.contains(&3));
}

#[test]
fn test_get_reachable_blocks_self_loop() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1), // Self loop
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&1));
}

#[test]
fn test_get_reachable_blocks_nested_loops() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );
    // Outer loop header
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 2,
                else_dst: 5,
            },
        },
    );
    // Inner loop header
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 3,
                else_dst: 4,
            },
        },
    );
    // Inner loop body
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(2), // Inner loop back edge
        },
    );
    // Outer loop body
    blocks.insert(
        4,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1), // Outer loop back edge
        },
    );
    // Exit
    blocks.insert(
        5,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 6);
    for i in 0..6 {
        assert!(reachable.contains(&i));
    }
}

// Return terminator tests

#[test]
fn test_get_reachable_blocks_return_terminates() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );
    // Block 1 should not be reachable
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 1);
    assert!(reachable.contains(&0));
    assert!(!reachable.contains(&1));
}

#[test]
fn test_get_reachable_blocks_early_return() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(
                1,
                MirType::Int(IntType::I32),
            )))),
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(3),
        },
    );
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(
                0,
                MirType::Int(IntType::I32),
            )))),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 4);
    for i in 0..4 {
        assert!(reachable.contains(&i));
    }
}

// Extern function tests

#[test]
fn test_get_reachable_blocks_extern_function() {
    let fun: Function<dyn FunctionLinkage> = Function {
        name: "extern_func".to_string(),
        ty: FunctionType::new(vec![MirType::Int(IntType::I32)], MirType::Void, false),
        linkage: Box::new(crate::core::FunExtern),
    };

    let reachable = fun.get_reachable_blocks();
    assert!(reachable.is_none());
}

// Complex control flow tests

#[test]
fn test_get_reachable_blocks_diamond_pattern() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(3),
        },
    );
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(3),
        },
    );
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 4);
    for i in 0..4 {
        assert!(reachable.contains(&i));
    }
}

#[test]
fn test_get_reachable_blocks_complex_cfg() {
    let mut blocks = HashMap::new();

    // Entry
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(1),
        },
    );

    // Branch 1
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(true)),
                then_dst: 2,
                else_dst: 3,
            },
        },
    );

    // Path A
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(4),
        },
    );

    // Path B
    blocks.insert(
        3,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(5),
        },
    );

    // Merge point
    blocks.insert(
        4,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(6),
        },
    );

    blocks.insert(
        5,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(6),
        },
    );

    // Another branch
    blocks.insert(
        6,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Iff {
                discriminant: Operand::Constant(Constant::Bool(false)),
                then_dst: 7,
                else_dst: 8,
            },
        },
    );

    blocks.insert(
        7,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    blocks.insert(
        8,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    // Unreachable block
    blocks.insert(
        99,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(blocks, Some(0));
    let reachable = fun.get_reachable_blocks().unwrap();

    assert_eq!(reachable.len(), 9);
    for i in 0..9 {
        assert!(reachable.contains(&i));
    }
    assert!(!reachable.contains(&99));
}
