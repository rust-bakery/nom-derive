use crate::config::*;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use proc_macro2::Span;
use syn::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserEndianness {
    Unspecified,
    LittleEndian,
    BigEndian,
    SetEndian,
}

pub fn get_object_endianness(config: &Config) -> ParserEndianness {
    // first, check struct endianness
    if config.object_endianness != ParserEndianness::Unspecified {
        return config.object_endianness;
    }
    // finally, return global endianness
    config.global_endianness
}

pub fn set_object_endianness(
    span: Span,
    endianness: ParserEndianness,
    meta_list: &[MetaAttr],
    config: &mut Config,
) -> Result<()> {
    config.object_endianness = endianness;
    // first, check local attribute
    let req_big_endian = meta_list.iter().any(|m| m.is_type(MetaAttrType::BigEndian));
    let req_little_endian = meta_list
        .iter()
        .any(|m| m.is_type(MetaAttrType::LittleEndian));
    let req_set_endian = meta_list.iter().any(|m| m.is_type(MetaAttrType::SetEndian));
    // test if 2 or more flags are set
    if two_or_more(req_big_endian, req_little_endian, req_set_endian) {
        return Err(Error::new(
            span,
            "cannot be both big, little and/or set endian",
        ));
    }
    if req_big_endian {
        config.object_endianness = ParserEndianness::BigEndian;
    } else if req_little_endian {
        config.object_endianness = ParserEndianness::LittleEndian;
    } else if req_set_endian {
        config.object_endianness = ParserEndianness::SetEndian;
    };
    Ok(())
}

fn two_or_more(a: bool, b: bool, c: bool) -> bool {
    if a {
        b | c
    } else {
        b & c
    }
}

pub fn get_local_endianness(
    span: Span,
    meta_list: &[MetaAttr],
    config: &Config,
) -> Result<ParserEndianness> {
    // first, check local attribute
    let req_big_endian = meta_list.iter().any(|m| m.is_type(MetaAttrType::BigEndian));
    let req_little_endian = meta_list
        .iter()
        .any(|m| m.is_type(MetaAttrType::LittleEndian));
    let req_set_endian = meta_list.iter().any(|m| m.is_type(MetaAttrType::SetEndian));
    // test if 2 or more flags are set
    if two_or_more(req_big_endian, req_little_endian, req_set_endian) {
        return Err(Error::new(
            span,
            "cannot be both big, little and/or set endian",
        ));
    }
    if req_big_endian {
        return Ok(ParserEndianness::BigEndian);
    } else if req_little_endian {
        return Ok(ParserEndianness::LittleEndian);
    } else if req_set_endian {
        return Ok(ParserEndianness::SetEndian);
    };
    // otherwise, get object-level endianness
    Ok(get_object_endianness(config))
}
