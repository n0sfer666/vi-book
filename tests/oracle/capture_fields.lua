local function str(value) return vim.json.encode(value) end

local function render_map(map)
  local fields = {
    '"mode": ' .. str(map.mode),
    '"lhs": ' .. str(map.lhs),
    '"rhs": ' .. str(map.rhs),
    '"noremap": ' .. tostring(map.noremap),
  }
  if map.nowait ~= nil then
    table.insert(fields, '"nowait": ' .. tostring(map.nowait))
  end
  return "        { " .. table.concat(fields, ", ") .. " }"
end

return function(out, case)
  if case.setup then table.insert(out, '      "setup": ' .. str(case.setup) .. ",") end
  if case.options ~= nil then
    table.insert(out, '      "options": ' .. tostring(case.options) .. ',')
  end
  if case.lua ~= nil then
    table.insert(out, '      "lua": ' .. tostring(case.lua) .. ',')
  end
  if case.chunks then
    local chunks = vim.tbl_map(str, case.chunks)
    table.insert(out, '      "chunks": [' .. table.concat(chunks, ", ") .. "],")
  end
  if case.maps then
    table.insert(out, '      "maps": [')
    for i, map in ipairs(case.maps) do
      table.insert(out, render_map(map) .. (i < #case.maps and "," or ""))
    end
    table.insert(out, "      ],")
  end
  if case.screen then
    table.insert(out, '      "screen": [' .. case.screen[1] .. ', ' .. case.screen[2] .. '],')
  end
  if case.messages then
    table.insert(out, '      "messages": [' .. table.concat(vim.tbl_map(vim.json.encode, case.messages), ", ") .. '],')
  end
end
