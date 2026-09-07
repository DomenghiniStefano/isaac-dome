use unpack::path_key;

#[test]
fn djb2_and_fnv_known_values() {
    let k = path_key("a");
    assert_eq!(k.djb2, 177670);
    assert_eq!(k.fnv, 3362534589);
}

#[test]
fn hashes_a_real_resource_path() {
    // Values verified against config.a's entry.
    let k = path_key("resources/achievements.xml");
    assert_eq!(k.djb2, 3628659754);
    assert_eq!(k.fnv, 3912015619);
}

#[test]
fn normalizes_case_and_separators() {
    // Backslash and uppercase must reduce to the same key.
    assert_eq!(
        path_key("RESOURCES\\Achievements.XML"),
        path_key("resources/achievements.xml")
    );
}
