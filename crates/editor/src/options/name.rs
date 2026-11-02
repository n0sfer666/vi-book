#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Name {
    Timeoutlen,
    Readonly,
    Number,
    Modeline,
    Modelines,
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
    pub const ALL: [Self; 5] = [
        Self::Modeline,
        Self::Modelines,
        Self::Number,
        Self::Readonly,
        Self::Timeoutlen,
    ];

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "timeoutlen" | "tm" => Some(Self::Timeoutlen),
            "readonly" | "ro" => Some(Self::Readonly),
            "number" | "nu" => Some(Self::Number),
            "modeline" | "ml" => Some(Self::Modeline),
            "modelines" | "mls" => Some(Self::Modelines),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Timeoutlen => "timeoutlen",
            Self::Readonly => "readonly",
            Self::Number => "number",
            Self::Modeline => "modeline",
            Self::Modelines => "modelines",
        }
    }

    pub fn scope(self) -> Scope {
        match self {
            Self::Timeoutlen | Self::Modelines => Scope::Global,
            Self::Readonly | Self::Modeline => Scope::Buffer,
            Self::Number => Scope::Window,
        }
    }

    pub fn default_value(self) -> Value {
        match self {
            Self::Timeoutlen => Value::Number(1000),
            Self::Readonly | Self::Number => Value::Bool(false),
            Self::Modelines => Value::Number(5),
            Self::Modeline => Value::Bool(modeline_default()),
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

fn modeline_default() -> bool {
    #[cfg(unix)]
    {
        !rustix::process::geteuid().is_root()
    }
    #[cfg(not(unix))]
    {
        true
    }
}
