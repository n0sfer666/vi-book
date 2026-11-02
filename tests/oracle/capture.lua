local INPUT_KEYS = { name = true, before = true, cursor = true, keys = true, setup = true, maps = true, lua = true, options = true, chunks = true, screen = true, messages = true, divergence = true, expect = true }

local snapshot_path = vim.fs.joinpath(vim.fs.dirname(vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p")), "capture_state.lua")
local snapshot = dofile(snapshot_path)

local function fail(err)
  io.stderr:write(tostring(err), "\n")
  os.exit(1)
end

local function run_child(case_path, out_path)
  local done = false
  local options = false
  local function finish(scheduled)
    if done then
      return
    end
    done = true
    local ok, err = pcall(function()
      assert(not scheduled or vim.fn.mode(1) == "i", "keys left an incomplete command")
      vim.fn.writefile({ vim.json.encode(snapshot(options)) }, out_path)
    end)
    if not ok then
      fail(err)
    end
    vim.cmd("qa!")
  end
  local ok, err = pcall(function()
    local case = vim.json.decode(table.concat(vim.fn.readfile(case_path), "\n"))
    options = case.options or false
    local lines = vim.split(case.before, "\n", { plain = true })
    vim.bo.undolevels = -1
    vim.api.nvim_buf_set_lines(0, 0, -1, true, lines)
    vim.bo.undolevels = -123456
    local line = lines[case.cursor[1] + 1]
    vim.api.nvim_win_set_cursor(0, { case.cursor[1] + 1, vim.fn.byteidxcomp(line, case.cursor[2]) })
    for _, map in ipairs(case.maps or {}) do
      if case.lua then
        vim.keymap.set(map.mode, map.lhs, map.rhs, { remap = not map.noremap, nowait = map.nowait or false })
      else
        vim.api.nvim_set_keymap(map.mode, map.lhs, map.rhs, { noremap = map.noremap, nowait = map.nowait or false })
      end
    end
    if case.setup then assert(loadstring("local vibook = vim\n" .. case.setup))() end
    vim.schedule(function()
      finish(true)
    end)
    vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(case.keys, true, false, true), "tx!", false)
  end)
  if not ok then
    fail(err)
  end
  finish(false)
end

local MAP_MODES = { n = true, i = true, x = true, o = true, c = true }
local MAP_KEYS = { mode = true, lhs = true, rhs = true, noremap = true, nowait = true }

local function validate_map(name, map)
  for key in pairs(map) do
    assert(MAP_KEYS[key], name .. ": unknown map key " .. key)
  end
  assert(MAP_MODES[map.mode], name .. ": unsupported map mode " .. tostring(map.mode))
  assert(type(map.lhs) == "string" and map.lhs ~= "", name .. ": map lhs must be a non-empty string")
  assert(type(map.rhs) == "string", name .. ": map rhs must be a string")
  assert(type(map.noremap) == "boolean", name .. ": map noremap must be a boolean")
  assert(map.nowait == nil or type(map.nowait) == "boolean", name .. ": nowait must be a boolean")
end

local function validate(case, seen)
  for key in pairs(case) do
    assert(INPUT_KEYS[key], tostring(case.name) .. ": unknown key " .. key)
  end
  assert(not seen[case.name], case.name .. ": duplicate name")
  seen[case.name] = true
  assert(not case.before:find("\r", 1, true), case.name .. ": before must not contain \\r")
  assert(case.before:sub(-1) ~= "\n", case.name .. ": before must not end with \\n")
  if case.chunks then
    assert(#case.chunks > 0 and table.concat(case.chunks) == case.keys, case.name .. ": chunks must concatenate to keys")
  end
  assert(case.setup == nil or type(case.setup) == "string", case.name .. ": setup must be a string")
  assert(case.options == nil or type(case.options) == "boolean", case.name .. ": options must be a boolean")
  assert(case.lua == nil or type(case.lua) == "boolean", case.name .. ": lua must be a boolean")
  assert(not case.lua or (not case.chunks and not case.screen), case.name .. ": Lua cases use the headless capture path")
  for _, map in ipairs(case.maps or {}) do
    validate_map(case.name, map)
  end
end

local function capture(script, case)
  if case.chunks and not case.screen then
    return dofile(vim.fs.joinpath(vim.fs.dirname(script), "capture_chunks.lua"))(case, snapshot_path)
  end
  local root = vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(script)))
  local tmp = vim.fs.joinpath(root, "target", "tmp", "oracle-capture-" .. vim.fn.getpid())
  vim.fn.mkdir(tmp, "p")
  if case.screen then
    return dofile(vim.fs.joinpath(vim.fs.dirname(script), "capture_pty.lua"))(case, snapshot_path, tmp)
  end
  local case_path, out_path = vim.fs.joinpath(tmp, "case.json"), vim.fs.joinpath(tmp, "snapshot.json")
  vim.fn.writefile({ vim.json.encode(case) }, case_path)
  local cmd = { vim.v.progpath, "--clean", "--headless", "-i", "NONE", "-l", script, "--child", case_path, out_path }
  local result = vim.system(cmd, { stdin = false }):wait(10000)
  local reason = result.code == 124 and "timed out" or ("exited with " .. result.code)
  assert(result.code == 0, case.name .. ": nvim " .. reason .. "\n" .. (result.stderr or ""))
  local expect = vim.json.decode(table.concat(vim.fn.readfile(out_path), "\n"))
  vim.fn.delete(case_path)
  vim.fn.delete(out_path)
  return expect
end

local function str(value)
  return vim.json.encode(value)
end

local function pos(value)
  return "[" .. value[1] .. ", " .. value[2] .. "]"
end


local function render_case(case)
  local e = case.expect
  local out = {
    "    {",
    '      "name": ' .. str(case.name) .. ",",
    '      "before": ' .. str(case.before) .. ",",
    '      "cursor": ' .. pos(case.cursor) .. ",",
    '      "keys": ' .. str(case.keys) .. ",",
  }
  dofile(vim.fs.joinpath(vim.fs.dirname(snapshot_path), "capture_fields.lua"))(out, case)
  if case.divergence then
    table.insert(out, '      "divergence": ' .. str(case.divergence) .. ",")
  end
  vim.list_extend(out, {
    '      "expect": {',
    '        "text": ' .. str(e.text) .. ",",
    '        "cursor": ' .. pos(e.cursor) .. ",",
    '        "mode": ' .. str(e.mode) .. ",",
    '        "unnamed": ' .. str(e.unnamed),
    "      }",
    "    }",
  })
  if e.options then
    out[#out - 2] = out[#out - 2] .. ","
    local fields = {}
    for _, name in ipairs({ "timeoutlen", "readonly", "global_readonly", "number", "global_number" }) do
      table.insert(fields, str(name) .. ": " .. str(e.options[name]))
    end
    table.insert(out, #out - 1, '        "options": { ' .. table.concat(fields, ", ") .. ' }')
  end
  return out
end

local function render(version, cases)
  local out = { "{", '  "nvim_version": ' .. str(version) .. ",", '  "cases": [' }
  for i, case in ipairs(cases) do
    local lines = render_case(case)
    if i < #cases then
      lines[#lines] = lines[#lines] .. ","
    end
    vim.list_extend(out, lines)
  end
  vim.list_extend(out, { "  ]", "}" })
  return out
end

local function run_parent(script)
  local dir = vim.fs.dirname(script)
  local v = vim.version()
  local version = v.major .. "." .. v.minor .. "." .. v.patch
  for name in vim.fs.dir(dir) do
    if name:match("%.json$") then
      local path = vim.fs.joinpath(dir, name)
      local fixture = vim.json.decode(table.concat(vim.fn.readfile(path), "\n"))
      local seen = {}
      for _, case in ipairs(fixture.cases) do
        validate(case, seen)
        case.expect = capture(script, case)
      end
      vim.fn.writefile(render(version, fixture.cases), path)
      io.stdout:write(name, ": ", #fixture.cases, " cases\n")
    end
  end
end

local script = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p")
if arg[1] == "--child" then
  run_child(arg[2], arg[3])
else
  run_parent(script)
end
