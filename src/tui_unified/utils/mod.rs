// Utils - placeholder implementations

use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout};

pub struct TerminalUtils;

impl TerminalUtils {
    pub fn enter_raw_mode() -> io::Result<()> {
        terminal::enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_raw_mode() -> io::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn get_size() -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn clear_screen() -> io::Result<()> {
        use crossterm::{cursor, terminal::Clear, terminal::ClearType};
        stdout()
            .execute(Clear(ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;
        Ok(())
    }
}

pub struct FormatUtils;

impl FormatUtils {
    pub fn truncate_string(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[..max_length.saturating_sub(3)])
        }
    }

    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    pub fn format_duration(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

pub struct ValidationUtils;

impl ValidationUtils {
    pub fn is_valid_branch_name(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('-')
            && !name.ends_with('/')
            && !name.contains("//")
            && !name.contains(' ')
    }

    pub fn is_valid_commit_hash(hash: &str) -> bool {
        hash.len() >= 7 && hash.len() <= 40 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn is_valid_tag_name(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('-')
            && !name.contains(' ')
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
    }
}
