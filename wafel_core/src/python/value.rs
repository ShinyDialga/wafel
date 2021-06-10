use super::PyAddress;
use crate::{error::Error, sm64::SM64ErrorCause};
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyFloat, PyList, PyLong},
};
use wafel_api::Value;

pub(crate) fn value_to_py_object(py: Python<'_>, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::None => Ok(py.None()),
        Value::Int(n) => Ok(n.to_object(py)),
        Value::Float(r) => Ok(r.to_object(py)),
        Value::String(s) => Ok(s.to_object(py)),
        Value::Address(address) => Ok(PyAddress { address: *address }.into_py(py)),
        Value::Struct { fields } => Ok(fields
            .iter()
            .map(|(name, value)| value_to_py_object(py, value).map(|object| (name, object)))
            .collect::<PyResult<Vec<_>>>()?
            .into_py_dict(py)
            .to_object(py)),
        Value::Array(items) => {
            let objects: Vec<PyObject> = items
                .iter()
                .map(|value| value_to_py_object(py, value))
                .collect::<PyResult<_>>()?;
            Ok(PyList::new(py, objects).to_object(py))
        }
    }
}

pub(crate) fn py_object_to_value(py: Python<'_>, value: &PyObject) -> PyResult<Value> {
    if value.is_none(py) {
        Ok(Value::None)
    } else if let Ok(long_value) = value.cast_as::<PyLong>(py) {
        Ok(Value::Int(long_value.extract()?))
    } else if let Ok(float_value) = value.cast_as::<PyFloat>(py) {
        Ok(Value::Float(float_value.extract()?))
    } else if let Ok(address) = value.cast_as::<PyAny>(py)?.extract::<PyAddress>() {
        Ok(Value::Address(address.address))
    } else {
        Err(Error::from(SM64ErrorCause::ValueFromPython {
            value: value.cast_as::<PyAny>(py)?.str()?.to_string(),
        })
        .into())
    }
}
