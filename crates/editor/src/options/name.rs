#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Name {
    Timeoutlen,
    Readonly,
    Number,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Global,
    Buffer,
    Window,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Effective,
    Global,
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Number(i64),
}

impl Name {
    pub const ALL: [Self; 3] = [Self::Number, Self::Readonly, Self::Timeoutlen];

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "timeoutlen" | "tm" => Some(Self::Timeoutlen),
            "readonly" | "ro" => Some(Self::Readonly),
            "number" | "nu" => Some(Self::Number),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Timeoutlen => "timeoutlen",
            Self::Readonly => "readonly",
            Self::Number => "number",
        }
    }

    pub fn scope(self) -> Scope {
        match self {
            Self::Timeoutlen => Scope::Global,
            Self::Readonly => Scope::Buffer,
            Self::Number => Scope::Window,
        }
    }

    pub fn default_value(self) -> Value {
        match self {
            Self::Timeoutlen => Value::Number(1000),
            Self::Readonly | Self::Number => Value::Bool(false),
        }
    }

    pub fn format(self, value: Value) -> String {
        let name = self.as_str();
        match value {
            Value::Bool(true) => format!("  {name}"),
            Value::Bool(false) => format!("no{name}"),
            Value::Number(value) => format!("  {name}={value}"),
        }
    }
}
