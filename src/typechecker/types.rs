use std::collections::HashMap;

use ordermap::OrderMap;

use super::TypecheckerError;

#[derive(Default)]
pub struct TypecheckerScope {
    type_map: HashMap<String, DefinedType>,
    function_map: HashMap<String, DefinedType>,
}

impl TypecheckerScope {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn define(&mut self, name: String, type_name: DefinedType) {
        self.type_map.insert(name, type_name);
    }

    pub fn define_function_return(&mut self, name: String, type_name: DefinedType) {
        self.function_map.insert(name, type_name);
    }

    pub fn get_type_for(&self, name: &str) -> Option<&DefinedType> {
        self.type_map.get(name)
    }

    pub fn get_function_return_for(&self, name: &str) -> Option<&DefinedType> {
        self.function_map.get(name)
    }
}

#[derive(Clone, Debug)]
pub enum DefinedType {
    String,
    I64,
    F64,
    Bool,
    Nil,
    // Not all objects are the same, we need more details, and to define Eq
    Object {
        fields: OrderMap<String, DefinedType>,
    },
    Array(Box<DefinedType>),
}

impl PartialEq for DefinedType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DefinedType::String => matches!(other, DefinedType::String),
            DefinedType::Bool => matches!(other, DefinedType::Bool),
            DefinedType::I64 => matches!(other, DefinedType::I64),
            DefinedType::F64 => matches!(other, DefinedType::F64),
            DefinedType::Object { fields } => match other {
                DefinedType::Object {
                    fields: other_fields,
                } => fields.eq(other_fields),

                _ => false,
            },
            DefinedType::Nil => matches!(other, DefinedType::Nil),
            DefinedType::Array(defined_type) => match other {
                DefinedType::Array(other_defined_type) => defined_type.eq(other_defined_type),
                _ => false,
            },
        }
    }
}

impl TryFrom<&String> for DefinedType {
    type Error = TypecheckerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "int" => Ok(Self::I64),
            "float" => Ok(Self::F64),
            "bool" => Ok(Self::Bool),
            "string" => Ok(Self::String),

            _ => Err(TypecheckerError::UnexpectedType {
                got: value.to_owned(),
            }),
        }
    }
}

impl std::fmt::Display for DefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
