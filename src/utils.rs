pub fn with_thousands(s: String) -> String {
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

pub struct IterVec<'a, T> {
    vec: &'a Vec<T>,
    index: usize,
}

impl<'a, T> IterVec<'a, T> {
    pub fn new(vec: &'a Vec<T>) -> Self {
        Self { vec, index: 0 }
    }

    pub fn current(&self) -> &T {
        &self.vec[self.index]
    }
    pub fn consume(&mut self) -> &T {
        self.index += 1;
        &self.vec[self.index - 1]
    }
    pub fn get(&self, offset: i32) -> &T {
        &self.vec[(self.index as i32 + offset) as usize]
    }
    pub fn finished(&self) -> bool {
        self.index >= self.vec.len()
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vec.len() == 0
    }
    pub fn current_index(&self) -> usize {
        self.index
    }
}
