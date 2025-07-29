#[derive(Debug)]
pub struct CommandBuilder {
    comamnd: Option<Vec<u8>>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        CommandBuilder { comamnd: None }
    }

    pub fn with_byte<T: Into<u8>>(&mut self, byte: T) {
        if let Some(c) = self.get_mut_buf() {
            c.push(byte.into());
        } else {
            self.comamnd = Some(Vec::new());
            self.with_byte(byte);
        }
    }

    pub fn with_bytes(&mut self, bytes: &[u8]) {
        if let Some(c) = self.get_mut_buf() {
            c.extend_from_slice(bytes);
        } else {
            self.comamnd = Some(Vec::new());
            self.with_bytes(bytes);
        }
    }

    fn get_mut_buf(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.comamnd
    }

    fn take(&mut self) -> Option<Vec<u8>> {
        self.comamnd.take()
    }
}
