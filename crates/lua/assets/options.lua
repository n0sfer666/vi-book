local api = vibook
local names = { number = true, nu = true, readonly = true, ro = true, timeoutlen = true, tm = true }
for _, interface in ipairs({ "o", "go", "bo", "wo" }) do
  api[interface] = setmetatable({}, {
    __index = function(_, name) return api.__option_get(interface, name) end,
    __newindex = function(_, name, value) api.__option_set(interface, name, value) end,
  })
end
local function accessor(interface, legacy)
  local object = {}
  local function make(name, value)
    return setmetatable({ _name = name, _value = value }, object)
  end
  local function arithmetic(self, right, operation)
    if operation == "^" then error("The '^' operator is not currently supported for numeric options") end
    if type(self._value) ~= "number" then error("arithmetic is not supported for this option") end
    if operation == "+" then return self._value * 1.0 + right end
    return self._value * 1.0 - right
  end
  function object:get() return self._value end
  function object:_set() api.__option_set(interface, self._name, self._value) end
  function object:append(right)
    self._value = arithmetic(self, right, "+")
    self:_set()
  end
  function object:remove(right)
    self._value = arithmetic(self, right, "-")
    self:_set()
  end
  function object:prepend(right)
    self._value = arithmetic(self, right, "^")
    self:_set()
  end
  object.__index = object
  object.__add = function(self, right) return make(self._name, arithmetic(self, right, "+")) end
  object.__sub = function(self, right) return make(self._name, arithmetic(self, right, "-")) end
  object.__pow = function(self, right) return make(self._name, arithmetic(self, right, "^")) end
  return setmetatable(legacy and api.opt or {}, {
    __index = function(_, name)
      if legacy and not names[name] then return nil end
      return make(name, api.__option_get(interface, name))
    end,
    __newindex = function(table, name, value)
      if legacy and not names[name] then rawset(table, name, value); return end
      if type(value) == "table" and getmetatable(value) == object then
        assert(name == value._name, "option object must have the same name")
        value = value._value
      end
      api.__option_set(interface, name, value)
    end,
  })
end
api.opt = accessor("opt", true)
api.opt_local = accessor("opt_local", false)
api.opt_global = accessor("opt_global", false)
