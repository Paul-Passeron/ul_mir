use ul_mir::core::{
    Context, PtrSize,
    builder::function_builder::FunctionBuilder,
    ctrl_flow::{Constant, Operand, Place, RValue},
    types::int::IntType,
};

pub fn fbuilder() {
    let mut ctx = Context::new(PtrSize::_64Bit);

    let int_type = IntType::I64.into_mir();

    let printf_fun = FunctionBuilder::new("ptintf")
        .param(IntType::U8.into_mir().wrap_ptr().into())
        .variadic(true)
        .ret_ty(int_type.clone())
        .build(&mut ctx);

    let builder = FunctionBuilder::new("main")
        .param(IntType::I64.into())
        .param(IntType::U8.into_mir().wrap_ptr().into())
        .ret_ty(int_type.clone())
        .define(&mut ctx);

    let mut bb1 = builder.build_block(builder.entry_block(&ctx));
    let call_place = Place::Local(builder.add_local(IntType::I64.into(), &mut ctx));
    bb1.assign(
        call_place,
        RValue::Use(Operand::Call {
            func: printf_fun,
            args: vec![Operand::Constant(Constant::Str("Hello, World !\n".into()))],
        }),
        &ctx,
    );
    bb1.ret(
        Some(Operand::Constant(Constant::Int(0, int_type.clone()))),
        &mut ctx,
    );

    let main_fn = builder.finish();
    println!("Main function created: {main_fn}");
}

pub fn main() {
    fbuilder();
}
