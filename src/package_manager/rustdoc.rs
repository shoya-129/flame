use std::fs;
use std::path::Path;
use super::meta::*;

pub fn enrich_with_syn(meta: &mut FlameMeta, plugin_path: &Path) {
    let src_dir = plugin_path.join("src");
    if !src_dir.exists() {
        return;
    }

    let mut files_to_scan = Vec::new();
    if let Ok(entries) = fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                files_to_scan.push(p);
            }
        }
    }

    for file_path in files_to_scan {
        if let Ok(code) = fs::read_to_string(&file_path) {
            if let Ok(syntax_tree) = syn::parse_file(&code) {
                for item in syntax_tree.items {
                    match item {
                        syn::Item::Fn(fn_item) => {
                            let (rename, skip, constructor, persistent_runtime) =
                                parse_flame_attrs(&fn_item.attrs);
                            if skip || !matches!(fn_item.vis, syn::Visibility::Public(_)) {
                                continue;
                            }
                            let fn_name = fn_item.sig.ident.to_string();
                            let flame_name = rename.unwrap_or_else(|| fn_name.clone());
                            let is_async = fn_item.sig.asyncness.is_some();
                            let return_type = parse_syn_return(&fn_item.sig.output);
                            let is_constructor = constructor || return_type == meta.module;
                            let params =
                                parse_syn_params(&fn_item.sig.inputs, &fn_item.sig.generics);

                            if let Some(existing) =
                                meta.functions.iter_mut().find(|f| f.name == fn_name)
                            {
                                existing.flame_name = flame_name;
                                existing.is_async = is_async;
                                existing.is_constructor = is_constructor;
                                existing.persistent_runtime = persistent_runtime;
                                existing.params = params;
                                existing.return_type = return_type;
                            } else {
                                meta.functions.push(FlameFunctionMeta {
                                    name: fn_name,
                                    flame_name,
                                    params,
                                    return_type,
                                    is_static: true,
                                    is_generic: !fn_item.sig.generics.params.is_empty(),
                                    requires: vec![],
                                    permissions: vec![],
                                    is_async,
                                    is_constructor,
                                    persistent_runtime,
                                    receiver: None,
                                    docs: extract_syn_docs(&fn_item.attrs),
                                });
                            }
                        }
                        syn::Item::Struct(struct_item) => {
                            let (rename, skip, _, _) = parse_flame_attrs(&struct_item.attrs);
                            if skip {
                                continue;
                            }
                            let struct_name = struct_item.ident.to_string();
                            let flame_name = rename.unwrap_or_else(|| struct_name.clone());
                            let fields = parse_syn_struct_fields(&struct_item.fields);
                            if !meta.structs.iter().any(|s| s.name == struct_name) {
                                meta.structs.push(FlameStructMeta {
                                    name: struct_name,
                                    flame_name,
                                    methods: Vec::new(),
                                    fields,
                                    docs: extract_syn_docs(&struct_item.attrs),
                                });
                            }
                        }
                        syn::Item::Impl(impl_item) => {
                            let self_ty = &impl_item.self_ty;
                            let struct_name = quote::quote!(#self_ty).to_string().replace(" ", "");
                            let struct_name_simple = struct_name
                                .rsplit("::")
                                .next()
                                .unwrap_or(&struct_name)
                                .split('<')
                                .next()
                                .unwrap_or(&struct_name)
                                .to_string();
                            if let Some(struct_meta) = meta.structs.iter_mut().find(|s| {
                                s.name == struct_name
                                    || s.name == struct_name_simple
                                    || struct_name.ends_with(&s.name)
                            }) {
                                for item_in_impl in impl_item.items {
                                    if let syn::ImplItem::Fn(method_item) = item_in_impl {
                                        let (rename, skip, constructor, persistent_runtime) =
                                            parse_flame_attrs(&method_item.attrs);
                                        if skip {
                                            continue;
                                        }
                                        let is_pub =
                                            matches!(method_item.vis, syn::Visibility::Public(_));
                                        if !is_pub {
                                            continue;
                                        }

                                        let m_name = method_item.sig.ident.to_string();
                                        let flame_name = rename.unwrap_or_else(|| m_name.clone());
                                        let is_async = method_item.sig.asyncness.is_some();
                                        let return_type = parse_syn_return(&method_item.sig.output);

                                        let mut receiver = None;
                                        let mut is_static = true;
                                        if let Some(first_arg) = method_item.sig.inputs.first() {
                                            if let syn::FnArg::Receiver(rec) = first_arg {
                                                is_static = false;
                                                let rec_str = quote::quote!(#rec).to_string();
                                                if rec_str.contains("mut") {
                                                    receiver = Some("&mut self".to_string());
                                                } else if rec_str.contains('&') {
                                                    receiver = Some("&self".to_string());
                                                } else {
                                                    receiver = Some("self".to_string());
                                                }
                                            }
                                        }

                                        let is_constructor = constructor
                                            || (is_static
                                                && (return_type == "Self"
                                                    || return_type == struct_name));
                                        let params = parse_syn_params(
                                            &method_item.sig.inputs,
                                            &method_item.sig.generics,
                                        );

                                        if let Some(existing) = struct_meta
                                            .methods
                                            .iter_mut()
                                            .find(|m| m.name == m_name)
                                        {
                                            existing.flame_name = flame_name;
                                            existing.is_async = is_async;
                                            existing.is_constructor = is_constructor;
                                            existing.receiver = receiver;
                                            existing.persistent_runtime = persistent_runtime;
                                            existing.params = params;
                                            existing.return_type = return_type;
                                            existing.is_static = is_static;
                                        } else {
                                            struct_meta.methods.push(FlameFunctionMeta {
                                                name: m_name,
                                                flame_name,
                                                params,
                                                return_type,
                                                is_static,
                                                is_generic: !method_item
                                                    .sig
                                                    .generics
                                                    .params
                                                    .is_empty(),
                                                requires: vec![],
                                                permissions: vec![],
                                                is_async,
                                                is_constructor,
                                                persistent_runtime,
                                                receiver,
                                                docs: extract_syn_docs(&method_item.attrs),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn parse_flame_attrs(attrs: &[syn::Attribute]) -> (Option<String>, bool, bool, bool) {
    let mut rename = None;
    let mut skip = false;
    let mut constructor = false;
    let mut persistent_runtime = false;
    for attr in attrs {
        if attr.path().is_ident("flame") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("constructor") {
                    constructor = true;
                } else if meta.path.is_ident("runtime") || meta.path.is_ident("daemon") {
                    persistent_runtime = true;
                } else if meta.path.is_ident("rename") {
                    if let Ok(value) = meta.value() {
                        if let Ok(s) = value.parse::<syn::LitStr>() {
                            rename = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }
    (rename, skip, constructor, persistent_runtime)
}

fn parse_syn_return(output: &syn::ReturnType) -> String {
    match output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => {
            let ty_str = quote::quote!(#ty).to_string();
            ty_str.replace(" ", "")
        }
    }
}

fn parse_syn_params(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    generics: &syn::Generics,
) -> Vec<FlameParamMeta> {
    let mut params = Vec::new();
    let callback_generics = find_callback_generics(generics);

    for input in inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let name = match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => format!("arg{}", params.len()),
            };
            let ty_node = &*pat_type.ty;
            let ty_str = quote::quote!(#ty_node).to_string();
            let clean_ty = ty_str.replace(" ", "");
            let is_ref = clean_ty.starts_with('&') && !clean_ty.starts_with("&mut");
            let is_mut = clean_ty.starts_with("&mut") || clean_ty.starts_with("mut");
            let is_callback = clean_ty.contains("Callback")
                || clean_ty.contains("Handler")
                || clean_ty.contains("Fn(")
                || callback_generics.contains(&clean_ty);

            params.push(FlameParamMeta {
                name,
                type_name: clean_ty,
                is_callback,
                is_ref,
                is_mut,
            });
        }
    }
    params
}

fn find_callback_generics(generics: &syn::Generics) -> Vec<String> {
    let mut cb_generics = Vec::new();
    for param in &generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let ident = type_param.ident.to_string();
            for bound in &type_param.bounds {
                let b_str = quote::quote!(#bound).to_string();
                if b_str.contains("Handler") || b_str.contains("Fn") || b_str.contains("Callback") {
                    cb_generics.push(ident.clone());
                }
            }
        }
    }
    if let Some(where_clause) = &generics.where_clause {
        for pred in &where_clause.predicates {
            if let syn::WherePredicate::Type(pred_type) = pred {
                let target = quote::quote!(#pred_type.bounded_ty).to_string();
                for bound in &pred_type.bounds {
                    let b_str = quote::quote!(#bound).to_string();
                    if b_str.contains("Handler")
                        || b_str.contains("Fn")
                        || b_str.contains("Callback")
                    {
                        cb_generics.push(target.replace(" ", ""));
                    }
                }
            }
        }
    }
    cb_generics
}

fn parse_syn_struct_fields(fields: &syn::Fields) -> Vec<FlameStructFieldMeta> {
    let mut parsed_fields = Vec::new();
    for field in fields {
        if let Some(ident) = &field.ident {
            let is_pub = matches!(field.vis, syn::Visibility::Public(_));
            if !is_pub {
                continue;
            }
            let name = ident.to_string();
            let ty_node = &field.ty;
            let ty_str = quote::quote!(#ty_node).to_string().replace(" ", "");
            let docs = extract_syn_docs(&field.attrs);
            parsed_fields.push(FlameStructFieldMeta {
                name,
                type_name: ty_str,
                docs,
            });
        }
    }
    parsed_fields
}

fn extract_syn_docs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr) = &meta.value {
                    if let syn::Lit::Str(s) = &expr.lit {
                        let line = s.value();
                        let line = line.strip_prefix(' ').unwrap_or(&line);
                        doc_lines.push(line.to_string());
                    }
                }
            }
        }
    }
    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n").trim().to_string())
    }
}

pub fn parse_rustdoc_json(rustdoc_json_path: &Path, target: &str) -> FlameMeta {
    let mut functions = Vec::new();
    let mut structs = Vec::new();

    if let Ok(json_str) = fs::read_to_string(rustdoc_json_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let paths = v.get("paths").and_then(|p| p.as_object());
            if let Some(index) = v.get("index").and_then(|i| i.as_object()) {
                for (id, item) in index {
                    if item.get("visibility").and_then(|v| v.as_str()) != Some("public") {
                        continue;
                    }
                    let name = match item.get("name").and_then(|n| n.as_str()) {
                        Some(n) => n,
                        None => continue,
                    };
                    if let Some(inner) = item.get("inner").and_then(|i| i.as_object()) {
                        if inner.contains_key("function") {
                            let mut is_top_level = false;
                            if let Some(p_obj) =
                                paths.and_then(|p| p.get(id)).and_then(|p| p.as_object())
                            {
                                if let Some(path_arr) =
                                    p_obj.get("path").and_then(|pa| pa.as_array())
                                {
                                    if path_arr.len() == 2 {
                                        is_top_level = true;
                                    }
                                }
                            }
                            if !is_top_level {
                                continue;
                            }
                            let mut param_types = vec![];
                            let mut return_type = "NativeObject".to_string();
                            let mut is_generic = false;
                            let mut is_async = false;
                            let mut has_bounds = false;
                            if let Some(func_obj) =
                                inner.get("function").and_then(|f| f.as_object())
                            {
                                if let Some(generics) =
                                    func_obj.get("generics").and_then(|g| g.as_object())
                                {
                                    if let Some(params) =
                                        generics.get("params").and_then(|p| p.as_array())
                                    {
                                        if !params.is_empty() {
                                            if params.len() == 1 {
                                                if let Some(kind) = params[0]
                                                    .get("kind")
                                                    .and_then(|k| k.as_object())
                                                {
                                                    if let Some(type_obj) =
                                                        kind.get("type").and_then(|t| t.as_object())
                                                    {
                                                        if let Some(bounds) = type_obj
                                                            .get("bounds")
                                                            .and_then(|b| b.as_array())
                                                        {
                                                            if !bounds.is_empty() {
                                                                has_bounds = true;
                                                            }
                                                        }
                                                    }
                                                }
                                                is_generic = true;
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }
                                if has_bounds {
                                    continue;
                                }
                                if let Some(header) =
                                    func_obj.get("header").and_then(|h| h.as_object())
                                {
                                    is_async = header
                                        .get("is_async")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                }
                                if let Some(sig) = func_obj.get("sig").and_then(|s| s.as_object()) {
                                    if let Some(inputs) =
                                        sig.get("inputs").and_then(|i| i.as_array())
                                    {
                                        for input in inputs {
                                            if let Some(arr) = input.as_array() {
                                                if arr.len() == 2 {
                                                    param_types.push(parse_type(&arr[1]));
                                                }
                                            }
                                        }
                                    }
                                    if let Some(output) = sig.get("output") {
                                        return_type = parse_type(output);
                                    } else {
                                        return_type = "()".to_string();
                                    }
                                }
                            }

                            if let Some(func_obj) =
                                inner.get("function").and_then(|f| f.as_object())
                            {
                                if let Some(generics) =
                                    func_obj.get("generics").and_then(|g| g.as_object())
                                {
                                    if let Some(params) =
                                        generics.get("params").and_then(|p| p.as_array())
                                    {
                                        if !params.is_empty() {
                                            is_generic = true;
                                        }
                                    }
                                }
                            }

                            let is_constructor = return_type == target;
                            functions.push(FlameFunctionMeta {
                                name: name.to_string(),
                                flame_name: name.to_string(),
                                is_generic,
                                is_async,
                                is_constructor,
                                requires: vec![],
                                permissions: vec![],
                                persistent_runtime: false,
                                receiver: None,
                                params: param_types
                                    .iter()
                                    .enumerate()
                                    .map(|(i, pt)| FlameParamMeta {
                                        name: format!("arg{}", i),
                                        type_name: pt.clone(),
                                        is_callback: pt == "Callback" || pt == "FlameCallback",
                                        is_ref: false,
                                        is_mut: false,
                                    })
                                    .collect(),
                                return_type: return_type.clone(),
                                is_static: true,
                                docs: item
                                    .get("docs")
                                    .and_then(|d| d.as_str())
                                    .map(|d| d.trim().to_string())
                                    .filter(|d| !d.is_empty()),
                            });
                        } else if inner.contains_key("struct") {
                            if name == "Hyphenated"
                                || name == "Simple"
                                || name == "Urn"
                                || name == "Braced"
                                || name == "ThreadLocalContext"
                                || name == "WeightedIndex"
                                || name == "Bernoulli"
                                || name == "StepRng"
                                || name == "ReseedingRng"
                                || name == "Choose"
                                || name == "ThreadRng"
                            {
                                continue;
                            }
                            let mut s_methods = vec![];
                            if let Some(impls) = inner
                                .get("struct")
                                .and_then(|s| s.get("impls"))
                                .and_then(|i| i.as_array())
                            {
                                for impl_id in impls {
                                    let impl_id_str = if let Some(s) = impl_id.as_str() {
                                        s.to_string()
                                    } else {
                                        impl_id.to_string()
                                    };
                                    if let Some(impl_item) =
                                        index.get(&impl_id_str).and_then(|i| i.as_object())
                                    {
                                        if let Some(impl_inner) =
                                            impl_item.get("inner").and_then(|i| i.as_object())
                                        {
                                            if let Some(impl_block) =
                                                impl_inner.get("impl").and_then(|i| i.as_object())
                                            {
                                                if let Some(items) = impl_block
                                                    .get("items")
                                                    .and_then(|i| i.as_array())
                                                {
                                                    for m_id in items {
                                                        let m_id_str =
                                                            if let Some(s) = m_id.as_str() {
                                                                s.to_string()
                                                            } else {
                                                                m_id.to_string()
                                                            };
                                                        if let Some(m_item) = index
                                                            .get(&m_id_str)
                                                            .and_then(|i| i.as_object())
                                                        {
                                                            if m_item
                                                                .get("visibility")
                                                                .and_then(|v| v.as_str())
                                                                != Some("public")
                                                            {
                                                                continue;
                                                            }
                                                            if let Some(m_name) = m_item
                                                                .get("name")
                                                                .and_then(|n| n.as_str())
                                                            {
                                                                if let Some(m_inner) = m_item
                                                                    .get("inner")
                                                                    .and_then(|i| i.as_object())
                                                                {
                                                                    if m_inner
                                                                        .contains_key("function")
                                                                    {
                                                                        let mut m_param_types =
                                                                            vec![];
                                                                        let mut m_return_type =
                                                                            "NativeObject"
                                                                                .to_string();

                                                                        let mut is_static = true;
                                                                        let mut consumes_self =
                                                                            false;
                                                                        let mut is_generic = false;
                                                                        let mut is_async = false;
                                                                        if let Some(func_obj) =
                                                                            m_inner
                                                                                .get("function")
                                                                                .and_then(|f| {
                                                                                    f.as_object()
                                                                                })
                                                                        {
                                                                            if let Some(generics) =
                                                                                func_obj
                                                                                    .get("generics")
                                                                                    .and_then(|g| {
                                                                                        g.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    params,
                                                                                ) = generics
                                                                                    .get("params")
                                                                                    .and_then(|p| {
                                                                                        p.as_array()
                                                                                    })
                                                                                {
                                                                                    if !params
                                                                                        .is_empty()
                                                                                    {
                                                                                        is_generic = true;
                                                                                    }
                                                                                }
                                                                            }
                                                                            if let Some(header) =
                                                                                func_obj
                                                                                    .get("header")
                                                                                    .and_then(|h| {
                                                                                        h.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                is_async = header
                                                                                    .get("is_async")
                                                                                    .and_then(|v| {
                                                                                        v.as_bool()
                                                                                    })
                                                                                    .unwrap_or(
                                                                                        false,
                                                                                    );
                                                                            }
                                                                            if let Some(sig) =
                                                                                func_obj
                                                                                    .get("sig")
                                                                                    .and_then(|s| {
                                                                                        s.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    inputs,
                                                                                ) = sig
                                                                                    .get("inputs")
                                                                                    .and_then(|i| {
                                                                                        i.as_array()
                                                                                    })
                                                                                {
                                                                                    for input in
                                                                                        inputs
                                                                                    {
                                                                                        if let Some(
                                                                                        arr,
                                                                                    ) = input
                                                                                        .as_array()
                                                                                    {
                                                                                        if arr.len()
                                                                                            == 2
                                                                                        {
                                                                                            let param_name = arr[0].as_str().unwrap_or_default();
                                                                                            if param_name == "self" {
                                                                                                        is_static = false;
                                                                                                        if let Some(g) = arr[1].as_object().and_then(|o| o.get("generic")).and_then(|v| v.as_str()) {
                                                                                                            if g == "Self" { consumes_self = true; }
                                                                                                        }
                                                                                                        continue;
                                                                                                    }
                                                                                            m_param_types.push(parse_type(&arr[1]));
                                                                                        }
                                                                                    }
                                                                                    }
                                                                                }
                                                                                if let Some(
                                                                                    output,
                                                                                ) = sig
                                                                                    .get("output")
                                                                                {
                                                                                    m_return_type =
                                                                                        parse_type(
                                                                                            output,
                                                                                        );
                                                                                } else {
                                                                                    m_return_type =
                                                                                    "()".to_string(
                                                                                    );
                                                                                }
                                                                            }
                                                                        }
                                                                        if let Some(func_obj) =
                                                                            m_inner
                                                                                .get("function")
                                                                                .and_then(|f| {
                                                                                    f.as_object()
                                                                                })
                                                                        {
                                                                            if let Some(generics) =
                                                                                func_obj
                                                                                    .get("generics")
                                                                                    .and_then(|g| {
                                                                                        g.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    params,
                                                                                ) = generics
                                                                                    .get("params")
                                                                                    .and_then(|p| {
                                                                                        p.as_array()
                                                                                    })
                                                                                {
                                                                                    if !params
                                                                                        .is_empty()
                                                                                    {
                                                                                        is_generic = true;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }

                                                                        let receiver = if !is_static
                                                                        {
                                                                            if consumes_self {
                                                                                Some(
                                                                                    "self"
                                                                                        .to_string(
                                                                                        ),
                                                                                )
                                                                            } else {
                                                                                Some(
                                                                                    "&mut self"
                                                                                        .to_string(
                                                                                        ),
                                                                                )
                                                                            }
                                                                        } else {
                                                                            None
                                                                        };
                                                                        let is_constructor =
                                                                            is_static
                                                                                && (m_return_type
                                                                                    == "Self"
                                                                                    || m_return_type
                                                                                        == name);
                                                                        s_methods.push(FlameFunctionMeta {
                                                                            name: m_name.to_string(),
                                                                            flame_name: m_name.to_string(),
                                                                            is_generic,
                                                                            is_async,
                                                                            is_constructor,
                                                                            requires: vec![],
                                                                            permissions: vec![],
                                                                            persistent_runtime: false,
                                                                            receiver,
                                                                            params: m_param_types.iter().enumerate().map(|(idx, pt)| FlameParamMeta {
                                                                                name: format!("arg{}", idx),
                                                                                type_name: pt.clone(),
                                                                                is_callback: pt == "Callback" || pt == "FlameCallback",
                                                                                is_ref: false,
                                                                                is_mut: false,
                                                                            }).collect(),
                                                                            return_type: m_return_type,
                                                                            is_static,
                                                                            docs: m_item
                                                                                .get("docs")
                                                                                .and_then(|d| d.as_str())
                                                                                .map(|d| d.trim().to_string())
                                                                                .filter(|d| !d.is_empty()),
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            structs.push(FlameStructMeta {
                                name: name.to_string(),
                                flame_name: name.to_string(),
                                fields: Vec::new(),
                                methods: s_methods,
                                docs: item
                                    .get("docs")
                                    .and_then(|d| d.as_str())
                                    .map(|d| d.trim().to_string())
                                    .filter(|d| !d.is_empty()),
                            });
                        }
                    }
                }
            }
        }
    } else {
        println!(
            "WARNING: Rustdoc generation failed for '{}'. Is cargo +nightly installed?",
            target
        );
    }

    let lib_filename = if cfg!(target_os = "windows") {
        format!("{}.dll", target)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", target)
    } else {
        format!("lib{}.so", target)
    };

    FlameMeta {
        module: target.to_string(),
        kind: "native".to_string(),
        lib: Some(lib_filename),
        functions,
        structs,
        docs: None,
    }
}

fn parse_type(ty: &serde_json::Value) -> String {
    if let Some(bref) = ty.get("borrowed_ref").and_then(|b| b.as_object()) {
        if let Some(inner) = bref.get("type") {
            let mut ty_str = String::from("&");
            if let Some(lt) = bref.get("lifetime").and_then(|l| l.as_str()) {
                if lt == "'static" {
                    ty_str.push_str("'static ");
                }
            }
            if bref
                .get("mutable")
                .and_then(|m| m.as_bool())
                .unwrap_or(false)
            {
                ty_str.push_str("mut ");
            }
            ty_str.push_str(&parse_type(inner));
            return ty_str;
        }
    }
    if let Some(prim) = ty.get("primitive").and_then(|p| p.as_str()) {
        return prim.to_string();
    }
    if let Some(res) = ty.get("resolved_path").and_then(|p| p.as_object()) {
        if let Some(name) = res
            .get("name")
            .or_else(|| res.get("path"))
            .and_then(|n| n.as_str())
        {
            return name.to_string();
        }
    }
    if ty.get("tuple").is_some() {
        return "(tuple)".to_string();
    }
    if ty.get("impl_trait").is_some() {
        return "impl trait".to_string();
    }
    if let Some(generic) = ty.get("generic").and_then(|g| g.as_str()) {
        return generic.to_string();
    }
    "NativeObject".to_string()
}
