local group=vibook.create_augroup('ExecModeline',{})
vibook.o.modeline=true;vibook.o.modelines=5
vibook.wo.number=false;vibook.bo.readonly=false
vibook.exec_autocmds('User',{group=group,pattern='Hit'})
assert(not vibook.wo.number)
local calls=0
vibook.create_autocmd({'User','BufEnter'},{group=group,pattern='*',callback=function()
  assert(not vibook.wo.number and not vibook.bo.readonly)
  calls=calls+1
end})
vibook.exec_autocmds('User',{group=group,pattern='Hit',modeline=false})
assert(calls == 1 and not vibook.wo.number)
local options=0
vibook.create_autocmd('OptionSet',{group=group,pattern='number,readonly',callback=function()options=options+1 end})
vibook.exec_autocmds({'User','BufEnter'},{group=group,pattern={'Hit','Other'}})
assert(calls == 5 and options == 2 and vibook.wo.number and vibook.bo.readonly)
assert(not vibook.go.number and not vibook.go.readonly)
vibook.wo.number=false;vibook.bo.readonly=false
assert(not pcall(vibook.exec_autocmds,{'User','Invalid'},{group=group,pattern='Hit'}))
assert(not vibook.wo.number and not vibook.bo.readonly)
vibook.bo.modeline=false
vibook.exec_autocmds('User',{group=group,pattern='Hit'})
assert(not vibook.wo.number)
vibook.bo.modeline=true;vibook.o.modelines=0
vibook.exec_autocmds('User',{group=group,pattern='Hit'})
assert(not vibook.wo.number)
vibook.del_augroup_by_id(group)
