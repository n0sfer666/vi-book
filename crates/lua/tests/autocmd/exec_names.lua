local group=vibook.create_augroup('ExecNames',{})
local trace={}
vibook.create_autocmd({'User','BufEnter','FileType'},{group=group,pattern='dir/*',callback=function(a)
  assert(a.file == 'dir/child.md' and a.buf == 1)
  if a.event == 'BufEnter' then
    assert(a.match ~= a.file and a.match:sub(-13) == '/dir/child.md')
  else assert(a.match == a.file) end
  trace[#trace+1]=a.event
end})
vibook.exec_autocmds({'User','BufEnter','FileType'},{group=group,pattern='dir/child.md',modeline=false})
assert(table.concat(trace,',') == 'User,BufEnter,FileType')
vibook.del_augroup_by_id(group)
