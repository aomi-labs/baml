use crate::{generated_types::RustEnum, package::CurrentRenderPackage};
use internal_baml_core::ir::Enum;

pub fn ir_enum_to_rust(enum_def: &Enum, _pkg: &CurrentRenderPackage) -> RustEnum {
    let values = enum_def
        .elem
        .values
        .iter()
        .map(|(name, _)| name.elem.0.clone())
        .collect();

    RustEnum {
        name: enum_def.elem.name.clone(),
        values,
    }
}
