vim.o.ignorecase = false
vim.o.fileignorecase = false
local classes = {lower = {}, upper = {}, print = {}}
local parents = {}
local function find(cp)
 local parent = parents[cp]
 if not parent then parents[cp] = cp; return cp end
 if parent ~= cp then parents[cp] = find(parent) end
 return parents[cp]
end
local function join(a, b)
 local left, right = find(a), find(b)
 if left < right then parents[right] = left else parents[left] = right end
end
local function range(rows, cp)
 local last = rows[#rows]
 if last and last[2] == cp - 1 then last[2] = cp
 else rows[#rows + 1] = {cp, cp} end
end
for cp = 1, 0x10ffff do
 if cp < 0xd800 or cp > 0xdfff then
  local ch = vim.fn.nr2char(cp)
  for name, rows in pairs(classes) do
   if vim.fn.match(ch, '^[[:' .. name .. ':]]$') == 0 then range(rows, cp) end
  end
  for _, other in ipairs({vim.fn.tolower(ch), vim.fn.toupper(ch)}) do
   if ch ~= other and vim.fn.strchars(other) == 1
    and vim.fn.match(other, '\\c\\V' .. ch) == 0 then
    join(cp, vim.fn.char2nr(other))
   end
  end
 end
end
local groups, keys = {}, {}
for cp in pairs(parents) do
 local root = find(cp)
 if not groups[root] then groups[root] = {}; keys[#keys + 1] = root end
 groups[root][#groups[root] + 1] = cp
end
table.sort(keys)
local chars = {}
for cp in pairs(parents) do chars[#chars + 1] = cp end
table.sort(chars)
local lines = {}
local function emit(value) lines[#lines + 1] = value end
local function char(cp) return string.format("'\\u{%x}'", cp) end
for _, name in ipairs({'lower', 'upper', 'print'}) do
 emit('#[rustfmt::skip]')
 emit('pub(super) const ' .. name:upper() .. ': &[(u32, u32)] = &[')
 for _, row in ipairs(classes[name]) do
  emit(string.format('    (0x%x, 0x%x),', row[1], row[2]))
 end
 emit('];')
end
emit('#[rustfmt::skip]')
emit('pub(super) const FOLD: &[(char, char)] = &[')
for _, cp in ipairs(chars) do
 if find(cp) ~= cp then emit('    (' .. char(cp) .. ', ' .. char(find(cp)) .. '),') end
end
emit('];')
emit('#[rustfmt::skip]')
emit('pub(super) const VARIANTS: &[(char, &[char])] = &[')
for _, root in ipairs(keys) do
 local members = groups[root]
 table.sort(members)
 local values = {}
 for _, cp in ipairs(members) do values[#values + 1] = char(cp) end
 emit('    (' .. char(root) .. ', &[' .. table.concat(values, ', ') .. ']),')
end
emit('];')
vim.fn.writefile(lines, 'crates/editor/src/autocmd/pattern/unicode_data.rs')
