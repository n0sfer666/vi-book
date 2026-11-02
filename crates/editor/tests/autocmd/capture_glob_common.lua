return function(patterns, names, output)
local function rust(value)
 local out = {'"'}
 for _, cp in ipairs(vim.fn.str2list(value, true)) do
  if cp == 34 then out[#out + 1] = '\\"'
  elseif cp == 92 then out[#out + 1] = '\\\\'
  elseif cp < 32 or cp >= 127 then out[#out + 1] = string.format('\\u{%x}', cp)
  else out[#out + 1] = string.char(cp) end
 end
 out[#out + 1] = '"'
 return table.concat(out)
end
local lines = {'#[rustfmt::skip]', 'pub const NAMES: &[&str] = &['}
for _,name in ipairs(names)do table.insert(lines,'    '..rust(name)..',')end
table.insert(lines,'];')
table.insert(lines,'#[rustfmt::skip]')
table.insert(lines,'pub const CASES: &[(&str, bool, &[usize])] = &[')
local buffers={}
for index,name in ipairs(names)do
 if name~='' and name~='.' and not name:find('/',1,true)then
  local b=vim.api.nvim_create_buf(false,true)
  vim.api.nvim_buf_set_name(b,'target/tmp/glob-gold/'..index..'/'..name)
  buffers[index]=b
 end
end
for _,pattern in ipairs(patterns)do
 for _,ignore in ipairs({false,true})do
  vim.o.fileignorecase=ignore
  vim.api.nvim_clear_autocmds({})
  local hit=false
  vim.api.nvim_create_autocmd('User',{pattern=pattern,callback=function()hit=true end})
  local hits={}
  for index,name in ipairs(names)do
   hit=false
   if not pattern:find('/',1,true) and buffers[index]then
    vim.api.nvim_exec_autocmds('User',{buffer=buffers[index],modeline=false})
   else
    vim.api.nvim_exec_autocmds('User',{pattern=name,modeline=false})
   end
   if hit then table.insert(hits,index-1)end
  end
  table.insert(lines,'    ('..rust(pattern)..', '..tostring(ignore)..', &['..table.concat(hits,', ')..']),')
 end
end
table.insert(lines,'];')
vim.fn.writefile(lines, output)
end
