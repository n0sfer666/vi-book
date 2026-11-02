local MODES = { n = "normal", i = "insert", v = "visual", V = "visual_line", ["\22"] = "visual_block" }

return function(options)
  local row, byte = unpack(vim.api.nvim_win_get_cursor(0))
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
  local mode = vim.fn.mode(1)
  assert(MODES[mode], "unsupported final mode " .. vim.inspect(mode))
  local result = {
    text = table.concat(lines, "\n"),
    cursor = { row - 1, vim.fn.charidx(lines[row], byte, true) },
    mode = MODES[mode],
    unnamed = vim.fn.getreg('"'),
  }
  if options then
    result.options = {
      timeoutlen = vim.o.timeoutlen,
      readonly = vim.bo.readonly,
      global_readonly = vim.go.readonly,
      number = vim.wo.number,
      global_number = vim.go.number,
    }
  end
  return result
end
