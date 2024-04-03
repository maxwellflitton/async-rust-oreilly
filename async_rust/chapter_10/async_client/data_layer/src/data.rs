use std::io::{self, Cursor, Read, Write};


#[derive(Debug)]
pub struct Data {
    pub field1: u32,
    pub field2: u16,
    pub field3: String,
}

impl Data {
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.write(&self.field1.to_ne_bytes())?;
        bytes.write(&self.field2.to_ne_bytes())?;
        let field3_len = self.field3.len() as u32;
        bytes.write(&field3_len.to_ne_bytes())?;
        bytes.extend_from_slice(self.field3.as_bytes());
        Ok(bytes)
    }
    pub fn deserialize(cursor: &mut Cursor<&[u8]>) -> io::Result<Data> {
        let mut field1_bytes = [0u8; 4];
        let mut field2_bytes = [0u8; 2];
        cursor.read_exact(&mut field1_bytes)?;
        cursor.read_exact(&mut field2_bytes)?;
        let field1 = u32::from_ne_bytes(field1_bytes);
        let field2 = u16::from_ne_bytes(field2_bytes);
        let mut len_bytes = [0u8; 4];
        cursor.read_exact(&mut len_bytes)?;
        let len = u32::from_ne_bytes(len_bytes) as usize;
        let mut field3_bytes = vec![0u8; len];
        cursor.read_exact(&mut field3_bytes)?;
        let field3 = String::from_utf8(field3_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        Ok(Data { field1, field2, field3 })
    }
}