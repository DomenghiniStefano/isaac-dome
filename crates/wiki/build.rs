//! Con la feature `embedded` comprime `dataset/wiki.json` in `$OUT_DIR/wiki.json.deflate`, il
//! blob che `Dataset::embedded` incorpora nel binario: il JSON leggibile pesa decine di
//! megabyte, il deflate una frazione. Un file mancante è un errore di build, con il percorso
//! atteso nel messaggio. Senza la feature (il tool `wiki-snapshot`, che il file lo produce)
//! non fa niente.

use std::path::Path;

/// Il livello massimo di `miniz_oxide`: si comprime una volta per build, si legge a ogni avvio.
const LEVEL: u8 = 10;

fn main() {
    println!("cargo:rerun-if-changed=../../dataset/wiki.json");
    if std::env::var_os("CARGO_FEATURE_EMBEDDED").is_none() {
        return;
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dataset/wiki.json");
    let bytes = std::fs::read(&source).unwrap_or_else(|e| {
        panic!(
            "dataset/wiki.json non trovato in {} ({e}): la feature `embedded` del crate `wiki` \
             incorpora il dataset derivato, che sta nel repo; per rigenerarlo `pnpm wiki:build` \
             (il tool compila senza la feature)",
            source.display()
        )
    });
    let deflated = miniz_oxide::deflate::compress_to_vec(&bytes, LEVEL);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR è impostata da cargo");
    let target = Path::new(&out_dir).join("wiki.json.deflate");
    std::fs::write(&target, deflated)
        .unwrap_or_else(|e| panic!("scrittura di {}: {e}", target.display()));
}
