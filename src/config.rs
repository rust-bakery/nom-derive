use crate::meta::attr::{MetaAttr, MetaAttrType};

#[derive(Debug)]
pub struct Config {
    pub struct_name: String,
    pub big_endian: bool,
    pub debug: bool,
    pub debug_derive: bool,
    pub input_name: String,
}

#[derive(Debug)]
pub struct ConfigError;

impl Config {
    pub fn from_meta_list(name: String, l: &[MetaAttr]) -> Result<Self, ConfigError> {
        let big_endian = if l.iter().any(|m| m.is_type(MetaAttrType::LittleEndian)) {
            if l.iter().any(|m| m.is_type(MetaAttrType::BigEndian)) {
                eprintln!("Struct cannot be both big and little endian");
                return Err(ConfigError);
            }
            false
        } else {
            true
        };
        let debug = l.iter().any(|m| m.is_type(MetaAttrType::Debug));
        let debug_derive = l.iter().any(|m| m.is_type(MetaAttrType::DebugDerive));
        let input_name = l
            .iter()
            .find(|m| m.is_type(MetaAttrType::InputName))
            .map(|m| m.arg().unwrap().to_string())
            .unwrap_or("i".to_string());
        Ok(Config {
            struct_name: name,
            big_endian,
            debug,
            debug_derive,
            input_name,
        })
    }
}
