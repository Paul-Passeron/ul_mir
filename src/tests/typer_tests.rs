use std::collections::{HashMap, HashSet};

use crate::core::{
    BinOp, Constant, Context, FunStatic, Function, FunctionLinkage, Operand, Place, ProjectionKind,
    PtrSize, RValue, UnOp,
    types::{
        MirType,
        function::FunctionType,
        int::IntType,
        struct_ty::{Struct, StructId},
        tuple::TupleType,
    },
};

fn create_test_function(locals: Vec<(u32, MirType)>) -> Function<dyn FunctionLinkage> {
    Function {
        name: "test_func".to_string(),
        ty: FunctionType::new(vec![], MirType::Void, false),
        linkage: Box::new(FunStatic {
            params: vec![],
            blocks: HashMap::new(),
            entry_block: None,
            reserved: HashSet::new(),
            last_reserved: 0,
            locals,
        }),
    }
}

// Constant type tests

#[test]
fn test_constant_get_type_int() {
    let c = Constant::Int(42, MirType::Int(IntType::I32));
    assert_eq!(c.get_type(), MirType::Int(IntType::I32));
}

#[test]
fn test_constant_get_type_bool() {
    let c = Constant::Bool(true);
    assert_eq!(c.get_type(), MirType::Int(IntType::Bool));
}

#[test]
fn test_constant_get_type_str() {
    let c = Constant::Str("hello".to_string());
    assert_eq!(
        c.get_type(),
        MirType::Int(IntType::U8).wrap_ptr().into_mir()
    );
}

#[test]
fn test_constant_get_type_null() {
    let c = Constant::Null;
    assert_eq!(c.get_type(), MirType::Void.wrap_ptr().into_mir());
}

// Place type tests

#[test]
fn test_place_get_type_local() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let place = Place::Local(0);
    assert_eq!(place.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_place_get_type_deref() {
    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr().into_mir();
    let locals = vec![(0, ptr_ty)];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let place = Place::Projection(Box::new(Place::Local(0)), ProjectionKind::Deref);
    assert_eq!(place.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_place_get_type_field_tuple() {
    let tuple_ty = TupleType::new(vec![
        MirType::Int(IntType::I32),
        MirType::Int(IntType::Bool),
        MirType::Int(IntType::U64),
    ])
    .into_mir();
    let locals = vec![(0, tuple_ty)];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let place = Place::Projection(Box::new(Place::Local(0)), ProjectionKind::Field(1));
    assert_eq!(
        place.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::Bool))
    );
}

#[test]
fn test_place_get_type_field_struct() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let struct_id: StructId = 0;
    ctx.structs.insert(
        struct_id,
        Struct {
            name: "TestStruct".to_string(),
            fields: vec![
                ("field1".to_string(), MirType::Int(IntType::I32)),
                ("field2".to_string(), MirType::Int(IntType::U8)),
            ],
            packed: false,
            align: 4,
        },
    );

    let locals = vec![(0, MirType::Struct(struct_id))];
    let fun = create_test_function(locals);

    let place = Place::Projection(Box::new(Place::Local(0)), ProjectionKind::Field(0));
    assert_eq!(place.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_place_get_type_index_pointer() {
    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr().into_mir();
    let locals = vec![(0, ptr_ty), (1, MirType::Int(IntType::U32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let place = Place::Projection(Box::new(Place::Local(0)), ProjectionKind::Index(1));
    assert_eq!(place.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

// Operand type tests

#[test]
fn test_operand_get_type_copy() {
    let locals = vec![(0, MirType::Int(IntType::I64))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let operand = Operand::Copy(Place::Local(0));
    assert_eq!(
        operand.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::I64))
    );
}

#[test]
fn test_operand_get_type_move() {
    let locals = vec![(0, MirType::Int(IntType::I16))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let operand = Operand::Move(Place::Local(0));
    assert_eq!(
        operand.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::I16))
    );
}

#[test]
fn test_operand_get_type_constant() {
    let fun = create_test_function(vec![]);
    let ctx = Context::new(PtrSize::_64Bit);

    let operand = Operand::Constant(Constant::Int(100, MirType::Int(IntType::U8)));
    assert_eq!(
        operand.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::U8))
    );
}

#[test]
fn test_operand_get_type_call_exact_args() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let func_ty = FunctionType::new(
        vec![MirType::Int(IntType::I32), MirType::Int(IntType::Bool)],
        MirType::Int(IntType::U64),
        false,
    );
    let target_fun: Function<dyn FunctionLinkage> = Function {
        name: "target".to_string(),
        ty: func_ty,
        linkage: Box::new(crate::core::FunExtern),
    };
    ctx.functions.insert(1, target_fun);

    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::Bool)),
    ];
    let fun = create_test_function(locals);

    let operand = Operand::Call {
        func: 1,
        args: vec![
            Operand::Copy(Place::Local(0)),
            Operand::Copy(Place::Local(1)),
        ],
    };
    assert_eq!(
        operand.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::U64))
    );
}

#[test]
fn test_operand_get_type_call_variadic() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let func_ty = FunctionType::new(
        vec![MirType::Int(IntType::I32)],
        MirType::Int(IntType::U64),
        true,
    );
    let target_fun: Function<dyn FunctionLinkage> = Function {
        name: "variadic".to_string(),
        ty: func_ty,
        linkage: Box::new(crate::core::FunExtern),
    };
    ctx.functions.insert(1, target_fun);

    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::Bool)),
    ];
    let fun = create_test_function(locals);

    let operand = Operand::Call {
        func: 1,
        args: vec![
            Operand::Copy(Place::Local(0)),
            Operand::Copy(Place::Local(1)),
        ],
    };
    assert_eq!(
        operand.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::U64))
    );
}

#[test]
fn test_operand_get_type_call_too_few_args() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let func_ty = FunctionType::new(
        vec![MirType::Int(IntType::I32), MirType::Int(IntType::Bool)],
        MirType::Int(IntType::U64),
        false,
    );
    let target_fun: Function<dyn FunctionLinkage> = Function {
        name: "target".to_string(),
        ty: func_ty,
        linkage: Box::new(crate::core::FunExtern),
    };
    ctx.functions.insert(1, target_fun);

    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);

    let operand = Operand::Call {
        func: 1,
        args: vec![Operand::Copy(Place::Local(0))],
    };
    assert_eq!(operand.get_type(&fun, &ctx), None);
}

// RValue type tests

#[test]
fn test_rvalue_use() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::Use(Operand::Copy(Place::Local(0)));
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_rvalue_binop_add_integers() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::BinOp(
        BinOp::Add,
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
    );
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_rvalue_binop_comparison_returns_bool() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::BinOp(
        BinOp::Lt,
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
    );
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::Bool)));
}

#[test]
fn test_rvalue_binop_mismatched_types() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I64)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::BinOp(
        BinOp::Add,
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
    );
    assert_eq!(rval.get_type(&fun, &ctx), None);
}

#[test]
fn test_rvalue_binop_bool_arithmetic() {
    let locals = vec![
        (0, MirType::Int(IntType::Bool)),
        (1, MirType::Int(IntType::Bool)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::BinOp(
        BinOp::Add,
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
    );
    assert_eq!(rval.get_type(&fun, &ctx), None);
}

#[test]
fn test_rvalue_binop_pointer_arithmetic() {
    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr().into_mir();
    let locals = vec![(0, ptr_ty.clone()), (1, ptr_ty)];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::BinOp(
        BinOp::Add,
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
    );
    assert_eq!(
        rval.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::I32).wrap_ptr().into_mir())
    );
}

#[test]
fn test_rvalue_unop_minus_integer() {
    let locals = vec![(0, MirType::Int(IntType::I64))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::UnOp(UnOp::Minus, Operand::Copy(Place::Local(0)));
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::I64)));
}

#[test]
fn test_rvalue_unop_not_bool() {
    let locals = vec![(0, MirType::Int(IntType::Bool))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::UnOp(UnOp::Not, Operand::Copy(Place::Local(0)));
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::Bool)));
}

#[test]
fn test_rvalue_unop_not_integer() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::UnOp(UnOp::Not, Operand::Copy(Place::Local(0)));
    assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
}

#[test]
fn test_rvalue_cast() {
    use crate::core::CastKind;

    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let target_ty = MirType::Int(IntType::U64);
    let rval = RValue::Cast(
        CastKind::Reinterpret,
        Operand::Copy(Place::Local(0)),
        target_ty.clone(),
    );
    assert_eq!(rval.get_type(&fun, &ctx), Some(target_ty));
}

#[test]
fn test_rvalue_ref() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::Ref(Place::Local(0));
    assert_eq!(
        rval.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::I32).wrap_ptr().into_mir())
    );
}

#[test]
fn test_rvalue_len_non_array() {
    let locals = vec![(0, MirType::Int(IntType::I32))];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::Len(Place::Local(0));
    assert_eq!(rval.get_type(&fun, &ctx), None);
}

#[test]
fn test_rvalue_aggregate() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::Bool)),
        (2, MirType::Int(IntType::U64)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let rval = RValue::Aggregate(vec![
        Operand::Copy(Place::Local(0)),
        Operand::Copy(Place::Local(1)),
        Operand::Copy(Place::Local(2)),
    ]);

    let expected = TupleType::new(vec![
        MirType::Int(IntType::I32),
        MirType::Int(IntType::Bool),
        MirType::Int(IntType::U64),
    ])
    .into_mir();

    assert_eq!(rval.get_type(&fun, &ctx), Some(expected));
}

// MirType helper method tests

#[test]
fn test_mirtype_as_ptr_success() {
    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr();
    let mir_ty = ptr_ty.clone().into_mir();

    assert!(mir_ty.as_ptr().is_some());
}

#[test]
fn test_mirtype_as_ptr_failure() {
    let mir_ty = MirType::Int(IntType::I32);
    assert!(mir_ty.as_ptr().is_none());
}

#[test]
fn test_mirtype_as_integer_all_types() {
    assert_eq!(MirType::Int(IntType::I8).as_integer(), Some(IntType::I8));
    assert_eq!(MirType::Int(IntType::I16).as_integer(), Some(IntType::I16));
    assert_eq!(MirType::Int(IntType::I32).as_integer(), Some(IntType::I32));
    assert_eq!(MirType::Int(IntType::I64).as_integer(), Some(IntType::I64));
    assert_eq!(MirType::Int(IntType::U8).as_integer(), Some(IntType::U8));
    assert_eq!(MirType::Int(IntType::U16).as_integer(), Some(IntType::U16));
    assert_eq!(MirType::Int(IntType::U32).as_integer(), Some(IntType::U32));
    assert_eq!(MirType::Int(IntType::U64).as_integer(), Some(IntType::U64));
    assert_eq!(
        MirType::Int(IntType::Bool).as_integer(),
        Some(IntType::Bool)
    );
}

#[test]
fn test_mirtype_as_integer_non_integer() {
    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr().into_mir();
    assert!(ptr_ty.as_integer().is_none());
}

#[test]
fn test_mirtype_fields_tuple() {
    let ctx = Context::new(PtrSize::_64Bit);
    let tuple_ty = TupleType::new(vec![
        MirType::Int(IntType::I32),
        MirType::Int(IntType::Bool),
    ])
    .into_mir();

    let fields = tuple_ty.fields(&ctx);
    assert!(fields.is_some());
    let fields = fields.unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0], MirType::Int(IntType::I32));
    assert_eq!(fields[1], MirType::Int(IntType::Bool));
}

#[test]
fn test_mirtype_fields_struct() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let struct_id: StructId = 0;
    ctx.structs.insert(
        struct_id,
        Struct {
            name: "TestStruct".to_string(),
            fields: vec![
                ("x".to_string(), MirType::Int(IntType::I32)),
                ("y".to_string(), MirType::Int(IntType::I32)),
            ],
            packed: false,
            align: 4,
        },
    );

    let struct_ty = MirType::Struct(struct_id);
    let fields = struct_ty.fields(&ctx);
    assert!(fields.is_some());
    let fields = fields.unwrap();
    assert_eq!(fields.len(), 2);
}

#[test]
fn test_mirtype_fields_primitive() {
    let ctx = Context::new(PtrSize::_64Bit);
    let int_ty = MirType::Int(IntType::I32);
    assert!(int_ty.fields(&ctx).is_none());
}

#[test]
fn test_mirtype_has_len_non_array() {
    let int_ty = MirType::Int(IntType::I32);
    assert!(!int_ty.has_len());

    let ptr_ty = MirType::Int(IntType::I32).wrap_ptr().into_mir();
    assert!(!ptr_ty.has_len());
}

#[test]
fn test_function_type_of_local() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (5, MirType::Int(IntType::Bool)),
        (10, MirType::Int(IntType::U64)),
    ];
    let fun = create_test_function(locals);

    assert_eq!(fun.type_of_local(0), Some(MirType::Int(IntType::I32)));
    assert_eq!(fun.type_of_local(5), Some(MirType::Int(IntType::Bool)));
    assert_eq!(fun.type_of_local(10), Some(MirType::Int(IntType::U64)));
    assert_eq!(fun.type_of_local(999), None);
}

#[test]
fn test_nested_projections() {
    let tuple_ty = TupleType::new(vec![
        MirType::Int(IntType::I32),
        MirType::Int(IntType::Bool),
    ])
    .into_mir();
    let ptr_ty = tuple_ty.wrap_ptr().into_mir();

    let locals = vec![(0, ptr_ty)];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    // *local.1
    let place = Place::Projection(
        Box::new(Place::Projection(
            Box::new(Place::Local(0)),
            ProjectionKind::Deref,
        )),
        ProjectionKind::Field(1),
    );

    assert_eq!(
        place.get_type(&fun, &ctx),
        Some(MirType::Int(IntType::Bool))
    );
}

#[test]
fn test_all_comparison_ops_return_bool() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let ops = vec![BinOp::Eq, BinOp::Neq, BinOp::Lt, BinOp::Leq];

    for op in ops {
        let rval = RValue::BinOp(
            op,
            Operand::Copy(Place::Local(0)),
            Operand::Copy(Place::Local(1)),
        );
        assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::Bool)));
    }
}

#[test]
fn test_all_arithmetic_ops_preserve_type() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let ops = vec![
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::Div,
        BinOp::Mod,
        BinOp::ShiftLeft,
        BinOp::ShiftRight,
    ];

    for op in ops {
        let rval = RValue::BinOp(
            op,
            Operand::Copy(Place::Local(0)),
            Operand::Copy(Place::Local(1)),
        );
        assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::I32)));
    }
}

#[test]
fn test_logical_ops_return_bool() {
    let locals = vec![
        (0, MirType::Int(IntType::I32)),
        (1, MirType::Int(IntType::I32)),
    ];
    let fun = create_test_function(locals);
    let ctx = Context::new(PtrSize::_64Bit);

    let ops = vec![BinOp::And, BinOp::Or];

    for op in ops {
        let rval = RValue::BinOp(
            op,
            Operand::Copy(Place::Local(0)),
            Operand::Copy(Place::Local(1)),
        );
        assert_eq!(rval.get_type(&fun, &ctx), Some(MirType::Int(IntType::Bool)));
    }
}
