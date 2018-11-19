/**
 * Some kind of port of
 * https://github.com/sui77/rc-switch/blob/c5645170be8cb3044f4a8ca8565bfd2d221ba182/RCSwitch.cpp
*/

use ::std::thread::sleep;
use ::std::time::Duration;
use super::{Gpio,Level};

struct HighLow {
    high: u8,
    low: u8,
}

struct Protocol {
    pulse_length: u8,
    sync_factor: HighLow,
    zero: HighLow,
    one: HighLow,
}

const PROTO: [Protocol; 1] = [Protocol { pulse_length:250,
                                        sync_factor: HighLow {high: 1, low: 31},
                                        zero: HighLow {high: 1, low: 3},
                                        one: HighLow {high: 3, low: 1}
                                    } ];

fn diff(a: u32, b: u32) -> u32 {
    if a >= b
    {
        a - b
    }else{
        b - a
    }
}

pub fn interrupt(duration: u32, recv_val: &mut u32)
{

    let n_separation_limit = 4600;
    let n_receive_tolerance = 60;
    const DUR_BUF_SIZE: usize = 67;
    static mut DUR_BUF: [u32; DUR_BUF_SIZE] = [0; DUR_BUF_SIZE];
    static mut CHANGE_COUNT: usize = 0;
    static mut REPEAT_COUNT: u8 = 0;

    unsafe { //static
        if duration > n_separation_limit {
            // A long stretch without signal level change occurred. This could
            // be the gap between two transmission.
            if diff(duration, DUR_BUF[0]) < 200 {
                // This long signal is close in length to the long signal which
                // started the previously recorded timings; this suggests that
                // it may indeed by a a gap between two transmissions (we assume
                // here that a sender will send the signal multiple times,
                // with roughly the same gap between them).
                REPEAT_COUNT += 1;
                if REPEAT_COUNT == 2 {
                    for pro in PROTO.iter() {

                        let mut code = 0;
                        let delay = DUR_BUF[0] / pro.sync_factor.low as u32;
                        let delay_tolerance = delay * n_receive_tolerance / 100;

                        for i in (1..CHANGE_COUNT - 1).step_by(2) {
                            code <<= 1;
                            if diff(DUR_BUF[i], delay * pro.zero.high as u32) < delay_tolerance &&
                               diff(DUR_BUF[i + 1], delay * pro.zero.low as u32) < delay_tolerance {
                                // zero
                            } else if diff(DUR_BUF[i], delay * pro.one.high as u32) < delay_tolerance &&
                                      diff(DUR_BUF[i + 1], delay * pro.one.low as u32) < delay_tolerance {
                                // one
                                code |= 1;
                            } else {
                                // Failed
                                continue;
                            }
                        }

                        if CHANGE_COUNT > 7 {    // ignore very short transmissions: no device sends them, so this must be noise
                            *recv_val = code;
                            /*RCSwitch::nReceivedBitlength = (CHANGE_COUNT - 1) / 2;
                            RCSwitch::nReceivedDelay = delay;
                            RCSwitch::nReceivedProtocol = p;*/
                        }


                        break;
                    }
                    REPEAT_COUNT = 0;
                }
            }
            CHANGE_COUNT = 0;
        }
        //RCSwitch:
        // detect overflow
        if CHANGE_COUNT >= DUR_BUF_SIZE {
            CHANGE_COUNT = 0;
            REPEAT_COUNT = 0;
        }
        DUR_BUF[CHANGE_COUNT] = duration;
        CHANGE_COUNT += 1;
    }
}

pub struct Transmitter<'a> {
    protocol: &'a Protocol,
    repeat_count: u8,
    pin: u8,
    gpio: Gpio,
}

impl<'a> Transmitter<'a> {
    pub fn new(proto: usize, pin: u8, repeat_count: u8, gpio: Gpio) -> Transmitter<'a> {
        Transmitter {
            protocol: &PROTO[proto],
            repeat_count,
            pin,
            gpio
        }
    }
    /**
     * Transmit the first 'bit_count' bits of the integer 'code'. The
     * bits are sent from MSB to LSB, i.e., first the bit at position length-1,
     * then the bit at position length-2, and so on, till finally the bit at position 0.
     */
    pub fn send(&mut self, code: u32, bit_count: u8) {
        //TODO make sure the receiver is disabled while we transmit

        for _n_repeat in 1..self.repeat_count {
            for i in (0..bit_count).rev() {
                if (code & (1 << i))!=0 {
                    self.transmit(&self.protocol.one);
                }else {
                    self.transmit(&self.protocol.zero);
                }
            }
            self.transmit(&self.protocol.sync_factor);
        }
        //TODO enable receiver again if we just disabled it
    }
    fn transmit(&self, pulse: &HighLow) {

        self.gpio.write(self.pin, Level::High);
        sleep(Duration::from_micros(self.protocol.pulse_length as u64 * pulse.high as u64));
        self.gpio.write(self.pin, Level::Low);
        sleep(Duration::from_micros( self.protocol.pulse_length as u64 * pulse.low as u64));
    }
}