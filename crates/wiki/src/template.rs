//! `{{name|arg|k=v}}` with nesting: arguments stay raw, `|` inside nested `{{…}}` and
//! `[[…]]` doesn't split.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub args: Vec<String>,
    pub named: BTreeMap<String, String>,
}

/// `s[at..]` starts with `{{`. Returns the template and the byte index after `}}`.
/// `None` if there's no template at `at`, if the closing is missing, or if `at` doesn't
/// fall on a character boundary: wikitext is external data and must not crash the process.
pub fn parse_template_at(s: &str, at: usize) -> Option<(Template, usize)> {
    if !s.get(at..)?.starts_with("{{") {
        return None;
    }
    let b = s.as_bytes();
    let mut i = at + 2;
    let mut depth_t = 0usize; // nested {{ }}
    let mut depth_l = 0usize; // nested [[ ]]
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    while i < b.len() {
        let rest = s.get(i..)?;
        if rest.starts_with("{{") {
            depth_t += 1;
            cur.push_str("{{");
            i += 2;
            continue;
        }
        if rest.starts_with("}}") {
            if depth_t == 0 {
                parts.push(std::mem::take(&mut cur));
                return Some((assemble(parts), i + 2));
            }
            depth_t -= 1;
            cur.push_str("}}");
            i += 2;
            continue;
        }
        if rest.starts_with("[[") {
            depth_l += 1;
            cur.push_str("[[");
            i += 2;
            continue;
        }
        if rest.starts_with("]]") {
            depth_l = depth_l.saturating_sub(1);
            cur.push_str("]]");
            i += 2;
            continue;
        }
        if b[i] == b'|' && depth_t == 0 && depth_l == 0 {
            parts.push(std::mem::take(&mut cur));
            i += 1;
            continue;
        }
        let ch = rest.chars().next()?;
        cur.push(ch);
        i += ch.len_utf8();
    }
    None
}

fn assemble(parts: Vec<String>) -> Template {
    let mut it = parts.into_iter();
    let name = it.next().unwrap_or_default().trim().to_lowercase();
    let mut args = Vec::new();
    let mut named = BTreeMap::new();
    for p in it {
        // `k=v` only if `k` doesn't contain `{{`/`[[`: an `=` inside a link isn't a name.
        match p.split_once('=') {
            Some((k, v)) if !k.contains("{{") && !k.contains("[[") && !k.trim().is_empty() => {
                named.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
            _ => args.push(p.trim().to_string()),
        }
    }
    Template { name, args, named }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Template {
        parse_template_at(s, 0).unwrap().0
    }

    #[test]
    fn name_and_positional_args() {
        let x = t("{{i|Little Baggy|Baggy}}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy", "Baggy"]);
        assert!(x.named.is_empty());
    }

    #[test]
    fn named_args_and_case() {
        let x = t("{{ I | Little Baggy | text = Baggy }}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy"]);
        assert_eq!(x.named.get("text").map(String::as_str), Some("Baggy"));
    }

    #[test]
    fn nested_templates_and_links_are_not_split() {
        let x = t("{{dlc+|a+}} Entrance to {{s|The Void}} [[a|b]] {{dlc-}}");
        // parse_template_at only reads the first template
        assert_eq!(x.name, "dlc+");
        assert_eq!(x.args, vec!["a+"]);
        let (y, end) = parse_template_at("{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail", 0).unwrap();
        assert_eq!(y.args, vec!["{{i|A|x=y}} and [[p|q]]"]);
        assert_eq!(y.named.get("dlc").map(String::as_str), Some("r"));
        assert_eq!(
            &"{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail"[end..],
            " tail"
        );
    }

    #[test]
    fn multiline_infobox() {
        let src = "{{infobox boss\n | dlc = a\n | id = 407\n | base hp = 6666\n}}\n'''Hush'''";
        let (x, end) = parse_template_at(src, 0).unwrap();
        assert_eq!(x.name, "infobox boss");
        assert_eq!(x.named.get("base hp").map(String::as_str), Some("6666"));
        assert_eq!(&src[end..], "\n'''Hush'''");
    }

    #[test]
    fn unterminated_returns_none() {
        assert!(parse_template_at("{{i|Breakfast", 0).is_none());
        assert!(parse_template_at("no template", 0).is_none());
    }

    #[test]
    fn parser_functions_keep_the_hash_name() {
        let x = t("{{#ev:youtube|dFwYucBWQ9k}}");
        assert_eq!(x.name, "#ev:youtube");
    }

    #[test]
    fn offset_past_the_end_or_inside_a_char_returns_none() {
        assert!(parse_template_at("{{i|x}}", 99).is_none());
        // `at` in the middle of a multibyte character: must not panic
        assert!(parse_template_at("é{{i|x}}", 1).is_none());
    }

    #[test]
    fn parses_from_a_later_offset() {
        let src = "testo {{i|Gimpy}} coda";
        let (x, end) = parse_template_at(src, 6).unwrap();
        assert_eq!(x.args, vec!["Gimpy"]);
        assert_eq!(&src[end..], " coda");
    }
}
