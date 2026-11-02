local group = vibook.create_augroup('ShapeValidation', {})
local id = vibook.create_autocmd('User', {group=group, command=''})
for _, key in ipairs({'event','pattern','buffer'}) do
  local opts = {group=group}
  opts[key] = vibook.empty_dict()
  assert(not pcall(vibook.get_autocmds, opts))
  assert(not pcall(vibook.clear_autocmds, opts))
end
assert(not pcall(vibook.create_autocmd, 'User', {pattern=vibook.empty_dict(), command=''}))
assert(not pcall(vibook.create_autocmd, vibook.empty_dict(), {command=''}))
assert(not pcall(vibook.get_autocmds, {}, 'extra'))
assert(not pcall(vibook.clear_autocmds, {}, 'extra'))
assert(not pcall(vibook.create_autocmd, 'User', {command=''}, 'extra'))
assert(not pcall(vibook.create_augroup, 'UnexpectedExtra', {}, 'extra'))
assert(not pcall(vibook.del_autocmd, id, 'extra'))
assert(not pcall(vibook.del_augroup_by_id, group, 'extra'))
assert(not pcall(vibook.del_augroup_by_name, 'ShapeValidation', 'extra'))
assert(#vibook.get_autocmds({id=id}) == 1)
vibook.del_augroup_by_id(group)
