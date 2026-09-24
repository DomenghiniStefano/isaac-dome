//! ipc — view-models for the UI. Pure logic: no I/O, no dependency on Tauri.

mod autostart;
mod catalog_view;
mod challenges;
mod collection;
pub mod contract;
mod error;
mod floor;
#[cfg(feature = "test-api")]
pub mod for_tests;
mod goals;
mod graph;
mod icon;
mod live;
mod mark_art;
mod marks;
mod preview;
mod profile;
mod progress;
mod queue;
mod reasons;
mod release_notes;
mod resources;
mod roll;
mod runs;
mod save_cache;
mod search;
mod settings;
mod sprite_png;
mod summary;
mod target_sprite;
mod tray;
mod update;
mod want;
mod wiki;
mod wiki_target;

pub use autostart::{
    launch_intent, AutostartFailure, AutostartReason, AutostartView, LaunchIntent, AUTOSTART_ENTRY,
    SILENT_ARG,
};
pub use catalog_view::{catalog_view, item_views, CatalogView, ItemKindView, ItemView, KindCounts};
pub use challenges::{
    challenges_view, ChallengeRow, ChallengeStateView, ChallengeTotals, ChallengesDiagnostic,
    ChallengesView, RewardView,
};
pub use collection::{
    collection_view, CollectionDiagnostic, CollectionItem, CollectionTotals, CollectionView,
    LockView,
};
pub use error::IpcError;
pub use floor::{
    floor_view, room_icons, AppliedRule, FloorCandidate, FloorDiagnostic, FloorSolutionView,
    FloorUnresolved, FloorView, RoomIconView, RoomKindView, TargetView,
};
pub use goals::{target_exists, Goal, GoalId, TargetKey, UnlockTarget};
pub use graph::{
    graph_views, next_steps, plan_view, resolve_target, target_of, unlock_view, AchievementRef,
    GoalView, GraphInfo, GraphViews, MarkColumnView, MarkLevelView, NextSteps, OriginView,
    PlanDiagnostic, PlanExpansion, PlanStep, PlanView, RequirementView, StepsBasis, StepsSection,
    ThresholdItemView, UnlockDiagnostic, UnlockNode, UnlockTotals, UnlockView, STEPS,
};
pub use icon::{
    icon_source, unknown_source, IconRef, MarkFill, MarkTier, ICON_SCHEME, UNKNOWN_SPRITE,
};
pub use live::{
    characters_named, live_mark_rows, live_marks, live_view, LiveAchievement, LiveDiagnostic,
    LiveGraph, LiveMarkRow, LiveMarks, LiveOpen, LiveView,
};
pub use mark_art::{
    mark_source, paper_source, widget_source, MarkFrames, WidgetArt, LOBBY_ANM2, WIDGET_ANM2,
};
pub use marks::{
    character_for, marks_matrix, Cell, CellLevel, CharacterGroup, CharacterRow, MarkArtView,
    MarksMatrix, MarksTotals, SecondLevelView, BOSSES, CHARACTERS,
};
// Reached only by tests: the layout tables and counter lookups are the crate's own business.
#[cfg(feature = "test-api")]
pub use floor::ROOM_KINDS;
#[cfg(feature = "test-api")]
pub use marks::{counter_index, marks_totals, CHARACTER_KEYS};
pub use preview::{preview_of, CandidatePreview, PreviewCount};
pub use profile::{
    candidates, profile_id, resolve_active, setup_state, ActiveProfile, CandidateSource,
    CandidateView, ChoiceReason, GameView, MissingReason, ProfileId, SetupDiagnostic, SetupState,
    SteamView,
};
pub use progress::SaveProgress;
pub use queue::{
    achievement_unlocking, achievements_unlocking, goals_pending, queue_view, GraphDeps,
    QueueDiagnostic, QueueInputs, QueueRow, QueueView,
};
pub use reasons::{IoReason, SaveReason, SettingsReason, StoreReason};
pub use release_notes::release_notes;
pub use resources::{
    archive_views, data_url, extraction_report, ArchiveMode, ArchiveView, ExtractionReport,
    SpriteView,
};
pub use roll::{
    deck_preset, preset_from_view, preset_view, roll_space, roll_view, DeckView, DrawnTargetView,
    DrawnView, PresetView, RollDiagnostic, RollInputs, RollRowView, RollView, SelectionView,
    StatusView,
};
pub use runs::{
    runs_view, CatalogKinds, RunItemRef, RunOutcomeView, RunSource, RunTotals, RunView,
    RunsDiagnostic, RunsInputs, RunsView,
};
pub use save_cache::SaveCache;
pub use search::{
    search, ProgressMark, SaveFlags, SearchDiagnostic, SearchHit, SearchIndex, SearchMatch,
    SearchView,
};
pub use settings::{
    session_document_fits, snap_percent, Settings, DEFAULT_SCALE, MAX_SESSION_BYTES, SCALE_PERCENTS,
};
pub use sprite_png::{centre_opaque, crop_png, decode_rgba, overlay, trim_opaque};
pub use summary::{save_summary, SaveDiagnostic, SaveSummary, SectionCount};
pub use target_sprite::{target_sprite, TargetSprite};
pub use tray::{tray_action, tray_locale, tray_text, TrayAction, TrayLocale, TrayText};
pub use update::{UpdateFailure, UpdatePhase, UpdateReason, UpdateState, UpdateView};
pub use want::{want_view, WantDiagnostic, WantRoute, WantState, WantView, WantedView};
pub use wiki::{
    rfc3339_to_unix, wiki_index, wiki_info, Block, Dlc, Entry, Infobox, Inline, ListItem,
    PatchView, Section, SectionKind, Style, Target, WikiCounts, WikiIndex, WikiInfo,
    WikiMissingReason, WikiPageRef,
};
