//! Recording and looking up type and global variable definitions.

use super::{IntValue, MemoryErrorCause::*};
use crate::error::Error;
use derive_more::Display;
use std::{collections::HashMap, fmt};
use wafel_types::{DataType, DataTypeRef, TypeName};

/// A description of accessible variables and types.
#[derive(Debug, Clone, Default)]
pub struct DataLayout {
    /// The definitions of structs, unions, and typedefs.
    pub type_defns: HashMap<TypeName, DataTypeRef>,
    /// The types of global variables and functions.
    pub globals: HashMap<String, DataTypeRef>,
    /// The values of integer constants.
    pub constants: HashMap<String, Constant>,
}

/// A constant's value and source.
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[display(fmt = "{} ({})", value, source)]
pub struct Constant {
    /// The integer value for the constant.
    pub value: IntValue,
    /// The source for the constant.
    pub source: ConstantSource,
}

/// The source for a constant value.
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
pub enum ConstantSource {
    /// The constant is defined as an enum variant.
    #[display(
        fmt = "{}",
        "name
            .as_deref()
            .map(|name| format!(\"enum {}\", name))
            .unwrap_or_else(|| \"anonymous enum\".to_owned())"
    )]
    Enum {
        /// The name of the enum, or None for an anonymous enum.
        name: Option<String>,
    },
    /// The constant is defined as a macro.
    #[display(fmt = "macro")]
    Macro,
}

impl DataLayout {
    /// Create an empty data layout.
    pub fn new() -> Self {
        Self {
            type_defns: HashMap::new(),
            globals: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    /// Look up the definition of a type name.
    pub fn get_type(&self, name: &TypeName) -> Result<&DataTypeRef, Error> {
        self.type_defns
            .get(name)
            .ok_or_else(|| UndefinedTypeName { name: name.clone() }.into())
    }

    /// Look up the definition of a type name.
    ///
    /// This returns a mutable reference to the DataTypeRef. This is only useful if
    /// the data type hasn't been used in multiple places.
    pub fn get_type_mut(&mut self, name: &TypeName) -> Result<&mut DataTypeRef, Error> {
        self.type_defns
            .get_mut(name)
            .ok_or_else(|| UndefinedTypeName { name: name.clone() }.into())
    }

    /// Recursively look up a type name.
    pub fn concrete_type(&self, data_type: &DataTypeRef) -> Result<DataTypeRef, Error> {
        let mut data_type = data_type.clone();
        while let DataType::Name(name) = data_type.as_ref() {
            data_type = self.get_type(name)?.clone();
        }
        Ok(data_type)
    }

    /// Look up the type of a global variable.
    pub fn get_global(&self, name: &str) -> Result<&DataTypeRef, Error> {
        self.globals.get(name).ok_or_else(|| {
            UndefinedGlobal {
                name: name.to_owned(),
            }
            .into()
        })
    }

    /// Look up the value of a constant.
    pub fn get_constant(&self, name: &str) -> Result<&Constant, Error> {
        self.constants.get(name).ok_or_else(|| {
            UndefinedConstant {
                name: name.to_owned(),
            }
            .into()
        })
    }
}

impl fmt::Display for DataLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, data_type) in &self.type_defns {
            writeln!(f, "{} = {}", name, data_type)?;
        }
        for (name, data_type) in &self.globals {
            writeln!(f, "{}: {}", name, data_type)?;
        }
        for (name, value) in &self.constants {
            writeln!(f, "{} := {}", name, value)?;
        }
        Ok(())
    }
}
