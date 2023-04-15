// pub struct Value {
//     value_type: ValueType,
// }
//
// impl Value {
//     pub fn to_byte(self) -> Option<u8> {
//         match self.value_type {
//             ValueType::VAL_BOOL(b) => Some(b as u8),
//             ValueType::VAL_NIL => None,
//             ValueType::VAL_NUMBER(n) => Some(n as u8),
//         }
//     }
// }
pub type Value = f32;
#[allow(non_camel_case_types)]
pub enum ValueType {
    VAL_BOOL(bool),
    VAL_NIL,
    VAL_NUMBER(f32),
}

#[derive(Default)]
pub struct ValueArray {
    pub count: usize,
    pub(crate) values: Vec<Value>,
}

impl ValueArray {
    pub fn write(&mut self, value: Value) {
        if self.values.len() < self.count + 1 {
            self.values.push(value);
        } else {
            self.values[self.count] = value;
        }
        self.count += 1;
    }
}
