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

/// The languages the app speaks. **Not an IPC type**: the tray is built before any window
/// exists, and these strings never cross to the frontend, which has vue-i18n.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayLocale {
    It,
    En,
}

/// The system's language tag, read as one of ours. Same rule as `ui/src/i18n/locale.ts`:
/// primary subtag, English as the fallback. Two implementations of one rule, which is the
/// price of a menu that has to exist before the frontend does.
pub fn tray_locale(tag: &str) -> TrayLocale {
    match tag
        .split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "it" => TrayLocale::It,
        _ => TrayLocale::En,
    }
}

/// Everything the tray shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayText {
    pub open: &'static str,
    pub quit: &'static str,
    pub notice_title: &'static str,
    pub notice_body: &'static str,
    pub tooltip: &'static str,
}

pub fn tray_text(locale: TrayLocale) -> TrayText {
    match locale {
        TrayLocale::It => TrayText {
            open: "Apri IsaacDome",
            quit: "Esci",
            notice_title: "IsaacDome resta aperta",
            notice_body: "Clicca l'icona accanto all'orologio per riaprire la finestra.",
            tooltip: "IsaacDome — clicca per aprire la finestra",
        },
        TrayLocale::En => TrayText {
            open: "Open IsaacDome",
            quit: "Quit",
            notice_title: "IsaacDome is still running",
            notice_body: "Click the icon next to the clock to bring the window back.",
            tooltip: "IsaacDome — click to open the window",
        },
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

    #[test]
    fn the_tray_speaks_the_systems_language_by_its_primary_subtag() {
        // The rule `ui/src/i18n/locale.ts` applies to navigator.languages, applied here to
        // the system tag: primary subtag, English when we don't have the language.
        assert_eq!(tray_locale("it-IT"), TrayLocale::It);
        assert_eq!(tray_locale("it"), TrayLocale::It);
        assert_eq!(tray_locale("IT"), TrayLocale::It);
        assert_eq!(tray_locale("en-GB"), TrayLocale::En);
        assert_eq!(tray_locale("fr"), TrayLocale::En);
        assert_eq!(tray_locale(""), TrayLocale::En);
    }

    #[test]
    fn every_string_the_tray_shows_exists_in_both_languages() {
        for locale in [TrayLocale::It, TrayLocale::En] {
            let t = tray_text(locale);
            for s in [t.open, t.quit, t.notice_title, t.notice_body, t.tooltip] {
                assert!(!s.trim().is_empty(), "empty string for {locale:?}");
            }
        }
        // Two languages, not one twice: a copy-paste would leave them equal.
        assert_ne!(
            tray_text(TrayLocale::It).quit,
            tray_text(TrayLocale::En).quit
        );
    }
}
