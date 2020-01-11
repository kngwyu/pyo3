// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

extern crate proc_macro;
use proc_macro::TokenStream;
use pyo3_derive_backend::*;
use quote::quote;
use syn::parse_macro_input;

/// Internally, this proc macro create a new c function called `PyInit_{my_module}`
/// that then calls the init function you provided
#[proc_macro_attribute]
pub fn pymodule(attr: TokenStream, input: TokenStream) -> TokenStream {
    fn pymodule_for_fn(attr: TokenStream, mut func: syn::ItemFn) -> TokenStream {
        let modname = if attr.is_empty() {
            func.sig.ident.clone()
        } else {
            parse_macro_input!(attr as syn::Ident)
        };

        process_functions_in_module(&mut func);

        let doc = match get_doc(&func.attrs, None, false) {
            Ok(doc) => doc,
            Err(err) => return err.to_compile_error().into(),
        };

        let expanded = py_init(&func.sig.ident, &modname, doc);

        quote!(
            #func
            #expanded
        )
        .into()
    }

    fn pymodule_for_mod(attr: TokenStream, mod_: syn::ItemMod) -> TokenStream {
        let modname = if attr.is_empty() {
            mod_.ident.clone()
        } else {
            parse_macro_input!(attr as syn::Ident)
        };

        let doc = match get_doc(&mod_.attrs, None, false) {
            Ok(doc) => doc,
            Err(err) => return err.to_compile_error().into(),
        };

        let expanded = py_init(&mod_.ident, &modname, doc);

        quote!(
            #mod_
            #expanded
        )
        .into()
    }

    match syn::parse_macro_input::parse::<syn::ItemFn>(input.clone()) {
        Ok(data) => pymodule_for_fn(attr, data),
        Err(_) => match syn::parse_macro_input::parse::<syn::ItemMod>(input) {
            Ok(data) => pymodule_for_mod(attr, data),
            Err(err) => TokenStream::from(err.to_compile_error()),
        },
    }
}

#[proc_macro_attribute]
pub fn pyproto(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_proto(&mut ast).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attr as PyClassArgs);
    let expanded = build_py_class(&mut ast, &args).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_methods(&mut ast).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyfunction(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(attr as PyFunctionAttr);

    let expanded = build_py_function(&mut ast, args).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}
