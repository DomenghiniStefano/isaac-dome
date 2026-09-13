//! What a click on the tray icon means. The tray itself is Tauri's, and lives in the `app`
//! crate; the decision is here, where it can be tested.

/// The label of the drag's preview window (`ui/src/lib/window/preview.ts`). It holds no tab
/// and must never be the window a click brings forward.
const PREVIEW_LABEL: &str = "tab-preview";

/// The first window's label, fixed by `tauri.conf.json`.
const MAIN_LABEL: &str = "main";

/// What the tray does with a click, given the windows that exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    /// Bring this window forward: it may be behind, hidden or minimized.
    Focus { label: String },
    /// There is nothing to bring forward. Build the main window.
    CreateMain,
}

/// The main window if it is open; otherwise the oldest window that holds tabs; otherwise
/// "create one".
///
/// **The oldest is the smallest label.** A torn-off window is `win-<milliseconds in base 36>`
/// (`newWindowLabel`), which is the same width from 2005 to 2059 — so lexicographic order is
/// chronological order. The day that stops being true, this comment is the one that explains
/// why the wrong window came forward.
pub fn tray_action(labels: &[String]) -> TrayAction {
    if labels.iter().any(|l| l == MAIN_LABEL) {
        return TrayAction::Focus {
            label: MAIN_LABEL.to_string(),
        };
    }
    match labels.iter().filter(|l| l.as_str() != PREVIEW_LABEL).min() {
        Some(label) => TrayAction::Focus {
            label: label.clone(),
        },
        None => TrayAction::CreateMain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(of: &[&str]) -> Vec<String> {
        of.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn with_no_window_open_a_click_creates_the_main_one() {
        assert_eq!(tray_action(&[]), TrayAction::CreateMain);
    }

    #[test]
    fn the_main_window_is_the_one_a_click_means_whatever_else_is_open() {
        let action = tray_action(&labels(&["win-abc", "main", "tab-preview"]));
        assert_eq!(
            action,
            TrayAction::Focus {
                label: "main".to_string()
            }
        );
    }

    #[test]
    fn without_the_main_window_a_click_means_the_oldest_torn_off_one() {
        // `win-<ms in base 36>`: same width, so the smallest string is the oldest window.
        let action = tray_action(&labels(&["win-mfk2h9", "win-mfk1zz"]));
        assert_eq!(
            action,
            TrayAction::Focus {
                label: "win-mfk1zz".to_string()
            }
        );
    }

    #[test]
    fn the_drag_preview_is_never_the_window_a_click_means() {
        // It carries no tab: focusing it would hand the user a ghost.
        assert_eq!(
            tray_action(&labels(&["tab-preview"])),
            TrayAction::CreateMain
        );
    }
}
