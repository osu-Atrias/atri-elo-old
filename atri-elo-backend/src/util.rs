use flexbuffers::{FlexbufferSerializer, Reader};
use serde::{Serialize, Deserialize};
use color_eyre::eyre::Result;

pub fn serialize<T: Serialize>(obj: &T) -> Result<Vec<u8>> {
    let mut s = FlexbufferSerializer::new();
    obj.serialize(&mut s)?;
    Ok(s.take_buffer())
}

pub fn deserialize<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Result<T> {
    let r = Reader::get_root(buf)?;
    Ok(T::deserialize(r)?)
}