use unpack::{extract_subset, path_key, Archive};

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
    assert!(a.read("resources/non_esiste_affatto.xml").is_none());
}

#[test]
fn extract_subset_writes_wanted_files_and_reports_missing() {
    let Some(path) = test_support::sample("config.a") else {
        return;
    };
    let Ok(a) = Archive::open(&path) else {
        test_support::skip("samples/config.a present but doesn't open");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let report = extract_subset(
        &a,
        &["resources/achievements.xml", "resources/non_esiste.xml"],
        tmp.path(),
    );

    assert_eq!(report.extracted.len(), 1);
    assert_eq!(report.missing, vec!["resources/non_esiste.xml".to_string()]);
    assert!(report.diagnostics.is_empty());

    let written = tmp.path().join("resources/achievements.xml");
    assert!(written.exists());
    let content = std::fs::read(&written).unwrap();
    assert_eq!(content, a.read("resources/achievements.xml").unwrap());
}

#[test]
fn extract_subset_refuses_paths_escaping_cache() {
    let Some(path) = test_support::sample("config.a") else {
        return;
    };
    let Ok(a) = Archive::open(&path) else {
        test_support::skip("samples/config.a present but doesn't open");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    // Paths with '..', POSIX absolute, Windows drive: all of them must end up in missing,
    // and no file must show up outside the tmp directory.
    let dangerous = [
        "../evasione.xml",
        "C:\\Windows\\evasione.xml",
        "/etc/evasione.xml",
    ];
    let report = extract_subset(&a, &dangerous, tmp.path());

    // No file extracted.
    assert!(
        report.extracted.is_empty(),
        "no file must be extracted from dangerous paths"
    );
    // All of them end up in missing (not in diagnostics).
    assert_eq!(
        report.missing.len(),
        dangerous.len(),
        "all dangerous paths must end up in missing"
    );
    // No file appeared in tmp's parent directory.
    let parent = tmp.path().parent().unwrap();
    assert!(
        !parent.join("evasione.xml").exists(),
        "evasione.xml must not be written outside the cache"
    );
    // The absolute Windows/POSIX paths must not have been created.
    assert!(
        !std::path::Path::new("C:\\Windows\\evasione.xml").exists(),
        "the absolute Windows path must not have been created"
    );
    assert!(
        !std::path::Path::new("/etc/evasione.xml").exists(),
        "the absolute POSIX path must not have been created"
    );
}
