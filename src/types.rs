#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccessType {
    INSTRUCTION,
    DATA,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccessOperation {
    READ,
    WRITE,
    DATA,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Access {
    pub address: u32,
    pub type_: AccessType, 
    pub operation: AccessOperation, 
}

#[derive(PartialEq, Eq, Debug)]
pub enum CacheType {
    INSTRUCTION,
    DATA,
    UNIFIED,
}
