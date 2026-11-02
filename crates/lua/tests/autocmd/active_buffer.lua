vibook.create_autocmd('User', {pattern='ActiveBuffer', once=true, callback=function(a)
  assert(a.buf == target_buffer)
  local id = vibook.create_autocmd('User', {pattern='<buffer=abuf>', command=''})
  local row = vibook.get_autocmds({id=id})[1]
  assert(row.buffer == target_buffer)
  assert(#vibook.get_autocmds({id=id, pattern='<buffer=abuf>'}) == 1)
  active_buffer_result = row.buffer
end})
