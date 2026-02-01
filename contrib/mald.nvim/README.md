# mald.nvim

Neovim integration for [MALD](https://github.com/NAME0x0/MALD) PKM.

## Requirements

- `mald` binary in PATH
- Neovim 0.9+
- [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) (optional, recommended)

## Install

### lazy.nvim

```lua
{ dir = "path/to/MALD/contrib/mald.nvim" }
```

### Manual

Symlink or copy `contrib/mald.nvim` into your Neovim plugin directory.

## Setup

```lua
require("mald").setup({
  mald_bin = "mald",  -- path to mald binary (default: "mald")
})
```

## Commands

| Command | Description |
|---|---|
| `:MaldSearch [query]` | Search notes (telescope if available, else vim.ui.select) |
| `:MaldCapture text` | Quick capture to daily note |
| `:MaldToday` | Open today's daily note |
| `:MaldTasks` | Show open tasks in quickfix list |
| `:MaldLinks` | Outgoing wikilinks from current file |
| `:MaldBacklinks` | Notes that link to current file |

## Keybindings (suggested)

```lua
vim.keymap.set("n", "<leader>ms", ":MaldSearch<CR>", { desc = "MALD search" })
vim.keymap.set("n", "<leader>mc", ":MaldCapture ", { desc = "MALD capture" })
vim.keymap.set("n", "<leader>mt", ":MaldToday<CR>", { desc = "MALD today" })
vim.keymap.set("n", "<leader>mk", ":MaldTasks<CR>", { desc = "MALD tasks" })
vim.keymap.set("n", "<leader>ml", ":MaldLinks<CR>", { desc = "MALD links" })
vim.keymap.set("n", "<leader>mb", ":MaldBacklinks<CR>", { desc = "MALD backlinks" })
```
