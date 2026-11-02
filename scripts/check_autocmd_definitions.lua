local names = {'create_augroup','del_augroup_by_id','del_augroup_by_name',
  'create_autocmd','del_autocmd','get_autocmds','clear_autocmds'}
vibook = {empty_dict=vim.empty_dict}
for _, name in ipairs(names) do vibook[name] = vim.api['nvim_' .. name] end
dofile('crates/lua/tests/autocmd/definitions.lua')
print('native autocmd definitions contract passed')
dofile('crates/lua/tests/autocmd/definition_edges.lua')
dofile('crates/lua/tests/autocmd/buffer_patterns.lua')
dofile('crates/lua/tests/autocmd/validation.lua')
print('native definition edges and buffer contracts passed')
target_buffer = vim.api.nvim_create_buf(false, true)
vim.api.nvim_buf_set_name(target_buffer, 'ActiveBuffer')
dofile('crates/lua/tests/autocmd/active_buffer.lua')
vim.api.nvim_exec_autocmds('User', {buffer=target_buffer, modeline=false})
assert(active_buffer_result == target_buffer)
dofile('crates/lua/tests/autocmd/named_callables.lua')
vim.api.nvim_exec_autocmds('User', {pattern='NamedCallable', modeline=false})
vim.api.nvim_exec_autocmds('User', {pattern='NamedCallable', modeline=false})
assert(callable_calls == 2 and #vibook.get_autocmds({group='NamedCallables'}) == 1)
