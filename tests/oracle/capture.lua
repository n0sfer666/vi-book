local MODES = { n = "normal", i = "insert", v = "visual", V = "visual_line", ["\22"] = "visual_block" }
local INPUT_KEYS = { name = true, before = true, cursor = true, keys = true, maps = true, divergence = true, expect = true }

local function snapshot()
  local row, byte = unpack(vim.api.nvim_win_get_cursor(0))
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
  local mode = vim.fn.mode(1)
  assert(MODES[mode], "unsupported final mode " .. vim.inspect(mode))
  return {
    text = table.concat(lines, "\n"),
    cursor = { row - 1, vim.fn.charidx(lines[row], byte, true) },
    mode = MODES[mode],
    unnamed = vim.fn.getreg('"'),
  }
end

local function fail(err)
  io.stderr:write(tostring(err), "\n")
  os.exit(1)
end

local function run_child(case_path, out_path)
  local done = false
  local function finish(scheduled)
    if done then
      return
    end
    done = true
    local ok, err = pcall(function()
      assert(not scheduled or vim.fn.mode(1) == "i", "keys left an incomplete command")
      vim.fn.writefile({ vim.json.encode(snapshot()) }, out_path)
    end)
    if not ok then
      fail(err)
    end
    vim.cmd("qa!")
  end
  local ok, err = pcall(function()
    local case = vim.json.decode(table.concat(vim.fn.readfile(case_path), "\n"))
    local lines = vim.split(case.before, "\n", { plain = true })
    vim.bo.undolevels = -1
    vim.api.nvim_buf_set_lines(0, 0, -1, true, lines)
    vim.bo.undolevels = -123456
    local line = lines[case.cursor[1] + 1]
    vim.api.nvim_win_set_cursor(0, { case.cursor[1] + 1, vim.fn.byteidxcomp(line, case.cursor[2]) })
    for _, map in ipairs(case.maps or {}) do
      vim.api.nvim_set_keymap(map.mode, map.lhs, map.rhs, { noremap = map.noremap })
    end
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
local MAP_KEYS = { mode = true, lhs = true, rhs = true, noremap = true }

local function validate_map(name, map)
  for key in pairs(map) do
    assert(MAP_KEYS[key], name .. ": unknown map key " .. key)
  end
  assert(MAP_MODES[map.mode], name .. ": unsupported map mode " .. tostring(map.mode))
  assert(type(map.lhs) == "string" and map.lhs ~= "", name .. ": map lhs must be a non-empty string")
  assert(type(map.rhs) == "string", name .. ": map rhs must be a string")
  assert(type(map.noremap) == "boolean", name .. ": map noremap must be a boolean")
end

local function validate(case, seen)
  for key in pairs(case) do
    assert(INPUT_KEYS[key], tostring(case.name) .. ": unknown key " .. key)
  end
  assert(not seen[case.name], case.name .. ": duplicate name")
  seen[case.name] = true
  assert(not case.before:find("\r", 1, true), case.name .. ": before must not contain \\r")
  assert(case.before:sub(-1) ~= "\n", case.name .. ": before must not end with \\n")
  for _, map in ipairs(case.maps or {}) do
    validate_map(case.name, map)
  end
end

local function capture(script, case)
  local case_path, out_path = vim.fn.tempname(), vim.fn.tempname()
  vim.fn.writefile({ vim.json.encode(case) }, case_path)
  local cmd = { vim.v.progpath, "--clean", "--headless", "-l", script, "--child", case_path, out_path }
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

local function render_map(map)
  local fields = {
    '"mode": ' .. str(map.mode),
    '"lhs": ' .. str(map.lhs),
    '"rhs": ' .. str(map.rhs),
    '"noremap": ' .. tostring(map.noremap),
  }
  return "        { " .. table.concat(fields, ", ") .. " }"
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
  if case.maps then
    table.insert(out, '      "maps": [')
    for i, map in ipairs(case.maps) do
      table.insert(out, render_map(map) .. (i < #case.maps and "," or ""))
    end
    table.insert(out, "      ],")
  end
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
