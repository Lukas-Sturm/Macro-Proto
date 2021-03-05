use embedded_hal::Qei;
use embedded_hal::digital::v2::InputPin;

pub struct Encoder<Q, B> {
    qei: Q,
    button: B
}

impl<Q, B> Encoder<Q, B> 
where 
    Q: Qei<>,
    B: InputPin<>
{
    pub fn new(qei: Q, button: B) -> Encoder<Q, B> {
        Encoder {
            qei,
            button
        }
    }

    pub fn count(&self) -> Q::Count {
        self.qei.count()
    }

    pub fn is_pressed(&self) -> Result<bool, B::Error> {
        self.button.is_low()
    }
}

