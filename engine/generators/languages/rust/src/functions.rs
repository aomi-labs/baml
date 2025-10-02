use crate::package::CurrentRenderPackage;
use crate::r#type::{SerializeType, TypeRust};
use askama::Template;

mod filters {
    use crate::utils::to_snake_case;

    pub fn snake_case(s: &str, _args: &dyn askama::Values) -> askama::Result<String> {
        Ok(to_snake_case(s))
    }
}

#[derive(Debug, Clone)]
pub struct FunctionRust {
    pub documentation: Option<String>,
    pub name: String,
    pub args: Vec<(String, TypeRust)>, // (name, type)
    pub return_type: TypeRust,
    pub stream_return_type: TypeRust,
}

/// Template for the complete functions module
#[derive(askama::Template)]
#[template(path = "client.rs.j2", escape = "none")]
pub struct RustFunctions<'a> {
    functions: &'a [SingleFunctionRust<'a>],
}

/// Individual function template
#[derive(askama::Template, Clone)]
#[template(path = "function.rs.j2", escape = "none")]
pub struct SingleFunctionRust<'a> {
    pub documentation: Option<String>,
    pub name: String,
    pub args: Vec<(String, TypeRust)>,
    pub return_type: TypeRust,
    pub stream_return_type: TypeRust,
    pub pkg: &'a CurrentRenderPackage,
}

pub fn render_functions(
    functions: &[FunctionRust],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    use askama::Template;

    let single_functions: Vec<SingleFunctionRust> = functions
        .iter()
        .map(|f| SingleFunctionRust {
            documentation: f.documentation.clone(),
            name: f.name.clone(),
            args: f.args.clone(),
            return_type: f.return_type.clone(),
            stream_return_type: f.stream_return_type.clone(),
            pkg,
        })
        .collect();

    RustFunctions {
        functions: &single_functions,
    }
    .render()
}

#[derive(Debug)]
struct SourceFileEntry {
    path_literal: String,
    contents: String,
}

#[derive(askama::Template)]
#[template(path = "source_map.rs.j2", escape = "none")]
struct SourceMapTemplate<'a> {
    files: &'a [SourceFileEntry],
}

pub fn render_source_files(file_map: Vec<(String, String)>) -> Result<String, askama::Error> {
    let mut files = file_map
        .into_iter()
        .map(|(raw_path, raw_contents)| {
            let mut path: String =
                serde_json::from_str(&raw_path).map_err(|e| askama::Error::Custom(Box::new(e)))?;
            path = path.replace('\\', "/");
            if let Some(stripped) = path.strip_prefix("./") {
                path = stripped.to_string();
            }
            if !path.starts_with("baml_src/") {
                path = format!("baml_src/{path}");
            }
            let contents: String = serde_json::from_str(&raw_contents)
                .map_err(|e| askama::Error::Custom(Box::new(e)))?;
            let path_literal =
                serde_json::to_string(&path).map_err(|e| askama::Error::Custom(Box::new(e)))?;

            Ok(SourceFileEntry {
                path_literal,
                contents,
            })
        })
        .collect::<Result<Vec<_>, askama::Error>>()?;

    files.sort_by(|a, b| a.path_literal.cmp(&b.path_literal));

    SourceMapTemplate { files: &files }.render()
}
