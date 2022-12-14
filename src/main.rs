use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::{prelude::*, delay, i2c::{self, I2cDriver}};
use std::time::Duration;
use mpu6050::Mpu6886;

mod complementary_filter;

const CONTROL_PERIOD: Duration = Duration::from_millis(10);

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    // Initialize I2C
    let i2c_config = i2c::config::Config {
        baudrate: KiloHertz(400).into(),
        sda_pullup_enabled: true,
        scl_pullup_enabled: true
    };
    let i2c = I2cDriver::new(
        peripherals.i2c0, peripherals.pins.gpio25, peripherals.pins.gpio21, &i2c_config
    ).unwrap();
    // Used to share the I2C bus with multiple drivers (IMU and motor)
    let i2c_bus = shared_bus::BusManagerSimple::new(i2c);

    // Initialize IMU
    let mut imu = Mpu6886::new(i2c_bus.acquire_i2c());
    imu.init(&mut delay::FreeRtos).unwrap();

    let mut comp_filter = complementary_filter::ComplemtaryFilter::new(CONTROL_PERIOD.as_secs_f32());

    let mut last_time = std::time::Instant::now();

    loop {
        let accel = imu.get_acc().unwrap();
        let gyro = imu.get_gyro().unwrap();

        let accel_angle = accel.z.atan2(-accel.y);  // Becomes 0 when USB port is facing up
        let angle = comp_filter.filter(accel_angle, gyro.x);

        // Control runs at constant frequency even if processing time changes
        std::thread::sleep(CONTROL_PERIOD - last_time.elapsed());
        last_time = std::time::Instant::now();
    }
}