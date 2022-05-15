//
// Module that exposes inkwell's types and functionality in a way that's actually usable
// I attempt to mirror inkwell's structures and philosophy as much as possible, with a few
// simplifications here and there
//

mod nib;
pub use nib::*;

mod quill;
pub use quill::*;

mod quill_types;
pub use quill_types::*;

mod quill_value;
pub use quill_value::*;

mod except;
pub use except::*;
