// use inkwell::types::BasicTypeEnum;
//
// impl<'a, 'ctx> Codegen<'a, 'ctx> {
//     // define and add the printf function to the module
//     pub fn add_printf(&mut self) {
//         let f64_type = self.context.f64_type();
//         let str_type = self
//             .context
//             .i8_type()
//             .ptr_type(inkwell::AddressSpace::Generic);
//         let printf_args_type = vec![BasicTypeEnum::PointerType(str_type)];
//
//         // printf needs to return a double to be used in compile_call
//         let printf_type = f64_type.fn_type(printf_args_type.as_slice(), true);
//
//         let printf_fn = self
//             .module
//             .add_function("printf", printf_type, Some(Linkage::External));
//
//         self.builtins.insert("printf", printf_fn);
//     }
//
//     // define and add exit function to module
//     pub fn add_exit(&mut self) {
//         let exit_fn = self.module.add_function()
//     }
// }