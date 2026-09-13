//! The TypeScript contract, assembled from the types that cross the boundary.
//!
//! `ts-rs` renders a fieldless enum as a union of string literals. Frontend rule 5 forbids
//! literal unions — every component compares against a symbol (`MissingReason.SteamNotFound`)
//! — so each one is rewritten into the `const … as const` pair the repo already spells by
//! hand. **By form and not by a list of names**: rule 5 admits no exception, so there is no
//! union that should survive and nothing for a list to forget.

/// `steamNotFound` → `SteamNotFound`, `rep_plus` → `RepPlus`.
pub fn pascal_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut upper = true;
    for c in value.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The string literals of a union, or `None` if any member is something else.
fn string_members(body: &str) -> Option<Vec<&str>> {
    let mut members = Vec::new();
    for part in body.split('|') {
        let part = part.trim();
        let inner = part.strip_prefix('"')?.strip_suffix('"')?;
        if inner.contains('"') {
            return None;
        }
        members.push(inner);
    }
    (!members.is_empty()).then_some(members)
}

/// Rewrite every union of string literals in `file` into the `const … as const` pair.
pub fn to_const_enums(file: &str) -> String {
    let mut out = String::with_capacity(file.len());
    for (i, line) in file.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        match rewrite_line(line) {
            Some(pair) => out.push_str(&pair),
            None => out.push_str(line),
        }
    }
    out
}

fn rewrite_line(line: &str) -> Option<String> {
    let rest = line.strip_prefix("export type ")?;
    let (name, body) = rest.split_once(" = ")?;
    let body = body.strip_suffix(';')?;
    let members = string_members(body)?;

    let mut pair = format!("export const {name} = {{\n");
    for m in &members {
        pair.push_str(&format!("  {}: \"{m}\",\n", pascal_case(m)));
    }
    pair.push_str("} as const;\n");
    pair.push_str(&format!(
        "export type {name} = (typeof {name})[keyof typeof {name}];"
    ));
    Some(pair)
}
