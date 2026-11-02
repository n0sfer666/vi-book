#[rustfmt::skip]
pub const CASES: &[(&str, &str, &str, &str)] = &[
    ("null", "input=vibook.NIL", "return type(copy)..':'..tostring(copy==vibook.NIL)", "userdata:true"),
    ("binary", "input={['a\\0b']='v\\0\\255'}", "local k=next(copy);local v=copy[k];return #k..':'..#v..':'..v:byte(3)", "3:3:255"),
    ("sparse", "input={[5]='e'}", "return #copy..':'..tostring(copy[1]==vibook.NIL)..':'..copy[5]", "5:true:e"),
    ("shared", "local t={n=1};input={a=t,b=t}", "copy.a.n=2;return tostring(copy.a~=copy.b)..':'..copy.b.n..':'..input.a.n", "true:1:1"),
    ("metatable", "input=setmetatable({x=1},{__pairs=function()error('pairs')end,__index={hidden=true}})", "return tostring(getmetatable(copy)==nil)..':'..tostring(copy.hidden==nil)", "true:true"),
    ("empty_kinds", "input={a={},d=vibook.empty_dict(),fake=setmetatable({},{__jsontype='object'})}", "return tostring(getmetatable(copy.a)==nil)..':'..tostring(getmetatable(copy.d)==getmetatable(vibook.empty_dict()))..':'..tostring(getmetatable(copy.fake)==nil)", "true:true:true"),
    ("function", "input=function()return 'ok'end", "return tostring(copy==input)..':'..copy()", "true:ok"),
    ("nested_function", "input={a=math.abs,b=math.abs}", "return tostring(copy.a==input.a)..':'..tostring(copy.a==copy.b)..':'..copy.a(-3)", "true:true:3"),
    ("nonfinite", "input={p=math.huge,n=0/0}", "return tostring(copy.p==math.huge)..':'..tostring(copy.n~=copy.n)", "true:true"),
    ("negative_zero", "input=-0.0", "return tostring(1/input<0)..':'..tostring(1/copy<0)", "true:false"),
];
