local group = vibook.create_augroup('ExecContract', {})
local shared = {value=3}
local fn = function() return 17 end
local input = {a=shared,b=shared,fn=fn,dict=vibook.empty_dict(),null=vibook.NIL}
local calls = 0
vibook.create_autocmd('User', {group=group, pattern='one,two', callback=function(a)
  calls=calls+1
  assert(a.event == 'User' and a.buf == 1 and a.group == group)
  assert(a.file == a.match and (a.match == 'one' or a.match == 'two'))
  assert(a.data.fn == fn and a.data.fn() == 17)
  assert(a.data.a ~= a.data.b and a.data.a.value == 3 and a.data.b.value == 3)
  assert(a.data.null == vibook.NIL)
  assert(getmetatable(a.data.dict) == getmetatable(vibook.empty_dict()))
  a.data.a.value=99
end})
assert(vibook.exec_autocmds('User', {group=group,pattern='one,two',modeline=false,data=input}) == nil)
assert(calls == 2 and input.a.value == 3)
vibook.clear_autocmds({group=group})
local nested=0
vibook.create_autocmd('User',{group=group,pattern='inner',callback=function(a)
  nested=nested+1
  assert(a.data() == 'temporary')
end})
vibook.create_autocmd('User',{group=group,pattern='outer',callback=function(a)
  assert(a.data == nil)
  vibook.exec_autocmds('User',{group=group,pattern='inner',modeline=false,data=function()return 'temporary'end})
end})
vibook.exec_autocmds('User',{group=group,pattern='outer',modeline=false})
assert(nested == 1)
local id=vibook.create_autocmd('User',{group=group,pattern='truthy,keep',callback=function()return 0 end})
vibook.exec_autocmds('User',{group=group,pattern='truthy',modeline=false})
local rows=vibook.get_autocmds({id=id})
assert(#rows == 1 and rows[1].pattern == 'keep')
local omitted=0
vibook.create_autocmd('User',{group=group,pattern='null',callback=function(a)
  omitted=omitted+1
  if omitted == 1 then assert(a.data == nil) else assert(a.data == vibook.NIL) end
end})
vibook.exec_autocmds('User',{group=group,pattern='null',modeline=false})
vibook.exec_autocmds('User',{group=group,pattern='null',modeline=false,data=vibook.NIL})
assert(omitted == 2)
local bad={};bad.self=bad
assert(not pcall(vibook.exec_autocmds,'User',{group=group,data=bad}))
assert(not pcall(vibook.exec_autocmds,'User',{group=group,data={[1]=1,key=2}}))
assert(not pcall(vibook.exec_autocmds,'User',{group=group,pattern=vibook.empty_dict()}))
assert(not pcall(vibook.exec_autocmds,'User',{buf=0,buffer=0}))
assert(not pcall(vibook.exec_autocmds,'User',{pattern='one',buffer=0}))
assert(not pcall(vibook.exec_autocmds,'User',{modeline='yes'}))
assert(not pcall(vibook.exec_autocmds,'User',{},'extra'))
vibook.del_augroup_by_id(group)
