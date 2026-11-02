use mlua::Value;
use v_ide_editor::autocmd::{Data, DataNode};

use super::{Codec, error, measure};

enum Task<'a> {
    Node(&'a DataNode),
    Array(usize),
    Dictionary(Vec<&'a [u8]>),
}

impl Codec {
    pub fn decode(&self, data: &Data) -> mlua::Result<Value> {
        let _invocation = self.budget.enter();
        measure::charge(data, &self.budget)?;
        let lua = self.lua()?;
        let mut tasks = vec![Task::Node(data.root())];
        let mut values = Vec::new();
        while let Some(task) = tasks.pop() {
            let value = match task {
                Task::Node(node) => match node {
                    DataNode::Null => Value::NULL,
                    DataNode::Boolean(value) => Value::Boolean(*value),
                    DataNode::Integer(value) => Value::Integer(*value),
                    DataNode::Float(value) => Value::Number(*value),
                    DataNode::String(value) => Value::String(lua.create_string(value)?),
                    DataNode::Function(reference) => Value::Function(self.function(*reference)?),
                    DataNode::Array(children) => {
                        tasks.push(Task::Array(children.len()));
                        for child in children.iter().rev() {
                            tasks.push(Task::Node(
                                data.get(*child)
                                    .ok_or_else(|| error("Invalid array value"))?,
                            ));
                        }
                        continue;
                    }
                    DataNode::Dictionary(entries) => {
                        tasks.push(Task::Dictionary(
                            entries.iter().map(|(key, _)| key.as_slice()).collect(),
                        ));
                        for (_, child) in entries.iter().rev() {
                            tasks.push(Task::Node(
                                data.get(*child)
                                    .ok_or_else(|| error("Invalid dictionary value"))?,
                            ));
                        }
                        continue;
                    }
                },
                Task::Array(length) => {
                    let children = take(&mut values, length)?;
                    let table = lua.create_table_with_capacity(length, 0)?;
                    for (index, value) in children.into_iter().enumerate() {
                        table.raw_set(index + 1, value)?;
                    }
                    Value::Table(table)
                }
                Task::Dictionary(keys) => {
                    let children = take(&mut values, keys.len())?;
                    let table = lua.create_table_with_capacity(0, keys.len())?;
                    if keys.is_empty() {
                        table.set_metatable(Some(self.empty_dict.clone()))?;
                    }
                    for (key, value) in keys.into_iter().zip(children) {
                        table.raw_set(lua.create_string(key)?, value)?;
                    }
                    Value::Table(table)
                }
            };
            values.push(value);
        }
        values.pop().ok_or_else(|| error("Missing decoded value"))
    }
}

fn take(values: &mut Vec<Value>, count: usize) -> mlua::Result<Vec<Value>> {
    let start = values
        .len()
        .checked_sub(count)
        .ok_or_else(|| error("Invalid conversion stack"))?;
    Ok(values.split_off(start))
}
