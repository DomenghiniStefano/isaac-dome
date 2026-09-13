/// The part of tailing a file that has no I/O in it: complete lines out of chunks that do not
/// respect line endings, and the one comparison that says the game relaunched.
///
/// Both halves exist because of how the game writes: the last line of a read may be
/// half-written, and `log.txt` is rewritten from scratch on every launch, so a file shorter
/// than what has been read from it is a new log rather than a corrupted one.
#[derive(Debug, Default)]
pub struct Tail {
    remainder: String,
    read: u64,
}

impl Tail {
    /// The complete lines in `chunk`, with whatever was left over from last time in front of
    /// them. A trailing partial line is kept, not returned: it is not a line yet.
    pub fn advance(&mut self, chunk: &[u8]) -> Vec<String> {
        self.read += chunk.len() as u64;
        // Lossy on purpose: mods write into this same file and one bad byte must not cost the
        // run around it.
        self.remainder.push_str(&String::from_utf8_lossy(chunk));
        let mut lines = Vec::new();
        while let Some(at) = self.remainder.find('\n') {
            let line = self.remainder[..at].trim_end_matches('\r').to_string();
            self.remainder.drain(..=at);
            lines.push(line);
        }
        lines
    }

    /// The file is shorter than what we have read from it: the game relaunched and rewrote it.
    pub fn restarted(&self, len: u64) -> bool {
        len < self.read
    }

    /// Start over on a new file. The old remainder is dropped rather than carried: it would
    /// arrive glued to the first line of the new log.
    pub fn restart(&mut self) {
        self.remainder.clear();
        self.read = 0;
    }
}
