use std::time::Duration;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::{prelude::*, delay, i2c::{self, I2cDriver}, gpio::{PinDriver, Level}};
use mpu6050::Mpu6886;

mod complementary_filter;
mod atom_motion;
mod pid;

const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

const CONTROL_PERIOD: Duration = Duration::from_millis(10);
const MOTOR_MIN: f32 = -1.0;
const MOTOR_MAX: f32 = 1.0;
const NEUTRAL_ANGLE: f32 = -0.105;

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

    // Initialize Atom Motion motor driver
    let mut motion = atom_motion::AtomMotion::new(i2c_bus.acquire_i2c());

    // Initialize button
    let button = PinDriver::input(peripherals.pins.gpio39).unwrap();

    let mut comp_filter = complementary_filter::ComplemtaryFilter::new(CONTROL_PERIOD.as_secs_f32());
    let mut pid = pid::PIDController::new(CONTROL_PERIOD.as_secs_f32(), 10.0, 130.0, 0.3);

    let mut last_button_state = Level::High;
    let mut motor_active = false;

    let mut last_time = std::time::Instant::now();

    loop {
        // Toggle motor on/off using a button
        let button_state = button.get_level();
        if last_button_state == Level::High && button_state == Level::Low {
            motor_active = !motor_active;
        }
        last_button_state = button_state;

        let accel = imu.get_acc().unwrap();
        let gyro = imu.get_gyro().unwrap();

        let accel_angle = accel.z.atan2(-accel.y);  // Becomes 0 when USB port is facing up
        let angle = comp_filter.filter(accel_angle, gyro.x);

        // Stop when falled down
        if (angle - NEUTRAL_ANGLE).abs() > 60.0 * DEG2RAD {
            motor_active = false;
        }

        pid.update(angle, NEUTRAL_ANGLE);

        if motor_active {
            pid.control = pid.control.clamp(MOTOR_MIN, MOTOR_MAX);
        } else {
            pid.control = 0.0;
        }

        motion.set_motor(atom_motion::MotorChannel::M1, -pid.control).unwrap();
        motion.set_motor(atom_motion::MotorChannel::M2, pid.control).unwrap();

        // Control runs at constant frequency even if processing time changes
        std::thread::sleep(CONTROL_PERIOD - last_time.elapsed());
        last_time = std::time::Instant::now();
    }
}