local api = vibook
local function wrap(name, count, call)
  return function(...)
    if select('#', ...) ~= count then error(name .. ': Expected ' .. count .. ' arguments') end
    return call(...)
  end
end
api.create_autocmd = wrap('create_autocmd', 2, function(...) return api.__autocmd_create(...) end)
api.get_autocmds = wrap('get_autocmds', 1, function(...) return api.__autocmd_get(...) end)
api.clear_autocmds = wrap('clear_autocmds', 1, function(...) return api.__autocmd_clear(...) end)
api.exec_autocmds = wrap('exec_autocmds', 2, function(...) return api.__autocmd_exec(...) end)
for _, name in ipairs({'create_augroup', 'del_autocmd', 'del_augroup_by_id', 'del_augroup_by_name'}) do
  api[name] = wrap(name, name == 'create_augroup' and 2 or 1, api[name])
end
