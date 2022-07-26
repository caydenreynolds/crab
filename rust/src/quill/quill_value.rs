use crate::quill::{
    PolyQuillType, QuillBoolType, QuillError, QuillFloatType, QuillFnType, QuillIntType,
    QuillListType, QuillPointerType, QuillStructType, QuillType, QuillVoidType,
};
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuillValue<T: QuillType> {
    id: usize,
    q_t: T,
}
impl From<QuillValue<T>> for QuillValue<PolyQuillType> {
    fn from(_: QuillValue<T>) -> Self {
        unreachable!()
    }
}

impl<T: QuillType> QuillValue<T> {
    pub fn get_type(&self) -> &T {
        &self.q_t
    }

    pub(super) fn new(id: usize, q_t: T) -> Self {
        Self { id, q_t }
    }

    pub(super) fn id(&self) -> usize {
        self.id
    }
}

macro_rules! poly_value_convert {
    (($quill_type: ident, $poly_quill_type: ident)) => {
        impl From<QuillValue<$quill_type>> for QuillValue<PolyQuillType> {
            fn from(q_v: QuillValue<$quill_type>) -> Self {
                Self {
                    id: q_v.id(),
                    q_t: q_v.get_type().clone().into(),
                }
            }
        }
        impl TryFrom<QuillValue<PolyQuillType>> for QuillValue<$quill_type> {
            type Error = QuillError;

            fn try_from(value: QuillValue<PolyQuillType>) -> Result<Self, Self::Error> {
                match value.get_type() {
                    PolyQuillType::$poly_quill_type(pt) => Ok(Self::new(value.id(), pt.clone())),
                    _ => Err(QuillError::WrongType(format!("{:?}", value), String::from(stringify!($quill_type))))
                }
            }
        }
    };
    (($quill_type: ident, $poly_quill_type: ident), $($tokens: tt),+) => {
        poly_value_convert!(($quill_type, $poly_quill_type));
        poly_value_convert!($($tokens),+);
    };
}

poly_value_convert!(
    (QuillPointerType, PointerType),
    (QuillBoolType, BoolType),
    (QuillIntType, IntType),
    (QuillStructType, StructType),
    (QuillListType, ListType),
    (QuillFloatType, FloatType),
    (QuillFnType, FnType),
    (QuillVoidType, VoidType)
);
