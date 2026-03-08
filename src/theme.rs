//! Mode-dependent colors and highlight group definitions.
//!
//! Defines highlight groups used by statusline components and provides
//! mode-to-color mapping for the mode indicator section.

use tane::highlight::Highlight;

/// Vim mode classification used for theming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Replace,
    Command,
    Terminal,
    Select,
}

impl Mode {
    /// Parse a Neovim mode string (from `nvim_get_mode`) into our enum.
    ///
    /// See `:help mode()` for the full list. We classify by the first
    /// character of the mode short-string.
    #[must_use]
    pub fn from_mode_str(s: &[u8]) -> Self {
        match s.first().copied() {
            Some(b'n') => Self::Normal,
            Some(b'i') => Self::Insert,
            Some(b'v' | b'V' | 0x16) => Self::Visual, // v, V, CTRL-V
            Some(b'R') => Self::Replace,
            Some(b'c') => Self::Command,
            Some(b't') => Self::Terminal,
            Some(b's' | b'S' | 0x13) => Self::Select, // s, S, CTRL-S
            _ => Self::Normal,
        }
    }

    /// Human-readable label for the mode indicator.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::Replace => "REPLACE",
            Self::Command => "COMMAND",
            Self::Terminal => "TERMINAL",
            Self::Select => "SELECT",
        }
    }

    /// Foreground color (hex) for the mode indicator.
    #[must_use]
    pub const fn fg(self) -> &'static str {
        "#1e1e2e" // dark background text for all modes
    }

    /// Background color (hex) for the mode indicator.
    #[must_use]
    pub const fn bg(self) -> &'static str {
        match self {
            Self::Normal => "#89b4fa",  // blue
            Self::Insert => "#a6e3a1",  // green
            Self::Visual => "#cba6f7",  // purple
            Self::Replace => "#f38ba8", // red
            Self::Command => "#fab387", // peach
            Self::Terminal => "#94e2d5", // teal
            Self::Select => "#f5c2e7",  // pink
        }
    }

    /// The highlight group name for this mode.
    #[must_use]
    pub const fn hl_group(self) -> &'static str {
        match self {
            Self::Normal => "TasukiModeNormal",
            Self::Insert => "TasukiModeInsert",
            Self::Visual => "TasukiModeVisual",
            Self::Replace => "TasukiModeReplace",
            Self::Command => "TasukiModeCommand",
            Self::Terminal => "TasukiModeTerminal",
            Self::Select => "TasukiModeSelect",
        }
    }
}

/// All mode variants for iteration.
pub const ALL_MODES: [Mode; 7] = [
    Mode::Normal,
    Mode::Insert,
    Mode::Visual,
    Mode::Replace,
    Mode::Command,
    Mode::Terminal,
    Mode::Select,
];

// ── Highlight group names for non-mode sections ──

/// Statusline background / default section.
pub const HL_DEFAULT: &str = "TasukiDefault";
/// File info section (filename, modified flag).
pub const HL_FILE: &str = "TasukiFile";
/// Git branch section.
pub const HL_GIT: &str = "TasukiGit";
/// Diagnostics — error count.
pub const HL_DIAG_ERROR: &str = "TasukiDiagError";
/// Diagnostics — warning count.
pub const HL_DIAG_WARN: &str = "TasukiDiagWarn";
/// Diagnostics — info count.
pub const HL_DIAG_INFO: &str = "TasukiDiagInfo";
/// Diagnostics — hint count.
pub const HL_DIAG_HINT: &str = "TasukiDiagHint";
/// Position / cursor location section.
pub const HL_POSITION: &str = "TasukiPosition";
/// Filetype section.
pub const HL_FILETYPE: &str = "TasukiFiletype";
/// Encoding section.
pub const HL_ENCODING: &str = "TasukiEncoding";

// ── Color palette (Catppuccin Mocha inspired) ──

const BG_DARK: &str = "#1e1e2e";
const BG_MID: &str = "#313244";
const FG_TEXT: &str = "#cdd6f4";
const FG_SUBTEXT: &str = "#a6adc8";
const RED: &str = "#f38ba8";
const YELLOW: &str = "#f9e2af";
const BLUE: &str = "#89b4fa";
const GREEN: &str = "#a6e3a1";
const TEAL: &str = "#94e2d5";
const LAVENDER: &str = "#b4befe";

/// Register all tasuki highlight groups.
///
/// Should be called once at plugin init and again on `ColorScheme`
/// autocommand to react to theme changes.
pub fn setup_highlights() -> tane::Result<()> {
    // Mode highlights
    for mode in &ALL_MODES {
        Highlight::new(mode.hl_group())
            .fg(mode.fg())
            .bg(mode.bg())
            .bold()
            .apply()?;
    }

    // Section highlights
    Highlight::new(HL_DEFAULT)
        .fg(FG_TEXT)
        .bg(BG_DARK)
        .apply()?;

    Highlight::new(HL_FILE)
        .fg(FG_TEXT)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_GIT)
        .fg(LAVENDER)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_DIAG_ERROR)
        .fg(RED)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_DIAG_WARN)
        .fg(YELLOW)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_DIAG_INFO)
        .fg(BLUE)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_DIAG_HINT)
        .fg(GREEN)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_POSITION)
        .fg(FG_SUBTEXT)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_FILETYPE)
        .fg(TEAL)
        .bg(BG_MID)
        .apply()?;

    Highlight::new(HL_ENCODING)
        .fg(FG_SUBTEXT)
        .bg(BG_MID)
        .apply()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_from_normal() {
        assert_eq!(Mode::from_mode_str(b"n"), Mode::Normal);
    }

    #[test]
    fn mode_from_insert() {
        assert_eq!(Mode::from_mode_str(b"i"), Mode::Insert);
    }

    #[test]
    fn mode_from_visual_variants() {
        assert_eq!(Mode::from_mode_str(b"v"), Mode::Visual);
        assert_eq!(Mode::from_mode_str(b"V"), Mode::Visual);
        assert_eq!(Mode::from_mode_str(&[0x16]), Mode::Visual); // CTRL-V
    }

    #[test]
    fn mode_from_replace() {
        assert_eq!(Mode::from_mode_str(b"R"), Mode::Replace);
    }

    #[test]
    fn mode_from_command() {
        assert_eq!(Mode::from_mode_str(b"c"), Mode::Command);
    }

    #[test]
    fn mode_from_terminal() {
        assert_eq!(Mode::from_mode_str(b"t"), Mode::Terminal);
    }

    #[test]
    fn mode_from_select_variants() {
        assert_eq!(Mode::from_mode_str(b"s"), Mode::Select);
        assert_eq!(Mode::from_mode_str(b"S"), Mode::Select);
        assert_eq!(Mode::from_mode_str(&[0x13]), Mode::Select); // CTRL-S
    }

    #[test]
    fn mode_from_empty_defaults_normal() {
        assert_eq!(Mode::from_mode_str(b""), Mode::Normal);
    }

    #[test]
    fn mode_from_unknown_defaults_normal() {
        assert_eq!(Mode::from_mode_str(b"x"), Mode::Normal);
    }

    #[test]
    fn mode_labels() {
        assert_eq!(Mode::Normal.label(), "NORMAL");
        assert_eq!(Mode::Insert.label(), "INSERT");
        assert_eq!(Mode::Visual.label(), "VISUAL");
        assert_eq!(Mode::Replace.label(), "REPLACE");
        assert_eq!(Mode::Command.label(), "COMMAND");
        assert_eq!(Mode::Terminal.label(), "TERMINAL");
        assert_eq!(Mode::Select.label(), "SELECT");
    }

    #[test]
    fn mode_hl_groups_are_distinct() {
        let groups: Vec<&str> = ALL_MODES.iter().map(|m| m.hl_group()).collect();
        for (i, a) in groups.iter().enumerate() {
            for b in &groups[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn all_modes_covered() {
        assert_eq!(ALL_MODES.len(), 7);
    }
}
