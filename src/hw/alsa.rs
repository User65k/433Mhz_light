
use super::{Gpio,Level};
use ::std::thread::sleep;
use ::std::time::Duration;
use std::sync::{Mutex,Arc};
use std::thread;

use std::fs;
use std::io::Read;

extern crate glob;
use self::glob::glob;

fn is_playing() -> bool {
    let mut plays = false;
    for filename in glob("/proc/asound/card*/pcm*/sub*/status").expect("Failed to read glob pattern") {
        if let Ok(filename) = filename {
            if let Ok(mut f) = fs::File::open(filename) {
                let mut contents = String::new();
                if f.read_to_string(&mut contents).is_ok()
                {
                    if contents.contains("RUNNING") {
                        plays = true;
                        break;
                    }
                }
            }
        }
    }
    plays
}

pub fn show_play_stat_on_pin(gpio: Arc<Mutex<Gpio>>, pin: u8) {
    thread::spawn(move || {
        let mut last_state = false;

        loop {
            let state = is_playing();
            if state != last_state {
                last_state = state;
                let gpio = gpio.lock().unwrap();
                if state {
                    gpio.write(pin, Level::High);
                }else{
                    gpio.write(pin, Level::Low);
                }
            }
            sleep(Duration::from_millis(500));
        }
    });
}