//! M4 2b: what the run being played would open, and the four ways the app can only say it
//! does not know.

use ipc::{
    live_view, AchievementRef, GraphInfo, LiveDiagnostic, LiveGraph, MarkColumnView, MarkLevelView,
    RequirementView, RunOutcomeView, RunSource, RunView, SecondLevelView, UnlockNode,
};

fn open_run(character: Option<&str>) -> RunView {
    RunView {
        source: RunSource::Live,
        ordinal: 1,
        character: character.map(str::to_string),
        character_id: None,
        seed_words: "FYQ8 QQ8G".into(),
        online: false,
        outcome: RunOutcomeView::Open,
        floors: 3,
        starting_items: Vec::new(),
        collected: Vec::new(),
        held_active: None,
        achievements: Vec::new(),
    }
}

fn mark(character: u32, name: &str, column: MarkColumnView) -> RequirementView {
    RequirementView::Mark {
        character,
        character_name: name.into(),
        column,
        level: MarkLevelView::Base,
    }
}

fn node(id: u32, missing: Vec<RequirementView>) -> UnlockNode {
    UnlockNode {
        achievement: AchievementRef::Known {
            id,
            text: format!("achievement {id}"),
            condition: None,
            icon_url: None,
        },
        done: false,
        unlocks: Vec::new(),
        origin: None,
        missing,
        graph: GraphInfo::Computed {
            available_now: false,
            blocked_by: 1,
            fan_out: 0,
            steps_missing: 1,
        },
    }
}

/// The names the catalog answers with: a name can reach two ids, because the game gives a
/// Tainted character the base form's name.
fn by_name(name: &str, id: Option<u32>) -> Vec<(u32, String)> {
    if let Some(id) = id {
        return match id {
            2 => vec![(2, "Cain".into())],
            23 => vec![(23, "Tainted Cain".into())],
            3 => vec![(3, "Judas".into())],
            _ => Vec::new(),
        };
    }
    match name {
        "Cain" => vec![(2, "Cain".into()), (23, "Tainted Cain".into())],
        "Judas" => vec![(3, "Judas".into())],
        _ => Vec::new(),
    }
}

#[test]
fn a_run_opens_what_only_its_own_mark_is_missing_from() {
    let nodes = vec![
        node(1, vec![mark(3, "Judas", MarkColumnView::MomsHeart)]),
        // Two marks of the same character: one run cannot give both, so it is not offered.
        node(
            2,
            vec![
                mark(3, "Judas", MarkColumnView::MomsHeart),
                mark(3, "Judas", MarkColumnView::Satan),
            ],
        ),
        // Someone else's mark is still in the way.
        node(
            3,
            vec![
                mark(3, "Judas", MarkColumnView::MomsHeart),
                mark(2, "Cain", MarkColumnView::MomsHeart),
            ],
        ),
    ];
    let view = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::Nodes(&nodes),
        None,
        by_name,
    );
    let ids: Vec<u32> = view
        .opens
        .iter()
        .flat_map(|o| o.achievements.iter())
        .filter_map(|a| match &a.achievement {
            AchievementRef::Known { id, .. } => Some(*id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1], "{:#?}", view.opens);
    assert!(view.diagnostics.is_empty(), "{:#?}", view.diagnostics);
}

#[test]
fn what_is_already_done_is_not_offered_again() {
    let mut done = node(1, vec![mark(3, "Judas", MarkColumnView::MomsHeart)]);
    done.done = true;
    let view = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::Nodes(&[done].to_vec()),
        None,
        by_name,
    );
    assert!(view.opens.is_empty());
}

#[test]
fn the_offers_are_grouped_by_the_cell_they_need() {
    let nodes = vec![
        node(1, vec![mark(3, "Judas", MarkColumnView::MomsHeart)]),
        node(2, vec![mark(3, "Judas", MarkColumnView::MomsHeart)]),
        node(3, vec![mark(3, "Judas", MarkColumnView::Satan)]),
    ];
    let view = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::Nodes(&nodes),
        None,
        by_name,
    );
    assert_eq!(view.opens.len(), 2, "{:#?}", view.opens);
    assert_eq!(view.opens[0].achievements.len(), 2);
    assert_eq!(view.opens[1].achievements.len(), 1);
}

/// The name the log prints reaches two characters, and the screen says both rather than
/// choosing: the base and the Tainted form share the name the game writes.
#[test]
fn a_name_that_reaches_two_characters_is_said_to_be_both() {
    let nodes = vec![
        node(1, vec![mark(2, "Cain", MarkColumnView::MomsHeart)]),
        node(2, vec![mark(23, "Tainted Cain", MarkColumnView::MomsHeart)]),
    ];
    let view = live_view(
        Some(open_run(Some("Cain"))),
        LiveGraph::Nodes(&nodes),
        None,
        by_name,
    );
    assert_eq!(view.opens.len(), 2);
    assert!(view
        .diagnostics
        .contains(&LiveDiagnostic::AmbiguousCharacter {
            name: "Cain".into(),
            forms: 2
        }));
}

#[test]
fn every_way_of_not_knowing_says_so() {
    // Nothing is being played.
    let idle = live_view(None, LiveGraph::Nodes(&Vec::new()), None, by_name);
    assert!(idle.run.is_none());
    assert!(idle.diagnostics.contains(&LiveDiagnostic::NoRun));

    // A run whose character no item line has named yet.
    let unnamed = live_view(
        Some(open_run(None)),
        LiveGraph::Nodes(&Vec::new()),
        None,
        by_name,
    );
    assert!(unnamed
        .diagnostics
        .contains(&LiveDiagnostic::CharacterNotNamed));

    // A name the catalog does not know.
    let unknown = live_view(
        Some(open_run(Some("Nobody"))),
        LiveGraph::Nodes(&Vec::new()),
        None,
        by_name,
    );
    assert!(unknown
        .diagnostics
        .contains(&LiveDiagnostic::UnknownCharacter {
            name: "Nobody".into()
        }));

    // No graph at all: the run still draws, and the screen must not read as "this opens
    // nothing".
    let blind = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::NoGraph,
        None,
        by_name,
    );
    assert!(blind.run.is_some());
    assert!(blind.diagnostics.contains(&LiveDiagnostic::NoGraph));
    assert!(blind.opens.is_empty());
}

/// "No profile" and "no graph" are different sentences to the person reading the screen: one
/// says we cannot read your progress, the other that the game is not installed. The run draws
/// either way.
#[test]
fn no_profile_is_not_the_same_as_no_graph() {
    let view = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::NoProfile,
        None,
        by_name,
    );
    assert!(view.run.is_some());
    assert_eq!(view.diagnostics, vec![LiveDiagnostic::NoProfile]);
}

/// The dashboard half: the row of the completion matrix for the character being played, so
/// the screen can say *these are the marks you are missing* beside *this run could give you
/// that one*. Two forms mean two rows, for the same reason the opens hold two.
#[test]
fn the_marks_of_the_character_being_played_travel_with_the_run() {
    let counters = vec![0u32; 600];
    let matrix = ipc::marks_matrix(&counters, None, |_| None);
    // Cain is row 2 of the matrix's own table, Tainted Cain the row keyed as its tainted form.
    let marks = ipc::live_marks(&matrix, &[2]);
    assert_eq!(marks.rows.len(), 1);
    assert_eq!(marks.rows[0].character, "Cain");
    assert_eq!(marks.rows[0].cells.len(), marks.bosses.len());
    // Nothing done on an empty profile: every cell of the row is still to be taken.
    assert_eq!(marks.rows[0].missing as usize, marks.bosses.len());
}

/// The end of the ambiguity, measured rather than inferred: the log states the character by
/// **id**, so when the archive carries one the screen stops saying "Cain or Tainted Cain" and
/// says which. The name stays as the fallback for a run folded before this line was read.
#[test]
fn an_id_from_the_log_settles_which_character_it_is() {
    let mut run = open_run(Some("Cain"));
    run.character_id = Some(23); // Tainted Cain
    let nodes = vec![
        node(1, vec![mark(2, "Cain", MarkColumnView::MomsHeart)]),
        node(2, vec![mark(23, "Tainted Cain", MarkColumnView::MomsHeart)]),
    ];
    let view = live_view(Some(run), LiveGraph::Nodes(&nodes), None, by_name);
    assert_eq!(view.opens.len(), 1, "{:#?}", view.opens);
    assert_eq!(view.opens[0].character, 23);
    assert!(
        !view
            .diagnostics
            .iter()
            .any(|d| matches!(d, LiveDiagnostic::AmbiguousCharacter { .. })),
        "nothing is ambiguous once the log has said it: {:#?}",
        view.diagnostics
    );
}

/// The second level of a cell is said in that column's own word (B66): Ultra Greedier in
/// Greed, which is where a second-level offer actually comes from, and hard elsewhere. A
/// base-level offer carries no word, because the screen says nothing extra for it.
#[test]
fn a_second_level_offer_names_its_level_the_way_its_column_does() {
    let second = |column| RequirementView::Mark {
        character: 3,
        character_name: "Judas".into(),
        column,
        level: MarkLevelView::Second,
    };
    let nodes = vec![
        node(1, vec![second(MarkColumnView::Greed)]),
        node(2, vec![second(MarkColumnView::Mother)]),
        node(3, vec![mark(3, "Judas", MarkColumnView::Satan)]),
    ];
    let view = live_view(
        Some(open_run(Some("Judas"))),
        LiveGraph::Nodes(&nodes),
        None,
        by_name,
    );
    let words: Vec<_> = view
        .opens
        .iter()
        .map(|o| (o.column, o.second_level))
        .collect();
    assert_eq!(
        words,
        vec![
            (MarkColumnView::Greed, Some(SecondLevelView::UltraGreedier)),
            (MarkColumnView::Mother, Some(SecondLevelView::Hard)),
            (MarkColumnView::Satan, None),
        ]
    );
}

/// Card #80, item 13: what the graph's answer means to Live, variant by variant. It used to be
/// `Err(_) => NoGraph`, so a save that did not read was told as "the game is not installed".
mod graph_of {
    use ipc::{live_graph, IpcError, LiveGraph, SaveReason};

    fn view() -> ipc::UnlockView {
        ipc::for_tests::unlock_view_of(vec![])
    }

    #[test]
    fn the_nodes_when_the_graph_answered() {
        let view = view();
        assert!(matches!(live_graph(Ok(&view)), LiveGraph::Nodes(_)));
    }

    #[test]
    fn no_profile_when_there_is_no_profile_to_read() {
        assert!(matches!(
            live_graph(Err(&IpcError::NoActiveProfile)),
            LiveGraph::NoProfile
        ));
        assert!(matches!(
            live_graph(Err(&IpcError::UnknownProfile {
                id: "gone".to_string()
            })),
            LiveGraph::NoProfile
        ));
    }

    #[test]
    fn an_unreadable_save_is_said_as_such_and_not_as_a_missing_game() {
        assert!(matches!(
            live_graph(Err(&IpcError::UnreadableSave {
                reason: SaveReason::TooShort
            })),
            LiveGraph::SaveUnreadable
        ));
    }

    #[test]
    fn no_graph_when_the_catalog_is_missing() {
        assert!(matches!(
            live_graph(Err(&IpcError::CatalogUnavailable)),
            LiveGraph::NoGraph
        ));
    }
}
