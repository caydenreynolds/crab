// use std::rc::Rc;
// use serde::Serialize;
// use crate::parse::ParseError;
// use crate::parse::node::ASTNode;
//
// #[derive(serde::Serialize)]
// pub struct ProgramNode {
//     below: Option<Rc<dyn ASTNode>>
// }
//
// impl ASTNode for ProgramNode {
//     fn get_right(&self) -> Option<Rc<dyn ASTNode>> {
//         None
//     }
//
//     fn get_below(&self) -> Option<Rc<dyn ASTNode>> {
//         return match self.below.clone() {
//             Some(below) => Some(below.clone()),
//             None => None,
//         }
//     }
//
//     fn set_right(&mut self, right: Rc<dyn ASTNode>) -> crate::parse::Result<()> {
//         Err(ParseError::ProgramRight)
//     }
//
//     fn set_below(&mut self, right: Rc<dyn ASTNode>) -> crate::parse::Result<()> {
//         return match self.below.clone() {
//             Some(_) => Err(ParseError::NodeAlreadySet),
//             None => {
//                 self.below = Some(right);
//                 Ok(())
//             }
//         }
//     }
// }
