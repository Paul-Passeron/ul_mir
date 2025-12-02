use std::collections::{HashMap, HashSet};

use crate::core::{Function, MirType};

#[test]
// fn test_mirtype_equality() {
//     let prim_tys = vec![
//         MirType::Void,
//         MirType::I8,
//         MirType::I16,
//         MirType::I32,
//         MirType::I64,
//         MirType::U8,
//         MirType::U16,
//         MirType::U32,
//         MirType::U64,
//         MirType::Bool,
//     ];
//     for i in 0..prim_tys.len() {
//         for j in 0..prim_tys.len() {
//             if i == j {
//                 assert_eq!(prim_tys[i], prim_tys[j])
//             } else {
//                 assert_ne!(prim_tys[i], prim_tys[j])
//             }
//         }
//     }

//     let ptr1 = MirType::Void.wrap_ptr();
//     let ptr2 = MirType::Bool.wrap_ptr().wrap_ptr();
//     assert_ne!(ptr1, ptr2);

//     let ptr3 = MirType::Void.wrap_ptr();
//     assert_eq!(ptr1, ptr3);

//     let str1 = MirType::Struct(1);
//     let str2 = MirType::Struct(58);
//     assert_ne!(str1, str2);

//     for ty in &prim_tys {
//         assert_ne!(&str1, ty);
//         assert_ne!(&str2, ty);
//         assert_ne!(&ptr1, ty);
//         assert_ne!(&ptr2, ty);
//     }

//     let str3 = MirType::Struct(58);
//     assert_eq!(str2, str3);
// }
#[test]
fn test_function_reserve_new_block() {
    let mut fun = Function {
        name: "my_function".to_string(),
        params: vec![],
        variadic: false,
        return_ty: MirType::Void,
        locals: vec![],
        blocks: HashMap::new(),
        entry_block: None,
        reserved: HashSet::new(),
        last_reserved: 0,
    };

    for i in 0..1500 {
        let reserved = fun.reserve_new_block();
        assert_eq!(i, reserved);
    }
}

// fn get_random_ty(depth: i32) -> MirType {
//     if depth <= 0 {
//         let random_int = random::random::<u32>(..) % 14;
//         match random_int {
//             0 => MirType::Void,
//             1 => MirType::I8,
//             2 => MirType::I16,
//             3 => MirType::I32,
//             4 => MirType::I64,
//             5 => MirType::U8,
//             6 => MirType::U16,
//             7 => MirType::U32,
//             8 => MirType::U64,
//             _ => MirType::Bool,
//         }
//     } else {
//         let random_int = random::random::<u32>(..) % 14;
//         match random_int {
//             0 => MirType::Void,
//             1 => MirType::I8,
//             2 => MirType::I16,
//             3 => MirType::I32,
//             4 => MirType::I64,
//             5 => MirType::U8,
//             6 => MirType::U16,
//             7 => MirType::U32,
//             8 => MirType::U64,
//             9 => MirType::Bool,
//             10 => MirType::Ptr(Box::new(get_random_ty(depth - 1))),
//             11 => MirType::Struct(random::random(..)),
//             12 => {
//                 let len = random::random::<i32>(..) % 15;
//                 let mut tys = vec![];
//                 for _ in 0..len {
//                     let rand_ty = get_random_ty(depth - 1);
//                     tys.push(rand_ty);
//                 }
//                 MirType::Tuple(tys)
//             }
//             _ => {
//                 let len = random::random::<u32>(..);
//                 let ty = get_random_ty(depth - 1);
//                 MirType::Array(Box::new(ty), len)
//             }
//         }
//     }
// }

// #[test]
// fn test_function_new_local() {
//     let mut fun = Function {
//         name: "my_function".to_string(),
//         params: vec![],
//         variadic: false,
//         return_ty: MirType::Void,
//         locals: vec![],
//         blocks: HashMap::new(),
//         entry_block: None,
//         reserved: HashSet::new(),
//         last_reserved: 0,
//     };

//     let mut locals = HashSet::new();

//     for _ in 0..1500 {
//         let ty = get_random_ty(10);
//         let reserved = fun.new_local(ty.clone());
//         if !locals.insert(reserved) {
//             panic!("Duplicate local reserved");
//         }
//         assert_eq!(fun.type_of_local(reserved).unwrap(), ty)
//     }
// }
