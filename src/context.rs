// use crate::core::{Context, MirType};

// pub struct FunctionBuilder {
//     name: String,
//     params: Vec<MirType>,
//     return_ty: MirType,
//     variadic: bool,
// }

// impl FunctionBuilder {
//     pub fn new(name: String) -> Self {
//         Self {
//             name,
//             params: vec![],
//             return_ty: MirType::Void,
//             variadic: false,
//         }
//     }

//     pub fn push_param(mut self, ty: MirType) -> Self {
//         self.params.push(ty);
//         self
//     }

//     pub fn insert_param(mut self, idx: usize, ty: MirType) -> Option<Self> {
//         if idx > self.params.len() {
//             None
//         } else {
//             self.params.insert(idx, ty);
//             Some(self)
//         }
//     }

//     pub fn build(self, ctx: &mut Context) -> Option<MirFunId> {
//         let mut params = vec![];

//         let mut current_local = 0;
//         for param in self.params {
//             params.push((current_local, param.clone()));
//             current_local += 1;
//         }

//         let fun = Function {
//             name: self.name,
//             params: params.clone(),
//             variadic: self.variadic,
//             return_ty: self.return_ty,
//             locals: params,
//             blocks: HashMap::new(),
//             entry_block: None,
//             reserved: HashSet::new(),
//             last_reserved: 0,
//         };

//         let fun_id = ctx.functions.keys().max().map_or(0, |x| x + 1);
//         ctx.functions.insert(fun_id, fun);

//         Some(fun_id)
//     }
// }

// impl Context {}
