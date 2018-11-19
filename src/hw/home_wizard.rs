/*
HomeWizard
S
11000101100011010000000000
0
1
0010
E

		_   _
'0':   | |_| |_____ (T,T,T,5T)
		_       _
'1':   | |_____| |_	(T,5T,T,T)

- start pulse: 1T high, 10T low
- 26 bit:  Address
- 1  bit:  group bit
- 1  bit:  on/off/[dim]
- 4  bit:  unit
- stop pulse: 1T high, 40T low

*/

use super::Level;

enum State {
    InSync,
    Probably0,
    Probably1,
    Nothing
}

pub fn interrupt(pin_val: Level, duration: u32, recv_val: &mut u32)
{
    static mut RECV_BUFF: u32 = 0;
    static mut STATE: State = State::Nothing;

    static mut PULSE_LEN_SHORT_MAX: u32 =0;
    static mut PULSE_LEN_LONG_MAX: u32 =0;
    static mut PULSE_LEN_SYNC_MAX: u32 =0;

    unsafe { //static
        if pin_val==Level::High
        {
            //rising -> delay is signal

            if duration < PULSE_LEN_SHORT_MAX {
                STATE = match STATE {
                    State::InSync => State::Probably0, //this was the fist pause of '0'
                    State::Probably1 => { //this was the second pause of '1'
                        RECV_BUFF = (RECV_BUFF << 1) | 1;
                        State::InSync
                    },
                    _ => State::Nothing,
                }
            }else if duration < PULSE_LEN_LONG_MAX {
                STATE = match STATE {
                    State::InSync => State::Probably1, //this was the fist pause of '1'
                    State::Probably0 => { //this was the second pause of '0'
                        RECV_BUFF = RECV_BUFF << 1;
                        State::InSync
                    },
                    _ => State::Nothing,
                }
            }else if duration < PULSE_LEN_SYNC_MAX {
                //sync
                STATE = State::InSync;
                //DUR_BUF_POINT = 0;
                RECV_BUFF = 0;
            }else {
                //longer than everything
                if let State::InSync = STATE
                {
                    //also we had something in the past -> this must be the end
                    *recv_val = RECV_BUFF;
                }
                STATE = State::Nothing;
            }

        }else{
            //falling -> delay is sync (HIGH is alway of duration T)
            if 240 < duration && duration < 280
            {
                PULSE_LEN_SHORT_MAX = duration * 3 as u32;
                PULSE_LEN_LONG_MAX = duration * 7.5 as u32;
                PULSE_LEN_SYNC_MAX = duration * 12 as u32;
            }
        }
    }
}

