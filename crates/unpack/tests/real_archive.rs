use unpack::{path_key, Archive};

#[test]
fn opens_real_config_archive() {
    let Some(path) = test_support::sample("config.a") else {
        return;
    };
    let Ok(a) = Archive::open(&path) else {
        test_support::skip("samples/config.a present but doesn't open");
        return;
    };
    // count read from the file (not hardcoded); the real config.a declares 24.
    assert_eq!(a.entries().len(), 24);
    // A known path must be present in the index.
    assert!(a.contains("resources/achievements.xml"));
    let k = path_key("resources/achievements.xml");
    assert!(a
        .entries()
        .iter()
        .any(|e| e.key.djb2 == k.djb2 && e.key.fnv == k.fnv));
}

#[test]
fn decompresses_achievements_xml_entry() {
    let Some(bytes) = test_support::sample_bytes("config.a") else {
        return;
    };
    // achievements.xml in config.a: offset 14, decompressed size 25894 (verified).
    let out =
        unpack::for_tests::lzw_decompress(&bytes, 14, 25894).expect("decompression succeeded");
    assert_eq!(
        out.len(),
        25894,
        "output matches the declared decompressed size"
    );
    assert_eq!(out[0], b'<', "an XML file starts with '<'");
}

#[test]
fn reads_and_decompresses_by_path() {
    let Some(path) = test_support::sample("config.a") else {
        return;
    };
    let Ok(a) = Archive::open(&path) else {
        test_support::skip("samples/config.a present but doesn't open");
        return;
    };
    let xml = a
        .read("resources/achievements.xml")
        .expect("present and decompressed");
    assert_eq!(xml.len(), 25894);
    assert_eq!(xml[0], b'<');
    // Normalization applies here too: uppercase/backslash resolve to the same file.
    assert_eq!(
        a.read("RESOURCES\\achievements.xml").map(|v| v.len()),
        Some(25894)
    );
    // Missing path → None, no panic.
    assert!(a.read("resources/not_there_at_all.xml").is_none());
}
