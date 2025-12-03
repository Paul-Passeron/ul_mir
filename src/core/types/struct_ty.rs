use crate::core::types::MirType;

// Struct ID
pub type StructId = u32;

pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, MirType)>,
    pub packed: bool,
    pub align: u32,
}
