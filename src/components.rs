//! Individual statusline components.
//!
//! Each component renders a small piece of the statusline as a plain string
//! (with `%#HlGroup#` highlight switches embedded). Components are pure
//! functions of their inputs so they remain testable without a running
//! Neovim instance.

use crate::theme::{
    HL_DEFAULT, HL_DIAG_ERROR, HL_DIAG_HINT, HL_DIAG_INFO, HL_DIAG_WARN, HL_ENCODING, HL_FILE,
    HL_FILETYPE, HL_GIT, HL_POSITION, Mode,
};

/// Wrap `text` with a statusline highlight group switch.
fn hl(group: &str, text: &str) -> String {
    format!("%#{group}# {text} ")
}

/// Render the mode indicator.
///
/// Example output: `%#TasukiModeNormal# NORMAL `
#[must_use]
pub fn mode(mode: Mode) -> String {
    hl(mode.hl_group(), mode.label())
}

/// Render the filename section.
///
/// Shows the tail of the file path (just the filename) plus a modified
/// indicator (`[+]`) when the buffer has unsaved changes.
///
/// `path` is the full buffer name from `nvim_buf_get_name`.
#[must_use]
pub fn filename(path: &str, modified: bool) -> String {
    let name = if path.is_empty() {
        "[No Name]"
    } else {
        path.rsplit('/').next().unwrap_or(path)
    };

    let mod_flag = if modified { " [+]" } else { "" };
    hl(HL_FILE, &format!("{name}{mod_flag}"))
}

/// Render the git branch section.
///
/// Returns an empty string when there is no branch (e.g. not in a git repo).
#[must_use]
pub fn git_branch(branch: &str) -> String {
    if branch.is_empty() {
        String::new()
    } else {
        hl(HL_GIT, &format!("\u{e0a0} {branch}")) // Nerd Font branch icon
    }
}

/// Diagnostics counts for the current buffer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagnosticCounts {
    pub errors: u32,
    pub warnings: u32,
    pub info: u32,
    pub hints: u32,
}

impl DiagnosticCounts {
    /// True when there are no diagnostics at all.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.errors == 0 && self.warnings == 0 && self.info == 0 && self.hints == 0
    }
}

/// Render the diagnostics section.
///
/// Shows error/warning/info/hint counts with icons. Omits the entire
/// section when there are no diagnostics.
#[must_use]
pub fn diagnostics(counts: DiagnosticCounts) -> String {
    if counts.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();

    if counts.errors > 0 {
        parts.push(format!("%#{HL_DIAG_ERROR}# \u{f057} {} ", counts.errors));
    }
    if counts.warnings > 0 {
        parts.push(format!("%#{HL_DIAG_WARN}# \u{f071} {} ", counts.warnings));
    }
    if counts.info > 0 {
        parts.push(format!("%#{HL_DIAG_INFO}# \u{f05a} {} ", counts.info));
    }
    if counts.hints > 0 {
        parts.push(format!("%#{HL_DIAG_HINT}# \u{f0eb} {} ", counts.hints));
    }

    parts.join("")
}

/// Render the cursor position section.
///
/// Format: `Ln X, Col Y` — using 1-based line and column.
#[must_use]
pub fn position(line: usize, col: usize) -> String {
    hl(HL_POSITION, &format!("Ln {line}, Col {col}"))
}

/// Render the filetype section.
///
/// Returns empty when filetype is empty or unset.
#[must_use]
pub fn filetype(ft: &str) -> String {
    if ft.is_empty() {
        String::new()
    } else {
        hl(HL_FILETYPE, ft)
    }
}

/// Render the encoding section (e.g. `utf-8`).
///
/// Returns empty when encoding is empty.
#[must_use]
pub fn encoding(enc: &str) -> String {
    if enc.is_empty() {
        String::new()
    } else {
        hl(HL_ENCODING, enc)
    }
}

/// Render the percentage through file indicator.
///
/// Uses Neovim's built-in `%p%%` for dynamic percentage.
#[must_use]
pub fn percent() -> String {
    hl(HL_POSITION, "%p%%")
}

/// Return a separator that resets to the default highlight.
#[must_use]
pub fn separator() -> String {
    format!("%#{HL_DEFAULT}#")
}

/// Return the alignment separator (`%=`) which pushes everything
/// after it to the right.
#[must_use]
pub fn align_right() -> String {
    format!("%#{HL_DEFAULT}#%=")
}

// ── File icon helper (uses kamon when available) ──

/// Look up a Nerd Font icon for the given filename.
///
/// Uses a built-in table of common extensions. The kamon crate will
/// eventually provide a richer lookup; for now we embed the essentials.
#[must_use]
pub fn file_icon(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    // Get the ext as &str for matching
    match ext.as_str() {
        "rs" => "\u{e7a8}",       // Rust
        "lua" => "\u{e620}",      // Lua
        "py" => "\u{e73c}",       // Python
        "js" => "\u{e74e}",       // JavaScript
        "ts" => "\u{e628}",       // TypeScript
        "tsx" => "\u{e7ba}",      // React TSX
        "jsx" => "\u{e7ba}",      // React JSX
        "go" => "\u{e626}",       // Go
        "rb" => "\u{e791}",       // Ruby
        "nix" => "\u{f313}",      // Nix
        "toml" => "\u{e6b2}",     // TOML
        "yaml" | "yml" => "\u{e6a8}", // YAML
        "json" => "\u{e60b}",     // JSON
        "md" => "\u{e73e}",       // Markdown
        "sh" | "bash" | "zsh" => "\u{e795}", // Shell
        "vim" => "\u{e62b}",      // Vim
        "html" => "\u{e736}",     // HTML
        "css" => "\u{e749}",      // CSS
        "c" => "\u{e61e}",        // C
        "cpp" | "cc" | "cxx" => "\u{e61d}", // C++
        "h" | "hpp" => "\u{e61e}", // Header
        "java" => "\u{e738}",     // Java
        "dockerfile" => "\u{e7b0}", // Docker
        "lock" => "\u{f023}",     // Lock file
        "txt" => "\u{f0f6}",      // Text
        _ => "\u{f15b}",          // Generic file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_normal_output() {
        let result = mode(Mode::Normal);
        assert!(result.contains("TasukiModeNormal"));
        assert!(result.contains("NORMAL"));
    }

    #[test]
    fn mode_insert_output() {
        let result = mode(Mode::Insert);
        assert!(result.contains("TasukiModeInsert"));
        assert!(result.contains("INSERT"));
    }

    #[test]
    fn mode_visual_output() {
        let result = mode(Mode::Visual);
        assert!(result.contains("TasukiModeVisual"));
        assert!(result.contains("VISUAL"));
    }

    #[test]
    fn filename_with_path() {
        let result = filename("/home/user/code/main.rs", false);
        assert!(result.contains("main.rs"));
        assert!(!result.contains("[+]"));
    }

    #[test]
    fn filename_modified() {
        let result = filename("/home/user/code/main.rs", true);
        assert!(result.contains("main.rs"));
        assert!(result.contains("[+]"));
    }

    #[test]
    fn filename_empty_path() {
        let result = filename("", false);
        assert!(result.contains("[No Name]"));
    }

    #[test]
    fn filename_no_slash() {
        let result = filename("file.txt", false);
        assert!(result.contains("file.txt"));
    }

    #[test]
    fn git_branch_present() {
        let result = git_branch("main");
        assert!(result.contains("TasukiGit"));
        assert!(result.contains("main"));
    }

    #[test]
    fn git_branch_empty() {
        let result = git_branch("");
        assert!(result.is_empty());
    }

    #[test]
    fn diagnostics_none() {
        let result = diagnostics(DiagnosticCounts::default());
        assert!(result.is_empty());
    }

    #[test]
    fn diagnostics_errors_only() {
        let counts = DiagnosticCounts {
            errors: 3,
            ..Default::default()
        };
        let result = diagnostics(counts);
        assert!(result.contains("TasukiDiagError"));
        assert!(result.contains("3"));
        assert!(!result.contains("TasukiDiagWarn"));
    }

    #[test]
    fn diagnostics_all() {
        let counts = DiagnosticCounts {
            errors: 1,
            warnings: 2,
            info: 3,
            hints: 4,
        };
        let result = diagnostics(counts);
        assert!(result.contains("TasukiDiagError"));
        assert!(result.contains("TasukiDiagWarn"));
        assert!(result.contains("TasukiDiagInfo"));
        assert!(result.contains("TasukiDiagHint"));
    }

    #[test]
    fn diagnostics_is_empty() {
        let empty = DiagnosticCounts::default();
        assert!(empty.is_empty());

        let non_empty = DiagnosticCounts {
            hints: 1,
            ..Default::default()
        };
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn position_format() {
        let result = position(42, 10);
        assert!(result.contains("Ln 42, Col 10"));
        assert!(result.contains("TasukiPosition"));
    }

    #[test]
    fn filetype_present() {
        let result = filetype("rust");
        assert!(result.contains("TasukiFiletype"));
        assert!(result.contains("rust"));
    }

    #[test]
    fn filetype_empty() {
        let result = filetype("");
        assert!(result.is_empty());
    }

    #[test]
    fn encoding_present() {
        let result = encoding("utf-8");
        assert!(result.contains("TasukiEncoding"));
        assert!(result.contains("utf-8"));
    }

    #[test]
    fn encoding_empty() {
        let result = encoding("");
        assert!(result.is_empty());
    }

    #[test]
    fn percent_has_statusline_token() {
        let result = percent();
        assert!(result.contains("%p%%"));
    }

    #[test]
    fn separator_uses_default_hl() {
        let result = separator();
        assert!(result.contains("TasukiDefault"));
    }

    #[test]
    fn align_right_has_equals() {
        let result = align_right();
        assert!(result.contains("%="));
    }

    #[test]
    fn file_icon_rust() {
        assert_eq!(file_icon("main.rs"), "\u{e7a8}");
    }

    #[test]
    fn file_icon_lua() {
        assert_eq!(file_icon("init.lua"), "\u{e620}");
    }

    #[test]
    fn file_icon_unknown() {
        assert_eq!(file_icon("file.xyz"), "\u{f15b}");
    }

    #[test]
    fn file_icon_case_insensitive() {
        assert_eq!(file_icon("Module.RS"), "\u{e7a8}");
    }

    #[test]
    fn file_icon_no_extension() {
        // No dot → entire filename treated as extension, falls through to default
        assert_eq!(file_icon("Makefile"), "\u{f15b}");
    }

    #[test]
    fn file_icon_nix() {
        assert_eq!(file_icon("flake.nix"), "\u{f313}");
    }
}
