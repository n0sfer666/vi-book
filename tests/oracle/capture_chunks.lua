return function(case, snapshot_path)
  local channel = vim.fn.jobstart({ vim.v.progpath, "--headless", "--embed", "-n", "-u", "NONE", "-i", "NONE" }, { rpc = true })
  assert(channel > 0, "could not start embedded nvim")
  local function request(method, ...)
    return vim.fn.rpcrequest(channel, method, ...)
  end
  local ok, result = pcall(function()
    request("nvim_set_option_value", "timeoutlen", 1000, {})
    local lines = vim.split(case.before, "\n", { plain = true })
    request("nvim_buf_set_lines", 0, 0, -1, true, lines)
    local byte = vim.fn.byteidxcomp(lines[case.cursor[1] + 1], case.cursor[2])
    request("nvim_win_set_cursor", 0, { case.cursor[1] + 1, byte })
    for _, map in ipairs(case.maps or {}) do
      request("nvim_set_keymap", map.mode, map.lhs, map.rhs, { noremap = map.noremap, nowait = map.nowait or false })
    end
    if case.setup then request("nvim_exec_lua", "local vibook = vim\n" .. case.setup, {}) end
    local chunks = vim.tbl_map(function(chunk)
      return request("nvim_replace_termcodes", chunk, true, false, true)
    end, case.chunks)
    for _, keys in ipairs(chunks) do
      assert(request("nvim_input", keys) == #keys, "nvim_input was truncated")
      vim.wait(40, function() return false end)
      request("nvim_get_mode")
    end
    vim.wait(1100, function() return false end)
    return request("nvim_exec_lua", "local path, options = ...; return dofile(path)(options)", { snapshot_path, case.options or false })
  end)
  vim.fn.jobstop(channel)
  assert(ok, case.name .. ": " .. tostring(result))
  return result
end
