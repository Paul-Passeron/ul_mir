use crate::core::types::MirType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtrType {
    pointee: Box<MirType>,
}

impl PtrType {
    pub fn new(pointee: MirType) -> Self {
        Self {
            pointee: Box::new(pointee),
        }
    }

    pub fn into_mir(self) -> MirType {
        MirType::Ptr(self)
    }
}
