use crate::hal::hal::PwmPin;

pub struct Alarm<PWM> {
    pwm: PWM,
    on: bool,
}

impl<PWM> Alarm<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pub fn new(mut pwm: PWM) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.set_duty(max_duty / 2);
        pwm.disable();
        Alarm { pwm, on: false }
    }

    pub fn disable(&mut self) {
        self.pwm.disable();
        self.on = false;
    }

    pub fn enable(&mut self) {
        self.pwm.enable();
        self.on = true;
    }

    pub fn is_on(&self) -> bool {
        self.on
    }
}
