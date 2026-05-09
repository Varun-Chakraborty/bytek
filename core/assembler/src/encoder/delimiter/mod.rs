#[derive(Clone, Debug)]
pub struct Delimiter {
    pub symbol: String,
    pub address: usize,
}

#[derive(Debug, Default)]
pub struct DelimiterTable {
    table: Vec<Delimiter>,
    current: Option<usize>,
}

impl DelimiterTable {
    pub fn new() -> Self {
        Self {
            table: Vec::new(),
            current: None,
        }
    }

    pub fn add_address(&mut self, address: u32) {
        self.append(format!("0x{:03}: ", address), address as usize);
    }

    pub fn append(&mut self, symbol: String, address: usize) {
        self.table.push(Delimiter { symbol, address });
    }

    pub fn delete_last(&mut self) {
        self.table.pop();
    }

    pub fn next(&mut self) {
        if let Some(c) = self.current {
            self.current = Some(c + 1);
        } else {
            self.current = Some(0);
        }
    }

    pub fn get_current(&self) -> Option<&Delimiter> {
        if let Some(c) = self.current {
            self.table.get(c)
        } else {
            None
        }
    }
}
