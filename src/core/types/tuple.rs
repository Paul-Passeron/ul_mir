use crate::core::types::MirType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleType {
    tys: Vec<MirType>,
}

impl TupleType {
    pub fn new(tys: Vec<MirType>) -> Self {
        Self { tys }
    }

    pub fn into_mir(self) -> MirType {
        MirType::Tuple(self)
    }

    pub fn fields(&self) -> &Vec<MirType> {
        &self.tys
    }
}

impl Into<MirType> for TupleType {
    fn into(self) -> MirType {
        self.into_mir()
    }
}
