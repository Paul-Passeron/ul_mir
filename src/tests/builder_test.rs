use crate::{
    builder::Builder,
    context::FunctionBuilder,
    core::{Context, IntType},
};

#[test]
fn test_builder_new() {
    let mut ctx = Context::new(crate::core::PtrSize::_64Bit);
    let fun = FunctionBuilder::new("test_builder_new".into())
        .push_param(IntType::Bool.into_mir())
        .build(&mut ctx)
        .unwrap();

    let entry_block = ctx.reserve_new_block(fun).unwrap();

    let _ = Builder::new(fun, entry_block, 0, &mut ctx);
}
