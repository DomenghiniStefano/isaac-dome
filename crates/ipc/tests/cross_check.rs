use core_save::{Kind, Save};
use ipc::{marks_matrix, Cell, BOSSES, CHARACTERS};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "20260905.rep+persistentgamedata1.dat";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Outcome of trying to call out to the Python reference. Distinguishes
/// "never started" (a legitimate skip: Python isn't installed on this
/// machine) from "started but failed" (the reference is broken: the independent
/// check isn't happening, and that must be flagged, not silenced).
enum ReferenceOutcome {
    Ready(HashMap<String, HashMap<String, u32>>),
    NotAvailable,
    Failed(String),
}

/// Whether `python` is a real interpreter. On Windows it's not enough to check that the
/// command runs at all: without Python installed, `python` is a **Microsoft Store
/// alias** that exists, launches, prints something like "run without arguments to
/// install" and exits with 49. Mistaken for a broken interpreter, it would fail the
/// test with "the reference is broken" when the reference isn't there at all. So we ask
/// the interpreter to say a word of our own choosing, and accept it only if it does.
fn python_answers() -> bool {
    Command::new("python")
        .args(["-c", "print('isaacdome')"])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("isaacdome"))
        .unwrap_or(false)
}

/// The Python reference's matrix: { boss: { character: value } }.
/// Combinations the reference doesn't locate simply aren't there.
fn reference_marks(sample: &Path) -> ReferenceOutcome {
    if !python_answers() {
        return ReferenceOutcome::NotAvailable;
    }
    let script = "import sys, json; sys.path.insert(0, 'reference'); \
                  from isaac_save import Save; from isaac_counters import marks; \
                  print(json.dumps(marks(Save(sys.argv[1]).u32s('counters'))))";
    let out = match Command::new("python")
        .current_dir(repo_root())
        .args(["-c", script, &sample.display().to_string()])
        .output()
    {
        Ok(out) => out,
        // The command didn't even start: Python isn't on this machine.
        Err(_) => return ReferenceOutcome::NotAvailable,
    };
    if !out.status.success() {
        return ReferenceOutcome::Failed(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    match serde_json::from_slice(&out.stdout) {
        Ok(reference) => ReferenceOutcome::Ready(reference),
        Err(e) => ReferenceOutcome::Failed(format!(
            "output isn't valid JSON ({e})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )),
    }
}

#[test]
fn rust_matrix_agrees_with_the_python_reference() {
    let Some(sample) = test_support::sample(SAMPLE) else {
        return;
    };
    let bytes = std::fs::read(&sample).expect("a present sample must be readable");
    let reference = match reference_marks(&sample) {
        ReferenceOutcome::Ready(r) => r,
        ReferenceOutcome::NotAvailable => {
            test_support::skip("python not available: the cross-check didn't run");
            return;
        }
        ReferenceOutcome::Failed(stderr) => {
            panic!(
                "the Python reference started but failed, the independent check isn't happening:\n{stderr}"
            );
        }
    };

    let counters = Save::parse(&bytes)
        .expect("the sample must be readable")
        .u32s(Kind::Counters)
        .expect("the counters section must be present");
    let matrix = marks_matrix(&counters);

    let compared = CHARACTERS
        .iter()
        .enumerate()
        .flat_map(|(c, &(name, _))| {
            BOSSES
                .iter()
                .enumerate()
                .map(move |(b, boss)| (c, name, b, *boss))
        })
        .filter(|&(c, name, b, boss)| {
            let expected = reference.get(boss).and_then(|row| row.get(name));
            let actual = matrix.characters[c].cells[b];
            match (expected, actual) {
                // The reference knows the cell: it must match.
                (Some(&v), Cell::Known { bits }) => {
                    assert_eq!(u32::from(bits), v, "{name} × {boss}");
                    true
                }
                (Some(&v), other) => {
                    panic!("{name} × {boss}: the reference says {v}, we say {other:?}")
                }
                // The reference doesn't know it: we must call it unknown too.
                (None, Cell::Unknown) => false,
                (None, other) => {
                    panic!("{name} × {boss}: the reference can't locate it, we say {other:?}")
                }
            }
        })
        .count();

    assert_eq!(
        compared, matrix.totals.readable,
        "all readable cells were compared"
    );
    assert_eq!(
        matrix.totals.unexpected, 0,
        "no suspicious value on a real save"
    );

    assert_eq!(matrix.totals.readable, 321);
    assert_eq!(
        matrix.totals.started, 93,
        "measured on the January 2025 sample"
    );
}
