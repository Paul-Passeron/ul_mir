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

    pub fn pointee(&self) -> &MirType {
        self.pointee.as_ref()
    }
}

impl Into<MirType> for PtrType {
    fn into(self) -> MirType {
        self.into_mir()
    }
}
