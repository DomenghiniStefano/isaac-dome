//! Final check: tries to extract EVERY path in filelist.txt from EVERY archive
//! that contains it, and reports per compression mode.

use unpack::{Archive, CompressionMode};

/// The repo's `samples/` folder, through `test-support`: `packed` is a junction to the
/// game's folder there.
fn samples(name: &str) -> std::path::PathBuf {
    test_support::samples_dir().join(name)
}

fn main() {
    let packed = samples("packed");
    let Ok(list) = std::fs::read_to_string(samples("filelist.txt")) else {
        println!("samples/filelist.txt missing: the path dictionary is needed");
        return;
    };
    let paths: Vec<&str> = list.lines().filter(|l| !l.is_empty()).collect();
    println!("paths in the filelist: {}\n", paths.len());

    let mut tot_ok = 0usize;
    let mut tot_ko = 0usize;
    for name in [
        "config.a",
        "fonts.a",
        "animations.a",
        "graphics.a",
        "music.a",
        "afterbirth.a",
        "afterbirthp.a",
        "repentance.a",
    ] {
        let Ok(a) = Archive::open(&packed.join(name)) else {
            continue;
        };
        let (mut ok, mut ko, mut bytes) = (0usize, 0usize, 0u64);
        for p in &paths {
            if !a.contains(p) {
                continue;
            }
            match a.read(p) {
                Some(v) => {
                    ok += 1;
                    bytes += v.len() as u64;
                }
                None => ko += 1,
            }
        }
        let modo = match a.mode() {
            CompressionMode::Bogocrypt1 => "Bogocrypt1",
            CompressionMode::Lzw => "LZW",
            CompressionMode::MiniZ => "MiniZ",
            CompressionMode::Bogocrypt2 => "Bogocrypt2",
            CompressionMode::Unknown(_) => "unknown",
        };
        println!(
            "{name:<16} {modo:<11} index={:<6} extracted={ok:<6} failed={ko:<5} ({:.1} MB)",
            a.entries().len(),
            bytes as f64 / 1_048_576.0
        );
        tot_ok += ok;
        tot_ko += ko;
    }
    println!("\nTOTAL extracted={tot_ok} failed={tot_ko}");
}
