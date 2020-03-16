// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate contains the implementation of the proc macro attributes

#![recursion_limit = "1024"]

mod defs;
mod func;
mod method;
mod module;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
mod pyproto;
mod utils;

pub use module::{add_fn_to_module, process_functions_in_module, py_init};
pub use pyclass::{build_py_class, PyClassArgs};
pub use pyfunction::{build_py_function, PyFunctionAttr};
pub use pyimpl::{build_py_methods, impl_methods};
pub use pyproto::build_py_proto;
pub use utils::get_doc;

#[cfg(test)]
mod test {
    use quote::quote;
    #[test]
    fn nothing() {
        println!("{}", 1 + 1);
    }
    #[test]
    fn use_proc_macro2() {
        use proc_macro2::{Ident, Span};
        let call_ident = Ident::new("calligraphy", Span::call_site());
        println!("{}", call_ident);
    }
    #[test]
    fn use_quote1() {
        let invalid = quote! { test="1", test2 };
        println!("{}", invalid);
    }
    #[test]
    fn use_quote2() {
        let invalid = quote! { fn f() {} };
        println!("{}", invalid);
    }
}
