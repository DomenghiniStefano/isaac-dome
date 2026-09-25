//! The game's stringtable: key -> text per language. It's XML, not binary.

use std::collections::HashMap;

use crate::diagnostics::{Diagnostic, Source};
use crate::text::Language;
use crate::xml::{self, Element};

pub struct Strings {
    /// The languages in the order of their declared indices, unknown ones skipped.
    languages: Vec<Language>,
    /// key -> texts, aligned to `languages` (`None` where the key has fewer).
    texts: HashMap<String, Vec<Option<String>>>,
}

impl Strings {
    pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Option<Strings> {
        let els = xml::read(bytes, Source::Strings, diagnostics)?;
        let slots = language_slots(&els, diagnostics);
        let languages: Vec<Language> = slots.iter().flatten().copied().collect();
        // Collected in file order, so a key declared twice is read as the last one.
        let texts = els
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name == "key")
            .filter_map(|(i, e)| {
                let key = e.attr("name")?;
                Some((key.to_string(), row_of(&els, i, &slots, &languages)))
            })
            .collect();
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

/// The language of the n-th `<string>` of every key, by declared index (`None` where the
/// language is one we don't know). Index 0 is "Key", not a language, and a declaration
/// without a usable index is none at all.
fn language_slots(els: &[Element], d: &mut Vec<Diagnostic>) -> Vec<Option<Language>> {
    let mut by_index: Vec<(usize, Option<Language>)> = els
        .iter()
        .filter(|e| e.name == "language")
        .filter_map(|e| declared_language(e, d))
        .collect();
    by_index.sort_by_key(|(i, _)| *i);
    by_index.into_iter().map(|(_, l)| l).collect()
}

/// One `<language>`: its index and what it is. An unknown name is diagnosed and keeps its
/// slot, so the languages after it do not slide into its place.
fn declared_language(e: &Element, d: &mut Vec<Diagnostic>) -> Option<(usize, Option<Language>)> {
    let index = e.attr("index")?.parse::<usize>().ok()?;
    let name = e.attr("name")?;
    if index == 0 {
        return None;
    }
    let lang = Language::from_name(name);
    if lang.is_none() {
        d.push(Diagnostic::UnknownLanguage {
            name: name.to_string(),
        });
    }
    Some((index, lang))
}

/// The texts of the key at `els[key]`, aligned to `languages`: the n-th `<string>` belongs
/// to the n-th slot, and one past the declared slots, or in an unknown one, is dropped.
fn row_of(
    els: &[Element],
    key: usize,
    slots: &[Option<Language>],
    languages: &[Language],
) -> Vec<Option<String>> {
    xml::children_named(els, key, "string")
        .into_iter()
        .zip(slots)
        .filter_map(|(s, slot)| {
            let lang = (*slot)?;
            // `lang` can't fail to be in `languages`: it comes from there. If it ever were,
            // the string is dropped instead of ending up in slot 0 (English), which would
            // be wrong data passed off as good.
            let pos = languages.iter().position(|l| *l == lang)?;
            Some((pos, s.text.clone()))
        })
        .fold(vec![None; languages.len()], |mut row, (pos, text)| {
            row[pos] = Some(text);
            row
        })
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

    // Languages declared out of index order, one without a usable index, a key without a
    // name, a key declared twice, a key with more strings than languages, and a `<string>`
    // that is not the key's direct child.
    const EDGES: &[u8] = br#"<stringtable><languages>
      <language id="3" index="2" name="French"/>
      <language id="21" index="0" name="Key"/>
      <language id="4" index="x" name="German"/>
      <language id="5" name="Spanish"/>
      <language id="0" index="1" name="English"/>
    </languages>
    <category name="Items">
      <key><string>orphan</string></key>
      <key name="TWICE"><string>first</string></key>
      <key name="TWICE"><string>second</string><string>deuxieme</string></key>
      <key name="LONG"><string>a</string><string>b</string><string>c</string></key>
      <key name="NESTED"><wrap><string>deep</string></wrap><string>top</string></key>
    </category></stringtable>"#;

    #[test]
    fn languages_follow_the_declared_index_and_unusable_declarations_are_ignored() {
        let mut d = Vec::new();
        let s = Strings::parse(EDGES, &mut d).expect("valid table");
        assert_eq!(s.languages(), &[Language::English, Language::French]);
        assert!(
            d.is_empty(),
            "a declaration without an index is no language: {d:?}"
        );
    }

    #[test]
    fn a_repeated_key_is_read_as_the_last_one_and_extra_strings_are_dropped() {
        let mut d = Vec::new();
        let s = Strings::parse(EDGES, &mut d).expect("valid table");
        assert_eq!(s.get("TWICE", Language::English), Some("second"));
        assert_eq!(s.get("TWICE", Language::French), Some("deuxieme"));
        assert_eq!(s.get("LONG", Language::French), Some("b"));
        assert_eq!(
            s.get("NESTED", Language::English),
            Some("top"),
            "only the key's direct children are its strings"
        );
        assert_eq!(s.get("", Language::English), None);
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
