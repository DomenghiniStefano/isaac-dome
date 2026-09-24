//! How many entries of each archive we manage to NAME (and hence extract),
//! trying the filelist with both known roots.

use std::collections::HashSet;

use unpack::Archive;

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

    // The filelist is rooted at "resources/"; Repentance uses "resources-dlc3/".
    let mut paths: HashSet<String> = HashSet::new();
    for l in list.lines().filter(|l| !l.is_empty()) {
        paths.insert(l.to_string());
        if let Some(rest) = l.strip_prefix("resources/") {
            paths.insert(format!("resources-dlc3/{rest}"));
        }
    }
    println!(
        "candidate paths: {} (filelist + dlc3 variant)\n",
        paths.len()
    );

    let mut tot_idx = 0usize;
    let mut tot_named = 0usize;
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
        let named = paths.iter().filter(|p| a.contains(p)).count();
        let idx = a.entries().len();
        println!(
            "{name:<16} index={idx:<6} named={named:<6} ({:.0}%)",
            named as f64 / idx as f64 * 100.0
        );
        tot_idx += idx;
        tot_named += named;
    }
    println!(
        "\nTOTAL index={tot_idx} named={tot_named} ({:.1}%) — missing {}",
        tot_named as f64 / tot_idx as f64 * 100.0,
        tot_idx - tot_named
    );
}
