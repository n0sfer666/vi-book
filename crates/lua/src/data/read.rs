use std::{
    collections::{HashMap, HashSet},
    ffi::c_void,
};

use mlua::Value;
use v_ide_editor::autocmd::{DataBuilder, DataNode, ValueId};

use super::{
    Codec, Encoded, error,
    rows::{self, Rows},
};

enum Task {
    Value(Value),
    Array {
        pointer: *const c_void,
        length: usize,
        positions: Vec<usize>,
    },
    Dictionary {
        pointer: *const c_void,
        keys: Vec<Vec<u8>>,
    },
}

impl Codec {
    pub fn encode(&self, value: Value) -> mlua::Result<Encoded> {
        let lua = self.lua()?;
        let _root = value.clone();
        let marker = self.empty_dict.to_pointer();
        let mut tasks = vec![Task::Value(value)];
        let mut builder = DataBuilder::new();
        let mut values = Vec::new();
        let mut memo = HashMap::new();
        let mut active = HashSet::new();
        let mut functions = Vec::new();
        let mut tables = Vec::new();
        while let Some(task) = tasks.pop() {
            let node = match task {
                Task::Value(value) => match value {
                    Value::Nil => DataNode::Null,
                    Value::LightUserData(pointer) if pointer.0.is_null() => DataNode::Null,
                    Value::Boolean(value) => DataNode::Boolean(value),
                    Value::Integer(value) => DataNode::Integer(value),
                    Value::Number(value) => lua
                        .coerce_integer(Value::Number(value))?
                        .map_or(DataNode::Float(value), DataNode::Integer),
                    Value::String(value) => DataNode::String(value.as_bytes().to_vec()),
                    Value::Function(function) => {
                        let stored = self.keep_function(function);
                        let reference = stored.reference;
                        functions.push(stored);
                        DataNode::Function(reference)
                    }
                    Value::Table(table) => {
                        let pointer = table.to_pointer();
                        if active.contains(&pointer) {
                            return Err(error("Lua failed to grow stack"));
                        }
                        if let Some(&id) = memo.get(&pointer) {
                            values.push(id);
                            continue;
                        }
                        tables.push(table.clone());
                        let empty = table
                            .metatable()
                            .is_some_and(|mt| mt.to_pointer() == marker);
                        let rows = rows::read(&table, empty)?;
                        active.insert(pointer);
                        let children = match rows {
                            Rows::Array {
                                length,
                                positions,
                                values,
                            } => {
                                tasks.push(Task::Array {
                                    pointer,
                                    length,
                                    positions,
                                });
                                values
                            }
                            Rows::Dictionary { keys, values } => {
                                tasks.push(Task::Dictionary { pointer, keys });
                                values
                            }
                        };
                        tasks.extend(children.into_iter().rev().map(Task::Value));
                        continue;
                    }
                    Value::UserData(_) | Value::LightUserData(_) | Value::Error(_) => {
                        return Err(error("Cannot convert userdata"));
                    }
                    _ => return Err(error("Cannot convert given Lua type")),
                },
                Task::Array {
                    pointer,
                    length,
                    positions,
                } => {
                    let children = take(&mut values, positions.len())?;
                    let mut array = Vec::new();
                    array
                        .try_reserve_exact(length)
                        .map_err(|_| error("Cannot allocate Lua array"))?;
                    if length > positions.len() {
                        let null = builder
                            .push(DataNode::Null)
                            .map_err(|err| error(&err.to_string()))?;
                        array.resize(length, null);
                        for (position, child) in positions.into_iter().zip(children) {
                            array[position] = child;
                        }
                    } else {
                        array.extend(children);
                    }
                    let id = builder
                        .push(DataNode::Array(array))
                        .map_err(|err| error(&err.to_string()))?;
                    memo.insert(pointer, id);
                    active.remove(&pointer);
                    values.push(id);
                    continue;
                }
                Task::Dictionary { pointer, keys } => {
                    let children = take(&mut values, keys.len())?;
                    let id = builder
                        .push(DataNode::Dictionary(
                            keys.into_iter().zip(children).collect(),
                        ))
                        .map_err(|err| error(&err.to_string()))?;
                    memo.insert(pointer, id);
                    active.remove(&pointer);
                    values.push(id);
                    continue;
                }
            };
            values.push(builder.push(node).map_err(|err| error(&err.to_string()))?);
        }
        let root = values.pop().ok_or_else(|| error("Missing data value"))?;
        Ok(Encoded {
            data: builder
                .finish(root)
                .map_err(|err| error(&err.to_string()))?,
            _functions: functions,
        })
    }
}

fn take(values: &mut Vec<ValueId>, count: usize) -> mlua::Result<Vec<ValueId>> {
    let start = values
        .len()
        .checked_sub(count)
        .ok_or_else(|| error("Invalid conversion stack"))?;
    Ok(values.split_off(start))
}
