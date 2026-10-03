use crate::parser::AccessParser;
use crate::types::Access;
use crate::types::AccessType;
use crate::types::AccessOperation;

pub struct LackeyParser {
    content: Vec<String>,
    cursor: usize,
}

impl LackeyParser {
    pub fn new(content: Vec<String>) -> Self {
        Self {
            content: content,
            cursor: 0,
        }
    }

    fn advance(&mut self) -> Option<String> {
        let line = match self.content.get(self.cursor)  {
            None => "",
            Some(line) => line,
        };

        if line.is_empty() {
            return None;
        }

        self.cursor += 1;

        return Some(String::from(line));
    }
}

impl AccessParser for LackeyParser {
    fn next_access(&mut self) -> Option<Access> {

        while let Some(line) = self.content.get(self.cursor) {
            self.cursor += 1;

            let trimmed = line.trim_start();

            let (type_, operation) = match trimmed.chars().next() {
                Some('I') => (AccessType::INSTRUCTION, AccessOperation::READ),
                Some('L') => (AccessType::DATA, AccessOperation::READ),
                Some('S') => (AccessType::DATA, AccessOperation::WRITE),
                Some('M') => (AccessType::DATA, AccessOperation::READ), // modify read + write
                _ => continue,
            };

            let Some(addr_str) = trimmed[1..].trim_start().split(',').next() else {
                continue;
            };

            let Ok(address) = u32::from_str_radix(addr_str, 16) else {
                continue;
            };

            return Some(Access{address, type_, operation});
        }

        return None;
    }
}
