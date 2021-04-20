use crate::endian::ParserEndianness;
use crate::meta::attr::{MetaAttr, MetaAttrType};

#[derive(Debug)]
pub struct Config {
    pub struct_name: String,
    /// Endianness for all parsers, if specified
    pub global_endianness: ParserEndianness,
    /// Endianness for this struct or enum, if specified
    pub object_endianness: ParserEndianness,
    /// Complete option for this struct (default: streaming)
    pub complete: bool,
    pub debug: bool,
    pub debug_derive: bool,
    pub generic_errors: bool,
    pub input_name: String,
}

#[derive(Debug)]
pub struct ConfigError;

impl Config {
    pub fn from_meta_list(name: String, l: &[MetaAttr]) -> Result<Self, ConfigError> {
        let req_big_endian = l.iter().any(|m| m.is_type(MetaAttrType::BigEndian));
        let req_little_endian = l.iter().any(|m| m.is_type(MetaAttrType::LittleEndian));
        if req_big_endian & req_little_endian {
            eprintln!("Struct cannot be both big and little endian");
            return Err(ConfigError);
        }
        let object_endianness = if req_big_endian {
            ParserEndianness::BigEndian
        } else if req_little_endian {
            ParserEndianness::LittleEndian
        } else {
            ParserEndianness::Unspecified
        };
        let complete = l.iter().any(|m| m.is_type(MetaAttrType::Complete));
        let debug = l.iter().any(|m| m.is_type(MetaAttrType::Debug));
        let debug_derive = l.iter().any(|m| m.is_type(MetaAttrType::DebugDerive));
        let generic_errors = l.iter().any(|m| m.is_type(MetaAttrType::GenericErrors));
        let input_name = l
            .iter()
            .find_map(|m| {
                if m.is_type(MetaAttrType::InputName) {
                    Some(m.arg().unwrap().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "i".to_string());
        Ok(Config {
            struct_name: name,
            global_endianness: ParserEndianness::Unspecified,
            object_endianness,
            complete,
            debug,
            debug_derive,
            generic_errors,
            input_name,
        })
    }
}
