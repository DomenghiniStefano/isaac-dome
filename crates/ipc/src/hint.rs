//! The one way a folder is shown to the user: the path as it is, with the Windows username
//! replaced. Every hint on the wire goes through here — a save's, the game's, the app's own
//! data — so the masking is written once.

/// What stands where the username was: the gap is declared, not silently removed.
const USER_PLACEHOLDER: &str = "<user>";

/// Replaces the segment that follows `Users` with [`USER_PLACEHOLDER`], and touches nothing
/// else: `path_hint` exists to say "found here", and a path reduced entirely to a
/// placeholder would no longer orient anyone.
///
/// It works on segments rather than the whole string because the username can be
/// anything — including a substring that also shows up elsewhere in the path.
/// Both separators are accepted: the path comes from disk on Windows, but the tests
/// write it with `/`.
pub(crate) fn mask_user_dir(path: &str) -> String {
    path.split_inclusive(['/', '\\'])
        .scan(false, |after_users, segment| {
            let name = segment.trim_end_matches(['/', '\\']);
            let separator = &segment[name.len()..];
            if name.is_empty() {
                // Consecutive separators (`\\?\`, UNC roots): not a segment, and they must
                // not consume a pending mask.
                return Some(["", separator]);
            }
            let shown = if *after_users { USER_PLACEHOLDER } else { name };
            *after_users = !*after_users && name.eq_ignore_ascii_case("users");
            Some([shown, separator])
        })
        .flatten()
        .collect()
}
