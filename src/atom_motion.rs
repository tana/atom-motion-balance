use embedded_hal::{blocking::i2c::Write};

const DEVICE_ADDR: u8 = 0x38;

pub struct AtomMotion<T: Write<u8>> {
    i2c: T
}

impl<T: Write<u8>> AtomMotion<T> {
    pub fn new(i2c: T) -> Self {
        Self {
            i2c
        }
    }

    pub fn set_motor(&mut self, channel: MotorChannel, value: f32) -> Result<(), T::Error> {
        let reg = match channel {
            MotorChannel::M1 => 0x20,
            MotorChannel::M2 => 0x21
        };

        self.i2c.write(DEVICE_ADDR, &[
            reg,
            ((127.0 * value).round().clamp(-128.0, 127.0) as i8) as u8
        ])?;

        Ok(())
    }
}

pub enum MotorChannel {
    M1,
    M2
}
