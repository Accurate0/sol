use crate::vm::VMValue;
use phf::phf_map;

mod print;

// FIXME: allow strict typing by native functions
// can do this once typechecking exists.
pub type NativeFunctionType = fn(Vec<VMValue>) -> Option<VMValue>;
pub static STANDARD_LIBRARY: phf::Map<&'static str, NativeFunctionType> = phf_map! {
    // FIXME: add serialise to string method and call it from print
    "print" => print::print,
};
