use crate::meta::Meta;

#[derive(Debug, Default)]
pub struct Config {
    pub big_endian: bool,
}

#[derive(Debug)]
pub struct ConfigError;

impl Config {
    pub fn from_meta_list(l: &[Meta]) -> Result<Self, ConfigError> {
        let big_endian = if l.contains(&Meta::LittleEndian) {
            false
        } else {
            true
        };
        if l.contains(&Meta::LittleEndian) && l.contains(&Meta::BigEndian) {
            eprintln!("Struct cannot be both big and little endian");
            return Err(ConfigError);
        }
        Ok(Config {
            big_endian
        })
    }
}
