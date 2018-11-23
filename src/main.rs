extern crate mio;

mod hw;
mod server;
use mio::{Poll, PollOpt, Ready, Events, Token};
use mio::net::TcpListener;

fn main() {
    let (mut lamp, erg) = hw::setup();

    // Setup some tokens to allow us to identify which event is
    // for which socket.
    const SERVER: Token = Token(0);
    const MHZ433: Token = Token(1);

    let addr = "0.0.0.0:1337".parse().unwrap();
    // Setup the server socket
    let server = TcpListener::bind(&addr).expect("could not get port 1337");

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).expect("register listener");

    poll.register(&lamp, MHZ433, Ready::readable(), PollOpt::edge()).expect("register 433");

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let stream = server.accept_std();
                    if let Ok(byte) = server::get_byte_from_stream(stream) {
                        let br = (byte & 0xF0)>>4;
                        if br==0 {
                            lamp.switch(false);
                        }else{
                            lamp.set_red(byte & 0xF);
                            lamp.dim_to(br - 1);
                            lamp.switch(true);
                        }
                    }

                }
                MHZ433 => {
                    let n_received_value = erg.lock().expect("recv buffer locked");
                    match *n_received_value {
                        3314352130 => { //Schalter aus
                            if lamp.is_on() {
                                println!("-> Aus");
                                lamp.switch(false);
                            }else{
                                println!("-> Dunkel Rot");
                                lamp.set_red(15);
                                lamp.dim_to(0);
                                lamp.switch(true);
                            }
                        },
                        3314352146 => { //Schalter an
                            if lamp.is_on() {
                                lamp.dim_to(15);
                                println!("-> Vollgas");
                            }else{
                                println!("-> Dunkelisch");
                                lamp.set_red(0);
                                lamp.dim_to(4);
                                lamp.switch(true);
                            }
                        },
                        14434000...14434999 => { //lampe
                            lamp.update_rf(*n_received_value);
                        },
                        _ => {
                            //jibberish
                            println!("got {}",n_received_value);
                        },
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
