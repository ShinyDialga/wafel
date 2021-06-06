//! Error definitions.

#![allow(missing_docs)]

use crate::{data_path::LocalDataPath, error::Error};
use derive_more::{Display, Error, From};
use std::io;
use wafel_data_type::{DataTypeRef, IntValue};

#[derive(Debug, Display, Error, From)]
pub enum SM64ErrorCause {
    #[display(fmt = "unhandled variable: {}", variable)]
    UnhandledVariable { variable: String },
    #[display(fmt = "variable is missing frame: {}", variable)]
    MissingFrame { variable: String },
    #[display(fmt = "variable is missing object: {}", variable)]
    MissingObject { variable: String },
    #[display(fmt = "variable is missing surface: {}", variable)]
    MissingSurface { variable: String },
    #[display(fmt = "invalid root type (must be object or surface): {}", path)]
    InvalidVariableRoot { path: LocalDataPath },
    #[display(fmt = "while loading layout extensions: {}", _0)]
    LoadObjectFieldsError(LayoutExtensionErrorCause),
    #[display(fmt = "unimplemented conversion from {} to python object", value)]
    ValueToPython { value: String },
    #[display(fmt = "unsupported conversion from {} to data value", value)]
    ValueFromPython { value: String },
    #[display(fmt = "invalid frame log event type: {}", value)]
    InvalidFrameLogEventType { value: IntValue },
    #[from]
    VariableSerdeError(serde_json::Error),
    #[display(fmt = "surface pool pointer does not have a stride")]
    UnsizedSurfacePoolPointer,
    #[display(fmt = "object pool array does not have a stride")]
    UnsizedObjectPoolArray,
}

#[derive(Debug, Display, Error, From)]
pub enum LayoutExtensionErrorCause {
    #[display(fmt = "object struct already in use")]
    ObjectStructInUse,
    #[display(fmt = "struct Object is not a struct: {}", object_struct)]
    ObjectStructNotStruct { object_struct: DataTypeRef },
    #[display(fmt = "struct Object missing field rawData: {}", object_struct)]
    MissingRawData { object_struct: DataTypeRef },
    #[display(fmt = "expected {}, found {}", expected, value)]
    WrongType { expected: String, value: String },
    #[display(fmt = "missing field {} in {}", field, object)]
    MissingField { object: String, field: String },
    #[from]
    IOError(io::Error),
    #[from]
    SerdeError(serde_json::Error),
}

impl From<LayoutExtensionErrorCause> for Error {
    fn from(cause: LayoutExtensionErrorCause) -> Self {
        SM64ErrorCause::LoadObjectFieldsError(cause).into()
    }
}
