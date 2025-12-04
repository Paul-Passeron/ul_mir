use crate::core::types::MirType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
}

impl IntType {
    pub fn into_mir(self) -> MirType {
        MirType::Int(self)
    }
}

impl Into<MirType> for IntType {
    fn into(self) -> MirType {
        self.into_mir()
    }
}
