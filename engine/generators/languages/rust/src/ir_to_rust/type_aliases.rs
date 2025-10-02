use internal_baml_core::ir::TypeAlias;

use crate::{generated_types::TypeAliasRust, package::CurrentRenderPackage};

use super::{stream_type_to_rust, type_to_rust};

pub fn ir_type_alias_to_rust<'a>(
    alias: &TypeAlias,
    pkg: &'a CurrentRenderPackage,
) -> TypeAliasRust<'a> {
    TypeAliasRust {
        name: alias.elem.name.clone(),
        type_: type_to_rust(
            &alias.elem.r#type.elem.to_non_streaming_type(pkg.lookup()),
            pkg.lookup(),
        ),
        docstring: alias
            .elem
            .docstring
            .clone()
            .map(|docstring| docstring.0.clone()),
        pkg,
    }
}

pub fn ir_type_alias_to_rust_stream<'a>(
    alias: &TypeAlias,
    pkg: &'a CurrentRenderPackage,
) -> TypeAliasRust<'a> {
    let partialized = alias.elem.r#type.elem.to_streaming_type(pkg.lookup());
    TypeAliasRust {
        name: alias.elem.name.clone(),
        type_: stream_type_to_rust(&partialized, pkg.lookup()),
        docstring: alias
            .elem
            .docstring
            .clone()
            .map(|docstring| docstring.0.clone()),
        pkg,
    }
}
