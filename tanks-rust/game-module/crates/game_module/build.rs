//! Generates the C FFI layer from Rust code.

use ::std::{env, fs, path::Path};

use syn::{FnArg, GenericArgument, Item, ItemFn, ItemStruct, PathArguments, Type};

const ARETE_PUBLIC_COMPONENTS: &[&str] = &[
    "Camera",
    "Color",
    "DirectionalLight",
    "DynamicStaticMesh",
    "PointLight",
    "Transform",
];

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ffi.rs");

    let input = env::current_dir().unwrap().join("src/lib.rs");
    let input = fs::read_to_string(input).unwrap();
    let file = syn::parse_file(&input).unwrap();

    let mut parsed_info = ParsedInfo::default();

    for item in &file.items {
        match item {
            Item::Fn(item) => parsed_info.parse_fn(item),
            Item::Struct(item) => parsed_info.parse_struct(item),
            _ => {}
        }
    }

    fs::write(dest_path, parsed_info.gen_ffi()).unwrap();
}

#[derive(Debug)]
enum StructType {
    Component,
    Resource,
}

#[derive(Debug)]
enum ArgType {
    DataAccessDirect,
    Query { inputs: Vec<SystemInputInfo> },
}

#[derive(Debug, Default)]
struct ParsedInfo {
    systems: Vec<SystemInfo>,
    structs: Vec<StructInfo>,
}

#[derive(Debug)]
struct SystemInfo {
    ident: String,
    is_once: bool,
    inputs: Vec<SystemInputInfo>,
}

#[derive(Debug)]
struct SystemInputInfo {
    ident: String,
    arg_type: ArgType,
    mutable: bool,
}

#[derive(Debug)]
struct StructInfo {
    ident: String,
    struct_type: StructType,
}

impl ParsedInfo {
    fn parse_fn(&mut self, item: &ItemFn) {
        let is_system = item.attrs.iter().any(|attr| attr.path().is_ident("system"));
        let is_system_once = item
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("system_once"));

        if !is_system && !is_system_once {
            return;
        }

        let mut inputs = Vec::new();

        for input in &item.sig.inputs {
            let FnArg::Typed(input) = input else {
                panic!("systems cannot take self")
            };

            let system_input = match input.ty.as_ref() {
                Type::Path(component) => {
                    // Query

                    let PathArguments::AngleBracketed(query_inputs) =
                        &component.path.segments.last().unwrap().arguments
                    else {
                        panic!("invalid query generics")
                    };

                    let param_type = component.path.segments.last().unwrap().ident.to_string();

                    let inputs = query_inputs
                        .args
                        .iter()
                        .flat_map(|input| {
                            let GenericArgument::Type(input) = input else {
                                panic!("invalid query generics")
                            };

                            if let Type::Reference(ty) = input {
                                let Type::Path(component) = ty.elem.as_ref() else {
                                    panic!("unsupported query input type")
                                };

                                Vec::from([SystemInputInfo {
                                    ident: component
                                        .path
                                        .segments
                                        .last()
                                        .unwrap()
                                        .ident
                                        .to_string(),
                                    arg_type: ArgType::DataAccessDirect,
                                    mutable: ty.mutability.is_some(),
                                }])
                            } else {
                                let Type::Tuple(tuple) = input else {
                                    panic!("unsupported query input type")
                                };

                                tuple
                                    .elems
                                    .iter()
                                    .map(|elem| {
                                        let Type::Reference(ty) = elem else {
                                            panic!("system inputs must be references")
                                        };

                                        let Type::Path(component) = ty.elem.as_ref() else {
                                            panic!("unsupported system input type")
                                        };

                                        SystemInputInfo {
                                            ident: component
                                                .path
                                                .segments
                                                .last()
                                                .unwrap()
                                                .ident
                                                .to_string(),
                                            arg_type: ArgType::DataAccessDirect,
                                            mutable: ty.mutability.is_some(),
                                        }
                                    })
                                    .collect()
                            }
                        })
                        .collect();

                    SystemInputInfo {
                        ident: param_type,
                        arg_type: ArgType::Query { inputs },
                        mutable: false,
                    }
                }
                Type::Reference(ty) => {
                    // Component or Resource

                    let Type::Path(component) = ty.elem.as_ref() else {
                        panic!("unsupported system input type")
                    };

                    let param_type = component.path.segments.last().unwrap().ident.to_string();

                    if param_type == "Query" {
                        panic!("query inputs must be taken by value");
                    }

                    SystemInputInfo {
                        ident: param_type,
                        arg_type: ArgType::DataAccessDirect,
                        mutable: ty.mutability.is_some(),
                    }
                }
                _ => panic!("system inputs must be references"),
            };

            inputs.push(system_input);
        }

        self.systems.push(SystemInfo {
            ident: item.sig.ident.to_string(),
            is_once: is_system_once,
            inputs,
        });
    }

    fn parse_struct(&mut self, item: &ItemStruct) {
        let struct_type = item
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("derive"))
            .find_map(|attr| {
                let mut is_component = false;
                let mut is_resource = false;
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Component") {
                        is_component = true;
                    }

                    if meta.path.is_ident("Resource") {
                        is_resource = true;
                    }

                    Ok(())
                })
                .unwrap();

                if is_component {
                    Some(StructType::Component)
                } else if is_resource {
                    Some(StructType::Resource)
                } else {
                    None
                }
            });

        let Some(struct_type) = struct_type else {
            return;
        };

        self.structs.push(StructInfo {
            ident: item.ident.to_string(),
            struct_type,
        });
    }

    fn gen_ffi(self) -> String {
        let mut output = String::new();

        output += &gen_version();
        output += &self.gen_components();
        output += &self.gen_resource_init();
        output += &self.gen_systems();
        output += &self.gen_callbacks();

        output
    }

    fn gen_components(&self) -> String {
        let mut output = String::new();

        output += "#[repr(C)]\n";
        output += "pub enum ComponentType {\n";
        output += "    Component,\n";
        output += "    Resource,\n";
        output += "}\n\n";

        output += "#[repr(C)]\n";
        output += "pub enum ArgType {\n";
        output += "    DataAccessMut,\n";
        output += "    DataAccessRef,\n";
        output += "    Query,\n";
        output += "}\n\n";

        output += &self.gen_component_string_id();
        output += &self.gen_component_size();
        output += &self.gen_component_align();
        output += &self.gen_component_type();
        output += &self.gen_set_component_ids();

        output
    }

    fn gen_component_string_id(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn component_string_id(index: usize) -> *const ::std::ffi::c_char {\n";
        output += "    match index {\n";

        for (i, struct_info) in self.structs.iter().enumerate() {
            output += &format!(
                "        {i} => {}::string_id().as_ptr(),\n",
                struct_info.ident
            );
        }

        output += "        _ => ::std::ptr::null(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_component_size(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn component_size(string_id: *const ::std::ffi::c_char) -> usize {\n";
        output += "    let string_id = ::std::ffi::CStr::from_ptr(string_id);\n\n";

        if self.structs.is_empty() {
            output += "    ::std::process::abort()\n;"
        } else {
            output += &format!(
                "    if string_id == {}::string_id() {{\n",
                self.structs[0].ident
            );
            output += &format!(
                "        ::std::mem::size_of::<{}>()\n",
                self.structs[0].ident
            );
            for struct_info in &self.structs[1..] {
                output += &format!(
                    "    }} else if string_id == {}::string_id() {{\n",
                    struct_info.ident
                );
                output += &format!("        ::std::mem::size_of::<{}>()\n", struct_info.ident);
            }

            output += "    } else {\n";
            output += "        ::std::process::abort()\n";
            output += "    }\n";
        }

        output += "}\n\n";

        output
    }

    fn gen_component_align(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn component_align(string_id: *const ::std::ffi::c_char) -> usize {\n";
        output += "    let string_id = ::std::ffi::CStr::from_ptr(string_id);\n\n";

        if self.structs.is_empty() {
            output += "    ::std::process::abort()\n";
        } else {
            output += &format!(
                "    if string_id == {}::string_id() {{\n",
                self.structs[0].ident
            );
            output += &format!(
                "        ::std::mem::align_of::<{}>()\n",
                self.structs[0].ident
            );
            for struct_info in &self.structs[1..] {
                output += &format!(
                    "    }} else if string_id == {}::string_id() {{\n",
                    struct_info.ident
                );
                output += &format!("        ::std::mem::align_of::<{}>()\n", struct_info.ident);
            }

            output += "    } else {\n";
            output += "        ::std::process::abort()\n";
            output += "    }\n";
        }

        output += "}\n\n";

        output
    }

    fn gen_component_type(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn component_type(string_id: *const ::std::ffi::c_char) -> ComponentType {\n";
        output += "    let string_id = ::std::ffi::CStr::from_ptr(string_id);\n\n";

        if self.structs.is_empty() {
            output += "    ::std::process::abort()\n";
        } else {
            output += &format!(
                "    if string_id == {}::string_id() {{\n",
                self.structs[0].ident
            );
            output += "        ComponentType::";
            output += match &self.structs[0].struct_type {
                StructType::Component => "Component\n",
                StructType::Resource => "Resource\n",
            };

            for struct_info in &self.structs[1..] {
                output += &format!(
                    "    }} else if string_id == {}::string_id() {{\n",
                    struct_info.ident
                );
                output += "        ComponentType::";
                output += match &struct_info.struct_type {
                    StructType::Component => "Component\n",
                    StructType::Resource => "Resource\n",
                };
            }

            output += "    } else {\n";
            output += "        ::std::process::abort()\n";
            output += "    }\n";
        }

        output += "}\n\n";

        output
    }

    fn gen_set_component_ids(&self) -> String {
        let mut components: Vec<_> = self
            .systems
            .iter()
            .flat_map(|s| &s.inputs)
            .filter_map(|i| {
                if i.ident != "Query" {
                    Some(i.ident.clone())
                } else {
                    None
                }
            })
            .chain(ARETE_PUBLIC_COMPONENTS.iter().map(|s| s.to_string()))
            .chain(self.structs.iter().map(|s| s.ident.clone()))
            .collect();

        components.sort_unstable();
        components.dedup();

        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn set_component_id(string_id: *const ::std::ffi::c_char, id: ComponentId) {\n";
        output += "    let string_id = ::std::ffi::CStr::from_ptr(string_id);\n\n";

        if !components.is_empty() {
            output += &format!("    if string_id == {}::string_id() {{\n", components[0]);
            output += &format!("        {}::set_id(id);\n", components[0]);

            for ident in &components[1..] {
                output += &format!("    }} else if string_id == {ident}::string_id() {{\n");
                output += &format!("        {ident}::set_id(id);\n");
            }

            output += "    }\n";
        }

        output += "}\n\n";

        output
    }

    fn gen_resource_init(&self) -> String {
        let resources: Vec<_> = self
            .structs
            .iter()
            .filter(|s| matches!(s.struct_type, StructType::Resource))
            .map(|s| s.ident.as_str())
            .collect();

        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub unsafe extern \"C\" fn resource_init(\n";
        output += "    string_id: *const ::std::ffi::c_char,\n";
        output += "    val: *mut ::std::ffi::c_void,\n";
        output += ") -> i32 {\n";
        output += "    let string_id = ::std::ffi::CStr::from_ptr(string_id);\n\n";

        if resources.is_empty() {
            output += "    1\n";
        } else {
            for (i, ident) in resources.iter().enumerate() {
                if i == 0 {
                    output += &format!("    if string_id == {ident}::string_id() {{\n");
                } else {
                    output += &format!("    }} else if string_id == {ident}::string_id() {{\n");
                }

                output += &format!("        (val as *mut {ident}).write(Default::default());\n");
            }

            output += "    } else {\n";
            output += "        return 1;\n";
            output += "    }\n\n";
            output += "    0\n";
        }

        output += "}\n\n";

        output
    }

    fn gen_systems(&self) -> String {
        let mut output = String::new();

        output += &self.gen_system_fn_ffi();
        output += &self.gen_systems_len();
        output += &self.gen_system_is_once();
        output += &self.gen_system_fn();
        output += &self.gen_system_args_len();
        output += &self.gen_system_arg_type();
        output += &self.gen_system_arg_component();

        output += &self.gen_system_query_args_len();
        output += &self.gen_system_query_arg_type();
        output += &self.gen_system_query_arg_component();

        output
    }

    fn gen_system_fn_ffi(&self) -> String {
        let mut output = String::new();

        let gen_system_fn = &mut |system: &SystemInfo| {
            output += "unsafe extern \"C\" fn ";
            output += &system.ident;
            output += "_ffi(data: *mut *mut ::std::ffi::c_void) -> i32 {\n";
            output += "    ::std::panic::catch_unwind(|| {\n";
            output += "        ";
            output += &system.ident;
            output += "(\n";

            for (i, input) in system.inputs.iter().enumerate() {
                if let ArgType::Query { .. } = &input.arg_type {
                    output += &format!("            Query::new(*data.offset({i})),\n");
                } else {
                    output += "            &";

                    if input.mutable {
                        output += "mut ";
                    }

                    output += &format!("*(*data.offset({i}) as *");

                    if input.mutable {
                        output += "mut ";
                    } else {
                        output += "const ";
                    }

                    output += &input.ident;
                    output += "),\n";
                }
            }

            output += "        );\n";
            output += "    })\n";
            output += "    .map(|_| 0)\n";
            output += "    .unwrap_or(1)\n";
            output += "}\n\n";
        };

        for system in &self.systems {
            gen_system_fn(system);
        }

        output
    }

    fn gen_systems_len(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn systems_len() -> usize {\n";
        output += &format!("    {}\n", self.systems.len());
        output += "}\n\n";

        output
    }

    fn gen_system_is_once(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_is_once(system_index: usize) -> bool {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate() {
            output += &format!("        {i} => {},\n", system.is_once);
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_fn(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_fn(system_index: usize) -> unsafe extern \"C\" fn(*mut *mut ::std::ffi::c_void) -> i32 {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate() {
            output += &format!("        {i} => {}_ffi,\n", system.ident);
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_args_len(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_args_len(system_index: usize) -> usize {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate() {
            output += &format!("        {i} => {},\n", system.inputs.len());
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_arg_type(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_arg_type(system_index: usize, arg_index: usize) -> ArgType {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate() {
            output += &format!("        {i} => match arg_index {{\n");

            for (i, input) in system.inputs.iter().enumerate() {
                output += &format!("            {i} => ArgType::");
                output += match &input.arg_type {
                    ArgType::DataAccessDirect if input.mutable => "DataAccessMut,\n",
                    ArgType::DataAccessDirect => "DataAccessRef,\n",
                    ArgType::Query { .. } => "Query,\n",
                };
            }

            output += "            _ => ::std::process::abort(),\n";
            output += "        },\n";
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_arg_component(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_arg_component(system_index: usize, arg_index: usize) -> *const ::std::ffi::c_char {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate() {
            output += &format!("        {i} => match arg_index {{\n");

            for (i, input) in system
                .inputs
                .iter()
                .enumerate()
                .filter(|(_, input)| matches!(input.arg_type, ArgType::DataAccessDirect))
            {
                output += &format!(
                    "            {i} => {}::string_id().as_ptr(),\n",
                    input.ident
                );
            }

            output += "            _ => ::std::process::abort(),\n";
            output += "        },\n";
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_query_args_len(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_query_args_len(system_index: usize, arg_index: usize) -> usize {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate().filter(|(_, system)| {
            system
                .inputs
                .iter()
                .any(|input| matches!(input.arg_type, ArgType::Query { .. }))
        }) {
            output += &format!("        {i} => match arg_index {{\n");

            for (i, input) in system.inputs.iter().enumerate() {
                if let ArgType::Query { inputs } = &input.arg_type {
                    output += &format!("            {i} => {},\n", inputs.len());
                }
            }

            output += "            _ => ::std::process::abort(),\n";
            output += "        },\n";
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_query_arg_type(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_query_arg_type(\n";
        output += "    system_index: usize,\n";
        output += "    arg_index: usize,\n";
        output += "    query_index: usize,\n";
        output += ") -> ArgType {\n";
        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate().filter(|(_, system)| {
            system
                .inputs
                .iter()
                .any(|input| matches!(input.arg_type, ArgType::Query { .. }))
        }) {
            output += &format!("        {i} => match arg_index {{\n");

            for (i, input) in system.inputs.iter().enumerate() {
                if let ArgType::Query { inputs } = &input.arg_type {
                    output += &format!("            {i} => match query_index {{\n");

                    for (i, input) in inputs.iter().enumerate() {
                        output += &format!("                {i} => ArgType::DataAccess");
                        output += match input.mutable {
                            true => "Mut,\n",
                            false => "Ref,\n",
                        };
                    }

                    output += "                _ => ::std::process::abort(),\n";
                    output += "            },\n";
                }
            }

            output += "            _ => ::std::process::abort(),\n";
            output += "        },\n";
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_system_query_arg_component(&self) -> String {
        let mut output = String::new();

        output += "#[no_mangle]\n";
        output += "pub extern \"C\" fn system_query_arg_component(\n";
        output += "    system_index: usize,\n";
        output += "    arg_index: usize,\n";
        output += "    query_index: usize,\n";
        output += ") -> *const ::std::ffi::c_char {\n";

        // SAFETY: verify tuple layout

        for query_inputs in self.systems.iter().flat_map(|system| {
            system
                .inputs
                .iter()
                .filter_map(|input| match &input.arg_type {
                    ArgType::Query { inputs } if inputs.len() > 1 => Some(inputs),
                    _ => None,
                })
        }) {
            output += "    let layout_check: (";

            for input in query_inputs {
                output += if input.mutable { "*mut " } else { "*const " };
                output += &input.ident;
                output += ", ";
            }

            output += ") = (";

            for _ in query_inputs {
                output += "::std::ptr::null_mut(), ";
            }

            output += ");\n";

            for i in 1..query_inputs.len() {
                output += &format!("    assert_eq!(&layout_check.{} as *const _ as usize - &layout_check.{} as *const _ as usize, ::std::mem::size_of::<*const ::std::ffi::c_void>());\n", i, i - 1);
            }

            output += "\n";
        }

        output += "    match system_index {\n";

        for (i, system) in self.systems.iter().enumerate().filter(|(_, system)| {
            system
                .inputs
                .iter()
                .any(|input| matches!(input.arg_type, ArgType::Query { .. }))
        }) {
            output += &format!("        {i} => match arg_index {{\n");

            for (i, input) in system.inputs.iter().enumerate() {
                if let ArgType::Query { inputs } = &input.arg_type {
                    output += &format!("            {i} => match query_index {{\n");

                    for (i, input) in inputs.iter().enumerate() {
                        output += &format!(
                            "                {i} => {}::string_id().as_ptr(),\n",
                            input.ident
                        );
                    }

                    output += "                _ => ::std::process::abort(),\n";
                    output += "            },\n";
                }
            }

            output += "            _ => ::std::process::abort(),\n";
            output += "        },\n";
        }

        output += "        _ => ::std::process::abort(),\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }

    fn gen_callbacks(&self) -> String {
        let mut output = String::new();

        output += "#[repr(C)]\npub enum CallbackType {\n";
        output += "    QueryGetFn,\n";
        output += "    QueryGetMutFn,\n";
        output += "    QueryGetFirstFn,\n";
        output += "    QueryGetFirstMutFn,\n";
        output += "    QueryForEachFn,\n";
        output += "    QueryParForEachFn,\n";
        output += "}\n\n";

        output += "#[no_mangle]\npub unsafe extern \"C\" fn set_callback_fn(\n";
        output += "    callback_type: CallbackType,\n";
        output += "    callback: *const ::std::ffi::c_void\n";
        output += ") {\n";
        output += "    match callback_type {\n";
        output += "        CallbackType::QueryGetFn => {\n";
        output += "            _QUERY_GET_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "        CallbackType::QueryGetMutFn => {\n";
        output += "            _QUERY_GET_MUT_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "        CallbackType::QueryGetFirstFn => {\n";
        output += "            _QUERY_GET_FIRST_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "        CallbackType::QueryGetFirstMutFn => {\n";
        output += "            _QUERY_GET_FIRST_MUT_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "        CallbackType::QueryForEachFn => {\n";
        output += "            _QUERY_FOR_EACH_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "        CallbackType::QueryParForEachFn => {\n";
        output += "            _QUERY_PAR_FOR_EACH_FN = ::std::mem::transmute(callback);\n";
        output += "        }\n";
        output += "    }\n";
        output += "}\n\n";

        output
    }
}

fn gen_version() -> String {
    let mut output = String::new();

    output += "#[no_mangle]\n";
    output += "pub unsafe extern \"C\" fn arete_target_version() -> u32 {\n";
    output += "    ::arete_public::ENGINE_VERSION\n";
    output += "}\n\n";

    output
}
