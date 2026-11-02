return function(case_path, output, ready, snapshot_path)
  local case = vim.json.decode(table.concat(vim.fn.readfile(case_path), "\n"))
  local snapshot = dofile(snapshot_path)
  vim.o.swapfile = false
  vim.o.verbose = 0
  vim.o.messagesopt = "wait:0,history:1000"
  vim.o.more = false
  vim.o.ttimeout = true
  vim.o.ttimeoutlen = 20
  vim.api.nvim_create_autocmd("VimEnter", { once = true, callback = function()
    vim.cmd("mapclear")
    vim.cmd("mapclear!")
    vim.cmd("messages clear")
    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
    local byte = vim.fn.byteidxcomp(lines[case.cursor[1] + 1], case.cursor[2])
    vim.api.nvim_win_set_cursor(0, { case.cursor[1] + 1, byte })
    for _, map in ipairs(case.maps or {}) do
      vim.api.nvim_set_keymap(map.mode, map.lhs, map.rhs, { noremap = map.noremap, nowait = map.nowait or false })
    end
    for _, text in ipairs(case.messages or {}) do vim.api.nvim_echo({ { text } }, true, {}) end
    vim.keymap.set("n", "<F12>", function()
      local ok, value = pcall(snapshot, case.options)
      vim.fn.writefile({ vim.json.encode(ok and value or { error = tostring(value) }) }, output)
      vim.cmd("qa!")
    end, { silent = true })
    vim.schedule(function()
      vim.cmd("redraw!")
      vim.o.more = true
      vim.o.messagesopt = "hit-enter,history:1000"
      vim.fn.writefile({ "ready" }, ready)
    end)
  end })
end
