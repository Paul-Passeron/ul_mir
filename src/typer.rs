use crate::core::{
    BinOp, Constant, Context, Function, LocalId, MirType, Operand, Place, ProjectionKind, RValue,
    UnOp,
};

impl RValue {
    pub fn get_type(&self, fun: &Function, ctx: &Context) -> Option<MirType> {
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
                        if lhs_t == MirType::Bool {
                            return None;
                        }
                        Some(lhs_t)
                    }
                    BinOp::And | BinOp::Or | BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Leq => {
                        Some(MirType::Bool)
                    }
                }
            }
            RValue::UnOp(un_op, operand) => {
                let operand_ty = operand.get_type(fun, ctx)?;
                match un_op {
                    UnOp::Not | UnOp::Minus => operand_ty
                        .as_integer()
                        .map_or_else(|| operand_ty.as_ptr(), Some),
                }
            }
            RValue::Cast(_, _, mir_type) => Some(mir_type.clone()),
            RValue::Ref(place) => Some(place.get_type(fun, ctx)?.wrap_ptr()),
            RValue::Len(place) => {
                if !place.get_type(fun, ctx)?.has_len() {
                    return None;
                }
                ctx.ptr_size.as_integer()
            }
            RValue::Aggregate(operands) => Some(MirType::Tuple(
                operands
                    .iter()
                    .map(|x| x.get_type(fun, ctx))
                    .collect::<Option<Vec<_>>>()?,
            )),
        }
    }
}

impl Operand {
    pub fn get_type(&self, fun: &Function, ctx: &Context) -> Option<MirType> {
        match self {
            Operand::Copy(place) | Operand::Move(place) => place.get_type(fun, ctx),
            Operand::Constant(constant) => Some(constant.get_type()),
            Operand::Call { func, args } => {
                let fun = ctx.functions.get(func)?;
                if args.len() < fun.params.len() {
                    return None;
                }
                if !fun.variadic && args.len() != fun.params.len() {
                    return None;
                }
                Some(fun.return_ty.clone())
            }
        }
    }
}

impl Place {
    pub fn get_type(&self, fun: &Function, ctx: &Context) -> Option<MirType> {
        match self {
            Place::Local(id) => fun.type_of_local(*id),
            Place::Projection(place, projection_kind) => {
                let place_ty = place.get_type(fun, ctx)?;
                match projection_kind {
                    ProjectionKind::Deref => place_ty.as_ptr(),
                    ProjectionKind::Field(index) => {
                        let fields = place_ty.fields(ctx)?;
                        fields.get(*index as usize).cloned()
                    }
                    ProjectionKind::Index(local) => {
                        fun.type_of_local(*local)?.as_integer()?;
                        place_ty.as_array().map_or_else(|| place_ty.as_ptr(), Some)
                    }
                }
            }
        }
    }
}

impl Function {
    pub fn type_of_local(&self, local: LocalId) -> Option<MirType> {
        self.locals
            .iter()
            .find(|x| x.0 == local)
            .map(|x| x.1.clone())
    }
}

impl MirType {
    pub fn as_ptr(&self) -> Option<MirType> {
        match self {
            MirType::Ptr(inner) => Some(inner.as_ref().clone()),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<MirType> {
        match self {
            MirType::I8
            | MirType::I16
            | MirType::I32
            | MirType::I64
            | MirType::U8
            | MirType::U16
            | MirType::U32
            | MirType::U64
            | MirType::Bool => Some(self.clone()),
            _ => None,
        }
    }

    pub fn fields(&self, ctx: &Context) -> Option<Vec<MirType>> {
        match self {
            MirType::Struct(struct_id) => {
                let as_struct = ctx.structs.get(struct_id)?;
                Some(as_struct.fields.iter().map(|x| &x.1).cloned().collect())
            }
            MirType::Tuple(mir_types) => Some(mir_types.clone()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<MirType> {
        match self {
            MirType::Array(ty, _) => Some(ty.as_ref().clone()),
            _ => None,
        }
    }

    pub fn has_len(&self) -> bool {
        match self {
            MirType::Array(_, _) => true,
            _ => false,
        }
    }

    pub fn wrap_ptr(&self) -> MirType {
        MirType::Ptr(Box::new(self.clone()))
    }
}

impl Constant {
    pub fn get_type(&self) -> MirType {
        match self {
            Constant::Int(_, ty) => ty.clone(),
            Constant::Bool(_) => MirType::Bool,
            Constant::Str(_) => MirType::U8.wrap_ptr(),
            Constant::Null => MirType::Void.wrap_ptr(),
        }
    }
}
