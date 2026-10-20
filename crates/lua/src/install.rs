use mlua::Table;
use v_ide_editor::module::VaultQuery;

use crate::config::Shared;
use crate::error::Error;
use crate::runtime::Runtime;

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
        lua.create_function(move |_, ()| query(&notes, |v| v.notes()))?,
    )?;
    let tags = bridge.clone();
    vault.set(
        "tags",
        lua.create_function(move |_, ()| query(&tags, |v| v.tags()))?,
    )?;
    let back = bridge.clone();
    vault.set(
        "backlinks",
        lua.create_function(move |_, path: String| query(&back, |v| v.backlinks(&path)))?,
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

pub fn install_open(rt: &Runtime, bridge: &Shared) -> Result<(), Error> {
    let lua = rt.lua();
    let vibook = lua.globals().get::<Table>("vibook")?;
    let open = bridge.clone();
    vibook.set(
        "open",
        lua.create_function(move |_, target: String| {
            open.borrow_mut().opens.push(target);
            Ok(())
        })?,
    )?;
    let external = bridge.clone();
    vibook.set(
        "open_external",
        lua.create_function(move |_, target: String| {
            external.borrow_mut().external_opens.push(target);
            Ok(())
        })?,
    )?;
    Ok(())
}

fn query(
    bridge: &Shared,
    run: impl Fn(&dyn VaultQuery) -> Result<Vec<String>, String>,
) -> mlua::Result<Vec<String>> {
    let vault = bridge.borrow().vault.clone();
    vault
        .map_or(Ok(Vec::new()), |vault| run(vault.as_ref()))
        .map_err(mlua::Error::RuntimeError)
}
