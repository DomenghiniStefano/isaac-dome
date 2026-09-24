/// The part of tailing a file that has no I/O in it: complete lines out of chunks that do not
/// respect line endings, and the one comparison that says the game relaunched.
///
/// Both halves exist because of how the game writes: the last line of a read may be
/// half-written, and `log.txt` is rewritten from scratch on every launch, so a file shorter
/// than what has been read from it is a new log rather than a corrupted one.
#[derive(Debug, Default)]
pub struct Tail {
    /// Raw bytes, not decoded text (card #80, P7): a read ends wherever 256 KiB ends, which can
    /// be inside a character, so decoding waits until a line is whole.
    remainder: Vec<u8>,
    read: u64,
}

impl Tail {
    /// The complete lines in `chunk`, with whatever was left over from last time in front of
    /// them. A trailing partial line is kept, not returned: it is not a line yet.
    pub fn advance(&mut self, chunk: &[u8]) -> Vec<String> {
        self.read += chunk.len() as u64;
        self.remainder.extend_from_slice(chunk);
        let Some(last) = self.remainder.iter().rposition(|&b| b == b'\n') else {
            return Vec::new();
        };
        let complete: Vec<u8> = self.remainder.drain(..=last).collect();
        complete[..last]
            .split(|&b| b == b'\n')
            // Lossy on purpose, and per line: mods write into this same file, and one bad byte
            // must cost its own line and not the run around it.
            .map(|line| {
                String::from_utf8_lossy(line)
                    .trim_end_matches('\r')
                    .to_string()
            })
            .collect()
    }

    /// Bytes held back because the last line is not finished — bytes **of the file**, so the
    /// offset the watcher stores (the end of the last complete line) is exact. Counting a
    /// half-written line as read would lose it, since the next read starts after it.
    pub fn pending(&self) -> usize {
        self.remainder.len()
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
