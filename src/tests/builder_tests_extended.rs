use std::collections::{HashMap, HashSet};

use crate::{
    builder::{BuildError, Builder},
    core::{
        Context, PtrSize,
        ctrl_flow::{
            BinOp, Constant, FunStatic, Function, FunctionLinkage, Operand, Place, RValue,
            Statement, UnOp,
        },
        types::{MirType, function::FunctionType, int::IntType},
    },
};

fn create_test_context_with_function() -> (Context, u32) {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let fun: Function<dyn FunctionLinkage> = Function {
        name: "test_function".to_string(),
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
    (ctx, 0)
}

// Builder creation tests

#[test]
fn test_builder_creation() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block_id = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block_id, 0, &mut ctx);

    assert_eq!(builder.current_block(), block_id);
    assert_eq!(builder.current_fun(), fun_id);
}

#[test]
fn test_builder_with_non_zero_instruction() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block_id = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block_id, 5, &mut ctx);

    assert_eq!(builder.current_block(), block_id);
}

// Goto tests

#[test]
fn test_build_goto_valid() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();
    let block2 = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_goto(block2);

    assert!(result.is_ok());
}

#[test]
fn test_build_goto_invalid() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_goto(999);

    assert!(result.is_err());
    match result {
        Err(BuildError::NoBlockInFun(_, _)) => (),
        _ => panic!("Expected NoBlockInFun error"),
    }
}

#[test]
fn test_build_goto_to_self() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block, 0, &mut ctx);
    let result = builder.build_goto(block);

    assert!(result.is_ok());
}

// Iff (conditional branch) tests

#[test]
fn test_build_iff_valid() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();
    let block2 = ctx.reserve_new_block(fun_id).unwrap();
    let block3 = ctx.reserve_new_block(fun_id).unwrap();

    let discriminant = Operand::Constant(Constant::Bool(true));
    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_iff(discriminant, block2, block3);

    assert!(result.is_ok());
}

#[test]
fn test_build_iff_invalid_then() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();
    let block2 = ctx.reserve_new_block(fun_id).unwrap();

    let discriminant = Operand::Constant(Constant::Bool(true));
    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_iff(discriminant, 999, block2);

    assert!(result.is_err());
}

#[test]
fn test_build_iff_invalid_else() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();
    let block2 = ctx.reserve_new_block(fun_id).unwrap();

    let discriminant = Operand::Constant(Constant::Bool(true));
    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_iff(discriminant, block2, 999);

    assert!(result.is_err());
}

#[test]
fn test_build_iff_both_invalid() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();

    let discriminant = Operand::Constant(Constant::Bool(true));
    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_iff(discriminant, 998, 999);

    assert!(result.is_err());
}

#[test]
fn test_build_iff_same_destinations() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block1 = ctx.reserve_new_block(fun_id).unwrap();
    let block2 = ctx.reserve_new_block(fun_id).unwrap();

    let discriminant = Operand::Constant(Constant::Bool(true));
    let builder = Builder::new(fun_id, block1, 0, &mut ctx);
    let result = builder.build_iff(discriminant, block2, block2);

    assert!(result.is_ok());
}

// Return tests

#[test]
fn test_build_return_void() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.build_return(None);

    // Check that the block was built
    let fun = &ctx.functions[&fun_id];
    let linkage = fun.linkage.get_linkage().unwrap();
    assert!(linkage.blocks.contains_key(&block));
}

#[test]
fn test_build_return_with_value() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let return_value = Operand::Constant(Constant::Int(42, MirType::Int(IntType::I32)));
    let builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.build_return(Some(return_value));

    let fun = &ctx.functions[&fun_id];
    let linkage = fun.linkage.get_linkage().unwrap();
    assert!(linkage.blocks.contains_key(&block));
}

// Position tests

#[test]
fn test_position_at_zero() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 5, &mut ctx);
    let result = builder.position_at(0);

    assert!(result.is_ok());
}

#[test]
fn test_position_at_beyond_statements() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
    let result = builder.position_at(10);

    assert!(result.is_err());
    match result {
        Err(BuildError::InvalidInstructionIndex(idx)) => assert_eq!(idx, 10),
        _ => panic!("Expected InvalidInstructionIndex error"),
    }
}

#[test]
fn test_position_at_after_push() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.push_stmt(Statement::Nop);
    let result = builder.position_at(0);

    assert!(result.is_ok());
}

// Statement pushing tests

#[test]
fn test_push_stmt_increments_position() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    builder.push_stmt(Statement::Nop);
    builder.push_stmt(Statement::Nop);
    builder.push_stmt(Statement::Nop);

    // After 3 pushes, position_at(2) should be valid but position_at(3) should not
    assert!(builder.position_at(2).is_ok());
}

#[test]
fn test_push_stmt_nop() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.push_stmt(Statement::Nop);

    builder.build_return(None);

    let fun = &ctx.functions[&fun_id];
    let linkage = fun.linkage.get_linkage().unwrap();
    let built_block = &linkage.blocks[&block];
    assert_eq!(built_block.statements.len(), 1);
}

#[test]
fn test_push_stmt_storage_live() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local_id = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.push_stmt(Statement::StorageLive(local_id));

    builder.build_return(None);
}

#[test]
fn test_push_stmt_storage_dead() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local_id = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
    builder.push_stmt(Statement::StorageDead(local_id));

    builder.build_return(None);
}

// Binary operation tests

#[test]
fn test_build_binop_matching_types() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let lhs = Operand::Copy(Place::Local(local1));
    let rhs = Operand::Copy(Place::Local(local2));

    let result = builder.build_binop(BinOp::Add, lhs, rhs, None);
    assert!(result.is_ok());
}

#[test]
fn test_build_binop_mismatched_types() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::I64)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let lhs = Operand::Copy(Place::Local(local1));
    let rhs = Operand::Copy(Place::Local(local2));

    let result = builder.build_binop(BinOp::Add, lhs, rhs, None);
    assert!(result.is_err());
    match result {
        Err(BuildError::MismatchedBinOp { .. }) => (),
        _ => panic!("Expected MismatchedBinOp error"),
    }
}

#[test]
fn test_build_binop_with_destination() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let dest_local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let lhs = Operand::Copy(Place::Local(local1));
    let rhs = Operand::Copy(Place::Local(local2));
    let dest = Place::Local(dest_local);

    let result = builder.build_binop(BinOp::Add, lhs, rhs, Some(dest.clone()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), dest);
}

#[test]
fn test_build_binop_all_ops() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let ops = vec![
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::Div,
        BinOp::Mod,
        BinOp::ShiftLeft,
        BinOp::ShiftRight,
        BinOp::And,
        BinOp::Or,
        BinOp::Eq,
        BinOp::Neq,
        BinOp::Lt,
        BinOp::Leq,
    ];

    for op in ops {
        let mut builder = Builder::new(fun_id, block, 0, &mut ctx);
        let lhs = Operand::Copy(Place::Local(local1));
        let rhs = Operand::Copy(Place::Local(local2));

        let result = builder.build_binop(op, lhs, rhs, None);
        assert!(result.is_ok(), "Failed for op: {:?}", op);
    }
}

// RValue building tests

#[test]
fn test_build_rval_creates_new_local() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let rval = RValue::Use(Operand::Copy(Place::Local(local)));
    let result = builder.build_rval(rval, None);

    assert!(result.is_some());
    // The result should be a new local
    if let Some(Place::Local(id)) = result {
        assert_ne!(id, local);
    } else {
        panic!("Expected a Place::Local");
    }
}

#[test]
fn test_build_rval_uses_provided_dest() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let dest = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let rval = RValue::Use(Operand::Copy(Place::Local(local)));
    let result = builder.build_rval(rval, Some(Place::Local(dest)));

    assert!(result.is_some());
    assert_eq!(result.unwrap(), Place::Local(dest));
}

#[test]
fn test_build_rval_unop() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let rval = RValue::UnOp(UnOp::Minus, Operand::Copy(Place::Local(local)));
    let result = builder.build_rval(rval, None);

    assert!(result.is_some());
}

#[test]
fn test_build_rval_ref() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let rval = RValue::Ref(Place::Local(local));
    let result = builder.build_rval(rval, None);

    assert!(result.is_some());
}

#[test]
fn test_build_rval_aggregate() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::Bool)).unwrap();
    let block = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, block, 0, &mut ctx);

    let rval = RValue::Aggregate(vec![
        Operand::Copy(Place::Local(local1)),
        Operand::Copy(Place::Local(local2)),
    ]);
    let result = builder.build_rval(rval, None);

    assert!(result.is_some());
}

// Context helper tests

#[test]
fn test_context_reserve_new_block() {
    let (mut ctx, fun_id) = create_test_context_with_function();

    let block1 = ctx.reserve_new_block(fun_id);
    let block2 = ctx.reserve_new_block(fun_id);
    let block3 = ctx.reserve_new_block(fun_id);

    assert!(block1.is_some());
    assert!(block2.is_some());
    assert!(block3.is_some());

    assert_ne!(block1.unwrap(), block2.unwrap());
    assert_ne!(block2.unwrap(), block3.unwrap());
}

#[test]
fn test_context_new_local() {
    let (mut ctx, fun_id) = create_test_context_with_function();

    let local1 = ctx.new_local(fun_id, MirType::Int(IntType::I32));
    let local2 = ctx.new_local(fun_id, MirType::Int(IntType::Bool));
    let local3 = ctx.new_local(fun_id, MirType::Int(IntType::U64));

    assert!(local1.is_some());
    assert!(local2.is_some());
    assert!(local3.is_some());

    assert_ne!(local1.unwrap(), local2.unwrap());
    assert_ne!(local2.unwrap(), local3.unwrap());
}

#[test]
fn test_context_new_local_invalid_function() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let result = ctx.new_local(999, MirType::Int(IntType::I32));
    assert!(result.is_none());
}

// Integration tests

#[test]
fn test_build_simple_function() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let entry = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, entry, 0, &mut ctx);
    builder.push_stmt(Statement::Nop);
    builder.build_return(None);

    let fun = &ctx.functions[&fun_id];
    let linkage = fun.linkage.get_linkage().unwrap();
    assert!(linkage.blocks.contains_key(&entry));
}

#[test]
fn test_build_function_with_assignment() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let local = ctx.new_local(fun_id, MirType::Int(IntType::I32)).unwrap();
    let entry = ctx.reserve_new_block(fun_id).unwrap();

    let mut builder = Builder::new(fun_id, entry, 0, &mut ctx);

    let rval = RValue::Use(Operand::Constant(Constant::Int(
        42,
        MirType::Int(IntType::I32),
    )));
    builder.push_stmt(Statement::Assign(Place::Local(local), rval));
    builder.build_return(Some(Operand::Copy(Place::Local(local))));
}

#[test]
fn test_build_function_with_branches() {
    let (mut ctx, fun_id) = create_test_context_with_function();
    let entry = ctx.reserve_new_block(fun_id).unwrap();
    let then_block = ctx.reserve_new_block(fun_id).unwrap();
    let else_block = ctx.reserve_new_block(fun_id).unwrap();

    // Entry block
    let builder = Builder::new(fun_id, entry, 0, &mut ctx);
    let discriminant = Operand::Constant(Constant::Bool(true));
    builder
        .build_iff(discriminant, then_block, else_block)
        .unwrap();

    // Then block
    let builder = Builder::new(fun_id, then_block, 0, &mut ctx);
    builder.build_return(Some(Operand::Constant(Constant::Int(
        1,
        MirType::Int(IntType::I32),
    ))));

    // Else block
    let builder = Builder::new(fun_id, else_block, 0, &mut ctx);
    builder.build_return(Some(Operand::Constant(Constant::Int(
        0,
        MirType::Int(IntType::I32),
    ))));
}
