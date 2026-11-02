local cases = {
 {'default', {'vim: set number:'}},
 {'vi', {'vi:number'}}, {'ex_start', {'ex:number'}}, {'ex_space', {' ex:number'}},
 {'upper', {'Vim: set number:'}}, {'upper_no_set', {' Vim: number'}},
 {'prefix', {'xvim: number'}}, {'multiple', {'vim: nonumber vim: number'}},
 {'version_old', {'vim800: number'}}, {'version_new', {'vim900: number'}},
 {'version_lt', {'vim<802: number'}}, {'version_eq', {'vim=801: number'}},
 {'version_gt', {'vim>801: number'}}, {'version_huge', {'vim999999999999999999999: number'}},
 {'reset', {'vim: set number&:'}}, {'all_reset', {'vim: set all&:'}},
 {'list', {'vim: set all:'}}, {'query', {'vim: number?'}},
 {'se', {'vim: se number: trailing'}}, {'missing_colon', {'vim: set number'}},
 {'pipe', {'vim: set number|set readonly:'}}, {'segments', {'vi:number:readonly'}},
 {'tail_ignored', {'vim: set number: trailing'}}, {'unknown', {'vim: set number unknown_xyz readonly:', 'vim: set timeoutlen=29:'}},
 {'disable', {'vim: set nomodeline number:', 'vim: set readonly:'}},
 {'global', {'vim: set timeoutlen=27:'}}, {'negative_mls', {'vim: set modelines=-1:'}},
 {'changed_mls', {'vim: set modelines=1:', 'vim: set readonly:'}},
 {'head_tail', {'vim: set readonly:', '', '', '', '', '', 'vim: set number:'}},
 {'middle_ignored', {'', '', '', '', '', 'vim: set readonly:', '', '', '', '', ''}},
 {'escaped_colon', {'vim: set number\\:readonly:'}},
}
cases[#cases+1]={'nested_set', {'vi:number:set readonly:'}}
cases[#cases+1]={'nested_set_unfinished', {'vi:number:set readonly'}}
cases[#cases+1]={'nested_se', {'vim: number: se readonly:'}}
cases[#cases+1]={'tail_reverse', {'','','','','','vim: set nonumber:','vim: set number readonly:'}}
cases[#cases+1]={'tail_disable', {'','','','','','vim: set number:','vim: set nomodeline:'}}
local function rust(s)
 return '"' .. s:gsub('\\','\\\\'):gsub('"','\\"'):gsub('\n','\\n'):gsub('\r','\\r'):gsub('\t','\\t') .. '"'
end
local group = vim.api.nvim_create_augroup('ModelineCapture', {})
vim.api.nvim_create_autocmd('User', {group=group, pattern='Hit', command=''})
local output = {
 'pub type Case = (', "    &'static str,", "    &'static [&'static str],",
 '    bool,', '    bool,', '    bool,', '    i64,', '    i64,', '    bool,', '    usize,', ');', '',
 '#[rustfmt::skip]', 'pub const CASES: &[Case] = &['}
for _, case in ipairs(cases) do
 vim.bo.modeline=true; vim.o.modelines=5; vim.wo.number=false; vim.bo.readonly=false; vim.o.timeoutlen=1000
 vim.api.nvim_buf_set_lines(0, 0, -1, false, case[2])
 local seen=0
 local observer=vim.api.nvim_create_autocmd('OptionSet',{callback=function()seen=seen+1 end})
 vim.v.errmsg=''
 vim.api.nvim_exec_autocmds('User', {group=group, pattern='Hit'})
 vim.api.nvim_del_autocmd(observer)
 local lines={};for _,line in ipairs(case[2])do lines[#lines+1]=rust(line)end
 output[#output+1]=string.format('    (%s, &[%s], %s, %s, %s, %d, %d, %s, %d),',
 rust(case[1]),table.concat(lines,', '),tostring(vim.wo.number),tostring(vim.bo.readonly),tostring(vim.bo.modeline),vim.o.timeoutlen,vim.o.modelines,tostring(vim.v.errmsg~=''),seen)
end
output[#output+1]='];'
vim.fn.writefile(output, 'crates/editor/tests/fixtures/modelines.rs')
