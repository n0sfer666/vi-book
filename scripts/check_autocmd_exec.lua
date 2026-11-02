vibook={NIL=vim.NIL,empty_dict=vim.empty_dict}
for _,name in ipairs({'create_augroup','del_augroup_by_id','create_autocmd','del_autocmd','get_autocmds','clear_autocmds','exec_autocmds'})do
 vibook[name]=vim.api['nvim_'..name]
end
dofile('crates/lua/tests/autocmd/exec.lua')
print('native exec data/functions contract passed')
vibook.o=vim.o;vibook.go=vim.go;vibook.wo=vim.wo;vibook.bo=vim.bo
vim.api.nvim_buf_set_lines(0,0,-1,false,{'vim: set number readonly:'})
dofile('crates/lua/tests/autocmd/exec_modeline.lua')
dofile('crates/lua/tests/autocmd/exec_names.lua')
print('native exec modeline/order/names contracts passed')
