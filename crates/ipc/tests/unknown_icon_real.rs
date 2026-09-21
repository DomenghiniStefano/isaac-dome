//! The stand-in picture, against the installed game.
//!
//! `ipc::UNKNOWN_SPRITE` is a path written into this repo, and a path written down is the
//! kind of thing that goes quiet when a patch moves it: the protocol would answer 404, the
//! interface would draw the empty square it drew before B69, and no test would say a word.
//! So the constant is checked against the archives it names.

use unpack::ResourceSet;

#[test]
fn the_question_mark_is_where_the_constant_says_it_is() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let rs = ResourceSet::open(&packed);
    let Some(bytes) = rs.read(ipc::UNKNOWN_SPRITE) else {
        panic!(
            "{} is not in the archives: the stand-in would be a 404 and nothing would say so",
            ipc::UNKNOWN_SPRITE
        );
    };
    // A PNG and not an empty entry: the protocol hands these bytes to the webview as
    // `image/png`, and an entry that reads as zero bytes draws the same nothing as a
    // missing file.
    assert_eq!(&bytes[..4], &[0x89, b'P', b'N', b'G'], "not a PNG at all");
    assert!(bytes.len() > 100, "{} bytes is not a picture", bytes.len());
}
