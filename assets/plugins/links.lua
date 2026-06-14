-- links.lua — default vi-book plugin.
--
-- Classifies the target of an internal [[link]] when you follow it:
--   * text notes  (.md / .markdown / .txt)  open inside vi-book
--   * code files  (.rs / .py / .js / ...)    open in your external editor
--   * bare names  ([[note]] with no extension) are left to vi-book's own
--     resolver (handles aliases, anchors and name collisions)
--
-- Configure the external editor:
--   vibook.opt.code_editor = "nvim"   -- falls back to $EDITOR
--
-- Disable this plugin:
--   vibook.opt.plugins = { links = false }

local text_ext = {
  md = true,
  markdown = true,
  txt = true,
  text = true,
  org = true,
  rst = true,
}

local function disabled()
  return vibook.opt.plugins ~= nil and vibook.opt.plugins.links == false
end

local function extension(target)
  return target:match("%.([%w]+)$")
end

vibook.on("follow_link", function(target)
  if disabled() then
    return
  end
  local ext = extension(target)
  if ext == nil then
    return
  end
  if text_ext[ext:lower()] then
    vibook.open(target)
  else
    vibook.open_external(target)
  end
end)
