use mlua::{Table, Value};

use super::{
    budget::{Budget, Cost},
    error,
};

pub(super) enum Rows {
    Array {
        length: usize,
        positions: Vec<usize>,
        values: Vec<Value>,
    },
    Dictionary {
        keys: Vec<Vec<u8>>,
        values: Vec<Value>,
    },
}

pub(super) fn read(table: &Table, empty_dict: bool, budget: &Budget) -> mlua::Result<Rows> {
    let mut numeric = Vec::new();
    let mut keys = Vec::new();
    let mut strings = Vec::new();
    for entry in table.pairs::<Value, Value>() {
        let (key, value) = entry?;
        budget.charge(Cost::VALUE)?;
        match key {
            Value::Integer(index) if index > 0 => {
                let index =
                    usize::try_from(index).map_err(|_| error("Cannot convert given Lua table"))?;
                budget.ensure(Cost::values(index.saturating_sub(numeric.len() + 1)))?;
                numeric.push((index, value));
            }
            Value::String(key) => {
                budget.charge(Cost::bytes(key.as_bytes().len()))?;
                keys.push(key.as_bytes().to_vec());
                strings.push(value);
            }
            _ => return Err(error("Cannot convert given Lua table")),
        }
    }
    if !numeric.is_empty() && !keys.is_empty() {
        return Err(error("Cannot convert given Lua table"));
    }
    if !keys.is_empty() || (numeric.is_empty() && empty_dict) {
        return Ok(Rows::Dictionary {
            keys,
            values: strings,
        });
    }
    numeric.sort_unstable_by_key(|(index, _)| *index);
    let length = numeric.last().map_or(0, |(index, _)| *index);
    let (positions, values) = numeric
        .into_iter()
        .map(|(index, value)| (index - 1, value))
        .unzip();
    Ok(Rows::Array {
        length,
        positions,
        values,
    })
}
