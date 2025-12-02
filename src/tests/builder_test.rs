use crate::{
    builder::Builder,
    context::FunctionBuilder,
    core::{Context, MirType},
};

#[test]
fn test_builder_new() {
    let mut ctx = Context::new();
    let fun = FunctionBuilder::new("test_builder_new".into())
        .push_param(MirType::Bool)
        .build(&mut ctx)
        .unwrap();

    let entry_block = ctx.reserve_new_block(fun).unwrap();

    let _ = Builder::new(fun, entry_block, 0, &mut ctx);
}
