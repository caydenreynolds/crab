use crate::quill::quill_types::QuillListSize;
use crate::quill::{
    PolyQuillType, Quill, QuillBoolType, QuillError, QuillFnType, QuillIntType, QuillListType,
    QuillPointerType, QuillStructType, QuillType, QuillValue, Result,
};
use crate::util::{ListFunctional, ListReplace};
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::AnyTypeEnum;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};
use log::trace;
use std::convert::TryFrom;
use std::fmt::Debug;

pub type IntCmpType = IntPredicate;

///
/// Enum of all the possible instructions that can be stored in a nib
///
#[derive(Debug, Clone)]
enum Instruction {
    Return(Option<usize>),                                // Return value
    ConditionalBranch(usize, ChildNib, Option<ChildNib>), // Condition id, t_branch, f_branch
    UnconditionalBranch(ChildNib),                        // Child to branch to
    ConditionalLoop(usize),                               // Condition id
    Unreachable,
    StructGet(usize, usize, String), // Source id, destination id, name of element to get
    StructSet(usize, usize, String), // Struct id, source id, name of element to set
    ConstInt(usize, u32, u64),       // Value id, bit width, value
    ConstBool(usize, bool),          // Value id, bool value
    ConstString(usize, String),      // Id, value
    Alloca(usize, PolyQuillType),    // Ptr id, type
    Malloc(usize, PolyQuillType),    // Ptr id, type
    Store(usize, usize),             // Ptr id, val id
    Load(usize, usize),              // Ptr id, val id
    FnCall(String, usize, Vec<usize>), // Fn name, return id, positional params
    FnParam(usize, String),          // Param id, param name
    IntAdd(usize, usize, usize),     // Result id, lhs id, rhs id
    ListValueSet(usize, usize, usize), // List id, value id, index id
    ListValueGet(usize, usize, usize), // List id, value id, index id
    ListCopy(usize, usize, usize, usize), // Old list id, new list id, list len, dest index id
    Free(usize),                     // Value id
    IntCmp(usize, usize, usize, IntCmpType), // Lhs id, rhs id, result id, comparison type
}

///
/// Nib trait defines all the behavior of a Nib
/// /// Essentially an inkwell builder for a specific codeblock
///
pub trait Nib: Debug {
    ///
    /// Creates a Nib that can be used as a child of this Nib
    ///
    fn create_child(&self) -> ChildNib;

    ///
    /// Add a return statement to the nib
    /// If value is none, a void return will be added
    /// If value is some, the PolyQuillValue will be returned
    ///
    /// Params:
    /// * `value` - The value to return
    ///
    fn add_return<T: QuillType>(&mut self, value: Option<&QuillValue<T>>);

    ///
    /// Adds a conditional branch instruction to the Nib
    /// If the branches do not terminate in a branch or return instruction, control will
    /// automatically be branched back to the parent nib once the end of the branch is reached
    ///
    /// If f_branch is not supplied and cond is false, the parent branch will continue
    ///
    /// Params:
    /// * `cond` - The condition to evaluate
    /// * `t_branch` - The branch to jump to if cond is true
    /// * `f_branch` - The optional branch to jump to if cond is false
    ///
    fn add_cond_branch(
        &mut self,
        cond: &QuillValue<QuillBoolType>,
        t_branch: ChildNib,
        f_branch: Option<ChildNib>,
    );

    ///
    /// Adds an unconditional branch instruction to the Nib
    /// Once the end of the branch is reached, the parent Nib will continue
    ///
    /// Params:
    /// * `branch` - The branch to jump to
    ///
    fn add_branch(&mut self, branch: ChildNib);

    ///
    /// Adds a conditional branch instruction to the Nib
    /// The target of the conditional branch will always be this Nib
    /// This way, a loop is formed
    ///
    /// If cond is true, the loop will continue
    /// If cond is false, the parent branch will continue
    ///
    /// Params:
    /// * `cond` - The condition to evaluate
    ///
    fn add_cond_loop(&mut self, cond: &QuillValue<QuillBoolType>);

    ///
    /// An unreachable statement is used to indicate that a portion of a Nib will never be reached
    /// The llvm compiler requires each codeblock to have a terminating instruction, and unreachable
    /// satisfies this requirement. For this reason, unreachables are sometimes required.
    ///
    fn build_unreachable(&mut self);

    ///
    /// Retrieves a field from a struct value by name
    ///
    /// Params:
    /// * `sv` - The struct value to retrieve the field from
    /// * `name` - The name of the field to retrieve
    /// * `expected_type` - The type that should be inside the pointer
    ///
    /// Returns:
    /// A pointer to a quill value with the given type
    ///
    fn get_value_from_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        expected_type: T,
    ) -> Result<QuillValue<T>>;

    ///
    /// Sets a field in a struct value by name
    ///
    /// Params:
    /// * `sv` - A pointer valeu to the struct value to set the field in
    /// * `name` - The name of the field to set
    /// * `value` - The value to set the field to
    ///
    fn set_value_in_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        value: &QuillValue<T>,
    ) -> Result<()>;

    ///
    /// Creates an unsigned integer with a given bit width and value
    ///
    /// Params:
    /// * `bits` - The bit width of the new value
    /// * `value` - The actual value of the integer
    ///
    /// Returns:
    /// The created integer value
    ///
    fn const_int(&mut self, bits: u32, value: u64) -> QuillValue<QuillIntType>;

    ///
    /// Creates a bool value with the given value
    ///
    /// Params:
    /// * `value` - The actual value of the bool
    ///
    /// Returns:
    /// The created bool value
    ///
    fn const_bool(&mut self, value: bool) -> QuillValue<QuillBoolType>;

    ///
    /// Creates a string value with the given value
    ///
    /// Params:
    /// * `value` - The actual value of the string
    /// * `null_term` - Whether or not the string should be null terminated
    ///
    /// Returns:
    /// A pointer to the created string value
    ///
    fn const_string(&mut self, value: String) -> QuillValue<QuillPointerType>;

    ///
    /// Adds an alloca instruction to the Nib
    /// This allocates stack memory
    ///
    /// Params:
    /// * `t` - The type to malloc
    ///
    /// Returns:
    /// A pointer to the created value
    ///
    fn add_alloca<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType>;

    ///
    /// Adds a malloc instruction to the Nib
    /// This allocates heap memory
    ///
    /// Params:
    /// * `t` - The type to malloc
    ///
    /// Returns:
    /// A pointer to the created value
    ///
    fn add_malloc<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType>;

    ///
    /// Stores a value into a pointer
    ///
    /// Params:
    /// * `ptr` - The pointer to store the value into
    /// * `value` - The value to store
    ///
    fn add_store<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
    ) -> Result<()>;

    ///
    /// Loads a value from a pointer
    ///
    /// Params:
    /// * `ptr` - The pointer to load the value from
    /// * `t` - The type of the value to fetch
    ///
    /// Returns:
    /// The loaded value
    ///
    fn add_load<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        expected_type: T,
    ) -> Result<QuillValue<T>>;

    ///
    /// Add a function call to the Nib
    ///
    /// Params:
    /// * `name` - The name of the function to call
    /// * `args` - The arguments to the function
    /// * `expected_type` - The expected return type of the function
    ///
    /// Returns:
    /// A value of the expected type
    ///
    fn add_fn_call<T: QuillType>(
        &mut self,
        name: String,
        args: Vec<QuillValue<PolyQuillType>>,
        expected_type: T,
    ) -> QuillValue<T>;

    ///
    /// Creates an integer addition instruction
    /// Both params must have the same bit width, and the result will have the same bit width as the params
    ///
    /// Params:
    /// * `lhs` - One of the ints to add
    /// * `rhs` - The other of the ints to add
    ///
    /// Returns:
    /// A value representing to two ints added together
    ///
    fn int_add(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
    ) -> Result<QuillValue<QuillIntType>>;

    ///
    /// Returns a reference to the fntype this nib is built from
    ///
    /// Returns:
    /// The QuillFnType of this Nib
    ///
    fn get_fn_t(&self) -> &QuillFnType;

    ///
    /// Sets a value in a list
    ///
    /// Params:
    /// * `lv` - A pointer to the list
    /// * `value` - The value to set the field to
    /// * `index` - The index in the list to set
    ///
    fn set_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
        index: &QuillValue<QuillIntType>,
    ) -> Result<()>;

    ///
    /// Gets a value from a list
    ///
    /// Params:
    /// * `lv` - A pointer to the list
    /// * `index` - The index in the list to set
    /// * `expected_type` - The type of element expected. Must match the type in the list
    ///
    /// Returns:
    /// The value fetched from the list
    ///
    fn get_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        index: &QuillValue<QuillIntType>,
        expected_type: T,
    ) -> Result<QuillValue<T>>;

    ///
    /// Copies one list from another
    ///
    /// Params:
    /// * `ol` - A pointer to the old list
    /// * `nl` - A pointer to the new list set
    ///
    /// Returns:
    /// The value fetched from the list
    ///
    fn list_copy(
        &mut self,
        ol: &QuillValue<QuillPointerType>,
        nl: &QuillValue<QuillPointerType>,
        len: &QuillValue<QuillIntType>,
        dest_index: &QuillValue<QuillIntType>,
    ) -> Result<()>;

    ///
    /// Free a value
    ///
    /// Params:
    /// * `val` - The value to free
    ///
    fn free(&mut self, val: QuillValue<QuillPointerType>);

    ///
    /// Compare two int types
    ///
    /// Params:
    /// * `lhs` - The left hand value
    /// * `rhs` - The right hand value
    /// * `cmp_type` - The type of comparison to perform
    ///
    /// Returns:
    /// The boolean result of the comparison
    ///
    fn int_cmp(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
        cmp_type: IntCmpType,
    ) -> Result<QuillValue<QuillBoolType>>;
}

///
/// A Nib that must be a child of a function
///
#[derive(Debug, Clone)]
pub struct FnNib {
    inner: ChildNib,
    fn_name: String,
}

impl FnNib {
    pub fn new(name: String, fn_type: QuillFnType) -> Self {
        Self {
            inner: ChildNib::new(fn_type, 0),
            fn_name: name,
        }
    }

    ///
    /// Gets a value from a function param
    ///
    /// Params:
    /// * `name` - The name of the value to get
    /// * `expected_type` - The expected type of the value
    ///
    /// Returns:
    /// A value that matches the expected type
    ///
    pub fn get_fn_param<T: QuillType>(&mut self, name: String, expected_type: T) -> QuillValue<T> {
        self.inner
            .instructions
            .push(Instruction::FnParam(self.inner.id_generator, name));
        let val = QuillValue::new(self.inner.id_generator, expected_type);
        self.inner.id_generator += 1;
        val
    }

    ///
    /// Gets the name of the function this nib is tied to
    ///
    /// Returns:
    /// The function's name
    ///
    pub fn get_fn_name(&self) -> &String {
        &self.fn_name
    }

    ///
    /// Inkwell's types behave unexpectedly, and may be modified at any time, by anybody. They don't even need to be mutable.
    /// We are beyond the help of rust's safety guarantees now...
    pub(super) fn commit<'ctx>(
        self,
        peter: &Quill,
        context: &'ctx Context,
        module: &Module<'ctx>,
        header: &QuillFnType,
    ) -> Result<()> {
        let fn_val = module
            .get_function(&self.fn_name)
            .ok_or(QuillError::FnNotFound(self.fn_name))?;
        self.inner
            .commit(peter, context, module, fn_val, &header, &vec![], None)?;
        Ok(())
    }
}
impl Nib for FnNib {
    fn create_child(&self) -> ChildNib {
        self.inner.create_child()
    }
    fn add_return<T: QuillType>(&mut self, value: Option<&QuillValue<T>>) {
        self.inner.add_return(value)
    }
    fn add_cond_branch(
        &mut self,
        cond: &QuillValue<QuillBoolType>,
        t_branch: ChildNib,
        f_branch: Option<ChildNib>,
    ) {
        self.inner.add_cond_branch(cond, t_branch, f_branch)
    }
    fn add_branch(&mut self, branch: ChildNib) {
        self.inner.add_branch(branch)
    }
    fn add_cond_loop(&mut self, cond: &QuillValue<QuillBoolType>) {
        self.inner.add_cond_loop(cond)
    }
    fn build_unreachable(&mut self) {
        self.inner.build_unreachable()
    }
    fn get_value_from_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        self.inner.get_value_from_struct(pv, name, expected_type)
    }
    fn set_value_in_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        value: &QuillValue<T>,
    ) -> Result<()> {
        self.inner.set_value_in_struct(pv, name, value)
    }
    fn const_int(&mut self, bits: u32, value: u64) -> QuillValue<QuillIntType> {
        self.inner.const_int(bits, value)
    }
    fn const_bool(&mut self, value: bool) -> QuillValue<QuillBoolType> {
        self.inner.const_bool(value)
    }
    fn const_string(&mut self, value: String) -> QuillValue<QuillPointerType> {
        self.inner.const_string(value)
    }
    fn add_alloca<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType> {
        self.inner.add_alloca(t)
    }
    fn add_malloc<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType> {
        self.inner.add_malloc(t)
    }
    fn add_store<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
    ) -> Result<()> {
        self.inner.add_store(ptr, value)
    }
    fn add_load<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        self.inner.add_load(ptr, expected_type)
    }
    fn add_fn_call<T: QuillType>(
        &mut self,
        name: String,
        args: Vec<QuillValue<PolyQuillType>>,
        expected_type: T,
    ) -> QuillValue<T> {
        self.inner.add_fn_call(name, args, expected_type)
    }
    fn int_add(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
    ) -> Result<QuillValue<QuillIntType>> {
        self.inner.int_add(lhs, rhs)
    }
    fn get_fn_t(&self) -> &QuillFnType {
        self.inner.get_fn_t()
    }
    fn set_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
        index: &QuillValue<QuillIntType>,
    ) -> Result<()> {
        self.inner.set_list_value(lv, value, index)
    }
    fn get_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        index: &QuillValue<QuillIntType>,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        self.inner.get_list_value(lv, index, expected_type)
    }
    fn list_copy(
        &mut self,
        ol: &QuillValue<QuillPointerType>,
        nl: &QuillValue<QuillPointerType>,
        len: &QuillValue<QuillIntType>,
        dest_index: &QuillValue<QuillIntType>,
    ) -> Result<()> {
        self.inner.list_copy(ol, nl, len, dest_index)
    }
    fn free(&mut self, val: QuillValue<QuillPointerType>) {
        self.inner.free(val)
    }
    fn int_cmp(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
        cmp_type: IntCmpType,
    ) -> Result<QuillValue<QuillBoolType>> {
        self.inner.int_cmp(lhs, rhs, cmp_type)
    }
}

///
/// A Nib that must be a child of a parent nib
///
#[derive(Debug, Clone)]
pub struct ChildNib {
    instructions: Vec<Instruction>,
    parent_fn: QuillFnType,
    id_generator: usize,
    instruction_pointer: usize,
}

impl ChildNib {
    fn new(parent_fn: QuillFnType, id_generator: usize) -> Self {
        Self {
            instructions: vec![],
            parent_fn,
            id_generator,
            instruction_pointer: 0,
        }
    }
    fn next_instruction(&mut self) -> Option<Instruction> {
        let instr = self.instructions.get(self.instruction_pointer);
        self.instruction_pointer += 1;
        instr.cloned()
    }
    fn get_num_values(&self) -> usize {
        self.id_generator
    }

    fn commit<'ctx>(
        mut self,
        peter: &Quill,
        context: &'ctx Context,
        module: &Module<'ctx>,
        fn_val: FunctionValue<'ctx>,
        header: &QuillFnType,
        parent_values: &Vec<Option<BasicValueEnum<'ctx>>>,
        after: Option<BasicBlock<'ctx>>,
    ) -> Result<BasicBlock<'ctx>> {
        trace!("ChildNib::commit() called");
        // Prepare the things we'll need
        let first_basic_block = context.append_basic_block(fn_val, "block");
        let mut curr_basic_block = first_basic_block;
        let builder = context.create_builder();
        builder.position_at_end(first_basic_block);
        let mut values: Vec<Option<BasicValueEnum<'ctx>>> = (0..self.get_num_values())
            .map(|i| {
                if i < parent_values.len() {
                    parent_values.get(i).unwrap().clone()
                } else {
                    None
                }
            })
            .collect();

        // Start building
        while let Some(instruction) = self.next_instruction() {
            trace!("Building instruction {:?}", instruction);
            match instruction {
                Instruction::Return(id) => {
                    builder.build_return(match id {
                        None => None,
                        Some(id) => values
                            .get(id)
                            .unwrap()
                            .as_ref()
                            .map(|v| v as &dyn BasicValue),
                    });
                }

                Instruction::ConditionalBranch(id, t_branch, f_branch) => {
                    curr_basic_block = context.append_basic_block(fn_val, "block");
                    let cond = values.get(id).unwrap().ok_or(QuillError::BadValueAccess)?;
                    let cond = match cond {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        t => Err(QuillError::WrongType(
                            format!("{:?}", t),
                            String::from("BoolValue"),
                            String::from("Nib::commit::ConditionalBranch"),
                        )),
                    }?;
                    if cond.get_type().get_bit_width() != 1 {
                        return Err(QuillError::WrongType(
                            format!("Integer with width {}", cond.get_type().get_bit_width()),
                            String::from("BoolValue"),
                            String::from("Nib::commit::ConditionalBranch"),
                        ));
                    }

                    let t_branch_block = t_branch.commit(
                        peter,
                        context,
                        module,
                        fn_val,
                        &header,
                        &values,
                        Some(curr_basic_block),
                    )?;
                    match f_branch {
                        None => {
                            builder.build_conditional_branch(cond, t_branch_block, curr_basic_block)
                        }
                        Some(f_branch) => {
                            let f_branch_block = f_branch.commit(
                                peter,
                                context,
                                module,
                                fn_val,
                                &header,
                                &values,
                                Some(curr_basic_block),
                            )?;
                            builder.build_conditional_branch(cond, t_branch_block, f_branch_block)
                        }
                    };
                    builder.position_at_end(curr_basic_block);
                }

                Instruction::UnconditionalBranch(to) => {
                    curr_basic_block = context.append_basic_block(fn_val, "block");
                    let to_block = to.commit(
                        peter,
                        context,
                        module,
                        fn_val,
                        &header,
                        &values,
                        Some(curr_basic_block),
                    )?;
                    builder.build_unconditional_branch(to_block);
                    builder.position_at_end(curr_basic_block);
                }

                Instruction::ConditionalLoop(cond_id) => {
                    let cond = values
                        .get(cond_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let cond = match cond {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        t => Err(QuillError::WrongType(
                            format!("{:?}", t),
                            String::from("BoolValue"),
                            String::from("Nib::commit::ConditionalLoop"),
                        )),
                    }?;
                    if cond.get_type().get_bit_width() != 1 {
                        return Err(QuillError::WrongType(
                            format!("Integer with width {}", cond.get_type().get_bit_width()),
                            String::from("BoolValue"),
                            String::from("Nib::commit::ConditionalLoop"),
                        ));
                    }
                    builder.build_conditional_branch(
                        cond,
                        curr_basic_block,
                        after.ok_or(QuillError::NoAfter)?,
                    );
                }

                Instruction::Unreachable => {
                    builder.build_unreachable();
                }

                Instruction::StructGet(source_id, dest_id, field_name) => {
                    let source = values
                        .get(source_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    match source {
                        BasicValueEnum::PointerValue(ptr) => {
                            match ptr.get_type().get_element_type() {
                                AnyTypeEnum::StructType(strct_type) => {
                                    let name = strct_type.get_name().unwrap().to_str()?;
                                    let q_strct = peter
                                        .get_struct_defintion(name)
                                        .ok_or(QuillError::NoStruct(name.into()))?;
                                    let field_ptr = builder
                                        .build_struct_gep(
                                            ptr,
                                            q_strct.get_index(&field_name)?,
                                            "struct_get",
                                        )
                                        .or(Err(QuillError::Gep))?;
                                    let loaded = builder.build_load(field_ptr, "loaded");
                                    values.replace(dest_id, Some(loaded));
                                }
                                t => {
                                    return Err(QuillError::WrongType(
                                        format!("{:?}", t),
                                        String::from("Struct pointer"),
                                        String::from("Nib::commit::StructGet"),
                                    ))
                                }
                            }
                        }
                        t => {
                            return Err(QuillError::WrongType(
                                format!("{:?}", t),
                                String::from("Struct pointer"),
                                String::from("Nib::commit::StructGet"),
                            ))
                        }
                    }
                }

                Instruction::StructSet(strct_id, source_id, field_name) => {
                    let source = values
                        .get(strct_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    match source {
                        BasicValueEnum::PointerValue(ptr) => {
                            match ptr.get_type().get_element_type() {
                                AnyTypeEnum::StructType(strct_type) => {
                                    let name = strct_type.get_name().unwrap().to_str()?;
                                    let q_strct = peter
                                        .get_struct_defintion(name)
                                        .ok_or(QuillError::NoStruct(name.into()))?;
                                    let field_ptr = builder
                                        .build_struct_gep(
                                            ptr,
                                            q_strct.get_index(&field_name)?,
                                            "struct_get",
                                        )
                                        .or(Err(QuillError::Gep))?;
                                    let source = values
                                        .get(source_id)
                                        .unwrap()
                                        .ok_or(QuillError::BadValueAccess)?;
                                    builder.build_store(field_ptr, source);
                                }
                                t => {
                                    return Err(QuillError::WrongType(
                                        format!("{:?}", t),
                                        String::from("Struct pointer"),
                                        String::from("Nib::commit::StructSet"),
                                    ))
                                }
                            }
                        }
                        t => {
                            return Err(QuillError::WrongType(
                                format!("{:?}", t),
                                String::from("Struct pointer"),
                                String::from("Nib::commit::StructSet"),
                            ))
                        }
                    }
                }

                Instruction::ConstInt(id, bits_width, value) => {
                    values.replace(
                        id,
                        Some(
                            context
                                .custom_width_int_type(bits_width)
                                .const_int(value, false)
                                .as_basic_value_enum(),
                        ),
                    );
                }

                Instruction::ConstBool(id, value) => {
                    values.replace(
                        id,
                        Some(
                            context
                                .custom_width_int_type(1)
                                .const_int(value as u64, false)
                                .as_basic_value_enum(),
                        ),
                    );
                }

                Instruction::ConstString(id, value) => {
                    let const_string = builder
                        .build_global_string_ptr(&value, "str_ptr")
                        .as_pointer_value();
                    // Using value.len() means we don't copy the null byte
                    let string_len = context.i64_type().const_int(value.len() as u64, false);
                    let string_array = builder
                        .build_array_malloc(context.i8_type(), string_len, "const_string")
                        .or(Err(QuillError::MallocErr))?;
                    builder
                        .build_memcpy(string_array, 1, const_string, 1, string_len)
                        .or(Err(QuillError::Memcpy))?;
                    values.replace(id, Some(string_array.as_basic_value_enum()));
                }

                Instruction::Alloca(dest_id, q_type) => match q_type {
                    PolyQuillType::PointerType(pt) => match pt.get_inner_type() {
                        PolyQuillType::StructType(qst) => {
                            let l_t = module
                                .get_struct_type(&qst.get_name())
                                .ok_or(QuillError::NoStruct(qst.get_name()))?
                                .ptr_type(AddressSpace::Generic);
                            let ptr = builder.build_alloca(l_t, "struct_alloca");
                            values.replace(dest_id, Some(ptr.as_basic_value_enum()));
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                },

                Instruction::Malloc(dest_id, q_type) => match q_type {
                    PolyQuillType::StructType(t) => {
                        let l_t = module
                            .get_struct_type(&t.get_name())
                            .ok_or(QuillError::NoStruct(t.get_name()))?;
                        let ptr = builder
                            .build_malloc(l_t, "struct_malloc")
                            .or(Err(QuillError::MallocErr))?;
                        values.replace(dest_id, Some(ptr.as_basic_value_enum()));
                    }
                    PolyQuillType::ListType(t) => {
                        let l_t = t.get_inner().as_llvm_type(&context, &module)?;
                        let size = match t.get_size() {
                            QuillListSize::Const(size) => context
                                .custom_width_int_type(64)
                                .const_int(*size as u64, false),
                            QuillListSize::Variable(qiv) => values
                                .get(qiv.id())
                                .unwrap()
                                .ok_or(QuillError::BadValueAccess)?
                                .into_int_value(),
                        };
                        let ptr = builder
                            .build_array_malloc(l_t, size, "list_malloc")
                            .or(Err(QuillError::MallocErr))?;
                        values.replace(dest_id, Some(ptr.as_basic_value_enum()));
                    }
                    PolyQuillType::BoolType(_) => todo!(),
                    PolyQuillType::IntType(_) => todo!(),
                    PolyQuillType::FloatType(_) => todo!(),
                    PolyQuillType::FnType(_) => todo!(),
                    PolyQuillType::PointerType(_) => todo!(),
                    PolyQuillType::VoidType(_) => todo!(),
                },

                Instruction::Store(ptr_id, value_id) => {
                    let ptr = values
                        .get(ptr_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let ptr_ptr = PointerValue::try_from(ptr).or(Err(QuillError::Convert))?;
                    let value = values
                        .get(value_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    builder.build_store(ptr_ptr, value);
                }

                Instruction::Load(ptr_id, value_id) => {
                    let ptr = values
                        .get(ptr_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let ptr_ptr = PointerValue::try_from(ptr).or(Err(QuillError::Convert))?;
                    let value = builder.build_load(ptr_ptr, "load");
                    values.replace(value_id, Some(value));
                }

                Instruction::FnCall(name, ret_id, pos_args) => {
                    let fn_l_t = module
                        .get_function(&name)
                        .ok_or(QuillError::FnNotFound(name))?;

                    let args = pos_args.into_iter().try_fold(vec![], |args, id| {
                        Result::Ok(args.fpush(BasicMetadataValueEnum::from(
                            values.get(id).unwrap().ok_or(QuillError::BadValueAccess)?,
                        )))
                    })?;

                    let ret_val = builder.build_call(fn_l_t, &args, "fn_call");
                    if let Some(bv) = ret_val.try_as_basic_value().left() {
                        values.replace(ret_id, Some(bv));
                    }
                }

                Instruction::FnParam(id, name) => {
                    let index = header.get_param_index(&name)?;
                    let val = fn_val
                        .get_nth_param(index as u32)
                        .ok_or(QuillError::NoSuchParam(name))?;
                    values.replace(id, Some(val));
                }

                Instruction::IntAdd(dest_id, lhs_id, rhs_id) => {
                    let lhs = values
                        .get(lhs_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let lhs = match lhs {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        t => Err(QuillError::WrongType(
                            format!("{:?}", t),
                            String::from("IntValue"),
                            String::from("Nib::commit::IntAdd"),
                        )),
                    }?;
                    let rhs = values
                        .get(rhs_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let rhs = match rhs {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        t => Err(QuillError::WrongType(
                            format!("{:?}", t),
                            String::from("IntValue"),
                            String::from("Nib::commit::IntAdd"),
                        )),
                    }?;
                    values.replace(
                        dest_id,
                        Some(builder.build_int_add(lhs, rhs, "add").as_basic_value_enum()),
                    );
                }

                Instruction::ListValueSet(list_id, value_id, index_id) => unsafe {
                    let list = values
                        .get(list_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let value = values
                        .get(value_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let index = values
                        .get(index_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let element_ptr = builder.build_gep(
                        PointerValue::try_from(list).or(Err(QuillError::Convert))?,
                        &[IntValue::try_from(index).or(Err(QuillError::Convert))?],
                        "list_set_gep",
                    );
                    builder.build_store(element_ptr, value);
                },

                Instruction::ListValueGet(list_id, value_id, index_id) => unsafe {
                    let list = values
                        .get(list_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let index = values
                        .get(index_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let element_ptr = builder.build_gep(
                        PointerValue::try_from(list).or(Err(QuillError::Convert))?,
                        &[IntValue::try_from(index).or(Err(QuillError::Convert))?],
                        "list_get_gep",
                    );
                    let value = builder.build_load(element_ptr, "list_get_load");
                    values.replace(value_id, Some(value));
                },

                Instruction::ListCopy(ol_id, nl_id, len_id, dest_index_id) => unsafe {
                    let ol = values
                        .get(ol_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let nl = values
                        .get(nl_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let len = values
                        .get(len_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let dest_index = values
                        .get(dest_index_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let nl_ptr = PointerValue::try_from(nl).or(Err(QuillError::Convert))?;
                    let ol_ptr = PointerValue::try_from(ol).or(Err(QuillError::Convert))?;
                    let dest_index_int =
                        IntValue::try_from(dest_index).or(Err(QuillError::Convert))?;
                    let len_val = IntValue::try_from(len).or(Err(QuillError::Convert))?;
                    let byte_len = builder.build_int_mul(
                        len_val,
                        nl_ptr.get_type().get_element_type().size_of().unwrap(),
                        "byte_len",
                    );
                    let byte_len_val = IntValue::try_from(byte_len).or(Err(QuillError::Convert))?;
                    let dest_ptr = builder.build_gep(nl_ptr, &[dest_index_int], "dest_index");
                    builder
                        .build_memcpy(dest_ptr, 1, ol_ptr, 1, byte_len_val)
                        .or(Err(QuillError::Memcpy))?;
                },

                Instruction::Free(val_id) => {
                    let val = values
                        .get(val_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let ptr = PointerValue::try_from(val).or(Err(QuillError::Convert))?;
                    builder.build_free(ptr);
                }

                Instruction::IntCmp(lhs_id, rhs_id, val_id, cmp_type) => {
                    let lhs = values
                        .get(lhs_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let rhs = values
                        .get(rhs_id)
                        .unwrap()
                        .ok_or(QuillError::BadValueAccess)?;
                    let lhs_int = IntValue::try_from(lhs).or(Err(QuillError::Convert))?;
                    let rhs_int = IntValue::try_from(rhs).or(Err(QuillError::Convert))?;
                    let cmp_result =
                        builder.build_int_compare(cmp_type, lhs_int, rhs_int, "int_cmp");
                    values.replace(val_id, Some(cmp_result.into()))
                }
            }
        }

        // Branch back to our parent if required
        if let Some(last_instruction) = self.instructions.get(self.instructions.len() - 1) {
            match &last_instruction {
                Instruction::Return(_)
                | Instruction::ConditionalLoop(_)
                | Instruction::Unreachable => {} // We already have a term instruction, do nothing,
                _ => {
                    trace!("No terminating instruction, branching to parent!");
                    builder.build_unconditional_branch(after.ok_or(QuillError::NoAfter)?);
                }
            }
        } else {
            trace!("Empty body, branching to parent!");
            builder.build_unconditional_branch(after.ok_or(QuillError::NoAfter)?);
        }

        Ok(first_basic_block)
    }
}
impl Nib for ChildNib {
    fn create_child(&self) -> ChildNib {
        ChildNib::new(self.parent_fn.clone(), self.id_generator)
    }

    fn add_return<T: QuillType>(&mut self, value: Option<&QuillValue<T>>) {
        self.instructions
            .push(Instruction::Return(value.map(|v| v.id())))
    }

    fn add_cond_branch(
        &mut self,
        cond: &QuillValue<QuillBoolType>,
        t_branch: ChildNib,
        f_branch: Option<ChildNib>,
    ) {
        self.instructions.push(Instruction::ConditionalBranch(
            cond.id(),
            t_branch,
            f_branch,
        ))
    }

    fn add_branch(&mut self, branch: ChildNib) {
        self.instructions
            .push(Instruction::UnconditionalBranch(branch))
    }

    fn add_cond_loop(&mut self, cond: &QuillValue<QuillBoolType>) {
        self.instructions
            .push(Instruction::ConditionalLoop(cond.id()))
    }

    fn build_unreachable(&mut self) {
        self.instructions.push(Instruction::Unreachable)
    }

    fn get_value_from_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        // Ensure we got a struct type
        QuillStructType::try_from(pv.get_type().get_inner_type())?;
        let value = QuillValue::new(self.id_generator, expected_type);
        self.instructions
            .push(Instruction::StructGet(pv.id(), self.id_generator, name));
        self.id_generator += 1;
        Ok(value)
    }

    fn set_value_in_struct<T: QuillType>(
        &mut self,
        pv: &QuillValue<QuillPointerType>,
        name: String,
        value: &QuillValue<T>,
    ) -> Result<()> {
        // Ensure we got a struct type
        QuillStructType::try_from(pv.get_type().get_inner_type())?;
        self.instructions
            .push(Instruction::StructSet(pv.id(), value.id(), name));
        Ok(())
    }

    fn const_int(&mut self, bits: u32, value: u64) -> QuillValue<QuillIntType> {
        self.instructions
            .push(Instruction::ConstInt(self.id_generator, bits, value));
        let qiv = QuillValue::new(self.id_generator, QuillIntType::new(bits));
        self.id_generator += 1;
        qiv
    }

    fn const_bool(&mut self, value: bool) -> QuillValue<QuillBoolType> {
        self.instructions
            .push(Instruction::ConstBool(self.id_generator, value));
        let qbv = QuillValue::new(self.id_generator, QuillBoolType::new());
        self.id_generator += 1;
        qbv
    }

    fn const_string(&mut self, value: String) -> QuillValue<QuillPointerType> {
        let strlen = value.len() + 1;
        let t = QuillListType::new_const_length(QuillIntType::new(8), strlen);
        self.instructions
            .push(Instruction::ConstString(self.id_generator, value));
        let v = QuillValue::new(self.id_generator, QuillPointerType::new(t));
        self.id_generator += 1;
        v
    }

    fn add_alloca<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType> {
        self.instructions
            .push(Instruction::Alloca(self.id_generator, t.clone().into()));
        let v = QuillValue::new(self.id_generator, QuillPointerType::new(t));
        self.id_generator += 1;
        v
    }

    fn add_malloc<T: QuillType>(&mut self, t: T) -> QuillValue<QuillPointerType> {
        self.instructions
            .push(Instruction::Malloc(self.id_generator, t.clone().into()));
        let v = QuillValue::new(self.id_generator, QuillPointerType::new(t));
        self.id_generator += 1;
        v
    }

    fn add_store<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
    ) -> Result<()> {
        if ptr.get_type().get_inner_type() != value.get_type().clone().into() {
            Err(QuillError::WrongType(
                format!("{:?}", ptr.get_type().get_inner_type()),
                format!("{:?}", value.get_type()),
                String::from("Nib::add_store"),
            ))
        } else {
            self.instructions
                .push(Instruction::Store(ptr.id(), value.id()));
            Ok(())
        }
    }

    fn add_load<T: QuillType>(
        &mut self,
        ptr: &QuillValue<QuillPointerType>,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        if ptr.get_type().get_inner_type() != expected_type.clone().into() {
            Err(QuillError::WrongType(
                format!("{:?}", ptr.get_type().get_inner_type()),
                format!("{:?}", expected_type),
                String::from("Nib::add_load"),
            ))
        } else {
            let v = QuillValue::new(self.id_generator, expected_type);
            self.instructions
                .push(Instruction::Load(ptr.id(), self.id_generator));
            self.id_generator += 1;
            Ok(v)
        }
    }

    fn add_fn_call<T: QuillType>(
        &mut self,
        name: String,
        args: Vec<QuillValue<PolyQuillType>>,
        expected_type: T,
    ) -> QuillValue<T> {
        self.instructions.push(Instruction::FnCall(
            name,
            self.id_generator,
            args.into_iter().map(|arg| arg.id()).collect(),
        ));
        let v = QuillValue::new(self.id_generator, expected_type);
        self.id_generator += 1;
        v
    }

    fn int_add(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
    ) -> Result<QuillValue<QuillIntType>> {
        if lhs.get_type().bit_width() != rhs.get_type().bit_width() {
            return Err(QuillError::IntSize(
                lhs.get_type().bit_width(),
                rhs.get_type().bit_width(),
            ));
        }
        self.instructions
            .push(Instruction::IntAdd(self.id_generator, lhs.id(), rhs.id()));
        let v = QuillValue::new(
            self.id_generator,
            QuillIntType::new(lhs.get_type().bit_width()),
        );
        self.id_generator += 1;
        Ok(v)
    }

    fn get_fn_t(&self) -> &QuillFnType {
        &self.parent_fn
    }

    fn set_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        value: &QuillValue<T>,
        index: &QuillValue<QuillIntType>,
    ) -> Result<()> {
        if lv.get_type().get_inner_type() != value.get_type().clone().into() {
            Err(QuillError::WrongType(
                format!("{:?}", lv.get_type().get_inner_type()),
                format!("{:?}", value.get_type()),
                String::from("Nib::set_list_value"),
            ))
        } else {
            self.instructions
                .push(Instruction::ListValueSet(lv.id(), value.id(), index.id()));
            Ok(())
        }
    }

    fn get_list_value<T: QuillType>(
        &mut self,
        lv: &QuillValue<QuillPointerType>,
        index: &QuillValue<QuillIntType>,
        expected_type: T,
    ) -> Result<QuillValue<T>> {
        if lv.get_type().get_inner_type() != expected_type.clone().into() {
            Err(QuillError::WrongType(
                format!("{:?}", lv.get_type().get_inner_type()),
                format!("{:?}", expected_type),
                String::from("Nib::get_list_value"),
            ))
        } else {
            let value = QuillValue::new(self.id_generator, expected_type);
            self.id_generator += 1;
            self.instructions
                .push(Instruction::ListValueGet(lv.id(), value.id(), index.id()));
            Ok(value)
        }
    }

    fn list_copy(
        &mut self,
        ol: &QuillValue<QuillPointerType>,
        nl: &QuillValue<QuillPointerType>,
        len: &QuillValue<QuillIntType>,
        dest_index: &QuillValue<QuillIntType>,
    ) -> Result<()> {
        // The type comparison is a little weird if we get list types, so we gotta deal with that
        let ol_type = match ol.get_type().get_inner_type() {
            PolyQuillType::ListType(lt) => QuillPointerType::new(lt.get_inner().clone()),
            _ => ol.get_type().clone().into(),
        };
        let nl_type = match nl.get_type().get_inner_type() {
            PolyQuillType::ListType(lt) => QuillPointerType::new(lt.get_inner().clone()),
            _ => nl.get_type().clone().into(),
        };
        if ol_type != nl_type {
            Err(QuillError::WrongType(
                format!("{:?}", ol_type),
                format!("{:?}", nl_type),
                String::from("Nib::list_copy"),
            ))
        } else {
            self.instructions.push(Instruction::ListCopy(
                ol.id(),
                nl.id(),
                len.id(),
                dest_index.id(),
            ));
            Ok(())
        }
    }

    fn free(&mut self, val: QuillValue<QuillPointerType>) {
        self.instructions.push(Instruction::Free(val.id()));
    }

    fn int_cmp(
        &mut self,
        lhs: &QuillValue<QuillIntType>,
        rhs: &QuillValue<QuillIntType>,
        cmp_type: IntCmpType,
    ) -> Result<QuillValue<QuillBoolType>> {
        if lhs.get_type().bit_width() != rhs.get_type().bit_width() {
            Err(QuillError::IntSize(
                lhs.get_type().bit_width(),
                rhs.get_type().bit_width(),
            ))
        } else {
            let result = QuillValue::new(self.id_generator, QuillBoolType);
            self.id_generator += 1;
            self.instructions.push(Instruction::IntCmp(
                lhs.id(),
                rhs.id(),
                result.id(),
                cmp_type,
            ));
            Ok(result)
        }
    }
}
