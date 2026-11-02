use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionRef {
    pub runtime: RuntimeId,
    pub function: FunctionId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataNode {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(Vec<u8>),
    Array(Vec<ValueId>),
    Dictionary(Vec<(Vec<u8>, ValueId)>),
    Function(FunctionRef),
}

#[derive(Debug, Clone)]
pub struct Data {
    nodes: Rc<[DataNode]>,
    root: ValueId,
}

impl Data {
    pub fn root(&self) -> &DataNode {
        &self.nodes[self.root.0]
    }

    pub fn get(&self, id: ValueId) -> Option<&DataNode> {
        self.nodes.get(id.0)
    }
}

#[derive(Debug, Default)]
pub struct DataBuilder {
    nodes: Vec<DataNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid data value id: {0:?}")]
pub struct DataError(ValueId);

impl DataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, node: DataNode) -> Result<ValueId, DataError> {
        match &node {
            DataNode::Array(children) => {
                for &id in children {
                    self.validate(id)?;
                }
            }
            DataNode::Dictionary(entries) => {
                for &(_, id) in entries {
                    self.validate(id)?;
                }
            }
            _ => {}
        }
        let id = ValueId(self.nodes.len());
        self.nodes.push(node);
        Ok(id)
    }

    pub fn finish(self, root: ValueId) -> Result<Data, DataError> {
        self.validate(root)?;
        Ok(Data {
            nodes: self.nodes.into(),
            root,
        })
    }

    fn validate(&self, id: ValueId) -> Result<(), DataError> {
        if id.0 < self.nodes.len() {
            Ok(())
        } else {
            Err(DataError(id))
        }
    }
}
