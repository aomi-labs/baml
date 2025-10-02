/// Utility functions for Rust code generation

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() && !result.is_empty() {
            if let Some(&next_char) = chars.peek() {
                if next_char.is_lowercase() {
                    result.push('_');
                }
            }
        }
        // Insert underscore between a digit and a following alphabetic character
        if c.is_ascii_digit() {
            result.push(c);
            if let Some(&next_char) = chars.peek() {
                if next_char.is_alphabetic() {
                    result.push('_');
                }
            }
            continue;
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }

    result
}

pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if !c.is_alphanumeric() {
            capitalize_next = true;
            continue;
        }

        if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            // Lowercase for consistency, digits remain unchanged
            if c.is_alphabetic() {
                result.push(c.to_lowercase().next().unwrap_or(c));
            } else {
                result.push(c);
            }
        }
    }

    result
}

pub fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

pub fn safe_rust_identifier(s: &str) -> String {
    if is_rust_keyword(s) {
        format!("r#{}", s)
    } else {
        s.to_string()
    }
}
