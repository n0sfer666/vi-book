local cases=vim.json.decode([==[
[
 [
  "null",
  "input=vibook.NIL",
  "return type(copy)..':'..tostring(copy==vibook.NIL)"
 ],
 [
  "binary",
  "input={['a\\0b']='v\\0\\255'}",
  "local k=next(copy);local v=copy[k];return #k..':'..#v..':'..v:byte(3)"
 ],
 [
  "sparse",
  "input={[5]='e'}",
  "return #copy..':'..tostring(copy[1]==vibook.NIL)..':'..copy[5]"
 ],
 [
  "shared",
  "local t={n=1};input={a=t,b=t}",
  "copy.a.n=2;return tostring(copy.a~=copy.b)..':'..copy.b.n..':'..input.a.n"
 ],
 [
  "metatable",
  "input=setmetatable({x=1},{__pairs=function()error('pairs')end,__index={hidden=true}})",
  "return tostring(getmetatable(copy)==nil)..':'..tostring(copy.hidden==nil)"
 ],
 [
  "empty_kinds",
  "input={a={},d=vibook.empty_dict(),fake=setmetatable({},{__jsontype='object'})}",
  "return tostring(getmetatable(copy.a)==nil)..':'..tostring(getmetatable(copy.d)==getmetatable(vibook.empty_dict()))..':'..tostring(getmetatable(copy.fake)==nil)"
 ],
 [
  "function",
  "input=function()return 'ok'end",
  "return tostring(copy==input)..':'..copy()"
 ],
 [
  "nested_function",
  "input={a=math.abs,b=math.abs}",
  "return tostring(copy.a==input.a)..':'..tostring(copy.a==copy.b)..':'..copy.a(-3)"
 ],
 [
  "nonfinite",
  "input={p=math.huge,n=0/0}",
  "return tostring(copy.p==math.huge)..':'..tostring(copy.n~=copy.n)"
 ],
 [
  "negative_zero",
  "input=-0.0",
  "return tostring(1/input<0)..':'..tostring(1/copy<0)"
 ]
]
]==])
vibook={NIL=vim.NIL,empty_dict=vim.empty_dict}
local function rust(value)
 local out={'"'}
 for _,cp in ipairs(vim.fn.str2list(value,true))do
  if cp==34 then out[#out+1]='\\"'
  elseif cp==92 then out[#out+1]='\\\\'
  elseif cp<32 or cp>=127 then out[#out+1]=string.format('\\u{%x}',cp)
  else out[#out+1]=string.char(cp)end
 end
 out[#out+1]='"'
 return table.concat(out)
end
local lines={'#[rustfmt::skip]','pub const CASES: &[(&str, &str, &str, &str)] = &['}
for _,case in ipairs(cases)do
 input=nil;copy=nil
 vim.api.nvim_clear_autocmds({event='User'})
 assert(loadstring(case[2]))()
 vim.api.nvim_create_autocmd('User',{pattern='Codec',once=true,callback=function(a)copy=a.data end})
 vim.api.nvim_exec_autocmds('User',{pattern='Codec',data=input,modeline=false})
 local observed=assert(loadstring(case[3]))()
 table.insert(lines,'    ('..rust(case[1])..', '..rust(case[2])..', '..rust(case[3])..', '..rust(observed)..'),')
end
table.insert(lines,'];')
vim.fn.writefile(lines,'crates/lua/tests/fixtures/data_native.rs')
