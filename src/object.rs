use std::cell::{Ref, RefCell};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Object {
    pub(crate) object_type: ObjectType,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ObjectType {
    OBJ_STRING(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object { object_type } => match object_type {
                ObjectType::OBJ_STRING(s) => writeln!(f, "{}", s),
            },
        }
    }
}

impl From<Object> for u8 {
    fn from(value: Object) -> Self {
        0 // TODO
    }
}
