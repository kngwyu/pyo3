// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the python interpreter such as `int`, `str` and `tuple`

pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::complex::PyComplex;
pub use self::datetime::PyDeltaAccess;
pub use self::datetime::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyTime, PyTimeAccess, PyTzInfo,
};
pub use self::dict::{IntoPyDict, PyDict};
pub use self::floatob::PyFloat;
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::module::PyModule;
#[cfg(not(Py_3))]
pub use self::num2::{PyInt, PyLong};
#[cfg(Py_3)]
pub use self::num3::PyLong;
#[cfg(Py_3)]
pub use self::num3::PyLong as PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
#[cfg(Py_3)]
pub use self::string::{PyBytes, PyString, PyString as PyUnicode};
#[cfg(not(Py_3))]
pub use self::string2::{PyBytes, PyString, PyUnicode};
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;
use crate::ffi;
use crate::PyObject;

/// Implements a typesafe conversions throught [FromPyObject], given a typecheck function as second
/// parameter
#[macro_export]
macro_rules! pyobject_downcast (
    ($name: ty, $checkfunction: path $(,$type_param: ident)*) => (
        impl<'a, $($type_param,)*> $crate::FromPyObject<'a> for &'a $name
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(ob: &'a $crate::types::PyBaseObject) -> $crate::PyResult<Self>
            {
                unsafe {
                    if $checkfunction(ob.as_ptr()) != 0 {
                        Ok(&*(ob as *const $crate::types::PyBaseObject as *const $name))
                    } else {
                        Err($crate::PyDowncastError.into())
                    }
                }
            }
        }
    );
);

#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name: ty $(,$type_param: ident)*) => {
        impl<$($type_param,)*> ::std::convert::AsRef<$crate::types::PyBaseObject> for $name {
            #[inline]
            fn as_ref(&self) -> &$crate::types::PyBaseObject {
                unsafe{&*(self as *const $name as *const $crate::types::PyBaseObject)}
            }
        }

        impl<$($type_param,)*> $crate::PyNativeType for $name {
            fn py(&self) -> $crate::Python {
                unsafe { $crate::Python::assume_gil_acquired() }
            }
        }

        impl<$($type_param,)*> $crate::AsPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$($type_param,)*> PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                use $crate::AsPyPointer;

                self.as_ptr() == o.as_ptr()
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type (
    ($name: ty, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_type_convert!($name, $typeobject, $checkfunction $(,$type_param)*);

        impl<'a, $($type_param,)*> ::std::convert::From<&'a $name> for &'a $crate::types::PyBaseObject {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::types::PyBaseObject)}
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type_convert(
    ($name: ty, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl<$($type_param,)*> $crate::type_object::PyTypeInfo for $name {
            type Type = ();
            type BaseType = $crate::types::PyBaseObject;

            const NAME: &'static str = stringify!($name);
            const SIZE: usize = ::std::mem::size_of::<$crate::ffi::PyObject>();
            const OFFSET: isize = 0;

            #[inline]
            unsafe fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                &mut $typeobject
            }

            #[allow(unused_unsafe)]
            fn is_instance(ptr: &$crate::types::PyBaseObject) -> bool {
                use $crate::AsPyPointer;

                unsafe { $checkfunction(ptr.as_ptr()) > 0 }
            }
        }

        impl<$($type_param,)*> $crate::type_object::PyObjectAlloc for $name {}

        impl<$($type_param,)*> $crate::type_object::PyTypeObject for $name {
            fn init_type() -> std::ptr::NonNull<$crate::ffi::PyTypeObject> {
                unsafe {
                    std::ptr::NonNull::new_unchecked(<Self as $crate::type_object::PyTypeInfo>::type_object() as *mut _)
                }
            }
        }

        impl<$($type_param,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;

                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())}
            }
        }

        impl<$($type_param,)*> ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.repr().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($type_param,)*> ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.str().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }
    };
);

/// Represents general python instance.
#[repr(transparent)]
pub struct PyBaseObject(PyObject);
pyobject_native_type_named!(PyBaseObject);
pyobject_native_type_convert!(PyBaseObject, ffi::PyBaseObject_Type, ffi::PyObject_Check);

mod boolobject;
mod bytearray;
mod complex;
mod datetime;
mod dict;
mod floatob;
mod iterator;
mod list;
mod module;
mod sequence;
mod set;
mod slice;
mod stringutils;
mod tuple;
mod typeobject;

#[macro_use]
mod num_common;

#[cfg(Py_3)]
mod num3;

#[cfg(not(Py_3))]
mod num2;

#[cfg(Py_3)]
mod string;

#[cfg(not(Py_3))]
mod string2;
