use std::collections::HashMap;

pub struct Serialize {
    bin_data: Vec<u8>,
    cursor: u16,
    name_compression: HashMap<String, u16>,
}

impl Serialize {
    pub fn new() -> Self {
        Serialize {
            bin_data: Vec::new(),
            cursor: 0,
            name_compression: HashMap::new(),
        }
    }

    pub fn write_u8(&mut self, data: u8) {
        self.cursor += 1;
        self.bin_data.push(data);
    }

    pub fn write_u16(&mut self, data: u16) {
        let mut data_as_bytes = data.to_be_bytes().to_vec();
        self.cursor += 2;
        self.bin_data.append(&mut data_as_bytes);
    }

    pub fn write_u32(&mut self, data: u32) {
        let mut data_as_bytes = data.to_be_bytes().to_vec();
        self.cursor += 4;
        self.bin_data.append(&mut data_as_bytes);
    }

    pub fn write_string(&mut self, data: &str) {
        self.write_u8(data.len() as u8);
        let mut str_bytes = data.as_bytes().to_vec();
        self.cursor += str_bytes.len() as u16;
        self.bin_data.append(&mut str_bytes);
    }

    pub fn write_n_bytes(&mut self, mut byte_data: Vec<u8>) {
        self.cursor += byte_data.len() as u16;
        self.bin_data.append(&mut byte_data);
    }

    pub fn set_name_compression(&mut self, partial_fqdn: String) -> Option<u16> {
        if self.name_compression.contains_key(&partial_fqdn) {
            return Some(self.name_compression[&partial_fqdn]);
        }

        self.name_compression.insert(partial_fqdn, self.cursor);
        None
    }

    pub fn bin_data(&self) -> Vec<u8> {
        self.bin_data.clone()
    }
}
