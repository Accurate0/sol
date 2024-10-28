use crate::vm::RegisterValue;
use phf::phf_map;

mod print;

pub type NativeFunctionType = fn(Vec<RegisterValue>);
pub static STANDARD_LIBRARY: phf::Map<&'static str, NativeFunctionType> = phf_map! {
    "print" => print::print,
};
