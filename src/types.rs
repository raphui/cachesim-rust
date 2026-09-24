pub enum AccessType {
    INSTRUCTION,
    DATA,
}

pub enum AccessOperation {
    READ,
    WRITE,
    DATA,
}

pub struct Access {
    pub address: u32,
    pub type_: AccessType, 
    pub operation: AccessOperation, 
}
