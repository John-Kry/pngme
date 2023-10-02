use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::str::FromStr;
use crc::{Crc, CRC_32_ISO_HDLC};
use crate::chunk_type::ChunkType;
use crate::Error;
use crate::Result;

pub(crate) struct Chunk{
    length : u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk {{", )?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}", )?;
        Ok(())
    }
}

impl Chunk {
    pub(crate) fn new(chunk_type: ChunkType, data:Vec<u8>) ->Self{
       Self{
           length: data.len() as u32,
           chunk_type,
           data,
       }
    }

    fn length(&self)->u32{
        return self.length;
    }

    pub fn data(&self) -> &[u8] {
        return &self.data
    }

    fn crc(&self)-> u32{
        pub const CRC_PNG: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        let mut msg = self.chunk_type.bytes().to_vec();
        msg.extend_from_slice(&self.data);
        CRC_PNG.checksum(&msg)
    }

    fn chunk_type(&self)->ChunkType{
        self.chunk_type
    }

    fn data_as_string(&self)->Result<String>{
        Ok(String::from_utf8(self.data.to_vec())?)
    }

    pub fn as_bytes(&self)->Vec<u8>{
       return self.data().to_vec();
    }
}
impl TryFrom<&[u8]> for Chunk{
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        type Error = crate::Error;

        let length = u32::from_be_bytes(bytes[0..=3].to_vec().try_into().unwrap());
        let chunk_type = ChunkType::try_from(
            <[u8; 4]>::try_from(bytes[4..=7].to_vec()).unwrap()
            )?;
        let mut data = Vec::<u8>::new();
        if length >0 {
           data = bytes[8..(8+length as usize)].to_vec();
        }
        let res =
        Self{
            length,
            chunk_type,
            data,
        };
        let crc = bytes[(8+length as usize)..(12+length as usize)].to_vec();
        if res.crc() != u32::from_be_bytes(crc.to_vec().try_into().unwrap()){
            return Err("Failed!".into());
        }
        Ok(res)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
