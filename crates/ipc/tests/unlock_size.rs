//! The size of the `unlock` payload, which is the whole point of C2.
//!
//! With the icons embedded as base64 this was **about 7 MB** on a real profile — 94% of it
//! images, measured on the design package's own excerpt. A row now carries a link that the
//! app's protocol handler serves, and this test is what stops anyone from putting the
//! pictures back: a regression here isn't a wrong value, it's a screen that takes seconds
//! to open.

use catalog::Catalog;
use core_save::{Kind, Save};
use unpack::ResourceSet;

/// A generous ceiling on purpose. The real figure is far below it, and pinning the exact
/// byte count would make this test fail every time the game adds an achievement — which is
/// noise, not a defect. What it forbids is the order of magnitude coming back.
const CEILING: usize = 1_000_000;

/// The era is in the name, as the samples are: the numbers below belong to this profile.
const SAMPLE: &str = "20260908.rep+persistentgamedata1.dat";

fn real() -> Option<(Catalog, Save)> {
    let packed = test_support::packed_dir()?;
    let path = test_support::sample(SAMPLE)?;
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    match Save::open(&path) {
        Ok(s) => Some((c, s)),
        Err(_) => {
            // Present but unreadable is declared, never a silent skip.
            test_support::skip("the sample exists but doesn't read");
            None
        }
    }
}

#[test]
fn the_unlock_payload_carries_links_and_stays_small() {
    let Some((c, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let view = ipc::unlock_view(Some(&c), Some(&flags), None, None, |r| {
        Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path()))
    });
    let json = serde_json::to_string(&view).expect("serializes");

    assert!(
        json.len() < CEILING,
        "unlock payload is {} bytes, over the {CEILING} ceiling",
        json.len()
    );
    // The ceiling alone would pass on a payload that simply had no icons at all. This is
    // the half that says the icons are still *there*, just not as pictures.
    assert!(
        !json.contains("data:image"),
        "an icon travelled as an embedded image, which is what C2 removed"
    );
    assert!(
        json.contains("isaac://achievement/"),
        "no icon link in the payload at all"
    );
}
