//! Process #[pyo3(add_class)] and #[pyo3(add_function)]
use proc_macro2::Span;
use syn::spanned::Spanned;

/// Finds and takes care of the #[pyo3(register)] in `#[pymodule]`
pub fn process_pyo3_add(mod_ident: &Option<syn::Ident>, func: &mut syn::ItemFn) -> syn::Result<()> {
    let mut stmts: Vec<syn::Stmt> = Vec::new();

    for mut stmt in func.block.stmts.drain(..) {
        if let syn::Stmt::Item(syn::Item::Use(ref mut use_)) = stmt {
            stmts.append(&mut extract_item(mod_ident, func.sig.inputs.span(), use_)?);
        };
        stmts.push(stmt);
    }

    func.block.stmts = stmts;
    Ok(())
}

fn extract_item(
    mod_ident: &Option<syn::Ident>,
    arg_span: Span,
    use_: &mut syn::ItemUse,
) -> syn::Result<Vec<syn::Stmt>> {
    let mut new_attrs = Vec::new();
    let mut return_items = vec![];
    let mut used = false;
    let syn::ItemUse {
        attrs,
        vis: _,
        use_token: _,
        leading_colon: _,
        tree,
        semi_token: _,
    } = use_;
    let get_ident_or = &|| match mod_ident.as_ref() {
        Some(i) => Ok(i),
        None => return Err(syn::Error::new(arg_span, "")),
    };
    for attr in attrs.drain(..) {
        if let Ok(syn::Meta::List(ref list)) = attr.parse_meta() {
            if list.path.is_ident("pyo3") {
                if used {
                    return Err(syn::Error::new_spanned(
                        list,
                        "use item can only 1 #[pyo3(add_*)]",
                    ));
                }
                for meta in list.nested.iter() {
                    if let syn::NestedMeta::Meta(ref metaitem) = meta {
                        if metaitem.path().is_ident("add_class") {
                            return_items.push(add_class(get_ident_or()?, tree)?);
                            used = true;
                        } else if metaitem.path().is_ident("add_function") {
                            return_items.push(add_function(get_ident_or()?, tree)?);
                            used = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                metaitem,
                                "Only add_class and add_function are supported",
                            ));
                        }
                    }
                }
            } else {
                new_attrs.push(attr.clone())
            }
        } else {
            new_attrs.push(attr.clone());
        }
    }
    *attrs = new_attrs;
    Ok(return_items)
}

fn get_use_item(tree: &syn::UseTree) -> syn::Result<syn::Ident> {
    match tree {
        syn::UseTree::Name(name) => Ok(name.ident.clone()),
        syn::UseTree::Path(path) => unimplemented!(),
        syn::UseTree::Group(group) => unimplemented!(),
        _ => Err(syn::Error::new_spanned(
            tree,
            "Invalid use item for #[pyo3(add_*)]",
        )),
    }
}

fn add_class(mod_ident: &syn::Ident, tree: &syn::UseTree) -> syn::Result<syn::Stmt> {
    unimplemented!()
}

fn add_function(mod_ident: &syn::Ident, tree: &syn::UseTree) -> syn::Result<syn::Stmt> {
    let function_name = get_use_item(tree)?;
    let wrapper_ident = crate::module::function_wrapper_ident(&function_name);
    Ok(syn::parse_quote! {
        #mod_ident.add_wrapped(&#wrapper_ident)?;
    })
}
