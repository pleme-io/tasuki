//! Statusline assembly — combines components into a full statusline string.
//!
//! The rendered string uses Neovim's `%#HlGroup#` syntax to switch
//! highlight groups inline. It is set as the global `'statusline'` option.

use crate::components::{self, DiagnosticCounts};
use crate::theme::Mode;

/// All the data needed to render one statusline frame.
///
/// This struct is populated from Neovim state in `lib.rs` and passed
/// here for pure rendering — keeping the rendering logic testable.
#[derive(Debug, Clone, Default)]
pub struct StatuslineData {
    /// Current Vim mode.
    pub mode: Mode,
    /// Full buffer file path.
    pub filepath: String,
    /// Whether the buffer has been modified.
    pub modified: bool,
    /// Current git branch (empty if not in a repo).
    pub git_branch: String,
    /// LSP diagnostic counts.
    pub diagnostics: DiagnosticCounts,
    /// 1-based cursor line.
    pub line: usize,
    /// 1-based cursor column.
    pub col: usize,
    /// Buffer filetype (e.g. `"rust"`, `"lua"`).
    pub filetype: String,
    /// File encoding (e.g. `"utf-8"`).
    pub encoding: String,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Render a complete statusline string from the given data.
///
/// Layout:
/// ```text
/// [MODE] [icon filename [+]] [branch] [diagnostics]  %= [filetype] [encoding] [Ln X, Col Y] [pct]
/// ```
#[must_use]
pub fn render(data: &StatuslineData) -> String {
    let mut left = String::with_capacity(256);
    let mut right = String::with_capacity(128);

    // ── Left side ──
    left.push_str(&components::mode(data.mode));

    // File icon + filename
    let icon = components::file_icon(&data.filepath);
    let fname = if data.filepath.is_empty() {
        "[No Name]".to_string()
    } else {
        data.filepath
            .rsplit('/')
            .next()
            .unwrap_or(&data.filepath)
            .to_string()
    };
    let mod_flag = if data.modified { " [+]" } else { "" };
    left.push_str(&format!(
        "%#{}# {icon} {fname}{mod_flag} ",
        crate::theme::HL_FILE,
    ));

    // Git branch
    left.push_str(&components::git_branch(&data.git_branch));

    // Diagnostics
    left.push_str(&components::diagnostics(data.diagnostics));

    // ── Alignment separator ──
    left.push_str(&components::align_right());

    // ── Right side ──
    right.push_str(&components::filetype(&data.filetype));
    right.push_str(&components::encoding(&data.encoding));
    right.push_str(&components::position(data.line, data.col));
    right.push_str(&components::percent());

    format!("{left}{right}")
}

/// Build a statusline string that uses Neovim's `%!` evaluation.
///
/// Instead of embedding literal data, this returns a `'statusline'` value
/// that calls back into Lua → Rust on every redraw.  This is the
/// preferred approach because Neovim re-evaluates `%{%…%}` expressions
/// on each redraw, so the mode and cursor position stay current.
///
/// The Lua bridge calls `require('tasuki').render_statusline()`.
#[must_use]
pub fn statusline_expr() -> String {
    String::from("%{%v:lua.require('tasuki').render_statusline()%}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> StatuslineData {
        StatuslineData {
            mode: Mode::Normal,
            filepath: "/home/user/project/src/main.rs".to_string(),
            modified: false,
            git_branch: "main".to_string(),
            diagnostics: DiagnosticCounts::default(),
            line: 42,
            col: 10,
            filetype: "rust".to_string(),
            encoding: "utf-8".to_string(),
        }
    }

    #[test]
    fn render_contains_mode() {
        let result = render(&sample_data());
        assert!(result.contains("NORMAL"));
        assert!(result.contains("TasukiModeNormal"));
    }

    #[test]
    fn render_contains_filename() {
        let result = render(&sample_data());
        assert!(result.contains("main.rs"));
    }

    #[test]
    fn render_contains_file_icon() {
        let result = render(&sample_data());
        // Rust icon
        assert!(result.contains('\u{e7a8}'));
    }

    #[test]
    fn render_contains_git_branch() {
        let result = render(&sample_data());
        assert!(result.contains("main"));
        assert!(result.contains("TasukiGit"));
    }

    #[test]
    fn render_contains_position() {
        let result = render(&sample_data());
        assert!(result.contains("Ln 42, Col 10"));
    }

    #[test]
    fn render_contains_filetype() {
        let result = render(&sample_data());
        assert!(result.contains("rust"));
        assert!(result.contains("TasukiFiletype"));
    }

    #[test]
    fn render_contains_encoding() {
        let result = render(&sample_data());
        assert!(result.contains("utf-8"));
    }

    #[test]
    fn render_contains_alignment() {
        let result = render(&sample_data());
        assert!(result.contains("%="));
    }

    #[test]
    fn render_contains_percent() {
        let result = render(&sample_data());
        assert!(result.contains("%p%%"));
    }

    #[test]
    fn render_modified_flag() {
        let mut data = sample_data();
        data.modified = true;
        let result = render(&data);
        assert!(result.contains("[+]"));
    }

    #[test]
    fn render_no_modified_flag() {
        let data = sample_data();
        let result = render(&data);
        assert!(!result.contains("[+]"));
    }

    #[test]
    fn render_no_git_branch() {
        let mut data = sample_data();
        data.git_branch.clear();
        let result = render(&data);
        assert!(!result.contains("TasukiGit"));
    }

    #[test]
    fn render_no_name_buffer() {
        let mut data = sample_data();
        data.filepath.clear();
        let result = render(&data);
        assert!(result.contains("[No Name]"));
    }

    #[test]
    fn render_with_diagnostics() {
        let mut data = sample_data();
        data.diagnostics = DiagnosticCounts {
            errors: 2,
            warnings: 1,
            info: 0,
            hints: 0,
        };
        let result = render(&data);
        assert!(result.contains("TasukiDiagError"));
        assert!(result.contains("2"));
        assert!(result.contains("TasukiDiagWarn"));
        assert!(result.contains("1"));
    }

    #[test]
    fn render_insert_mode() {
        let mut data = sample_data();
        data.mode = Mode::Insert;
        let result = render(&data);
        assert!(result.contains("INSERT"));
        assert!(result.contains("TasukiModeInsert"));
    }

    #[test]
    fn statusline_expr_format() {
        let expr = statusline_expr();
        assert!(expr.contains("v:lua.require('tasuki')"));
        assert!(expr.contains("render_statusline"));
    }
}
