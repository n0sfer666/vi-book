use std::collections::HashMap;

use v_ide_editor::autocmd::{Data, DataNode};

use super::{
    budget::{Budget, Cost},
    error,
};

enum Task<'a> {
    Visit(&'a DataNode),
    Finish(&'a DataNode),
}

pub(super) fn charge(data: &Data, budget: &Budget) -> mlua::Result<()> {
    let mut costs: HashMap<*const DataNode, Cost> = HashMap::new();
    let mut tasks = vec![Task::Visit(data.root())];
    budget.charge(Cost::VALUE)?;
    while let Some(task) = tasks.pop() {
        let node = match task {
            Task::Visit(node) => {
                if let Some(cost) = costs.get(&(node as *const DataNode)) {
                    budget.charge(Cost {
                        values: cost.values - 1,
                        bytes: cost.bytes,
                    })?;
                    continue;
                }
                tasks.push(Task::Finish(node));
                match node {
                    DataNode::Array(children) => {
                        budget.charge(Cost::values(children.len()))?;
                        for child in children.iter().rev() {
                            tasks.push(Task::Visit(
                                data.get(*child)
                                    .ok_or_else(|| error("Invalid array value"))?,
                            ));
                        }
                    }
                    DataNode::Dictionary(entries) => {
                        budget.charge(Cost::values(entries.len()))?;
                        for (key, child) in entries.iter().rev() {
                            budget.charge(Cost::bytes(key.len()))?;
                            tasks.push(Task::Visit(
                                data.get(*child)
                                    .ok_or_else(|| error("Invalid dictionary value"))?,
                            ));
                        }
                    }
                    _ => {}
                }
                continue;
            }
            Task::Finish(node) => node,
        };
        let mut cost = Cost::VALUE;
        match node {
            DataNode::String(value) => {
                let bytes = Cost::bytes(value.len());
                budget.charge(bytes)?;
                cost = cost.add(bytes);
            }
            DataNode::Array(children) => {
                for child in children {
                    let child = data
                        .get(*child)
                        .ok_or_else(|| error("Invalid array value"))?;
                    cost = cost.add(costs[&(child as *const DataNode)]);
                }
            }
            DataNode::Dictionary(entries) => {
                for (key, child) in entries {
                    let child = data
                        .get(*child)
                        .ok_or_else(|| error("Invalid dictionary value"))?;
                    cost = cost
                        .add(Cost::bytes(key.len()))
                        .add(costs[&(child as *const DataNode)]);
                }
            }
            _ => {}
        }
        costs.insert(node as *const DataNode, cost);
    }
    Ok(())
}
