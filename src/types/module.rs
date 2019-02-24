// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi;
use crate::instance::PyNativeType;
use crate::object::PyObject;
use crate::objectprotocol::ObjectProtocol;
use crate::type_object::PyTypeCreate;
use crate::type_object::PyTypeObject;
use crate::types::PyTuple;
use crate::types::{PyBaseObject, PyDict};
use crate::AsPyPointer;
use crate::IntoPy;
use crate::Py;
use crate::Python;
use crate::ToPyObject;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str;

/// Represents a Python `module` object.
#[repr(transparent)]
pub struct PyModule(PyObject);

pyobject_native_type!(PyModule, ffi::PyModule_Type, ffi::PyModule_Check);

impl PyModule {
    /// Create a new module object with the `__name__` attribute set to name.
    pub fn new<'p>(py: Python<'p>, name: &str) -> PyResult<&'p PyModule> {
        let name = CString::new(name)?;
        unsafe { py.from_owned_ptr_or_err(ffi::PyModule_New(name.as_ptr())) }
    }

    /// Import the Python module with the specified name.
    pub fn import<'p>(py: Python<'p>, name: &str) -> PyResult<&'p PyModule> {
        let name = CString::new(name)?;
        unsafe { py.from_owned_ptr_or_err(ffi::PyImport_ImportModule(name.as_ptr())) }
    }

    /// Loads the python code specified into a new module
    /// 'code' is the raw Python you want to load into the module
    /// 'file_name' is the file name to associate with the module
    ///     (this is used when Python reports errors, for example)
    /// 'module_name' is the name to give the module
    #[cfg(Py_3)]
    pub fn from_code<'p>(
        py: Python<'p>,
        code: &str,
        file_name: &str,
        module_name: &str,
    ) -> PyResult<&'p PyModule> {
        let data = CString::new(code)?;
        let filename = CString::new(file_name)?.as_ptr();
        let module = CString::new(module_name)?;

        unsafe {
            let cptr = ffi::Py_CompileString(data.as_ptr(), filename, ffi::Py_file_input);
            if cptr.is_null() {
                return Err(PyErr::fetch(py));
            }

            let mptr = ffi::PyImport_ExecCodeModuleEx(module.as_ptr(), cptr, filename);
            if mptr.is_null() {
                return Err(PyErr::fetch(py));
            }

            <&PyModule as crate::FromPyObject>::extract(py.from_owned_ptr_or_err(mptr)?)
        }
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self) -> &PyDict {
        unsafe {
            self.py()
                .from_owned_ptr::<PyDict>(ffi::PyModule_GetDict(self.as_ptr()))
        }
    }

    unsafe fn str_from_ptr(&self, ptr: *const c_char) -> PyResult<&str> {
        if ptr.is_null() {
            Err(PyErr::fetch(self.py()))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::from_instance(
                    exceptions::UnicodeDecodeError::new_utf8(self.py(), slice, e)?,
                )),
            }
        }
    }

    /// Gets the module name.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name(&self) -> PyResult<&str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetName(self.as_ptr())) }
    }

    /// Gets the module filename.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    pub fn filename(&self) -> PyResult<&str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)(*args, **kwargs)`
    pub fn call(
        &self,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyBaseObject> {
        self.getattr(name)?.call(args, kwargs)
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)()`
    pub fn call0(&self, name: &str) -> PyResult<&PyBaseObject> {
        self.getattr(name)?.call0()
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)(*args)`
    pub fn call1(&self, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyBaseObject> {
        self.getattr(name)?.call1(args)
    }

    /// Gets a member from the module.
    /// This is equivalent to the Python expression: `getattr(module, name)`
    pub fn get(&self, name: &str) -> PyResult<&PyBaseObject> {
        self.getattr(name)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<()>
    where
        V: ToPyObject,
    {
        self.setattr(name, value)
    }

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that initializes the `class`,
    /// sets `new_type.__module__` to this module's name,
    /// and adds the type to this module.
    pub fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyTypeCreate,
    {
        self.setattr(T::NAME, <T as PyTypeObject>::type_object())
    }

    /// Adds a function or a (sub)module to a module, using the functions __name__ as name.
    ///
    /// Use this together with the`#[pyfunction]` and [wrap_pyfunction!] or `#[pymodule]` and
    /// [wrap_pymodule!].
    ///
    /// ```rust,ignore
    /// m.add_wrapped(wrap_pyfunction!(double));
    /// m.add_wrapped(wrap_pymodule!(utils));
    /// ```
    ///
    /// You can also add a function with a custom name using [add](PyModule::add):
    ///
    /// ```rust,ignore
    /// m.add("also_double", wrap_pyfunction!(double)(py));
    /// ```
    pub fn add_wrapped(&self, wrapper: &Fn(Python) -> PyObject) -> PyResult<()> {
        let function = wrapper(self.py());
        let name = function
            .getattr(self.py(), "__name__")
            .expect("A function or module must have a __name__");
        self.add(name.extract(self.py()).unwrap(), function)
    }
}
