use crate::core::{
    BinOp, Constant, Context, Function, FunctionLinkage, LocalId, Operand, Place, ProjectionKind,
    RValue, UnOp,
    types::{MirType, int::IntType, tuple::TupleType},
};

impl RValue {
    pub fn get_type(&self, fun: &Function<dyn FunctionLinkage>, ctx: &Context) -> Option<MirType> {
        match self {
            RValue::Use(operand) => operand.get_type(fun, ctx),
            RValue::BinOp(bin_op, lhs, rhs) => {
                let lhs_t = lhs.get_type(fun, ctx)?;
                let rhs_t = rhs.get_type(fun, ctx)?;
                if lhs_t != rhs_t || (lhs_t.as_integer().is_none() && lhs_t.as_ptr().is_none()) {
                    return None;
                }
                match bin_op {
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Mod
                    | BinOp::ShiftRight
                    | BinOp::ShiftLeft => {
                        if lhs_t == MirType::Int(IntType::Bool) {
                            return None;
                        }
                        Some(lhs_t)
                    }
                    BinOp::And | BinOp::Or | BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Leq => {
                        Some(MirType::Int(IntType::Bool))
                    }
                }
            }
            RValue::UnOp(un_op, operand) => {
                let operand_ty = operand.get_type(fun, ctx)?;
                match un_op {
                    UnOp::Not | UnOp::Minus => operand_ty.as_integer().map_or_else(
                        || operand_ty.as_ptr().cloned().map(|x| x.into_mir()),
                        |x| Some(x.into_mir()),
                    ),
                }
            }
            RValue::Cast(_, _, mir_type) => Some(mir_type.clone()),
            RValue::Ref(place) => Some(place.get_type(fun, ctx)?.wrap_ptr().into_mir()),
            RValue::Len(place) => {
                if !place.get_type(fun, ctx)?.has_len() {
                    return None;
                }
                Some(ctx.ptr_size.corresponding_int().into_mir())
            }
            RValue::Aggregate(operands) => Some(
                TupleType::new(
                    operands
                        .iter()
                        .map(|x| x.get_type(fun, ctx))
                        .collect::<Option<Vec<_>>>()?,
                )
                .into_mir(),
            ),
        }
    }
}

impl Operand {
    pub fn get_type(&self, fun: &Function<dyn FunctionLinkage>, ctx: &Context) -> Option<MirType> {
        match self {
            Operand::Copy(place) | Operand::Move(place) => place.get_type(fun, ctx),
            Operand::Constant(constant) => Some(constant.get_type()),
            Operand::Call { func, args } => {
                let fun = &ctx.functions.get(func)?.ty;
                if args.len() < fun.params().len() {
                    return None;
                }
                if !fun.variadic() && args.len() != fun.params().len() {
                    return None;
                }
                Some(fun.ret_ty().clone())
            }
        }
    }
}

impl Place {
    pub fn get_type(&self, fun: &Function<dyn FunctionLinkage>, ctx: &Context) -> Option<MirType> {
        match self {
            Place::Local(id) => fun.type_of_local(*id),
            Place::Projection(place, projection_kind) => {
                let place_ty = place.get_type(fun, ctx)?;
                match projection_kind {
                    ProjectionKind::Deref => place_ty.as_ptr().map(|x| x.pointee().clone()),
                    ProjectionKind::Field(index) => {
                        let fields = place_ty.fields(ctx)?;
                        fields.get(*index as usize).cloned()
                    }
                    ProjectionKind::Index(local) => {
                        fun.type_of_local(*local)?.as_integer()?;
                        place_ty.as_array().map_or_else(
                            || place_ty.as_ptr().map(|x| x.pointee().clone()),
                            |x| Some(x.pointee().clone()),
                        )
                    }
                }
            }
        }
    }
}

impl Function<dyn FunctionLinkage> {
    pub fn type_of_local(&self, local: LocalId) -> Option<MirType> {
        self.linkage
            .get_linkage()?
            .locals
            .iter()
            .find(|x| x.0 == local)
            .map(|x| x.1.clone())
    }
}

impl Constant {
    pub fn get_type(&self) -> MirType {
        match self {
            Constant::Int(_, ty) => ty.clone(),
            Constant::Bool(_) => IntType::Bool.into_mir(),
            Constant::Str(_) => IntType::U8.into_mir().wrap_ptr().into_mir(),
            Constant::Null => MirType::Void.wrap_ptr().into_mir(),
        }
    }
}
