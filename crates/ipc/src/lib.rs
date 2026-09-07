//! ipc — view-models for the UI. Pure logic: no I/O, no dependency on Tauri.

mod catalog_view;
mod goals;
mod graph;
mod marks;
mod profile;
mod queue;
mod resources;
mod settings;
mod summary;
mod target_sprite;
mod wiki;

pub use catalog_view::{catalog_view, item_views, CatalogView, ItemKindView, ItemView, KindCounts};
pub use goals::{target_exists, Goal, GoalId, TargetKey, UnlockTarget};
pub use graph::{
    next_steps, plan_view, resolve_target, target_of, unlock_view, AchievementRef, GoalView,
    GraphInfo, NextSteps, OriginView, PlanDiagnostic, PlanExpansion, PlanStep, PlanView,
    RequirementView, StepsBasis, UnlockDiagnostic, UnlockNode, UnlockTotals, UnlockView, STEPS,
};
pub use marks::{
    character_for, counter_index, marks_matrix, Cell, CharacterGroup, CharacterRow, MarksMatrix,
    MarksTotals, BOSSES, CHARACTERS, CHARACTER_KEYS,
};
pub use profile::{
    candidates, profile_id, resolve_active, setup_state, ActiveProfile, CandidateSource,
    CandidateView, ChoiceReason, GameView, MissingReason, ProfileId, SetupDiagnostic, SetupState,
    SteamView,
};
pub use queue::{
    achievement_unlocking, queue_view, QueueDiagnostic, QueueInputs, QueueRow, QueueView,
};
pub use resources::{
    archive_views, data_url, extraction_report, ArchiveMode, ArchiveView, ExtractionReport,
    SpriteView,
};
pub use settings::Settings;
pub use summary::{save_summary, SaveDiagnostic, SaveSummary, SectionCount};
pub use target_sprite::{target_sprite, TargetSprite};
pub use wiki::{
    rfc3339_to_unix, wiki_info, Block, Dlc, Entry, Infobox, Inline, ListItem, PatchView, Section,
    SectionKind, Style, Target, WikiCounts, WikiInfo, WikiMissingReason,
};
