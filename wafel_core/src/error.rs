//! Error definitions.

#![allow(missing_docs)]

use std::{
    backtrace::Backtrace,
    fmt::{self, Display},
};

use derive_more::{Display, Error, From};
use wafel_data_type::ValueTypeError;
use wafel_layout::{LayoutLookupError, SM64ExtrasError};

use crate::{
    data_path::DataPathErrorCause,
    dll::{DllError, DllErrorCause},
    memory::MemoryErrorCause,
    sm64::SM64ErrorCause,
};

pub type Error = WithContext<ErrorCause>;

#[derive(Debug, Display, Error, From)]
pub enum ErrorCause {
    #[from]
    ApiError(wafel_api::Error),
    #[from]
    ValueTypeError(ValueTypeError),
    #[from]
    LayoutLookupError(LayoutLookupError),
    #[from]
    SM64ExtrasError(SM64ExtrasError),
    #[from]
    MemoryError(MemoryErrorCause),
    #[from]
    DataPathError(DataPathErrorCause),
    #[from]
    DllError(DllErrorCause),
    #[from]
    SM64Error(SM64ErrorCause),
}

impl From<wafel_api::Error> for Error {
    fn from(value: wafel_api::Error) -> Self {
        ErrorCause::ApiError(value).into()
    }
}

impl From<ValueTypeError> for Error {
    fn from(value: ValueTypeError) -> Self {
        ErrorCause::ValueTypeError(value).into()
    }
}

impl From<LayoutLookupError> for Error {
    fn from(value: LayoutLookupError) -> Self {
        ErrorCause::LayoutLookupError(value).into()
    }
}

impl From<SM64ExtrasError> for Error {
    fn from(value: SM64ExtrasError) -> Self {
        ErrorCause::SM64ExtrasError(value).into()
    }
}

impl From<MemoryErrorCause> for Error {
    fn from(value: MemoryErrorCause) -> Self {
        ErrorCause::MemoryError(value).into()
    }
}

impl From<DataPathErrorCause> for Error {
    fn from(value: DataPathErrorCause) -> Self {
        ErrorCause::DataPathError(value).into()
    }
}

impl From<DllErrorCause> for Error {
    fn from(value: DllErrorCause) -> Self {
        ErrorCause::DllError(value).into()
    }
}

impl From<DllError> for Error {
    fn from(value: DllError) -> Self {
        value.cause_into()
    }
}

impl From<SM64ErrorCause> for Error {
    fn from(value: SM64ErrorCause) -> Self {
        ErrorCause::SM64Error(value).into()
    }
}

/// An error with extra context.
#[derive(Debug, Error)]
pub struct WithContext<E> {
    /// The root source of the error.
    #[error(source)]
    pub cause: E,
    /// The additional context for the error.
    ///
    /// The outermost context is at the front of the vector.
    pub context: Vec<String>,
    /// The backtrace for the error.
    pub backtrace: Backtrace,
}

impl<E> WithContext<E> {
    /// Add additional context to the error.
    pub fn context(mut self, context: String) -> Self {
        self.context.insert(0, context);
        self
    }

    /// Change the error's cause.
    pub fn map_cause<R>(self, f: impl FnOnce(E) -> R) -> WithContext<R> {
        WithContext {
            cause: f(self.cause),
            context: self.context,
            backtrace: self.backtrace,
        }
    }

    /// Convert the cause into another type.
    pub fn cause_into<R: From<E>>(self) -> WithContext<R> {
        self.map_cause(R::from)
    }
}

impl<E> From<E> for WithContext<E> {
    fn from(cause: E) -> Self {
        Self {
            cause,
            context: Vec::new(),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl<E: Display> Display for WithContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for context in &self.context {
            write!(f, "{}: ", context)?;
        }
        write!(f, "{}", self.cause)
    }
}
