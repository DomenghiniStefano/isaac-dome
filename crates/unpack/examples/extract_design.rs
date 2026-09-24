//! Extracts a set of real sprites for design work, giving DLCs precedence.
//! Also verifies the IHDR header: a valid PNG declares its width and height.

use unpack::Archive;

/// The repo's `samples/` folder, through `test-support`: `packed` is a junction to the
/// game's folder there.
fn samples(name: &str) -> std::path::PathBuf {
    test_support::samples_dir().join(name)
}

fn ihdr(png: &[u8]) -> Option<(u32, u32)> {
    if png.get(0..8)? != [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a] {
        return None;
    }
    if png.get(12..16)? != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(png.get(16..20)?.try_into().ok()?);
    let h = u32::from_be_bytes(png.get(20..24)?.try_into().ok()?);
    Some((w, h))
}

fn main() {
    let packed = samples("packed");
    // Precedence: the most recent DLC wins.
    let archives = [
        "repentance.a",
        "afterbirthp.a",
        "afterbirth.a",
        "graphics.a",
    ];
    let opened: Vec<_> = archives
        .iter()
        .filter_map(|n| Archive::open(&packed.join(n)).ok().map(|a| (*n, a)))
        .collect();

    let wanted = [
        "resources/gfx/ui/completion_widget.png",
        "resources/gfx/ui/boss/portrait_102.0_isaac.png",
        "resources/gfx/ui/boss/portrait_102.1_bluebaby.png",
        "resources/gfx/ui/boss/portrait_273.0_thelamb.png",
        "resources/gfx/ui/boss/portrait_274.0_megasatan.png",
        "resources/gfx/ui/achievement/achievement_180_purity.png",
        "resources/gfx/items/collectibles/collectibles_001_thesadonion.png",
        "resources/gfx/items/collectibles/collectibles_002_theinnereye.png",
        "resources/gfx/items/trinkets/trinket_001_swallowedpenny.png",
        "resources/gfx/characters/costumes/character_001_isaac.png",
    ];

    let out = samples("sprites");
    std::fs::create_dir_all(&out).expect("creates the folder");

    let (mut ok, mut ko) = (0, 0);
    for w in wanted {
        let mut done = false;
        for (name, a) in &opened {
            let Some(bytes) = a.read(w) else { continue };
            let leaf = w.rsplit('/').next().unwrap();
            match ihdr(&bytes) {
                Some((iw, ih)) => {
                    std::fs::write(out.join(leaf), &bytes).expect("writes");
                    println!("{leaf:<44} {iw}x{ih}  {} bytes  from {name}", bytes.len());
                    ok += 1;
                }
                None => {
                    println!("{leaf:<44} INVALID PNG from {name}");
                    ko += 1;
                }
            }
            done = true;
            break;
        }
        if !done {
            println!("{w}: absent from all archives");
        }
    }
    println!("\nvalid={ok} invalid={ko}");
}
