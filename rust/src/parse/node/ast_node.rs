// use std::fmt::Debug;
// use std::ops::Sub;
// use std::rc::Rc;
// use crate::parse::Result;
// use serde::Serialize;
//
// /// All pieces used in our AST must implement this trait
// ///
// pub trait ASTNode : Sized + Debug + Serialize + IntoIterator {
//     type SubnodeType: ASTNode;
//
//     /// Get the AST Node to the right of this one
//     ///
//     fn get_subnode(&self, i: usize) -> Self::SubnodeType;
//
//     /// Add a new subnode to this node
//     fn add_subnode(&self, new_node: Self::SubnodeType);
// }
//
// macro_rules! subnode_enum_types {
//     ($subNodeType:ty) => {
//         a(i32),
//     };
//     // ($subNodeType:ty, $($subNodeTypes:ty),+) => {
//     //     subnode_enum_types!($subNodeType)
//     //     subnode_enum_types!($($subNodeTypes),+)
//     // }
// }
//
// // macro_rules! subnode_get_subnode {
// //     ($name:ident, $subNodeType:ty, ) => {
// //
// //     }
// // }
// //
// // macro_rules! subnode_add_subnode {
// //
// // }
//
// /// Used to generate a type that is compliant with ASTNode::SubnodeType
// ///
// macro_rules! subnode_type {
//     ($name:ident, $subnodeType:ty) => {
//         #[derive(Debug, serde::Serialize)]
//         pub enum $name {
//             subnode_enum_types!($subNodeType);
//         }
//
//         // impl ASTNode for $name {
//         //     fn get_subnode(&self, i: usize) -> Self::SubnodeType {
//         //         return match self {
//         //             $subNodeType(a) => {
//         //                 a.get_subnode(i)
//         //             }
//         //         }
//         //     }
//         // }
//     }
// }
//
// subnode_type!(UwU, i32);