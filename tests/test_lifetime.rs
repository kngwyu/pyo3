#![feature(specialization)]

#[macro_use]
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyDict};

#[pymodule]
fn module_with_functions(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "bytes_plus1")]
    fn bytes_plus1(py: Python, b: &PyByteArray) -> &PyByteArray {
        let mut data = b.data.to_owned();
        for x in &mut data {
            *x += 1;
        }
        PyByteArray::new(py, &data)
    }
    Ok(())
}

#[test]
fn test_fn_lifetime() {
    let gil = GILGuard::acquire();
    let py = gil.python();
    let d = PyDict::new(py);
    d.set_item(
        "module_with_functions",
        wrap_module!(module_with_functions)(py),
    )
    .unwrap();
    d.set_item("bytes", PyByteArray::new(py, "abcdefg".as_bytes()))
        .unwrap();
    py.run("assert module_with_functions.bytes_plus1(bytes) == b'bcdefgh'")
        .unwrap();
}
