//! Extracts the paths passed on the command line into samples/sprites/, using the resolver.

use unpack::ResourceSet;

fn main() {
    let rs = ResourceSet::open(&test_support::sample_path("packed"));
    let out = test_support::sample_path("sprites");
    std::fs::create_dir_all(&out).expect("creates the folder");
    for arg in std::env::args().skip(1) {
        match rs.read_with_source(&arg) {
            Some((bytes, source)) => {
                let leaf = arg.rsplit('/').next().unwrap().replace(' ', "_");
                std::fs::write(out.join(&leaf), &bytes).expect("writes");
                println!("{arg} -> {leaf} ({} bytes, from {source})", bytes.len());
            }
            None => println!("{arg}: not found"),
        }
    }
}
