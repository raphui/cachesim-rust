use crate::cache::CacheLine;
use crate::types::Access;
use crate::types::AccessOperation;

pub struct PseudorandomPolicy {
    counter: u32,
}

impl PseudorandomPolicy {
    pub fn new() -> Self {
        PseudorandomPolicy {
            counter: 0,
        }
    }

   fn get_line(&self, associativity: u32) -> u32 {
       self.counter % associativity
   }

   pub fn find_evicted(&self, associativity: u32) -> u32 {
       return self.get_line(associativity)
   }

   pub fn execute(&mut self, line: &mut CacheLine, tag: u32) {
       line.tag = tag;
       line.valid = true;
       line.block = AccessOperation::DATA as u8;
   }

   pub fn post_op(&mut self, associativity: u32, access: &Access) {
        if (associativity > 1) && (((access.address >> 2) & 1) == 1) {
            self.counter += 1;
        }
   }
}
