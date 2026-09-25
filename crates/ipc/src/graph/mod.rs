//! The contracts for the graph screens: Unlock, Next Steps, Plan, and the one node all three
//! share. What the graph can't interpret travels as a declared `Partial`, never as a value
//! that looks computed.

pub use crate::goals::UnlockTarget;

mod missing;
mod plan;
mod steps;
mod target;
mod types;
mod unlock;

pub use plan::plan_view;
pub use steps::{graph_views, next_steps, STEPS};
pub use target::{resolve_target, target_of};
pub use types::{
    AchievementRef, GoalView, GraphInfo, GraphViews, MarkColumnView, MarkLevelView, NextSteps,
    OriginView, PlanDiagnostic, PlanView, RequirementView, StepsBasis, StepsSection,
    ThresholdItemView, UnlockDiagnostic, UnlockNode, UnlockTotals, UnlockView,
};
pub use unlock::{unlock_view, UnlockInputs};
