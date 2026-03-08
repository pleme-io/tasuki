//! Tasuki (襷) — configurable statusline for Neovim with mode, file, git, and diagnostics sections
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.

use nvim_oxi as oxi;

#[oxi::plugin]
fn tasuki() -> oxi::Result<()> {
    Ok(())
}
