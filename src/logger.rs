//! tracing ベースのファイルロガー。設定ディレクトリに `tamago.log` を出力。
//! 5000 行を超えたら `tamago.log.1` にローテし最新1世代だけ残す。

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MAX_LINES: usize = 5000;
const LOG_FILE: &str = "tamago.log";
const ROTATED_FILE: &str = "tamago.log.1";

static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

pub fn init(base_dir: &Path) {
    if std::fs::create_dir_all(base_dir).is_err() {
        return;
    }
    let writer = match RotatingWriter::open(base_dir.join(LOG_FILE), MAX_LINES) {
        Ok(w) => w,
        Err(_) => return,
    };
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);
    let _ = GUARD.set(guard);

    let _ = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .try_init();
}

struct RotatingWriter {
    path: PathBuf,
    rotated: PathBuf,
    file: File,
    line_count: usize,
    max_lines: usize,
}

impl RotatingWriter {
    fn open(path: PathBuf, max_lines: usize) -> io::Result<Self> {
        let rotated = path.with_file_name(ROTATED_FILE);
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let line_count = count_lines(&path).unwrap_or(0);
        Ok(Self {
            path,
            rotated,
            file,
            line_count,
            max_lines,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;
        std::fs::rename(&self.path, &self.rotated)?;
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.line_count = 0;
        Ok(())
    }
}

impl Write for RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.file.write(buf)?;
        self.line_count += buf[..n].iter().filter(|&&b| b == b'\n').count();
        if self.line_count >= self.max_lines {
            let _ = self.rotate();
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn count_lines(path: &Path) -> io::Result<usize> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut n = 0;
    for line in reader.lines() {
        line?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn rotates_after_max_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(LOG_FILE);
        let mut w = RotatingWriter::open(path.clone(), 3).unwrap();
        for i in 0..5 {
            writeln!(w, "line {i}").unwrap();
        }
        w.flush().unwrap();
        assert!(
            dir.path().join(ROTATED_FILE).exists(),
            "rotated file missing"
        );
        let current = std::fs::read_to_string(&path).unwrap();
        assert!(current.lines().count() < 5, "current should be reset");
    }

    #[test]
    fn resumes_line_count_from_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(LOG_FILE);
        std::fs::write(&path, "a\nb\nc\n").unwrap();
        let w = RotatingWriter::open(path, 5).unwrap();
        assert_eq!(w.line_count, 3);
    }
}
