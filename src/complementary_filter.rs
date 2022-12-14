/// Complementary filter for calculating angle from accelerometer and gyroscope measurement
/// Reference:
///  Pieter-Jan Van de Maele, "Reading a IMU Without Kalman: The Complementary Filter", http://web.archive.org/web/20170913021102/https://www.pieter-jan.com/node/11 (accessed 2022/12/14)

pub struct ComplemtaryFilter {
    pub angle: f32,
    accel_weight: f32,
    period: f32
}

impl ComplemtaryFilter {
    pub fn new(period: f32) -> Self {
        Self {
            angle: 0.0,
            accel_weight: 0.02,
            period
        }
    }

    pub fn filter(&mut self, accel_angle: f32, gyro_rate: f32) -> f32 {
        self.angle = (1.0 - self.accel_weight) * (self.angle + gyro_rate * self.period) + self.accel_weight * accel_angle;
        self.angle
    }
}