use std::collections::{HashMap, HashSet};

use crate::{
    builder::Builder,
    core::{
        Context, PtrSize,
        ctrl_flow::{
            BinOp, CastKind, Constant, FunExtern, FunStatic, Function, FunctionLinkage, Operand,
            Place, ProjectionKind, RValue, Statement, UnOp,
        },
        types::{
            MirType,
            function::FunctionType,
            int::IntType,
            struct_ty::{Struct, StructId},
            tuple::TupleType,
        },
    },
};

// Helper function to create a basic static function
fn create_static_function(
    name: &str,
    params: Vec<MirType>,
    ret_ty: MirType,
) -> Function<dyn FunctionLinkage> {
    Function {
        name: name.to_string(),
        ty: FunctionType::new(params, ret_ty, false),
        linkage: Box::new(FunStatic {
            params: vec![],
            blocks: HashMap::new(),
            entry_block: None,
            reserved: HashSet::new(),
            last_reserved: 0,
            locals: vec![],
        }),
    }
}

// Integration test: Build and verify a simple function
#[test]
fn test_build_and_verify_simple_function() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("simple", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let mut builder = Builder::new(0, entry, 0, &mut ctx);
    builder.push_stmt(Statement::Nop);
    builder.build_return(None);

    // Function should have one block
    let fun = &ctx.functions[&0];
    let linkage = fun.linkage.get_linkage().unwrap();
    assert_eq!(linkage.blocks.len(), 1);
}

// Integration test: Build function with conditional branches
#[test]
fn test_build_and_verify_function_with_branches() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("branching", vec![], MirType::Int(IntType::I32));
    ctx.functions.insert(0, fun);

    let entry = ctx.reserve_new_block(0).unwrap();
    let then_block = ctx.reserve_new_block(0).unwrap();
    let else_block = ctx.reserve_new_block(0).unwrap();

    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    // Build entry block
    let builder = Builder::new(0, entry, 0, &mut ctx);
    let discriminant = Operand::Constant(Constant::Bool(true));
    builder
        .build_iff(discriminant, then_block, else_block)
        .unwrap();

    // Build then block
    let builder = Builder::new(0, then_block, 0, &mut ctx);
    builder.build_return(Some(Operand::Constant(Constant::Int(
        1,
        MirType::Int(IntType::I32),
    ))));

    // Build else block
    let builder = Builder::new(0, else_block, 0, &mut ctx);
    builder.build_return(Some(Operand::Constant(Constant::Int(
        0,
        MirType::Int(IntType::I32),
    ))));

    // Check all blocks are reachable
    let fun = &ctx.functions[&0];
    let reachable = fun.get_reachable_blocks().unwrap();
    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains(&entry));
    assert!(reachable.contains(&then_block));
    assert!(reachable.contains(&else_block));
}

// Integration test: Build function with dead code
#[test]
fn test_build_function_with_dead_code() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("dead_code", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let entry = ctx.reserve_new_block(0).unwrap();
    let dead_block = ctx.reserve_new_block(0).unwrap();

    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    // Build entry block that returns immediately
    let builder = Builder::new(0, entry, 0, &mut ctx);
    builder.build_return(None);

    // Build unreachable block
    let builder = Builder::new(0, dead_block, 0, &mut ctx);
    builder.build_return(None);

    // Check only entry is reachable
    let fun = &ctx.functions[&0];
    let reachable = fun.get_reachable_blocks().unwrap();
    assert_eq!(reachable.len(), 1);
    assert!(reachable.contains(&entry));
    assert!(!reachable.contains(&dead_block));
}

// Integration test: Complex type hierarchy
#[test]
fn test_complex_type_hierarchy() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    // Create a struct type
    let struct_id: StructId = 0;
    ctx.structs.insert(
        struct_id,
        Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), MirType::Int(IntType::I32)),
                ("y".to_string(), MirType::Int(IntType::I32)),
            ],
            packed: false,
            align: 4,
        },
    );

    // Create function with complex types
    let fun = create_static_function("complex", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    // Create locals with various types
    let struct_local = ctx.new_local(0, MirType::Struct(struct_id)).unwrap();
    let tuple_local = ctx
        .new_local(
            0,
            TupleType::new(vec![
                MirType::Int(IntType::I32),
                MirType::Int(IntType::Bool),
            ])
            .into_mir(),
        )
        .unwrap();
    let ptr_local = ctx
        .new_local(0, MirType::Int(IntType::I32).wrap_ptr().into_mir())
        .unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let builder = Builder::new(0, entry, 0, &mut ctx);

    // Test projection through struct
    let field_place = Place::Projection(
        Box::new(Place::Local(struct_local)),
        ProjectionKind::Field(0),
    );

    // Test projection through tuple
    let tuple_field_place = Place::Projection(
        Box::new(Place::Local(tuple_local)),
        ProjectionKind::Field(1),
    );

    // Test pointer dereference
    let deref_place = Place::Projection(Box::new(Place::Local(ptr_local)), ProjectionKind::Deref);

    builder.build_return(None);

    // Verify types
    let fun = &ctx.functions[&0];
    assert_eq!(
        field_place.get_type(fun, &ctx),
        Some(MirType::Int(IntType::I32))
    );
    assert_eq!(
        tuple_field_place.get_type(fun, &ctx),
        Some(MirType::Int(IntType::Bool))
    );
    assert_eq!(
        deref_place.get_type(fun, &ctx),
        Some(MirType::Int(IntType::I32))
    );
}

// Integration test: Variadic function calls
#[test]
fn test_variadic_function_calls() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    // Create variadic function (like printf)
    let variadic_fun: Function<dyn FunctionLinkage> = Function {
        name: "printf".to_string(),
        ty: FunctionType::new(
            vec![MirType::Int(IntType::U8).wrap_ptr().into_mir()],
            MirType::Int(IntType::I32),
            true,
        ),
        linkage: Box::new(FunExtern),
    };
    ctx.functions.insert(0, variadic_fun);

    // Create caller function
    let caller = create_static_function("caller", vec![], MirType::Void);
    ctx.functions.insert(1, caller);

    let entry = ctx.reserve_new_block(1).unwrap();
    ctx.functions
        .get_mut(&1)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let builder = Builder::new(1, entry, 0, &mut ctx);

    // Call with extra arguments (variadic)
    let call_operand = Operand::Call {
        func: 0,
        args: vec![
            Operand::Constant(Constant::Str("Hello %d\n".to_string())),
            Operand::Constant(Constant::Int(42, MirType::Int(IntType::I32))),
        ],
    };

    builder.build_return(None);

    // Verify call type
    let fun = &ctx.functions[&1];
    assert_eq!(
        call_operand.get_type(fun, &ctx),
        Some(MirType::Int(IntType::I32))
    );
}

// Integration test: All binary operations
#[test]
fn test_all_binary_operations() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("binops", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let a = ctx.new_local(0, MirType::Int(IntType::I32)).unwrap();
    let b = ctx.new_local(0, MirType::Int(IntType::I32)).unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let mut builder = Builder::new(0, entry, 0, &mut ctx);

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
        let result = builder.build_binop(
            op,
            Operand::Copy(Place::Local(a)),
            Operand::Copy(Place::Local(b)),
            None,
        );
        assert!(result.is_ok(), "Binary operation {:?} failed", op);
    }

    builder.build_return(None);
}

// Integration test: All unary operations
#[test]
fn test_all_unary_operations() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("unops", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let int_local = ctx.new_local(0, MirType::Int(IntType::I32)).unwrap();
    let bool_local = ctx.new_local(0, MirType::Int(IntType::Bool)).unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let mut builder = Builder::new(0, entry, 0, &mut ctx);

    // Test Minus on integer
    let rval = RValue::UnOp(UnOp::Minus, Operand::Copy(Place::Local(int_local)));
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    // Test Not on boolean
    let rval = RValue::UnOp(UnOp::Not, Operand::Copy(Place::Local(bool_local)));
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    // Test Not on integer (bitwise not)
    let rval = RValue::UnOp(UnOp::Not, Operand::Copy(Place::Local(int_local)));
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    builder.build_return(None);
}

// Integration test: All cast kinds
#[test]
fn test_all_cast_kinds() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("casts", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let int_local = ctx.new_local(0, MirType::Int(IntType::U64)).unwrap();
    let ptr_local = ctx
        .new_local(0, MirType::Int(IntType::I32).wrap_ptr().into_mir())
        .unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    let mut builder = Builder::new(0, entry, 0, &mut ctx);

    // PtrToInt cast
    let rval = RValue::Cast(
        CastKind::PtrToInt,
        Operand::Copy(Place::Local(ptr_local)),
        MirType::Int(IntType::U64),
    );
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    // IntToPtr cast
    let rval = RValue::Cast(
        CastKind::IntToPtr,
        Operand::Copy(Place::Local(int_local)),
        MirType::Int(IntType::I32).wrap_ptr().into_mir(),
    );
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    // Reinterpret cast
    let rval = RValue::Cast(
        CastKind::Reinterpret,
        Operand::Copy(Place::Local(int_local)),
        MirType::Int(IntType::I64),
    );
    let result = builder.build_rval(rval, None);
    assert!(result.is_some());

    builder.build_return(None);
}

// Integration test: Complex nested projections
#[test]
fn test_projection_chains() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    // Create nested struct types
    let inner_struct_id: StructId = 0;
    ctx.structs.insert(
        inner_struct_id,
        Struct {
            name: "Inner".to_string(),
            fields: vec![("value".to_string(), MirType::Int(IntType::I32))],
            packed: false,
            align: 4,
        },
    );

    let outer_struct_id: StructId = 1;
    ctx.structs.insert(
        outer_struct_id,
        Struct {
            name: "Outer".to_string(),
            fields: vec![
                ("inner".to_string(), MirType::Struct(inner_struct_id)),
                ("flag".to_string(), MirType::Int(IntType::Bool)),
            ],
            packed: false,
            align: 4,
        },
    );

    let fun = create_static_function("projections", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    // Create a pointer to pointer to outer struct
    let ptr_ty = MirType::Struct(outer_struct_id)
        .wrap_ptr()
        .into_mir()
        .wrap_ptr()
        .into_mir();
    let local = ctx.new_local(0, ptr_ty).unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    // Complex projection: (**local).inner.value
    let place = Place::Projection(
        Box::new(Place::Projection(
            Box::new(Place::Projection(
                Box::new(Place::Projection(
                    Box::new(Place::Local(local)),
                    ProjectionKind::Deref,
                )),
                ProjectionKind::Deref,
            )),
            ProjectionKind::Field(0), // inner
        )),
        ProjectionKind::Field(0), // value
    );

    let fun = &ctx.functions[&0];
    assert_eq!(place.get_type(fun, &ctx), Some(MirType::Int(IntType::I32)));
}

// Integration test: Loop with lifetime tracking
#[test]
fn test_function_with_loop_and_lifetime() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let fun = create_static_function("loop_func", vec![], MirType::Void);
    ctx.functions.insert(0, fun);

    let counter = ctx.new_local(0, MirType::Int(IntType::I32)).unwrap();

    let entry = ctx.reserve_new_block(0).unwrap();
    let loop_header = ctx.reserve_new_block(0).unwrap();
    let loop_body = ctx.reserve_new_block(0).unwrap();
    let exit = ctx.reserve_new_block(0).unwrap();

    ctx.functions
        .get_mut(&0)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    // Entry block
    let mut builder = Builder::new(0, entry, 0, &mut ctx);
    builder.push_stmt(Statement::StorageLive(counter));
    let init_rval = RValue::Use(Operand::Constant(Constant::Int(
        0,
        MirType::Int(IntType::I32),
    )));
    builder.push_stmt(Statement::Assign(Place::Local(counter), init_rval));
    builder.build_goto(loop_header).unwrap();

    // Loop header
    let builder = Builder::new(0, loop_header, 0, &mut ctx);
    let condition = Operand::Constant(Constant::Bool(true));
    builder.build_iff(condition, loop_body, exit).unwrap();

    // Loop body
    let mut builder = Builder::new(0, loop_body, 0, &mut ctx);
    let inc_rval = RValue::BinOp(
        BinOp::Add,
        Operand::Copy(Place::Local(counter)),
        Operand::Constant(Constant::Int(1, MirType::Int(IntType::I32))),
    );
    builder.push_stmt(Statement::Assign(Place::Local(counter), inc_rval));
    builder.build_goto(loop_header).unwrap();

    // Exit block
    let mut builder = Builder::new(0, exit, 0, &mut ctx);
    builder.push_stmt(Statement::StorageDead(counter));
    builder.build_return(None);

    // Verify all blocks are reachable
    let fun = &ctx.functions[&0];
    let reachable = fun.get_reachable_blocks().unwrap();
    assert_eq!(reachable.len(), 4);

    // Verify lifetime map
    let lifetime_map = fun.build_lifetime_map();
    assert!(lifetime_map.contains_key(&entry));
    assert!(lifetime_map.contains_key(&loop_header));
    assert!(lifetime_map.contains_key(&loop_body));
    assert!(lifetime_map.contains_key(&exit));
}

// Integration test: Multiple functions with cross-calls
#[test]
fn test_multiple_functions_with_calls() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    // Create helper function
    let helper = create_static_function(
        "helper",
        vec![MirType::Int(IntType::I32)],
        MirType::Int(IntType::I32),
    );
    ctx.functions.insert(0, helper);

    // Create main function
    let main = create_static_function("main", vec![], MirType::Int(IntType::I32));
    ctx.functions.insert(1, main);

    let entry = ctx.reserve_new_block(1).unwrap();
    ctx.functions
        .get_mut(&1)
        .unwrap()
        .linkage
        .get_linkage_mut()
        .unwrap()
        .entry_block = Some(entry);

    // Call helper function
    let call = Operand::Call {
        func: 0,
        args: vec![Operand::Constant(Constant::Int(
            42,
            MirType::Int(IntType::I32),
        ))],
    };

    let fun = &ctx.functions[&1];
    assert_eq!(call.get_type(fun, &ctx), Some(MirType::Int(IntType::I32)));
    let builder = Builder::new(1, entry, 0, &mut ctx);
    // let mut builder = builder;
    builder.build_return(Some(call));

    // Verify call returns correct type
}

// Integration test: Context with multiple structs
#[test]
fn test_context_with_multiple_structs() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    // Create multiple struct types
    for i in 0..10 {
        let struct_id: StructId = i;
        ctx.structs.insert(
            struct_id,
            Struct {
                name: format!("Struct{}", i),
                fields: vec![
                    ("field1".to_string(), MirType::Int(IntType::I32)),
                    ("field2".to_string(), MirType::Int(IntType::Bool)),
                ],
                packed: false,
                align: 4,
            },
        );
    }

    assert_eq!(ctx.structs.len(), 10);

    // Verify each struct can be accessed
    for i in 0..10 {
        assert!(ctx.structs.contains_key(&i));
        assert_eq!(ctx.structs[&i].name, format!("Struct{}", i));
    }
}

// Integration test: Entry point tracking
#[test]
fn test_context_entry_point() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let main = create_static_function("main", vec![], MirType::Void);
    ctx.functions.insert(0, main);

    assert!(ctx.entry_point.is_none());

    ctx.entry_point = Some(0);
    assert_eq!(ctx.entry_point, Some(0));
}

// Integration test: Different pointer sizes
#[test]
fn test_different_pointer_sizes() {
    let sizes = vec![
        PtrSize::_8Bit,
        PtrSize::_16Bit,
        PtrSize::_32Bit,
        PtrSize::_64Bit,
    ];

    let expected_int_types = vec![IntType::U8, IntType::U16, IntType::U32, IntType::U64];

    for (size, expected) in sizes.iter().zip(expected_int_types.iter()) {
        let ctx = Context::new(*size);
        assert_eq!(ctx.ptr_size.corresponding_int(), *expected);
    }
}
