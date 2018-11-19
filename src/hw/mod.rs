extern crate rppal;
use self::rppal::gpio::{Gpio, Mode, Level, Trigger};
use ::std::time::{SystemTime, Duration, UNIX_EPOCH};
use ::std::thread::sleep;
use std::sync::{Mutex, Arc};

// The GPIO module uses BCM pin numbering. BCM GPIO 18 is tied to physical pin 12.
const GPIO_SOUND_ON: u8 = 18;
const GPIO_433_IN: u8 = 27;
const GPIO_433_OUT: u8 = 24;

pub fn setup() -> LampStatus<'static> {
    let mut gpio = Gpio::new().expect("GPIOs already in use");
    gpio.set_mode(GPIO_SOUND_ON, Mode::Output);
    gpio.set_mode(GPIO_433_OUT, Mode::Output);
    gpio.set_mode(GPIO_433_IN, Mode::Input);

    //DEMO HW
    gpio.set_mode (17, Mode::Input );
    gpio.set_mode (22, Mode::Output );
    gpio.set_mode (23, Mode::Output );
    gpio.write (22, Level::High) ;
    gpio.write (23, Level::High) ;

    // Blink an LED attached to the pin on and off
    //gpio.write(GPIO_SOUND_ON, Level::High);
    //gpio.write(GPIO_SOUND_ON, Level::Low);
    let erg = Arc::new(Mutex::new(0));
    gpio.set_async_interrupt(GPIO_433_IN, Trigger::Both, move |val: Level| {
        let time = SystemTime::now();
        let duration = time.duration_since(UNIX_EPOCH).expect("Time went backwards");
        static mut LAST_TIME: u64 = 0;
        let time = (duration.as_secs() * 1_000_000) + (duration.subsec_nanos() / 1_000) as u64;
        let duration: u32; //μs
        unsafe { //static
            duration = (time - LAST_TIME) as u32;
        }

        if duration < 150
        {
            return; //jitter
        }

        let mut n_received_value: u32 = 0;
        home_wizard::interrupt(val, duration, &mut n_received_value);

        rcswitch::interrupt(duration, &mut n_received_value);

        if n_received_value != 0 {
            let mut num = erg.lock().unwrap();
            *num = n_received_value;
            println!("{}",n_received_value);
        }

        unsafe { //static
            LAST_TIME = time;  
        }
    }).expect("could not set interrupt");

    LampStatus::new(gpio)
}

mod home_wizard;
mod rcswitch;
pub struct LampStatus<'a> {
    bright: u8,
    red: u8,
    on: bool,
    con: rcswitch::Transmitter<'a>,
}
impl<'a> LampStatus<'a> {
    fn new(gpios : Gpio) -> LampStatus<'a> {
        let mut lamp = LampStatus {
            bright:15,
            red:0,
            on: true,
            con: rcswitch::Transmitter::new(0,GPIO_433_OUT,7,gpios),
        };

        lamp.set_red(15);
        lamp.dim_to(0);
        lamp.switch(false);
        lamp
    }

    pub fn got_brighter(&mut self) {
        let br  = self.bright + 1;
        self.bright = if br > 15 {
            15
        }else {
            br
        }
    }
    pub fn got_darker(&mut self) {
        self.bright = if self.bright > 0 {
            self.bright - 1
        }else {
            0
        }
    }

    pub fn set_red(&mut self, new: u8) {
        let (cmd, steps) = if new > self.red {
            (800, new - self.red)
        }else{
            (608, self.red - new)
        };
        for _i in 0..steps {
            self.send(cmd);
            self.got_brighter(); // lamp always gets brighter
        }
        self.red = new;
    }
    pub fn dim_to(&mut self, new: u8) {
        let (cmd, steps) = if new > self.bright {
            (572, new - self.bright)
        }else{
            (620, self.bright - new)
        };
        for _i in 0..steps {
            self.send(cmd);
        }
        self.bright = new;
    }
    pub fn switch(&mut self, stat: bool) {
        self.on = stat;
        let cmd = if stat {
            755
        }else{
            611
        };
        self.send(cmd);
    }
    fn send(&mut self, cmd: u32) {
        let code = 14434000+cmd;
        self.con.send(code, 24);
        sleep(Duration::from_micros(50));
    }
}