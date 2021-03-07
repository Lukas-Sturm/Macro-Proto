use embedded_hal::PwmPin;
use rtt_target::rprintln;

pub struct Vibrator<C1> 
where 
    C1: PwmPin<Duty=u16>
{
    motor: C1,
    rumbling: bool,
    cycles: u16,
    max_duty: u16
}

impl<C1> Vibrator<C1> 
where
    C1: PwmPin<Duty=u16>
{
    
    pub fn new<C2: PwmPin<Duty=u16>>(motors: (C1, C2)) -> Vibrator<C1> {
        let (mut motor, mut ground) = motors;

        ground.set_duty(0);
        motor.set_duty(motor.get_max_duty());
        
        Vibrator {
            max_duty: motor.get_max_duty(), // cache
            motor,
            rumbling: false,
            cycles: 0
        }
    }

    pub fn disable(&mut self) {
        if self.rumbling {
            self.motor.disable();
            self.rumbling = false;
        }
    }

    // TODO: sometimes does not stop?
    pub fn update(&mut self) {
        if self.rumbling {
            rprintln!("u");

            self.cycles -= 1;

            if self.cycles == 0 {
                rprintln!("done");
                self.motor.disable();
                self.rumbling = false;
            } else {
                self.motor.set_duty(self.max_duty - self.max_duty / self.cycles);
            }
        }
    }

    pub fn enable(&mut self, cycles: u16) {
        self.cycles = cycles;
        rprintln!("{}", cycles);

        if !self.rumbling {
            self.motor.set_duty(self.max_duty);
            self.motor.enable();
            self.rumbling = true;
        }
    }
}