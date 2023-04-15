pub type Value = f32;

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
