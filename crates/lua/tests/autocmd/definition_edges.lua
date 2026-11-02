local group = vibook.create_augroup('DefinitionEdges', {})
local function rows() return vibook.get_autocmds({group=group}) end
assert(not pcall(vibook.create_autocmd, {'User', 'Invalid'}, {group=group, pattern='one', command=''}))
assert(#rows() == 1 and rows()[1].pattern == 'one')
vibook.clear_autocmds({group=group})
assert(not pcall(vibook.create_autocmd, 'User', {group=group, pattern={'one','{'}, command=''}))
assert(#rows() == 1 and rows()[1].pattern == 'one')
vibook.clear_autocmds({group=group})
assert(not pcall(vibook.create_autocmd, {}, {group=group, command=''}))
for _, pattern in ipairs({'', ',', {}}) do
  local id = vibook.create_autocmd('User', {group=group, pattern=pattern, command=''})
  assert(vibook.get_autocmds({id=id})[1].pattern == '*')
end
assert(#vibook.get_autocmds({group=group, id=-2}) == 0)
assert(#vibook.get_autocmds({group=group, buffer={}}) == 3)
local id = vibook.create_autocmd('User', {group=group, once=0.5, nested=1, command=''})
assert(vibook.get_autocmds({id=id})[1].once)
assert(not pcall(vibook.create_autocmd, 'User', {group=group, once='yes', command=''}))
vibook.del_augroup_by_id(group)
