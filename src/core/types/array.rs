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

    pub fn pointee(&self) -> &MirType {
        self.ty.as_ref()
    }

    pub fn length(&self) -> u32 {
        self.length
    }
}

impl Into<MirType> for ArrayType {
    fn into(self) -> MirType {
        self.into_mir()
    }
}
