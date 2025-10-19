use std::collections::VecDeque;

#[derive(Debug)]
pub struct Parser<SymbolType> {
    symbols: VecDeque<SymbolType>,
}

pub trait SymbolMetadata: Sized {
    fn reduce(slice: &[Self]) -> Option<(usize, Self)>;
}

impl<SymbolType: SymbolMetadata> Parser<SymbolType> {
    pub fn new(tokens: impl Iterator<Item = impl Into<SymbolType>>) -> Self {
        Self {
            symbols: tokens.map(Into::into).collect(),
        }
    }
    pub fn parse(&mut self) -> Result<SymbolType, String> {
        let mut stack = Vec::new();
        while self.symbols.len() > 1 {
            let symbol = SymbolMetadata::reduce(self.symbols.make_contiguous());
            if let Some((n, symbol)) = symbol {
                for _ in 0..n {
                    self.symbols.pop_front();
                }
                self.symbols.push_front(symbol);
                while let Some(symbol) = stack.pop() {
                    self.symbols.push_front(symbol);
                }
            } else if let Some(symbol) = self.symbols.pop_front() {
                stack.push(symbol);
            }
        }
        if let Some(symbol) = self.symbols.pop_front()
            && stack.is_empty()
        {
            Ok(symbol)
        } else {
            Err("Failed to parse input".to_string())
        }
    }
}
