vibook={o=vim.o,bo=vim.bo}
for _,name in ipairs({'create_augroup','del_augroup_by_id','get_autocmds','create_autocmd'})do vibook[name]=vim.api['nvim_'..name]end
vibook.cmd=function(command)vim.cmd(command)end
vibook.buf={get_text=function()return table.concat(vim.api.nvim_buf_get_lines(0,0,-1,false),'\n')end,
 set_text=function(text)vim.api.nvim_buf_set_lines(0,0,-1,false,vim.split(text,'\n',{plain=true}))end}
vim.api.nvim_buf_set_name(0,'target/first.md');vim.api.nvim_buf_set_lines(0,0,-1,false,{'seed'})
dofile('crates/lua/tests/autocmd/ex.lua')
print('native Ex/scoped core contract passed')
dofile('crates/lua/tests/autocmd/ex_shapes.lua')
print('native Ex shapes contract passed')
