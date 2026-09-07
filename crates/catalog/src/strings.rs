//! The game's stringtable: key -> text per language. It's XML, not binary.

use std::collections::HashMap;

use crate::diagnostics::{Diagnostic, Source};
use crate::text::Language;
use crate::xml::{elements, Element};

pub struct Strings {
    /// The languages in the order of their declared indices, unknown ones skipped.
    languages: Vec<Language>,
    /// key -> texts, aligned to `languages` (`None` where the key has fewer).
    texts: HashMap<String, Vec<Option<String>>>,
}

impl Strings {
    pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Option<Strings> {
        let els = match elements(bytes) {
            Ok(els) => els,
            Err(_) => {
                diagnostics.push(Diagnostic::SourceUnreadable {
                    source: Source::Strings,
                });
                return None;
            }
        };

        // Declared index -> language. Index 0 is "Key", not a language.
        let mut by_index: Vec<(usize, Option<Language>)> = Vec::new();
        for e in els.iter().filter(|e| e.name == "language") {
            let (Some(index), Some(name)) = (e.attr("index"), e.attr("name")) else {
                continue;
            };
            let Ok(index) = index.parse::<usize>() else {
                continue;
            };
            if index == 0 {
                continue;
            }
            let lang = Language::from_name(name);
            if lang.is_none() {
                diagnostics.push(Diagnostic::UnknownLanguage {
                    name: name.to_string(),
                });
            }
            by_index.push((index, lang));
        }
        by_index.sort_by_key(|(i, _)| *i);
        // Position of the n-th <string> -> language (or None if unknown).
        let slots: Vec<Option<Language>> = by_index.iter().map(|(_, l)| *l).collect();
        let languages: Vec<Language> = slots.iter().flatten().copied().collect();

        let mut texts = HashMap::new();
        let mut i = 0;
        while i < els.len() {
            if els[i].name == "key" {
                let Some(key) = els[i].attr("name") else {
                    i += 1;
                    continue;
                };
                let strings = children_named(&els, i, "string");
                let mut row = vec![None; languages.len()];
                for (n, s) in strings.iter().enumerate() {
                    // The n-th <string> belongs to the language at index n+1.
                    let Some(Some(lang)) = slots.get(n) else {
                        continue;
                    };
                    // `lang` can't fail to be in `languages`: it comes from there. If it
                    // ever were, the string is dropped instead of ending up in slot 0
                    // (English), which would be wrong data passed off as good.
                    let Some(pos) = languages.iter().position(|l| l == lang) else {
                        continue;
                    };
                    row[pos] = Some(s.text.clone());
                }
                texts.insert(key.to_string(), row);
            }
            i += 1;
        }
        Some(Strings { languages, texts })
    }

    pub fn get(&self, key: &str, lang: Language) -> Option<&str> {
        let pos = self.languages.iter().position(|l| *l == lang)?;
        self.texts.get(key)?.get(pos)?.as_deref()
    }

    pub fn languages(&self) -> &[Language] {
        &self.languages
    }
}

/// The direct children of element `parent` with that name: those that follow at depth
/// +1, up to the next element at depth <= the parent's.
pub(crate) fn children_named<'a>(
    els: &'a [Element],
    parent: usize,
    name: &str,
) -> Vec<&'a Element> {
    let depth = els[parent].depth;
    els[parent + 1..]
        .iter()
        .take_while(|e| e.depth > depth)
        .filter(|e| e.depth == depth + 1 && e.name == name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Language;

    const TABLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<stringtable>
  <info version="1.0" />
  <languages>
    <language id="21" index="0" name="Key"/>
    <language id="0" index="1" name="English"/>
    <language id="3" index="2" name="French"/>
    <language id="99" index="3" name="Klingon"/>
  </languages>
  <category name="Items">
    <key name="THE_SAD_ONION_NAME">
      <string>The Sad Onion</string>
      <string>Oignon Triste</string>
      <string>nuqneH</string>
    </key>
    <key name="ONLY_ENGLISH">
      <string>Alone</string>
    </key>
  </category>
</stringtable>"#;

    #[test]
    fn strings_follow_the_declared_language_order_not_a_fixed_one() {
        let mut d = Vec::new();
        let s = Strings::parse(TABLE, &mut d).expect("valid table");
        assert_eq!(
            s.get("THE_SAD_ONION_NAME", Language::English),
            Some("The Sad Onion")
        );
        assert_eq!(
            s.get("THE_SAD_ONION_NAME", Language::French),
            Some("Oignon Triste")
        );
        assert_eq!(s.get("THE_SAD_ONION_NAME", Language::German), None);
        assert_eq!(s.get("NOPE", Language::English), None);
    }

    #[test]
    fn an_unknown_language_is_diagnosed_and_its_strings_ignored() {
        let mut d = Vec::new();
        let s = Strings::parse(TABLE, &mut d).unwrap();
        assert!(d.contains(&Diagnostic::UnknownLanguage {
            name: "Klingon".to_string()
        }));
        assert_eq!(s.languages(), &[Language::English, Language::French]);
    }

    #[test]
    fn a_key_with_fewer_strings_than_languages_is_fine() {
        let mut d = Vec::new();
        let s = Strings::parse(TABLE, &mut d).unwrap();
        assert_eq!(s.get("ONLY_ENGLISH", Language::English), Some("Alone"));
        assert_eq!(s.get("ONLY_ENGLISH", Language::French), None);
    }

    // Klingon is in the middle (index 2), not at the end like in TABLE: checks that
    // French, right after it, stays aligned to its own slot instead of sliding into
    // the English slot (what the removed `unwrap_or(0)` would have produced).
    const LANGUAGE_UNKNOWN_IN_THE_MIDDLE: &[u8] = br#"<stringtable><languages>
      <language id="21" index="0" name="Key"/>
      <language id="0" index="1" name="English"/>
      <language id="99" index="2" name="Klingon"/>
      <language id="3" index="3" name="French"/>
    </languages>
    <category name="Items"><key name="THE_SAD_ONION_NAME">
      <string>The Sad Onion</string>
      <string>nuqneH</string>
      <string>Oignon Triste</string>
    </key></category></stringtable>"#;

    #[test]
    fn an_unknown_language_in_the_middle_of_the_indices_does_not_shift_the_ones_after_it() {
        let mut d = Vec::new();
        let s = Strings::parse(LANGUAGE_UNKNOWN_IN_THE_MIDDLE, &mut d).unwrap();
        assert!(d.contains(&Diagnostic::UnknownLanguage {
            name: "Klingon".to_string()
        }));
        assert_eq!(s.languages(), &[Language::English, Language::French]);
        assert_eq!(
            s.get("THE_SAD_ONION_NAME", Language::English),
            Some("The Sad Onion")
        );
        assert_eq!(
            s.get("THE_SAD_ONION_NAME", Language::French),
            Some("Oignon Triste"),
            "French must not end up in English's slot"
        );
    }

    #[test]
    fn junk_yields_none_and_a_diagnostic() {
        let mut d = Vec::new();
        assert!(Strings::parse(b"<stringtable><key></stringtable>", &mut d).is_none());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Strings
            }]
        );
    }
}
