use crate::parser::AccessParser;
use crate::types::Access;
use crate::types::AccessType;
use crate::types::AccessOperation;

pub struct BasicParser {
    content: Vec<String>,
    cursor: usize,
}

impl BasicParser {
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


impl AccessParser for BasicParser {
    fn next_access(&mut self) -> Option<Access> {

        while let Some(line) = self.content.get(self.cursor) {
            self.cursor += 1;

            let mut trimmed = line.trim_start().split(" ").into_iter();

            let Some(addr_str) = trimmed.next() else {
                continue;
            };

            let Ok(address) = u32::from_str_radix(addr_str, 10) else {
                continue;
            };

            let (type_, operation) = match trimmed.next() {
                Some("R") => (AccessType::INSTRUCTION, AccessOperation::READ),
                Some("W") => (AccessType::DATA, AccessOperation::WRITE),
                _ => continue,
            };

            return Some(Access{address, type_, operation});
        }

        return None;
    }
}
