use mlua::Table;

use crate::lua::config::Shared;
use crate::lua::error::Error;
use crate::lua::runtime::Runtime;

pub fn install_buf(rt: &Runtime, bridge: &Shared) -> Result<(), Error> {
    let lua = rt.lua();
    let buf = lua.create_table()?;
    let get = bridge.clone();
    buf.set(
        "get_text",
        lua.create_function(move |_, ()| Ok(get.borrow().buf_text.clone()))?,
    )?;
    let set = bridge.clone();
    buf.set(
        "set_text",
        lua.create_function(move |_, text: String| {
            set.borrow_mut().buf_text = text;
            Ok(())
        })?,
    )?;
    let cur = bridge.clone();
    buf.set(
        "get_cursor",
        lua.create_function(move |_, ()| Ok(cur.borrow().buf_cursor))?,
    )?;
    lua.globals().get::<Table>("vibook")?.set("buf", buf)?;
    Ok(())
}

pub fn install_vault(rt: &Runtime, bridge: &Shared) -> Result<(), Error> {
    let lua = rt.lua();
    let vault = lua.create_table()?;
    let notes = bridge.clone();
    vault.set(
        "notes",
        lua.create_function(move |_, ()| Ok(notes.borrow().vault_notes.clone()))?,
    )?;
    let tags = bridge.clone();
    vault.set(
        "tags",
        lua.create_function(move |_, ()| Ok(tags.borrow().vault_tags.clone()))?,
    )?;
    let back = bridge.clone();
    vault.set(
        "backlinks",
        lua.create_function(move |_, path: String| {
            Ok(back
                .borrow()
                .backlinks
                .get(&path)
                .cloned()
                .unwrap_or_default())
        })?,
    )?;
    let create = bridge.clone();
    vault.set(
        "create",
        lua.create_function(move |_, name: String| {
            create.borrow_mut().creates.push(name);
            Ok(())
        })?,
    )?;
    let open = bridge.clone();
    vault.set(
        "open",
        lua.create_function(move |_, name: String| {
            open.borrow_mut().opens.push(name);
            Ok(())
        })?,
    )?;
    lua.globals().get::<Table>("vibook")?.set("vault", vault)?;
    Ok(())
}
