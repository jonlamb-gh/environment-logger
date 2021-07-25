use crate::hal::hal::PwmPin;

pub struct Alarm<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pwm: PWM,
}

impl<PWM> Alarm<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pub fn new(mut pwm: PWM) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.set_duty(max_duty / 2);
        pwm.disable();
        Alarm { pwm }
    }

    pub fn disable(&mut self) {
        self.pwm.disable();
    }

    pub fn enable(&mut self) {
        self.pwm.enable();
    }
}
