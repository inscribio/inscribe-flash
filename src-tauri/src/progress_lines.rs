use std::io::{self, BufRead};

/// Buffered stream reader that treats both \r and \n as line termination.
/// This allows to iterate in real time over output of commands with progress bars.
///
/// Based on https://stackoverflow.com/a/55145242 and implementations in std.
#[derive(Debug)]
pub struct ProgressLines<B> {
    reader: B,
}

impl<B: BufRead> ProgressLines<B> {
    #[allow(dead_code)]
    pub fn new(reader: B) -> Self {
        Self { reader }
    }

    // Keep reading and appending to buf until delimiter is found or we read EOF.
    // Returns true if found delimiter, else (=> found EOF) false.
    fn read_until_delimiter(&mut self, buf: &mut Vec<u8>) -> io::Result<bool> {
        loop {
            // Perform single IO operation and check if we found delimiter
            let (found, consumed) = {
                // Fill the buffer from stream (performs IO)
                let bytes = match self.reader.fill_buf() {
                    Ok(b) => b,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };

                // Find the delimiter
                let is_delim = |c: &u8| *c == b'\n' || *c == b'\r';
                match bytes.iter().position(is_delim) {
                    Some(n) => {
                        // Drop delimiter byte
                        buf.extend_from_slice(&bytes[..n]);
                        // Consume with the delimiter
                        (true, n + 1)
                    }
                    None => {
                        // Append all data that has been read
                        buf.extend_from_slice(bytes);
                        (false, bytes.len())
                    }
                }
            };

            // Drop used data from reader
            self.reader.consume(consumed);

            // Stop reading if we have a line, or if we encounter EOF
            let end_of_file = consumed == 0;
            if found || end_of_file {
                return Ok(found);
            }
        }
    }
}

impl<B: BufRead> Iterator for ProgressLines<B> {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::<u8>::new();

        let bytes = match self.read_until_delimiter(&mut buf) {
            Err(e) => return Some(Err(e)),
            Ok(false) if buf.len() == 0 => None, // EOF and no data
            Ok(_) => Some(&buf[..]),
        };

        bytes.map(|b| {
            // TODO: it's probably possible to keep Cow by storing `buf` in `self`
            let line = String::from_utf8_lossy(b).to_string();
            Ok(line)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_by_newline() -> anyhow::Result<()> {
        let text = "Sodales latine detracto commodo\n est philosophia definitiones\n sem. Dicant\n modus...";
        let lines = ProgressLines::new(text.as_bytes()).collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            lines,
            [
                "Sodales latine detracto commodo",
                " est philosophia definitiones",
                " sem. Dicant",
                " modus..."
            ]
        );
        Ok(())
    }

    #[test]
    fn splits_by_carriage_return() -> anyhow::Result<()> {
        let text = "Sodales latine detracto commodo\r est philosophia definitiones\n sem. Dicant\r modus...";
        let lines = ProgressLines::new(text.as_bytes()).collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            lines,
            [
                "Sodales latine detracto commodo",
                " est philosophia definitiones",
                " sem. Dicant",
                " modus..."
            ]
        );
        Ok(())
    }

    #[test]
    fn includes_empty_lines() -> anyhow::Result<()> {
        let text = "Sodales latine detracto commodo\n\n est philosophia definitiones\r\n sem. Dicant\r\n modus...";
        let lines = ProgressLines::new(text.as_bytes()).collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            lines,
            [
                "Sodales latine detracto commodo",
                "",
                " est philosophia definitiones",
                "",
                " sem. Dicant",
                "",
                " modus..."
            ]
        );
        Ok(())
    }

    #[test]
    fn empty_string_no_lines() -> anyhow::Result<()> {
        let text = "";
        let lines = ProgressLines::new(text.as_bytes()).collect::<Result<Vec<_>, _>>()?;
        assert_eq!(lines, []);
        Ok(())
    }
}
