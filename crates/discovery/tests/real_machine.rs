use discovery::{discover, Options};

/// End-to-end on the real machine. Doesn't hardcode paths: asserts THAT it finds, not WHERE.
/// Skips with a note if Steam or Isaac aren't present (machines without the game).
#[test]
fn discovers_game_and_at_least_one_save_if_present() {
    let d = discover(&Options::default());

    let Some(steam) = &d.steam else {
        test_support::skip("Steam not found on this machine");
        return;
    };
    eprintln!(
        "steam root: {:?}, libraries: {}",
        steam.root,
        steam.libraries.len()
    );

    let Some(game) = &d.game else {
        test_support::skip("Isaac (250900) not installed");
        return;
    };
    assert!(game.dir.exists(), "the reported game folder must exist");
    eprintln!("game: {:?} edition {:?}", game.dir, game.edition);

    if d.saves.is_empty() {
        eprintln!("note: no saves found (profile never launched?)");
    } else {
        eprintln!("saves found: {}", d.saves.len());
        for s in &d.saves {
            assert!(
                s.path.exists(),
                "every candidate must point to a file that exists"
            );
        }
    }
}
