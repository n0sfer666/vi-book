for _,row in ipairs(vim.api.nvim_get_autocmds({}))do pcall(vim.api.nvim_del_autocmd,row.id)end
local source=table.concat(vim.fn.readfile('crates/editor/src/autocmd/event.rs'),'\n')
local body=source:match('const NAMES:.-= &%[(.-)%];')
local relative={};local absolute={};local errors={}
for event in body:gmatch('"([%w]+)"')do
 local seen
 local id=vim.api.nvim_create_autocmd(event,{pattern='*',callback=function(a)seen=a end})
 local ok,r=pcall(vim.api.nvim_exec_autocmds,event,{pattern='relative-name',modeline=false})
 vim.api.nvim_del_autocmd(id)
 if not ok or not seen then errors[#errors+1]=event..':'..tostring(r)
 elseif seen.match=='relative-name'then relative[#relative+1]=event
 elseif seen.match==vim.fn.fnamemodify('relative-name',':p')then absolute[#absolute+1]=event
 else errors[#errors+1]=event..':'..seen.match end
end
local output={'#[rustfmt::skip]', 'pub const CASES: &[(&str, bool)] = &['}
for _,event in ipairs(relative)do output[#output+1]='    ("'..event..'", false),'end
for _,event in ipairs(absolute)do output[#output+1]='    ("'..event..'", true),'end
assert(#errors==0,table.concat(errors,'\n'))
output[#output+1]='];'
vim.fn.writefile(output,'crates/editor/tests/fixtures/autocmd_names.rs')
local root=vim.fn.getcwd()
local parent=vim.fn.fnamemodify(root,':h')
local paths={'./name','././name','dir/./name','a//b','//a/b','~/name','~abc/name','http://a/b',
 'term://a/b','file:///a/b','../name','./../name','.','target/../child','target/./child','target//child'}
local rows={'#[rustfmt::skip]','pub const PATHS: &[(&str, &str)] = &['}
local match
vim.api.nvim_create_autocmd('BufEnter',{pattern='*',callback=function(a)match=a.match end})
for _,path in ipairs(paths)do
 vim.api.nvim_exec_autocmds('BufEnter',{pattern=path,modeline=false})
 if match:sub(1,#root)==root then match='$CWD'..match:sub(#root+1)
 elseif match:sub(1,#parent)==parent then match='$PARENT'..match:sub(#parent+1)end
 rows[#rows+1]='    ('..string.format('%q',path)..', '..string.format('%q',match)..'),'
end
rows[#rows+1]='];'
vim.fn.writefile(rows,'crates/editor/tests/fixtures/autocmd_paths.rs')
