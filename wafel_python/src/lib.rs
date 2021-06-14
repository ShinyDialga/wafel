//! Python bindings for [wafel_api].
//!
//! Currently only the [Game](api::Game) API is implemented, not [Timeline](api::Timeline).

#![warn(missing_debug_implementations, rust_2018_idioms, unreachable_pub)]

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use pyo3::{
    basic::CompareOp,
    prelude::*,
    types::{IntoPyDict, PyDict, PyFloat, PyList, PyLong, PyString},
    PyObjectProtocol,
};
use wafel_api as api;

#[pymodule]
pub fn wafel(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("WafelError", py.get_type::<WafelError>())?;
    m.add_class::<Address>()?;
    m.add_class::<Surface>()?;
    m.add_class::<ObjectHitbox>()?;
    m.add_class::<Game>()?;
    m.add_class::<SaveState>()?;
    Ok(())
}

pyo3::create_exception!(wafel, WafelError, pyo3::exceptions::PyException);

fn err(error: api::Error) -> PyErr {
    let message = format!("\n  {}", error);
    PyErr::new::<WafelError, _>(message)
}

fn value_to_py_object(py: Python<'_>, value: api::Value) -> PyResult<PyObject> {
    match value {
        api::Value::None => Ok(py.None()),
        api::Value::Int(n) => Ok(n.to_object(py)),
        api::Value::Float(r) => Ok(r.to_object(py)),
        api::Value::String(s) => Ok(s.to_object(py)),
        api::Value::Address(address) => Ok(Address { inner: address }.into_py(py)),
        api::Value::Struct(fields) => Ok(fields
            .into_iter()
            .map(|(name, value)| value_to_py_object(py, value).map(|object| (name, object)))
            .collect::<PyResult<Vec<_>>>()?
            .into_py_dict(py)
            .to_object(py)),
        api::Value::Array(items) => {
            let objects: Vec<PyObject> = items
                .into_iter()
                .map(|value| value_to_py_object(py, value))
                .collect::<PyResult<_>>()?;
            Ok(PyList::new(py, objects).to_object(py))
        }
    }
}

fn py_object_to_value(py: Python<'_>, value: PyObject) -> PyResult<api::Value> {
    if value.is_none(py) {
        Ok(api::Value::None)
    } else if let Ok(long_value) = value.cast_as::<PyLong>(py) {
        Ok(api::Value::Int(long_value.extract()?))
    } else if let Ok(float_value) = value.cast_as::<PyFloat>(py) {
        Ok(api::Value::Float(float_value.extract()?))
    } else if let Ok(string_value) = value.cast_as::<PyString>(py) {
        Ok(api::Value::String(string_value.extract()?))
    } else if let Ok(address) = value.cast_as::<PyAny>(py)?.extract::<Address>() {
        Ok(api::Value::Address(address.inner))
    } else if let Ok(dict_value) = value.cast_as::<PyDict>(py) {
        match dict_value.extract::<HashMap<String, PyObject>>() {
            Ok(entries) => {
                let mut fields = HashMap::new();
                for (name, value) in entries {
                    fields.insert(name, py_object_to_value(py, value)?);
                }
                Ok(api::Value::Struct(Box::new(fields)))
            }
            Err(_) => Err(PyErr::new::<WafelError, _>(format!(
                "invalid data value: {}",
                value
            ))),
        }
    } else if let Ok(list_value) = value.cast_as::<PyList>(py) {
        let mut elements = Vec::new();
        for value in list_value.extract::<Vec<PyObject>>()? {
            elements.push(py_object_to_value(py, value)?);
        }
        Ok(api::Value::Array(elements))
    } else {
        Err(PyErr::new::<WafelError, _>(format!(
            "invalid data value: {}",
            value
        )))
    }
}

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address {
    inner: api::Address,
}

#[pymethods]
impl Address {
    fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

#[pyproto]
impl PyObjectProtocol for Address {
    fn __str__(&'p self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&'p self) -> String {
        format!("{}", self.inner)
    }

    fn __hash__(&'p self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&'p self, other: Address, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == &other),
            CompareOp::Ne => Ok(self != &other),
            _ => Err(PyErr::new::<WafelError, _>("unimplemented comparison")),
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Surface {
    inner: api::Surface,
}

#[pymethods]
impl Surface {
    #[getter]
    fn normal(&self) -> [f32; 3] {
        self.inner.normal
    }

    #[getter]
    fn vertices(&self) -> Vec<[i16; 3]> {
        self.inner.vertices.to_vec()
    }
}

#[pyproto]
impl PyObjectProtocol for Surface {
    fn __str__(&'p self) -> String {
        format!("{:?}", self.inner)
    }

    fn __repr__(&'p self) -> String {
        format!("{:?}", self.inner)
    }
}

#[pyclass]
#[derive(Debug)]
pub struct ObjectHitbox {
    inner: api::ObjectHitbox,
}

#[pymethods]
impl ObjectHitbox {
    #[getter]
    fn pos(&self) -> [f32; 3] {
        self.inner.pos
    }

    #[getter]
    fn hitbox_height(&self) -> f32 {
        self.inner.hitbox_height
    }

    #[getter]
    fn hitbox_radius(&self) -> f32 {
        self.inner.hitbox_radius
    }
}

#[pyproto]
impl PyObjectProtocol for ObjectHitbox {
    fn __str__(&'p self) -> String {
        format!("{:?}", self.inner)
    }

    fn __repr__(&'p self) -> String {
        format!("{:?}", self.inner)
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Game {
    dll_path: String,
    inner: api::Game,
}

#[pymethods]
impl Game {
    #[new]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn open(dll_path: &str) -> PyResult<Self> {
        let inner = api::Game::try_new(dll_path).map_err(err)?;
        Ok(Game {
            dll_path: dll_path.to_string(),
            inner,
        })
    }

    pub fn read(&self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        let value = self.inner.try_read(path).map_err(err)?;
        let object = value_to_py_object(py, value)?;
        Ok(object)
    }

    pub fn read_string_at(&self, address: Address) -> PyResult<Vec<u8>> {
        self.inner.try_read_string_at(address.inner).map_err(err)
    }

    pub fn address(&self, path: &str) -> PyResult<Option<Address>> {
        let address = self.inner.try_address(path).map_err(err)?;
        Ok(address.map(|inner| Address { inner }))
    }

    pub fn address_to_symbol(&self, address: Address) -> Option<String> {
        self.inner.address_to_symbol(address.inner)
    }

    pub fn data_type(&self, path: &str) -> PyResult<String> {
        let data_type = self.inner.try_data_type(path).map_err(err)?;
        Ok(format!("{}", data_type))
    }

    pub fn write(&mut self, py: Python<'_>, path: &str, value: PyObject) -> PyResult<()> {
        self.inner
            .try_write(path, py_object_to_value(py, value)?)
            .map_err(err)
    }

    pub fn frame(&self) -> u32 {
        self.inner.frame()
    }

    pub fn advance(&mut self) {
        self.inner.advance()
    }

    pub fn advance_n(&mut self, num_frames: u32) {
        self.inner.advance_n(num_frames)
    }

    pub fn save_state(&self) -> SaveState {
        let inner = self.inner.save_state();
        SaveState { inner }
    }

    pub fn load_state(&mut self, state: &SaveState) -> PyResult<()> {
        self.inner.try_load_state(&state.inner).map_err(err)
    }

    pub fn constant(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        let value = self.inner.try_constant(name).map_err(err)?;
        let object = value_to_py_object(py, value)?;
        Ok(object)
    }

    pub fn mario_action_names(&self) -> HashMap<u32, String> {
        self.inner.mario_action_names()
    }

    pub fn frame_log(&self, py: Python<'_>) -> PyResult<Vec<HashMap<String, PyObject>>> {
        let frame_log = self.inner.try_frame_log().map_err(err)?;
        let object = convert_frame_log(py, frame_log)?;
        Ok(object)
    }

    pub fn surfaces(&self) -> PyResult<Vec<Surface>> {
        let surfaces = self.inner.try_surfaces().map_err(err)?;
        Ok(surfaces
            .into_iter()
            .map(|inner| Surface { inner })
            .collect())
    }

    pub fn object_hitboxes(&self) -> PyResult<Vec<ObjectHitbox>> {
        let hitboxes = self.inner.try_object_hitboxes().map_err(err)?;
        Ok(hitboxes
            .into_iter()
            .map(|inner| ObjectHitbox { inner })
            .collect())
    }
}

#[pyproto]
impl PyObjectProtocol for Game {
    fn __str__(&'p self) -> String {
        format!("Game({:?})", self.dll_path)
    }

    fn __repr__(&'p self) -> String {
        format!("Game({:?})", self.dll_path)
    }
}

fn convert_frame_log(
    py: Python<'_>,
    events: Vec<HashMap<String, api::Value>>,
) -> PyResult<Vec<HashMap<String, PyObject>>> {
    let mut py_events = Vec::new();
    for event in events {
        let mut py_event = HashMap::new();
        for (key, value) in event.into_iter() {
            let object = value_to_py_object(py, value)?;
            py_event.insert(key, object);
        }
        py_events.push(py_event);
    }
    Ok(py_events)
}

#[pyclass]
#[derive(Debug)]
pub struct SaveState {
    inner: api::SaveState,
}

#[pymethods]
impl SaveState {
    fn frame(&self) -> u32 {
        self.inner.frame()
    }
}