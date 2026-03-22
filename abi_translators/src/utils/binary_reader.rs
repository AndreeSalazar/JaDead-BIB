// ============================================================
// Binary Reader — Cursor-based reader for binary data
// ============================================================

pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BinaryReader { data, pos: 0 }
    }

    pub fn at(data: &'a [u8], offset: usize) -> Self {
        BinaryReader { data, pos: offset }
    }

    pub fn pos(&self) -> usize { self.pos }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn remaining(&self) -> usize { self.data.len().saturating_sub(self.pos) }
    pub fn is_eof(&self) -> bool { self.pos >= self.data.len() }

    pub fn seek(&mut self, pos: usize) { self.pos = pos; }
    pub fn skip(&mut self, n: usize) { self.pos += n; }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() { return None; }
        let v = self.data[self.pos];
        self.pos += 1;
        Some(v)
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.data.len() { return None; }
        let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Some(v)
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.data.len() { return None; }
        let v = u32::from_le_bytes([
            self.data[self.pos], self.data[self.pos+1],
            self.data[self.pos+2], self.data[self.pos+3],
        ]);
        self.pos += 4;
        Some(v)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        if self.pos + 8 > self.data.len() { return None; }
        let v = u64::from_le_bytes(self.data[self.pos..self.pos+8].try_into().unwrap());
        self.pos += 8;
        Some(v)
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.pos + n > self.data.len() { return None; }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Some(slice)
    }

    pub fn read_cstring(&mut self) -> Option<String> {
        if self.pos >= self.data.len() { return None; }
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
        if self.pos < self.data.len() { self.pos += 1; } // skip null
        Some(s)
    }

    pub fn peek_u8(&self) -> Option<u8> {
        if self.pos >= self.data.len() { return None; }
        Some(self.data[self.pos])
    }

    pub fn peek_u16(&self) -> Option<u16> {
        if self.pos + 2 > self.data.len() { return None; }
        Some(u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]))
    }

    pub fn slice(&self, offset: usize, len: usize) -> Option<&'a [u8]> {
        if offset + len > self.data.len() { return None; }
        Some(&self.data[offset..offset + len])
    }

    pub fn slice_from(&self, offset: usize) -> &'a [u8] {
        if offset >= self.data.len() { return &[]; }
        &self.data[offset..]
    }
}
