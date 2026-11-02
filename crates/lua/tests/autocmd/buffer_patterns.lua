local group = vibook.create_augroup('BufferPatterns', {})
for _, pattern in ipairs({'<buffer>', '<buffer=0>', '<buffer=1>'}) do
  local id = vibook.create_autocmd('User', {group=group, pattern=pattern, command=''})
  local row = vibook.get_autocmds({id=id})[1]
  assert(row.buflocal and row.buffer == 1 and row.buf == 1 and row.pattern == '<buffer=1>')
  assert(#vibook.get_autocmds({id=id, pattern='<buffer>'}) == 1)
end
local ok = pcall(vibook.create_autocmd, 'User', {group=group, pattern='one,<buffer=2000>', command=''})
assert(not ok)
assert(#vibook.get_autocmds({group=group, pattern='one'}) == 1)
vibook.del_augroup_by_name('BufferPatterns')
