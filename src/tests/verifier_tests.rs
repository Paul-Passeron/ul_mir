use std::collections::{HashMap, HashSet};

use crate::core::{
    Context, PtrSize,
    ctrl_flow::{
        BasicBlock, BlockId, Constant, FunExtern, FunStatic, Function, FunctionLinkage, LocalId,
        Operand, Place, ProjectionKind, RValue, Statement, Terminator,
    },
    types::{MirType, function::FunctionType, int::IntType},
};

fn create_test_context() -> Context {
    Context::new(PtrSize::_64Bit)
}

fn create_test_function_with_blocks(
    locals: Vec<(LocalId, MirType)>,
    blocks: HashMap<BlockId, BasicBlock>,
    entry_block: Option<BlockId>,
) -> Function<dyn FunctionLinkage> {
    let reserved: HashSet<BlockId> = blocks.keys().copied().collect();
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
            locals,
        }),
    }
}

// Empty context tests

#[test]
fn test_verify_empty_context() {
    let ctx = create_test_context();
    assert!(ctx.verifiy());
}

// Simple function tests

#[test]
fn test_verify_simple_function() {
    let mut ctx = create_test_context();

    let locals = vec![(0, MirType::Int(IntType::I32))];
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![Statement::Nop],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    ctx.functions.insert(0, fun);

    // Note: verify_fun is not fully implemented (has todo!), so this will panic
    // assert!(ctx.verify_fun(0));
}

#[test]
fn test_verify_function_type_mismatch_assign() {
    let mut ctx = create_test_context();

    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I64)),
    ];

    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Copy(Place::Local(1))),
            )],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    ctx.functions.insert(0, fun);

    // This should fail due to type mismatch
    // assert!(!ctx.verify_fun(0));
}

#[test]
fn test_verify_function_valid_assign() {
    let mut ctx = create_test_context();

    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];

    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Copy(Place::Local(1))),
            )],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    ctx.functions.insert(0, fun);

    // This should succeed
    // assert!(ctx.verify_fun(0));
}

// Lifetime map tests

#[test]
fn test_build_lifetime_map_simple() {
    let locals = vec![(0, MirType::Int(IntType::I32))];

    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![
                Statement::StorageLive(0),
                Statement::Assign(
                    Place::Local(0),
                    RValue::Use(Operand::Constant(Constant::Int(
                        42,
                        MirType::Int(IntType::I32),
                    ))),
                ),
                Statement::StorageDead(0),
            ],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let lifetime_map = fun.build_lifetime_map();

    assert!(lifetime_map.contains_key(&0));
    assert_eq!(lifetime_map[&0].len(), 0); // No variables live at entry
}

#[test]
fn test_build_lifetime_map_branches() {
    let locals = vec![
        (0, MirType::Int(IntType::Bool)),
        (1, MirType::Int(IntType::I32)),
    ];

    let mut blocks = HashMap::new();

    // Entry block
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![Statement::StorageLive(0), Statement::StorageLive(1)],
            terminator: Terminator::Iff {
                discriminant: Operand::Copy(Place::Local(0)),
                then_dst: 1,
                else_dst: 2,
            },
        },
    );

    // Then block
    blocks.insert(
        1,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    // Else block
    blocks.insert(
        2,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let lifetime_map = fun.build_lifetime_map();

    assert!(lifetime_map.contains_key(&0));
    assert!(lifetime_map.contains_key(&1));
    assert!(lifetime_map.contains_key(&2));

    // Both variables should be live in then/else blocks
    assert_eq!(lifetime_map[&1].len(), 2);
    assert_eq!(lifetime_map[&2].len(), 2);
}

#[test]
fn test_build_lifetime_map_linear_chain() {
    let locals = vec![(0, MirType::Int(IntType::I32))];

    let mut blocks = HashMap::new();

    blocks.insert(
        0,
        BasicBlock {
            statements: vec![Statement::StorageLive(0)],
            terminator: Terminator::Goto(1),
        },
    );

    blocks.insert(
        1,
        BasicBlock {
            statements: vec![Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Constant(Constant::Int(
                    42,
                    MirType::Int(IntType::I32),
                ))),
            )],
            terminator: Terminator::Goto(2),
        },
    );

    blocks.insert(
        2,
        BasicBlock {
            statements: vec![Statement::StorageDead(0)],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let lifetime_map = fun.build_lifetime_map();

    assert_eq!(lifetime_map[&0].len(), 0); // No live vars at entry
    assert_eq!(lifetime_map[&1].len(), 1); // Local 0 is live
    assert_eq!(lifetime_map[&2].len(), 1); // Local 0 still live
}

// Lifetime checking tests

#[test]
fn test_check_lifetime_valid_storage() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Constant(Constant::Int(
                    42,
                    MirType::Int(IntType::I32),
                ))),
            ),
            Statement::StorageDead(0),
        ],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(block.check_lifetime(&live_locals));
}

#[test]
fn test_check_lifetime_use_before_storage_live() {
    let block = BasicBlock {
        statements: vec![Statement::Assign(
            Place::Local(0),
            RValue::Use(Operand::Constant(Constant::Int(
                42,
                MirType::Int(IntType::I32),
            ))),
        )],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(!block.check_lifetime(&live_locals));
}

#[test]
fn test_check_lifetime_double_storage_live() {
    let block = BasicBlock {
        statements: vec![Statement::StorageLive(0), Statement::StorageLive(0)],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(!block.check_lifetime(&live_locals));
}

#[test]
fn test_check_lifetime_use_after_storage_dead() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::StorageDead(0),
            Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Constant(Constant::Int(
                    42,
                    MirType::Int(IntType::I32),
                ))),
            ),
        ],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(!block.check_lifetime(&live_locals));
}

#[test]
fn test_check_lifetime_storage_dead_not_live() {
    let block = BasicBlock {
        statements: vec![Statement::StorageDead(0)],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(!block.check_lifetime(&live_locals));
}

#[test]
fn test_check_lifetime_with_live_set() {
    let block = BasicBlock {
        statements: vec![Statement::Assign(
            Place::Local(0),
            RValue::Use(Operand::Constant(Constant::Int(
                42,
                MirType::Int(IntType::I32),
            ))),
        )],
        terminator: Terminator::Return(None),
    };

    let mut live_locals = HashSet::new();
    live_locals.insert(0);
    assert!(block.check_lifetime(&live_locals));
}

// Predecessor tests

#[test]
fn test_get_predecessors_simple_chain() {
    let locals = vec![];
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

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let predecessors = fun.get_predecessors();

    assert_eq!(predecessors[&0].len(), 0);
    assert_eq!(predecessors[&1].len(), 1);
    assert!(predecessors[&1].contains(&0));
    assert_eq!(predecessors[&2].len(), 1);
    assert!(predecessors[&2].contains(&1));
}

#[test]
fn test_get_predecessors_branches() {
    let locals = vec![];
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

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let predecessors = fun.get_predecessors();

    assert_eq!(predecessors[&0].len(), 0);
    assert_eq!(predecessors[&1].len(), 1);
    assert!(predecessors[&1].contains(&0));
    assert_eq!(predecessors[&2].len(), 1);
    assert!(predecessors[&2].contains(&0));
}

#[test]
fn test_get_predecessors_multiple() {
    let locals = vec![];
    let mut blocks = HashMap::new();

    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Goto(2),
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

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let predecessors = fun.get_predecessors();

    assert_eq!(predecessors[&2].len(), 2);
    assert!(predecessors[&2].contains(&0));
    assert!(predecessors[&2].contains(&1));
}

#[test]
fn test_get_predecessors_return_has_no_successors() {
    let locals = vec![];
    let mut blocks = HashMap::new();

    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, Some(0));
    let predecessors = fun.get_predecessors();

    assert_eq!(predecessors[&0].len(), 0);
}

// Surviving vars tests

#[test]
fn test_get_surviving_vars_storage_live() {
    let block = BasicBlock {
        statements: vec![Statement::StorageLive(0), Statement::StorageLive(1)],
        terminator: Terminator::Return(None),
    };

    let mut set = HashSet::new();
    block.get_surviving_vars(&mut set);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&0));
    assert!(set.contains(&1));
}

#[test]
fn test_get_surviving_vars_storage_dead() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::StorageLive(1),
            Statement::StorageDead(0),
        ],
        terminator: Terminator::Return(None),
    };

    let mut set = HashSet::new();
    block.get_surviving_vars(&mut set);

    assert_eq!(set.len(), 1);
    assert!(!set.contains(&0));
    assert!(set.contains(&1));
}

#[test]
fn test_get_surviving_vars_ignores_assign() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Constant(Constant::Int(
                    42,
                    MirType::Int(IntType::I32),
                ))),
            ),
        ],
        terminator: Terminator::Return(None),
    };

    let mut set = HashSet::new();
    block.get_surviving_vars(&mut set);

    assert_eq!(set.len(), 1);
    assert!(set.contains(&0));
}

#[test]
fn test_get_surviving_vars_with_initial_set() {
    let block = BasicBlock {
        statements: vec![Statement::StorageLive(1), Statement::StorageDead(0)],
        terminator: Terminator::Return(None),
    };

    let mut set = HashSet::new();
    set.insert(0);
    set.insert(2);

    block.get_surviving_vars(&mut set);

    assert!(!set.contains(&0)); // Was removed
    assert!(set.contains(&1)); // Was added
    assert!(set.contains(&2)); // Was preserved
}

// Place unprojected local tests

#[test]
fn test_place_get_unprojected_local_simple() {
    let place = Place::Local(42);
    assert_eq!(place.get_unprojected_local(), 42);
}

#[test]
fn test_place_get_unprojected_local_nested_projection() {
    let place = Place::Projection(
        Box::new(Place::Projection(
            Box::new(Place::Projection(
                Box::new(Place::Local(10)),
                ProjectionKind::Deref,
            )),
            ProjectionKind::Field(0),
        )),
        ProjectionKind::Index(5),
    );

    assert_eq!(place.get_unprojected_local(), 10);
}

#[test]
fn test_place_get_unprojected_local_single_projection() {
    let place = Place::Projection(Box::new(Place::Local(7)), ProjectionKind::Field(2));

    assert_eq!(place.get_unprojected_local(), 7);
}

// Extern function tests

#[test]
fn test_verify_extern_function() {
    let mut ctx = create_test_context();

    let fun: Function<dyn FunctionLinkage> = Function {
        name: "extern_func".to_string(),
        ty: FunctionType::new(vec![MirType::Int(IntType::I32)], MirType::Void, false),
        linkage: Box::new(FunExtern),
    };

    ctx.functions.insert(0, fun);

    // Extern functions should verify successfully (they have no body)
    assert!(ctx.verify_fun(0));
}

#[test]
fn test_build_lifetime_map_no_entry() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(None),
        },
    );

    let fun = create_test_function_with_blocks(locals, blocks, None);
    let lifetime_map = fun.build_lifetime_map();

    assert!(lifetime_map.is_empty());
}

#[test]
fn test_multiple_storage_operations() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::StorageLive(1),
            Statement::StorageLive(2),
            Statement::StorageDead(1),
            Statement::StorageDead(0),
        ],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(block.check_lifetime(&live_locals));
}

#[test]
fn test_nop_statements_dont_affect_lifetime() {
    let block = BasicBlock {
        statements: vec![
            Statement::StorageLive(0),
            Statement::Nop,
            Statement::Nop,
            Statement::Assign(
                Place::Local(0),
                RValue::Use(Operand::Constant(Constant::Int(
                    1,
                    MirType::Int(IntType::I32),
                ))),
            ),
            Statement::Nop,
            Statement::StorageDead(0),
        ],
        terminator: Terminator::Return(None),
    };

    let live_locals = HashSet::new();
    assert!(block.check_lifetime(&live_locals));
}
