use aes::cipher::StreamCipher;
use aes::Aes128;
use cfb8::Cfb8;
use std::io::{Read, Write};

pub struct AesStream<S> {
    inner: S,
    aes: Cfb8<Aes128>,
}

impl<S> AesStream<S> {
    pub fn new(inner: S, aes: Cfb8<Aes128>) -> Self {
        AesStream { inner, aes }
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<R: Read> Read for AesStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read_amount = self.inner.read(buf)?;
        self.aes.decrypt(&mut buf[..read_amount]);
        Ok(read_amount)
    }
}

impl<W: Write> Write for AesStream<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut copy = Vec::from(buf);
        self.aes.encrypt(&mut copy);
        self.inner.write_all(&copy)?;
        Ok(copy.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
