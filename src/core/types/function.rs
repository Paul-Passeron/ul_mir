use crate::core::types::MirType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    args: Vec<MirType>,
    ret_ty: Box<MirType>,
    variadic: bool,
}

impl FunctionType {
    pub fn into_mir(self) -> MirType {
        MirType::Function(self)
    }

    pub fn new(args: Vec<MirType>, ret_ty: MirType, variadic: bool) -> Self {
        Self {
            args,
            ret_ty: Box::new(ret_ty),
            variadic,
        }
    }

    pub fn params(&self) -> &[MirType] {
        &self.args
    }

    pub fn ret_ty(&self) -> &MirType {
        &self.ret_ty
    }

    pub fn variadic(&self) -> bool {
        self.variadic
    }
}
