/// Velocity-type PID controller

pub struct PIDController {
    pub dt: f32,
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub control: f32,
    prev_err: f32,
    prev_prev_err: f32
}

impl PIDController {
    pub fn new(dt: f32, kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            dt,
            kp, ki, kd,
            control: 0.0,
            prev_err: 0.0,
            prev_prev_err: 0.0
        }
    }

    pub fn update(&mut self, value: f32, target: f32) {
        let err = target - value;
        let diff_err = (err - self.prev_err) / self.dt; // Derivative of error
        let diff_diff_err = (err - 2.0 * self.prev_err + self.prev_prev_err) / (self.dt * self.dt);   // Second-order derivative of error
        let diff_control = self.kp * diff_err + self.ki * err + self.kd * diff_diff_err;
        self.control += diff_control * self.dt;

        self.prev_prev_err = self.prev_err;
        self.prev_err = err;
    }
}