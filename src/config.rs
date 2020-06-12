use crate::meta::Meta;

#[derive(Debug)]
pub struct Config {
    pub struct_name: String,
    pub big_endian: bool,
    pub debug: bool,
    pub debug_derive: bool,
}

#[derive(Debug)]
pub struct ConfigError;

impl Config {
    pub fn from_meta_list(name: String, l: &[Meta]) -> Result<Self, ConfigError> {
        let big_endian = if l.contains(&Meta::LittleEndian) {
            false
        } else {
            true
        };
        let debug = l.contains(&Meta::Debug);
        if l.contains(&Meta::LittleEndian) && l.contains(&Meta::BigEndian) {
            eprintln!("Struct cannot be both big and little endian");
            return Err(ConfigError);
        }
        let debug_derive = l.contains(&Meta::DebugDerive);
        Ok(Config {
            struct_name: name,
            big_endian,
            debug,
            debug_derive,
        })
    }
}
