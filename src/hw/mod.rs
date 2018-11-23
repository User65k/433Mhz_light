extern crate rppal;
use self::rppal::gpio::{Gpio, Mode, Level, Trigger};
use ::std::time::{SystemTime, Duration, UNIX_EPOCH};
use ::std::thread::sleep;
use std::sync::{Mutex, Arc};

mod alsa;

mod home_wizard;
mod rcswitch;

use mio::{Ready, Registration, Poll, PollOpt, Token};
use mio::event::Evented;
use std::io;

// The GPIO module uses BCM pin numbering. BCM GPIO 18 is tied to physical pin 12.
const GPIO_SOUND_ON: u8 = 22;
const GPIO_433_IN: u8 = 27;
const GPIO_433_OUT: u8 = 24;

pub fn setup() -> (LampStatus<'static>, Arc<Mutex<u32>>) {
    let mut gpio = Gpio::new().expect("GPIOs already in use");
    gpio.set_mode(GPIO_SOUND_ON, Mode::Output);
    gpio.set_mode(GPIO_433_OUT, Mode::Output);
    gpio.set_mode(GPIO_433_IN, Mode::Input);

    let (registration, set_readiness) = Registration::new2();

    // Blink an LED attached to the pin on and off
    //gpio.write(GPIO_SOUND_ON, Level::High);
    //gpio.write(GPIO_SOUND_ON, Level::Low);
    let erg = Arc::new(Mutex::new(0));
    let ret = erg.clone();
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
            let mut num = ret.lock().expect("recv buffer locked");
            *num = n_received_value;
            if set_readiness.set_readiness(Ready::readable()).is_err() {
                println!("faild to notify {}",n_received_value);
            }
        }

        unsafe { //static
            LAST_TIME = time;  
        }
    }).expect("could not set interrupt");

    let gpio = Arc::new(Mutex::new(gpio));
    alsa::show_play_stat_on_pin(gpio.clone(), GPIO_SOUND_ON);

    (LampStatus::new(gpio, registration), erg)
}

pub struct LampStatus<'a> {
    bright: u8,
    red: u8,
    on: bool,
    con: rcswitch::Transmitter<'a>,
    registration: Registration,
}
const LAMP_ID: u32 = 14434000;
impl<'a> LampStatus<'a> {

    fn new(gpios : Arc<Mutex<Gpio>>, registration: Registration) -> LampStatus<'a> {
        let mut lamp = LampStatus {
            bright:15,
            red:0,
            on: true,
            con: rcswitch::Transmitter::new(0,GPIO_433_OUT,7,gpios),
            registration,
        };

        lamp.set_red(15);
        lamp.dim_to(0);
        lamp.switch(false);
        lamp
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    fn got_brighter(&mut self) {
        let br  = self.bright + 1;
        self.bright = if br > 15 {
            15
        }else {
            br
        }
    }
    fn got_darker(&mut self) {
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
        let code = LAMP_ID+cmd;
        self.con.send(code, 24);
        sleep(Duration::from_micros(100));
    }

    pub fn update_rf(&mut self, cmd: u32) {
        let cmd = cmd - LAMP_ID;
        match cmd {
            611 => self.on = false,
            755 => self.on = true,
            800 => { // Warm
                let br  = self.red + 1;
                self.red = if br > 15 {
                    15
                }else {
                    br
                };
                self.got_brighter();
            },
            608 => { //Cold
                self.red = if self.red > 0 {
                    self.red - 1
                }else {
                    0
                };
                self.got_brighter();
            },
            620 => self.got_darker(),
            572 => self.got_brighter(),
            _ => {}
        }
    }
}

impl<'a> Evented for LampStatus<'a> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.registration.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.registration.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.registration.deregister(poll)
    }
}