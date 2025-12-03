use crate::core::{
    Context,
    types::{
        array::ArrayType, function::FunctionType, int::IntType, ptr::PtrType, struct_ty::StructId,
        tuple::TupleType,
    },
};

pub mod array;
pub mod function;
pub mod int;
pub mod ptr;
pub mod struct_ty;
pub mod tuple;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirType {
    Void,
    Int(IntType),
    Ptr(PtrType),
    Array(ArrayType),
    Function(FunctionType),
    Tuple(TupleType),
    Struct(StructId),
}

impl MirType {
    pub fn as_ptr(&self) -> Option<&PtrType> {
        match self {
            MirType::Ptr(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<IntType> {
        match self {
            MirType::Int(x) => Some(*x),
            _ => None,
        }
    }

    pub fn fields(&self, ctx: &Context) -> Option<Vec<MirType>> {
        match self {
            MirType::Struct(struct_id) => {
                let as_struct = ctx.structs.get(struct_id)?;
                Some(as_struct.fields.iter().map(|x| &x.1).cloned().collect())
            }
            MirType::Tuple(tuple) => Some(tuple.fields().clone()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&ArrayType> {
        match self {
            MirType::Array(arr_ty) => Some(arr_ty),
            _ => None,
        }
    }

    pub fn has_len(&self) -> bool {
        match self {
            MirType::Array(_) => true,
            _ => false,
        }
    }

    pub fn wrap_ptr(&self) -> PtrType {
        PtrType::new(self.clone())
    }
}
