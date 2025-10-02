use crate::{
    generated_types::{RustClass, RustField},
    package::CurrentRenderPackage,
    r#type::SerializeType,
    utils::{safe_rust_identifier, to_snake_case},
};
use internal_baml_core::ir::Class;

pub fn ir_class_to_rust(class: &Class, pkg: &CurrentRenderPackage) -> RustClass {
    let fields = class
        .elem
        .static_fields
        .iter()
        .map(|field| {
            let field_type = &field.elem.r#type.elem;
            let mut rust_type = crate::ir_to_rust::type_to_rust(
                &field_type.to_non_streaming_type(pkg.lookup()),
                pkg.lookup(),
            );
            if rust_type.is_class_named(&class.elem.name) {
                rust_type.make_boxed();
            }
            let mut rust_type_string = rust_type.serialize_type(pkg);
            rust_type_string = apply_boxing_if_recursive(rust_type_string, &class.elem.name);

            let field_name = to_snake_case(&field.elem.name);
            RustField {
                name: safe_rust_identifier(&field_name),
                original_name: field.elem.name.clone(),
                rust_type: rust_type_string,
                optional: rust_type.meta().is_optional(),
            }
        })
        .collect();

    RustClass {
        name: class.elem.name.clone(),
        fields,
    }
}

fn apply_boxing_if_recursive(type_str: String, class_name: &str) -> String {
    if let Some(inner) = type_str
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        if inner.ends_with(&format!("::{}", class_name)) && !inner.starts_with("Box<") {
            println!("boxing optional recursive field for {}", class_name);
            return format!("Option<Box<{}>>", inner);
        }
    }

    if type_str.ends_with(&format!("::{}", class_name)) && !type_str.starts_with("Box<") {
        println!("boxing direct recursive field for {}", class_name);
        return format!("Box<{}>", type_str);
    }

    type_str
}
