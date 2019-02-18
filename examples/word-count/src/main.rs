use pyo3::derive_utils::make_module;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::ffi;
use std::ffi::CString;
use word_count;

fn example1() {
    extern "C" fn add_module() -> *mut ffi::PyObject {
        unsafe { make_module("word_count", "", word_count::word_count) }
    }
    let name = CString::new("word_count").unwrap().into_raw();
    unsafe {
        ffi::PyImport_AppendInittab(name, Some(add_module));
    }
    let gil = Python::acquire_gil();
    let py = gil.python();
    py.run("import word_count; print(word_count.count_line('Simple is better', 'is'))", None, None)
      .map_err(|e| e.print(py))
      .unwrap();
}


fn example2() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mod_: &pyo3::types::PyModule = unsafe {
        let m = make_module("word_count", "", word_count::word_count);
        py.from_borrowed_ptr(m)
    };
    let locals = PyDict::new(py);
    locals.set_item("word_count", mod_).unwrap();
    py.run("print(word_count.count_line('Simple is better', 'is'))", None, Some(locals))
      .map_err(|e| e.print(py))
      .unwrap();
}

fn main() {
    example2();
}
