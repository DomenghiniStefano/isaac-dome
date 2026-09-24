//! Extracts the paths passed on the command line into samples/sprites/, using the resolver.

use unpack::ResourceSet;

/// The repo's `samples/` folder, through `test-support`: `packed` is a junction to the
/// game's folder there.
fn samples(name: &str) -> std::path::PathBuf {
    test_support::samples_dir().join(name)
}

fn main() {
    let rs = ResourceSet::open(&samples("packed"));
    let out = samples("sprites");
    std::fs::create_dir_all(&out).expect("creates the folder");
    for arg in std::env::args().skip(1) {
        match rs.read_with_source(&arg) {
            Some((bytes, da)) => {
                let leaf = arg.rsplit('/').next().unwrap().replace(' ', "_");
                std::fs::write(out.join(&leaf), &bytes).expect("writes");
                println!("{arg} -> {leaf} ({} bytes, from {da})", bytes.len());
            }
            None => println!("{arg}: not found"),
        }
    }
}
