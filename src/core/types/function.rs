use crate::core::types::MirType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    params: Vec<MirType>,
    ret_ty: Box<MirType>,
    variadic: bool,
}

impl FunctionType {
    pub fn into_mir(self) -> MirType {
        MirType::Function(self)
    }

    pub fn new(params: Vec<MirType>, ret_ty: MirType, variadic: bool) -> Self {
        Self {
            params,
            ret_ty: Box::new(ret_ty),
            variadic,
        }
    }

    pub fn params(&self) -> &[MirType] {
        &self.params
    }

    pub fn ret_ty(&self) -> &MirType {
        &self.ret_ty
    }

    pub fn variadic(&self) -> bool {
        self.variadic
    }

    pub fn set_variadic(&mut self, v: bool) {
        self.variadic = v;
    }

    pub fn set_return_ty(&mut self, ty: MirType) {
        *self.ret_ty = ty;
    }

    pub fn set_params(&mut self, params: Vec<MirType>) {
        self.params = params
    }
}
