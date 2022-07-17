use std::convert::{TryFrom, TryInto};
use crate::compile::{Result, CompileError};
use crate::parse::ast::StructIdent;
use crate::quill::{PolyQuillType, QuillPointerType, QuillStructType, QuillValue};

///
/// A struct that represents a CrabStruct identifier and quill type paired together
///
#[derive(Clone, Debug)]
pub(super) struct IdentifiedValue {
    id: StructIdent,
    value: QuillValue<QuillPointerType>,
}

impl IdentifiedValue {
    pub(super) fn new(id: StructIdent, value: QuillValue<QuillPointerType>) -> Result<Self> {
         match value.get_type().get_inner_type() {
             PolyQuillType::StructType(_) => Ok(Self { id, value }),
            _ => Err(CompileError::ValueNotStruct),
        }
    }

    pub(super) fn try_from_parts(id: Option<StructIdent>, value: QuillValue<PolyQuillType>) -> Result<Self> {
        match id {
            Some(id) => Self::new(id, value.try_into()?),
            None => Err(CompileError::FromParts)
        }
    }

    pub(super) fn into_inner(self) -> (StructIdent, QuillValue<QuillPointerType>) {
        (self.id, self.value)
    }

    pub(super) fn into_parts(self) -> (Option<StructIdent>, QuillValue<PolyQuillType>) {
        (Some(self.id), self.value.into())
    }

    pub(super) fn get_quill_name(self) -> String {
        QuillStructType::try_from(self.value.get_type().get_inner_type()).unwrap().get_name()
    }
}
