local replies = {
  { "\27[c", "\27[?62;22c" }, { "\27[0c", "\27[?62;22c" },
  { "\27[5n", "\27[0n" }, { "\27[6n", "\27[1;1R" },
  { "\27]10;?", "\27]10;rgb:ffff/ffff/ffff\27\\" },
  { "\27]11;?", "\27]11;rgb:0000/0000/0000\27\\" },
}

local terminal = {
  up = "\27[A", down = "\27[B", right = "\27[C", left = "\27[D",
  pageup = "\27[5~", pagedown = "\27[6~", home = "\27[H", ["end"] = "\27[F",
}

return function(case, snapshot_path, tmp)
  local case_path = vim.fs.joinpath(tmp, "pty-case.json")
  local input = vim.fs.joinpath(tmp, "input.txt")
  local output = vim.fs.joinpath(tmp, "pty-snapshot.json")
  local ready = vim.fs.joinpath(tmp, "ready")
  local init = vim.fs.joinpath(tmp, "init.lua")
  local child = vim.fs.joinpath(vim.fs.dirname(snapshot_path), "capture_pty_child.lua")
  vim.fn.delete(output)
  vim.fn.delete(ready)
  vim.fn.writefile({ vim.json.encode(case) }, case_path)
  vim.fn.writefile(vim.split(case.before, "\n", { plain = true }), input, "b")
  vim.fn.writefile({ string.format("dofile(%q)(%q, %q, %q, %q)", child, case_path, output, ready, snapshot_path) }, init)
  local chunks = vim.tbl_map(function(chunk)
    local keys = chunk:gsub("<([^>]+)>", function(name) return terminal[name:lower()] or ("<" .. name .. ">") end)
    return vim.api.nvim_replace_termcodes(keys, true, false, true)
  end, case.chunks or { case.keys })
  local job
  local transcript = {}
  job = vim.fn.jobstart({ vim.v.progpath, "--noplugin", "-u", init, "-i", "NONE", "-n", input }, {
    pty = true, width = case.screen[1], height = case.screen[2],
    env = {
      HOME = tmp, TERM = "xterm-256color", NVIM_LOG_FILE = vim.fs.joinpath(tmp, "nvim.log"),
      XDG_CONFIG_HOME = vim.fs.joinpath(tmp, "config"), XDG_DATA_HOME = vim.fs.joinpath(tmp, "data"),
      XDG_STATE_HOME = vim.fs.joinpath(tmp, "state"), NVIM = "", VIMINIT = "",
    },
    on_stdout = function(_, data)
      local text = table.concat(data, "\n")
      table.insert(transcript, text)
      local answers = {}
      for _, pair in ipairs(replies) do
        local at = text:find(pair[1], 1, true)
        if at then table.insert(answers, { at, pair[2] }) end
      end
      table.sort(answers, function(a, b) return a[1] < b[1] end)
      for _, answer in ipairs(answers) do vim.fn.chansend(job, answer[2]) end
    end,
  })
  assert(job > 0, "could not start PTY nvim")
  local pid = vim.fn.jobpid(job)
  local watchdog = vim.uv.new_timer()
  watchdog:start(10000, 0, function() vim.uv.kill(pid, 9) end)
  local ok, result = pcall(function()
    assert(vim.wait(5000, function() return vim.fn.filereadable(ready) == 1 end, 10), "nvim startup did not finish")
    for _, keys in ipairs(chunks) do
      vim.fn.chansend(job, keys)
      vim.wait(80, function() return false end)
    end
    vim.fn.chansend(job, "\27[24~")
    assert(vim.wait(3000, function() return vim.fn.filereadable(output) == 1 end, 10), "input left a prompt or incomplete command")
    local snapshot = vim.json.decode(table.concat(vim.fn.readfile(output), "\n"))
    assert(not snapshot.error, snapshot.error)
    return snapshot
  end)
  watchdog:stop()
  watchdog:close()
  vim.uv.kill(pid, 9)
  vim.fn.jobwait({ job }, 1000)
  vim.fn.writefile({ vim.inspect(table.concat(transcript)) }, vim.fs.joinpath(tmp, "pty-transcript.txt"))
  assert(ok, case.name .. ": " .. tostring(result))
  return result
end
