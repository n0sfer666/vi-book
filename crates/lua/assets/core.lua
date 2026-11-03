local api=vibook
function api.buf.get_text(...) return api.__buffer_get(...) end
function api.buf.set_text(...) return api.__buffer_set(...) end
function api.buf.get_cursor(...) return api.__buffer_cursor(...) end
function api.cmd(...)
  if select('#',...) ~= 1 then error('cmd: Expected 1 argument') end
  return api.__command(...)
end
