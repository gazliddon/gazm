/// GDB remote reply


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

pub fn swap_32(val : u32) -> u32 {
    let b0 = val & 0xff;
    let b1 = ( val >> 8 ) & 0xff;
    let b2 = ( val >> 16 ) & 0xff;
    let b3 = ( val >> 24 ) & 0xff;

    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

pub fn swap_16(val : u16) -> u16 {
    let b0 = val & 0xff;
    let b1 = ( val >> 8 ) & 0xff;

    (b0 << 8) | b1
}


pub struct Reply {
    /// Packet data
    data: Vec<u8>,
    /// Checksum
    csum: u8,

    // endian for pushes
    endian : Endian
}

impl Reply {
    pub fn new(endian : &Endian) -> Reply {
        // 32bytes is probably sufficient for the majority of replies
        let mut data = Vec::with_capacity(32);

        // Each reply begins with a dollar sign
        data.push(b'$');

        Reply {
            data,
            csum: 0,
            endian : endian.clone(),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        // Update checksum
        for &b in data {
            self.csum = self.csum.wrapping_add(b);

            if  b == b'$' {
                panic!("Invalid char in GDB response");
            }
        }

        self.data.extend(data.iter().cloned());
    }

    pub fn push_u8(&mut self, byte: u8) {
        let to_hex = b"0123456789abcdef";

        self.push(&[
                  to_hex[(byte >> 4) as usize],
                  to_hex[(byte & 0xf) as usize],
        ])
    }

    /// Push an u16 as 2 little endian bytes
    pub fn push_u16(&mut self, v: u16) {

        let v = match self.endian {
            Endian::Big => swap_16(v),
            _ => v,
        };

        for i in 0..2 {
            self.push_u8((v >> (i * 8)) as u8);
        }
    }

    /// Push an u32 as 4 little endian bytes
    pub fn push_u32(&mut self, v: u32) {

        let v = match self.endian {
            Endian::Big => swap_32(v),
            _ => v,
        };
        for i in 0..4 {
            self.push_u8((v >> (i * 8)) as u8);
        }
    }

    /// Finalize the reply: append the checksum and return the
    /// complete packet
    pub fn into_packet(mut self) -> Vec<u8> {
        // End of packet
        self.data.push(b'#');
        // Append checksum.
        let csum = self.csum;
        self.push_u8(csum);

        self.data
    }
}
