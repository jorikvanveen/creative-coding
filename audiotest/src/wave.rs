pub struct SinWave {
    amplitude: f32,
    period: f32,
    offset: f32,
}

impl SinWave {
    pub fn new(amplitude: f32, period: f32, offset: f32) -> SinWave {
        SinWave {
            amplitude,
            period,
            offset,
        }
    }

    pub fn evaluate(&self, x: f32) -> f32 {
        use std::f32::consts::PI;
        (((2.0 * PI) / self.period) * (x - self.offset)).sin() * self.amplitude + self.amplitude
    }
}
