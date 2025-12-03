use crate::core::types::MirType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    ty: Box<MirType>,
    length: u32,
}

impl ArrayType {
    pub fn into_mir(self) -> MirType {
        MirType::Array(self)
    }
}
