use std::{fs::File, io::Write, path::PathBuf, process::Command};
use ul_mir::{
    core::{
        Context, PtrSize,
        builder::function_builder::FunctionBuilder,
        ctrl_flow::{BinOp, Constant, Operand, Place, ProjectionKind, RValue},
        types::int::IntType,
    },
    output::dot::{DotOutput, function::DotFormatter},
};

pub fn fbuilder() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let int_type = IntType::I64.into_mir();

    let printf_fun = FunctionBuilder::new("printf")
        .param(IntType::U8.into_mir().wrap_ptr().into())
        .variadic(true)
        .ret_ty(int_type.clone())
        .build(&mut ctx);

    let mut builder = FunctionBuilder::new("main")
        .param(IntType::I64.into())
        .param(IntType::U8.into_mir().wrap_ptr().into())
        .ret_ty(int_type.clone())
        .define(&mut ctx);

    let call_ret = builder.add_local(IntType::I64.into(), &mut ctx);

    builder
        .build_block(builder.entry_block(&ctx))
        // .stmt(Statement::StorageLive(call_ret))
        .assign(
            Place::Local(call_ret),
            RValue::Use(Operand::Call {
                func: printf_fun,
                args: vec![Operand::Constant(Constant::Str("Hello, World !\n".into()))],
            }),
            &ctx,
        )
        .unwrap()
        .ret(
            Some(Operand::Constant(Constant::Int(
                0,
                int_type.as_integer().unwrap(),
            ))),
            &mut ctx,
        );

    let main_fn = builder.finish();

    println!("{}", DotFormatter::new(&ctx, main_fn).to_dot());
}

pub fn fib() {
    let mut ctx = Context::new(PtrSize::_64Bit);
    let int_type = IntType::I64;
    let mut builder = FunctionBuilder::new("fib")
        .param(int_type.into())
        .ret_ty(int_type.into())
        .define(&mut ctx);

    let n_ptr = builder.add_local(int_type.into_mir().wrap_ptr().into(), &mut ctx);
    let a_ptr = builder.add_local(int_type.into_mir().wrap_ptr().into(), &mut ctx);
    let b_ptr = builder.add_local(int_type.into_mir().wrap_ptr().into(), &mut ctx);
    let cond = builder.add_local(IntType::Bool.into(), &mut ctx);

    let entry_id = builder.entry_block(&ctx);
    let cond_id = builder.reserve_block(&mut ctx);
    let body_id = builder.reserve_block(&mut ctx);
    let end_id = builder.reserve_block(&mut ctx);

    let deref_n = Place::Projection(Box::new(Place::Local(n_ptr)), ProjectionKind::Deref);
    let deref_a = Place::Projection(Box::new(Place::Local(a_ptr)), ProjectionKind::Deref);
    let deref_b = Place::Projection(Box::new(Place::Local(b_ptr)), ProjectionKind::Deref);
    let cond_val = Operand::Copy(Place::Local(cond));
    let param = Place::Local(builder.param(0, &ctx).unwrap());

    builder
        .build_block(entry_id)
        // .stmt(Statement::StorageLive(builder.param(0, &ctx).unwrap()))
        // .stmt(Statement::StorageLive(n_ptr))
        // .stmt(Statement::StorageLive(a_ptr))
        // .stmt(Statement::StorageLive(b_ptr))
        .assign(
            deref_a.clone(),
            RValue::Use(Operand::Constant(Constant::Int(1, int_type.clone()))),
            &ctx,
        )
        .unwrap()
        .assign(
            deref_b.clone(),
            RValue::Use(Operand::Constant(Constant::Int(1, int_type.clone()))),
            &ctx,
        )
        .unwrap()
        .assign(
            deref_n.clone(),
            RValue::Use(Operand::Constant(Constant::Int(2, int_type.clone()))),
            &ctx,
        )
        .unwrap()
        .goto(cond_id, &mut ctx)
        .unwrap();

    builder
        .build_block(cond_id)
        // .stmt(Statement::StorageLive(cond))
        .binop(
            BinOp::Lt,
            Operand::Copy(deref_n.clone()),
            Operand::Copy(param.clone()),
            Place::Local(cond),
            &ctx,
        )
        .unwrap()
        .br(cond_val, body_id, end_id, &mut ctx)
        .unwrap();

    builder
        .build_block(body_id)
        .binop(
            BinOp::Add,
            Operand::Copy(deref_a.clone()),
            Operand::Copy(deref_b.clone()),
            deref_a.clone(),
            &mut ctx,
        )
        .unwrap()
        .binop(
            BinOp::Sub,
            Operand::Copy(deref_a.clone()),
            Operand::Copy(deref_b.clone()),
            deref_b.clone(),
            &mut ctx,
        )
        .unwrap()
        .binop(
            BinOp::Add,
            Operand::Copy(deref_n.clone()),
            Operand::Constant(Constant::Int(1, int_type.clone())),
            deref_n.clone(),
            &mut ctx,
        )
        .unwrap()
        .goto(cond_id, &mut ctx)
        .unwrap();

    builder
        .build_block(end_id)
        // .stmt(Statement::StorageDead(cond))
        .ret(Some(Operand::Copy(deref_a)), &mut ctx);

    let fib = builder.finish();

    let mut f = File::create(PathBuf::from("fib.dot")).unwrap();
    f.write_fmt(format_args!("{}", DotFormatter::new(&ctx, fib).to_dot()))
        .unwrap();
    Command::new("dot")
        .arg("fib.dot")
        .arg("-Tpng")
        .arg("-o")
        .arg("fib.png")
        .spawn()
        .unwrap();
}

pub fn main() {
    fib();
}
