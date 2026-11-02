local group = vibook.create_augroup('NamedCallables', {})
callable_calls=0
Callable=setmetatable({}, {__call=function(self,...)
  assert(self == Callable and select('#',...) == 0)
  callable_calls=callable_calls+1
  return true
end})
vibook.create_autocmd('User', {group=group, pattern='NamedCallable', callback='v:lua.Callable'})
