//! Tasuki (襷) — configurable statusline for Neovim with mode, file, git, and diagnostics sections
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.
//!
//! # Architecture
//!
//! The statusline is rendered via a `%{%…%}` expression that Neovim
//! re-evaluates on every redraw. The expression calls back into Rust
//! through `v:lua.require('tasuki').render_statusline()`, which:
//!
//! 1. Reads current mode, buffer name, cursor position, etc.
//! 2. Passes them to [`render::render`] as a pure [`render::StatuslineData`]
//! 3. Returns a string with `%#HlGroup#` highlight switches
//!
//! Highlight groups are created at init and refreshed on `ColorScheme`.

pub mod components;
pub mod render;
pub mod theme;

use nvim_oxi as oxi;
use nvim_oxi::api;
use nvim_oxi::api::opts::OptionOpts;

use render::StatuslineData;
use theme::Mode;

/// Gather current state from Neovim and return a rendered statusline string.
///
/// Called on every statusline redraw via the `%{%…%}` expression.
fn render_statusline() -> oxi::Result<String> {
    let got_mode = api::get_mode()?;
    let mode = Mode::from_mode_str(got_mode.mode.as_bytes());

    // Re-apply mode highlight so colors track the current mode.
    // Only the mode-specific group needs updating per-redraw;
    // the rest are stable until ColorScheme changes.
    tane::highlight::Highlight::new(mode.hl_group())
        .fg(mode.fg())
        .bg(mode.bg())
        .bold()
        .apply()
        .ok();

    let buf = api::get_current_buf();
    let win = api::get_current_win();

    let filepath = buf
        .get_name()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let modified = buf_modified(&buf);

    let git_branch = get_git_branch();

    let diagnostics = get_diagnostics(&buf);

    let (line, col) = win.get_cursor().unwrap_or((1, 0));
    // nvim_win_get_cursor returns (1-based line, 0-based col)
    let col = col + 1;

    let filetype = get_buf_option_string(&buf, "filetype");
    let encoding = get_buf_option_string(&buf, "fileencoding");
    let encoding = if encoding.is_empty() {
        get_global_option_string("encoding")
    } else {
        encoding
    };

    let data = StatuslineData {
        mode,
        filepath,
        modified,
        git_branch,
        diagnostics,
        line,
        col,
        filetype,
        encoding,
    };

    Ok(render::render(&data))
}

/// Check if a buffer is modified.
fn buf_modified(buf: &api::Buffer) -> bool {
    let opts = OptionOpts::builder().buffer(buf.clone()).build();
    api::get_option_value::<bool>("modified", &opts).unwrap_or(false)
}

/// Get a string buffer-local option, returning empty string on failure.
fn get_buf_option_string(buf: &api::Buffer, name: &str) -> String {
    let opts = OptionOpts::builder().buffer(buf.clone()).build();
    api::get_option_value::<String>(name, &opts).unwrap_or_default()
}

/// Get a global string option.
fn get_global_option_string(name: &str) -> String {
    let opts = OptionOpts::builder().build();
    api::get_option_value::<String>(name, &opts).unwrap_or_default()
}

/// Retrieve the current git branch via `b:gitsigns_head` (gitsigns.nvim).
///
/// If the plugin is not available, returns an empty string.
fn get_git_branch() -> String {
    // Try buffer-local gitsigns variable
    let buf = api::get_current_buf();
    if let Ok(branch) = buf.get_var::<String>("gitsigns_head") {
        if !branch.is_empty() {
            return branch;
        }
    }
    String::new()
}

/// Retrieve LSP diagnostic counts for the given buffer.
///
/// Uses `luaeval()` via Neovim's eval API since nvim-oxi does not expose
/// the diagnostics API directly.
fn get_diagnostics(buf: &api::Buffer) -> components::DiagnosticCounts {
    let bufnr = buf.handle();

    // Use luaeval() to call vim.diagnostic.count() and return a
    // comma-separated string for easy parsing.
    let lua_expr = format!(
        concat!(
            "(function() ",
            "local ok, counts = pcall(vim.diagnostic.count, {bufnr}) ",
            "if not ok or not counts then return '0,0,0,0' end ",
            "local e = counts[vim.diagnostic.severity.ERROR] or 0 ",
            "local w = counts[vim.diagnostic.severity.WARN] or 0 ",
            "local i = counts[vim.diagnostic.severity.INFO] or 0 ",
            "local h = counts[vim.diagnostic.severity.HINT] or 0 ",
            "return string.format('%d,%d,%d,%d', e, w, i, h) ",
            "end)()",
        ),
        bufnr = bufnr,
    );

    let result: String = api::eval(&lua_expr).unwrap_or_default();
    parse_diagnostic_string(&result)
}

/// Parse the `"e,w,i,h"` diagnostic string into counts.
fn parse_diagnostic_string(s: &str) -> components::DiagnosticCounts {
    let parts: Vec<u32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();

    components::DiagnosticCounts {
        errors: parts.first().copied().unwrap_or(0),
        warnings: parts.get(1).copied().unwrap_or(0),
        info: parts.get(2).copied().unwrap_or(0),
        hints: parts.get(3).copied().unwrap_or(0),
    }
}

/// Convert a `tane::Error` into an `oxi::Error`.
fn tane_to_oxi(err: tane::Error) -> oxi::Error {
    oxi::Error::Api(nvim_oxi::api::Error::Other(err.to_string()))
}

/// Plugin entry point.
///
/// Sets up highlight groups, registers autocommands for `ColorScheme`,
/// exports `render_statusline` to Lua, and sets the global statusline option.
#[oxi::plugin]
fn tasuki() -> oxi::Result<oxi::Dictionary> {
    // Set up highlight groups.
    theme::setup_highlights().map_err(tane_to_oxi)?;

    // Re-create highlights when the colorscheme changes.
    tane::autocmd::Autocmd::on(&["ColorScheme"])
        .pattern("*")
        .group("tasuki")
        .desc("Tasuki: refresh statusline highlights on colorscheme change")
        .register(|_args| {
            theme::setup_highlights()?;
            Ok(false)
        })
        .map_err(tane_to_oxi)?;

    // Build the render function to export via the Lua module table.
    let render_fn: oxi::Function<(), String> =
        oxi::Function::from_fn(|(): ()| render_statusline());

    // Set the global statusline to use our expression.
    let stl = render::statusline_expr();
    let opts = OptionOpts::builder().build();
    api::set_option_value("statusline", stl, &opts)?;

    // Use a global statusline (one line at the bottom, not per-window).
    api::set_option_value("laststatus", 3i64, &opts)?;

    // Return the module table — #[oxi::plugin] exports it as require('tasuki').
    let mut module = oxi::Dictionary::new();
    module.insert("render_statusline", oxi::Object::from(render_fn));

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_diagnostic_string_valid() {
        let counts = parse_diagnostic_string("3,1,0,2");
        assert_eq!(counts.errors, 3);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.info, 0);
        assert_eq!(counts.hints, 2);
    }

    #[test]
    fn parse_diagnostic_string_empty() {
        let counts = parse_diagnostic_string("");
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 0);
        assert_eq!(counts.info, 0);
        assert_eq!(counts.hints, 0);
    }

    #[test]
    fn parse_diagnostic_string_partial() {
        let counts = parse_diagnostic_string("5,2");
        assert_eq!(counts.errors, 5);
        assert_eq!(counts.warnings, 2);
        assert_eq!(counts.info, 0);
        assert_eq!(counts.hints, 0);
    }

    #[test]
    fn parse_diagnostic_string_with_spaces() {
        let counts = parse_diagnostic_string(" 1 , 2 , 3 , 4 ");
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 2);
        assert_eq!(counts.info, 3);
        assert_eq!(counts.hints, 4);
    }

    #[test]
    fn parse_diagnostic_string_invalid() {
        let counts = parse_diagnostic_string("abc,def");
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 0);
    }
}
