use crate::quill::{QuillError, QuillValue, Result};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::AddressSpace;
use std::convert::TryFrom;
use std::fmt::Debug;

///
/// The trait that defines behaviors that are applicable to all quill types
/// A quill type is equivalent to a data type
/// For example:
/// `QuillType var_name = QuillValue`
///
pub trait QuillType: Debug + Into<PolyQuillType> + TryFrom<PolyQuillType> + Eq + Clone {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>>;
}

///
/// An enum that can be one of any of the valid quill types
/// Note: This type implements `from` for each of it's subtypes. This automagically implements Into on each of it's subtypes, which satisfies the QuillType trait
/// Additionally, any type implicitly implements From<Self>
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PolyQuillType {
    StructType(QuillStructType),
    BoolType(QuillBoolType),
    IntType(QuillIntType),
    FloatType(QuillFloatType),
    ListType(QuillListType),
    FnType(QuillFnType),
    PointerType(QuillPointerType),
    VoidType(QuillVoidType),
}
impl QuillType for PolyQuillType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        match self {
            Self::StructType(st) => st.as_llvm_type(context, module),
            Self::BoolType(bt) => bt.as_llvm_type(context, module),
            Self::IntType(it) => it.as_llvm_type(context, module),
            Self::FloatType(ft) => ft.as_llvm_type(context, module),
            Self::ListType(lt) => lt.as_llvm_type(context, module),
            Self::FnType(ft) => ft.as_llvm_type(context, module),
            Self::PointerType(pt) => pt.as_llvm_type(context, module),
            Self::VoidType(vt) => vt.as_llvm_type(context, module),
        }
    }
}

///
/// A struct type, defined as the list of all the fields contained within the struct
/// Struct fields are identified by index, rather than by name
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuillStructType(String);
impl QuillStructType {
    pub fn new(name: String) -> Self {
        Self(name)
    }
    pub fn get_name(&self) -> String {
        self.0.clone()
    }
}
impl QuillType for QuillStructType {
    fn as_llvm_type<'ctx>(
        &self,
        _: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(module
            .get_struct_type(&self.get_name())
            .ok_or(QuillError::NoStruct(self.get_name()))?
            .as_basic_type_enum())
    }
}

///
/// A boolean type
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct QuillBoolType;
impl QuillBoolType {
    pub fn new() -> Self {
        Self
    }
}
impl QuillType for QuillBoolType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        _: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(context.custom_width_int_type(1).as_basic_type_enum())
    }
}

///
/// An integer type of the given bit width
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct QuillIntType(u32);
impl QuillIntType {
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }
    pub(super) fn bit_width(&self) -> u32 {
        self.0
    }
}
impl QuillType for QuillIntType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        _: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(context
            .custom_width_int_type(self.bit_width())
            .as_basic_type_enum())
    }
}

///
/// A float type
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct QuillFloatType;
impl QuillFloatType {
    pub fn new() -> Self {
        Self
    }
}
impl QuillType for QuillFloatType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        _: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(context.f64_type().as_basic_type_enum())
    }
}

///
/// A list type. The internal list type is another QuillType
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuillListType {
    internal: Box<PolyQuillType>,
    size: QuillListSize,
}
impl QuillListType {
    fn new(internal: impl QuillType, size: QuillListSize) -> Self {
        Self {
            internal: Box::new(internal.into()),
            size,
        }
    }

    pub fn new_var_length(internal: impl QuillType, size: QuillValue<QuillIntType>) -> Self {
        Self::new(internal, QuillListSize::Variable(size))
    }

    pub fn new_const_length(internal: impl QuillType, size: usize) -> Self {
        Self::new(internal, QuillListSize::Const(size))
    }

    pub(super) fn get_inner(&self) -> &PolyQuillType {
        &self.internal
    }

    pub(super) fn get_size(&self) -> &QuillListSize {
        &self.size
    }
}
impl QuillType for QuillListType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(self
            .internal
            .as_llvm_type(context, module)?
            .ptr_type(AddressSpace::Generic)
            .as_basic_type_enum())
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum QuillListSize {
    Const(usize),
    Variable(QuillValue<QuillIntType>),
}

///
/// A type that can be used to declare functions and pass fn pointers around
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuillFnType {
    return_type: Box<Option<PolyQuillType>>,
    params: Vec<(String, PolyQuillType)>,
}
impl QuillFnType {
    pub fn new(return_t: Option<impl QuillType>, params: Vec<(String, PolyQuillType)>) -> Self {
        Self {
            return_type: Box::new(return_t.map(|rt| rt.into())),
            params,
        }
    }

    pub fn void_return() -> Option<PolyQuillType> {
        None
    }

    pub fn void_return_value() -> Option<&'static QuillValue<PolyQuillType>> {
        Option::<&'static QuillValue<PolyQuillType>>::None
    }

    pub fn get_param_index(&self, name: &str) -> Result<usize> {
        for (i, (p_name, _)) in self.params.iter().enumerate() {
            if name == p_name {
                return Ok(i);
            }
        }

        Err(QuillError::NoSuchParam(String::from(name)))
    }

    pub fn get_params(&self) -> &[(String, PolyQuillType)] {
        &self.params
    }

    pub(super) fn get_ret_type(&self) -> &Option<PolyQuillType> {
        &self.return_type
    }
}
impl QuillType for QuillFnType {
    fn as_llvm_type<'ctx>(
        &self,
        _: &'ctx Context,
        _: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Err(QuillError::FnAsBTE)
    }
}

///
/// A pointer type
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuillPointerType(Box<PolyQuillType>);
impl QuillPointerType {
    pub fn new(internal: impl QuillType) -> Self {
        Self(Box::new(internal.into()))
    }
    pub fn get_inner_type(&self) -> PolyQuillType {
        *self.0.clone()
    }
}
impl QuillType for QuillPointerType {
    fn as_llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Ok(self
            .get_inner_type()
            .as_llvm_type(context, module)?
            .ptr_type(AddressSpace::Generic)
            .as_basic_type_enum())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct QuillVoidType;
impl QuillVoidType {
    pub fn new() -> Self {
        Self
    }
}
impl QuillType for QuillVoidType {
    fn as_llvm_type<'ctx>(
        &self,
        _: &'ctx Context,
        _: &Module<'ctx>,
    ) -> Result<BasicTypeEnum<'ctx>> {
        Err(QuillError::VoidType)
    }
}

macro_rules! poly_type_convert {
    (($quill_type: ident, $poly_quill_type: ident)) => {
        impl From<$quill_type> for PolyQuillType {
            fn from(q_t: $quill_type) -> Self {
                PolyQuillType::$poly_quill_type(q_t)
            }
        }
        impl TryFrom<PolyQuillType> for $quill_type {
            type Error = QuillError;

            fn try_from(value: PolyQuillType) -> Result<Self> {
                match value {
                    PolyQuillType::$poly_quill_type(pt) => Ok(pt),
                    _ => Err(
                        QuillError::WrongType(format!("{:?}", value),
                        String::from(stringify!($quill_type))),
                        String::from(format!("{}::try_from(PolyQuillType)", stringify!($quill_type))),
                    )
                }
            }
        }
    };
    (($quill_type: ident, $poly_quill_type: ident), $($tokens: tt),+) => {
        poly_type_convert!(($quill_type, $poly_quill_type));
        poly_type_convert!($($tokens),+);
    };
}

poly_type_convert!(
    (QuillPointerType, PointerType),
    (QuillBoolType, BoolType),
    (QuillIntType, IntType),
    (QuillStructType, StructType),
    (QuillListType, ListType),
    (QuillFloatType, FloatType),
    (QuillFnType, FnType),
    (QuillVoidType, VoidType)
);
