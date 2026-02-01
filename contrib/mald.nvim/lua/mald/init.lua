-- mald.nvim — Neovim integration for MALD PKM
--
-- Install: add to lazy.nvim or packer as a local plugin:
--   { dir = "path/to/MALD/contrib/mald.nvim" }
--
-- Or symlink into your neovim plugin directory.
--
-- Requires: mald binary in PATH, telescope.nvim (optional but recommended)

local M = {}

-- Configuration
M.config = {
  mald_bin = "mald",
}

function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})

  -- Register commands
  vim.api.nvim_create_user_command("MaldSearch", function(args)
    M.search(args.args)
  end, { nargs = "?", desc = "Search MALD notes" })

  vim.api.nvim_create_user_command("MaldCapture", function(args)
    M.capture(args.args)
  end, { nargs = "+", desc = "Quick capture to daily note" })

  vim.api.nvim_create_user_command("MaldToday", function()
    M.today()
  end, { desc = "Open today's daily note" })

  vim.api.nvim_create_user_command("MaldTasks", function()
    M.tasks()
  end, { desc = "Show open tasks in quickfix" })

  vim.api.nvim_create_user_command("MaldLinks", function()
    M.links()
  end, { desc = "Show wikilinks from current file" })

  vim.api.nvim_create_user_command("MaldBacklinks", function()
    M.backlinks()
  end, { desc = "Show backlinks to current file" })
end

-- Search notes using telescope (falls back to vim.ui.select)
function M.search(query)
  local ok, telescope = pcall(require, "telescope")
  if ok then
    M._telescope_search(query)
  else
    M._fallback_search(query)
  end
end

function M._telescope_search(initial_query)
  local pickers = require("telescope.pickers")
  local finders = require("telescope.finders")
  local conf = require("telescope.config").values
  local actions = require("telescope.actions")
  local action_state = require("telescope.actions.state")

  pickers.new({}, {
    prompt_title = "MALD Search",
    default_text = initial_query or "",
    finder = finders.new_async_job({
      command_generator = function(prompt)
        if not prompt or prompt == "" then
          return nil
        end
        return { M.config.mald_bin, "search", prompt, "--json" }
      end,
      entry_maker = function(line)
        local ok, item = pcall(vim.json.decode, line)
        if not ok or type(item) ~= "table" then
          return nil
        end
        -- Handle both single items and array
        local items = item
        if item.path then
          items = { item }
        end
        -- This gets called per-line of JSON output; handle array format
        return nil
      end,
    }),
    -- Use a simpler approach: run search, parse results, show them
    finder = finders.new_oneshot_job(
      vim.list_extend(
        { M.config.mald_bin, "search", initial_query or "", "--json" },
        {}
      ),
      {
        entry_maker = function(line)
          -- The JSON output is pretty-printed, so we collect it and parse
          return {
            value = line,
            display = line,
            ordinal = line,
          }
        end,
      }
    ),
    sorter = conf.generic_sorter({}),
    attach_mappings = function(prompt_bufnr)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
        local selection = action_state.get_selected_entry()
        if selection and selection.value then
          -- Try to extract path from JSON
          local path = selection.value:match('"path"%s*:%s*"([^"]+)"')
          if path then
            vim.cmd("edit " .. vim.fn.fnameescape(path))
          end
        end
      end)
      return true
    end,
  }):find()
end

-- Simple telescope-free approach using mald search --json
function M._fallback_search(query)
  if not query or query == "" then
    query = vim.fn.input("Search: ")
    if query == "" then return end
  end

  local cmd = string.format("%s search %s --json", M.config.mald_bin, vim.fn.shellescape(query))
  local output = vim.fn.system(cmd)
  local ok, results = pcall(vim.json.decode, output)

  if not ok or type(results) ~= "table" or #results == 0 then
    vim.notify("No results for: " .. query, vim.log.levels.INFO)
    return
  end

  local items = {}
  for _, r in ipairs(results) do
    table.insert(items, {
      display = (r.title or r.path or ""),
      path = r.path,
    })
  end

  vim.ui.select(items, {
    prompt = "Open note:",
    format_item = function(item) return item.display end,
  }, function(choice)
    if choice and choice.path then
      vim.cmd("edit " .. vim.fn.fnameescape(choice.path))
    end
  end)
end

-- Quick capture
function M.capture(text)
  if not text or text == "" then
    text = vim.fn.input("Capture: ")
    if text == "" then return end
  end
  local cmd = string.format("%s q %s", M.config.mald_bin, text)
  local output = vim.fn.system(cmd)
  vim.notify(vim.trim(output), vim.log.levels.INFO)
end

-- Open today's daily note
function M.today()
  -- Get the daily note path by running mald with a helper
  local today = os.date("%Y-%m-%d")
  local home = vim.fn.system(M.config.mald_bin .. " config get default_kb"):gsub("%s+$", "")
  if home == "" then home = "personal" end

  -- Determine MALD home
  local mald_home = os.getenv("MALD_HOME")
  if not mald_home then
    if vim.fn.has("win32") == 1 then
      mald_home = os.getenv("USERPROFILE") .. "\\.mald"
    else
      mald_home = os.getenv("HOME") .. "/.mald"
    end
  end

  local path = mald_home .. "/kb/" .. home .. "/" .. today .. ".md"
  vim.cmd("edit " .. vim.fn.fnameescape(path))
end

-- Show open tasks in quickfix list
function M.tasks()
  local cmd = string.format("%s tasks --json", M.config.mald_bin)
  local output = vim.fn.system(cmd)
  local ok, tasks = pcall(vim.json.decode, output)

  if not ok or type(tasks) ~= "table" or #tasks == 0 then
    vim.notify("No open tasks", vim.log.levels.INFO)
    return
  end

  local qf_items = {}
  for _, t in ipairs(tasks) do
    table.insert(qf_items, {
      text = string.format("[ ] %s (%s/%s)", t.task, t.kb, t.note),
    })
  end

  vim.fn.setqflist(qf_items, "r")
  vim.cmd("copen")
end

-- Show outgoing wikilinks from current file
function M.links()
  local file = vim.fn.expand("%:t:r") -- current filename without extension
  if file == "" then
    vim.notify("No file open", vim.log.levels.WARN)
    return
  end
  local cmd = string.format("%s links %s", M.config.mald_bin, vim.fn.shellescape(file))
  local output = vim.fn.system(cmd)
  vim.notify(output, vim.log.levels.INFO)
end

-- Show backlinks to current file
function M.backlinks()
  local file = vim.fn.expand("%:t:r")
  if file == "" then
    vim.notify("No file open", vim.log.levels.WARN)
    return
  end
  local cmd = string.format("%s backlinks %s", M.config.mald_bin, vim.fn.shellescape(file))
  local output = vim.fn.system(cmd)
  vim.notify(output, vim.log.levels.INFO)
end

return M
