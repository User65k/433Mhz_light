use ::std::net::{TcpListener,TcpStream};
use std::io::{Read,Error,ErrorKind};

/*pub fn serve() {
    let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
    for stream in listener.incoming() {
        if let Ok(byte) = get_byte_from_stream(stream) {
            let br = (byte & 0xF0)>>4;
        }
    }
}*/

pub fn get_byte_from_stream(stream: Result<TcpStream,Error>) -> Result<u8, Error> {
    match stream {
        Ok(stream) => {
            for byte in stream.bytes() {
                let byte = byte?;
                return Ok(byte);
            }
            // TCP end?
        },
        Err(e) => return Err(e),
    }
    Err(Error::new(ErrorKind::Other, "oh no!"))
}