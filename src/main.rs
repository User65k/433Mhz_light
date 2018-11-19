mod hw;
mod server;
use ::std::net::TcpListener;

fn main() {
    let mut lamp = hw::setup();
    //server::serve();
    let listener = TcpListener::bind("0.0.0.0:1337").expect("could not get port 1337");
    for stream in listener.incoming() {
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
}


/*

    let lt = 0
    loop {
        ready = select.select([sck],[],[])[0] # proc.stdout

        if sck in ready:
            val = None
            try:
                dsck, peer = sck.accept()
                val = dsck.recv(1)
                if val:
                    val = val[0]
                dsck.close()
            except Exception:
                pass

            if val==None:
                continue

            br = (val & 0xF0)>>4
            if br==0:
                lamp.switch(False)
            else:
                lamp.setRed(val & 0xF)
                br -= 1
                lamp.dimTo(br)
                lamp.switch(True)
        
            continue

        line = proc.stdout.readline()
        
        ts = time()
        dur = ts - lt
        if line == '':
            break

        number = line.rstrip().decode("ascii")

        #print(number)

        if ( dur < 0.1 ):
            #print("repeat")
            continue

        if number[:5]=="14434":
            #Lampe
            code = number[5:]

            if code=="611": # Aus
                lamp.on = False
                print("Aus")
            elif code=="755": # An
                lamp.on = True
                print("An")
            elif code=="800": # Warm
                lamp.red += 1
                lamp.bright += 1
                print("Warm")
                print(lamp.red)
            elif code=="608": # Cold
                lamp.red -= 1
                lamp.bright += 1
                print("Kalt")
            elif code=="620": # Dunkler
                lamp.bright -= 1
                print("Dunkel")
            elif code=="572": # Heller
                lamp.bright += 1
                print("Hell")
                print(lamp.bright)

        elif number[:8]=="33143521":
            #Schalter
            code = number[8:]

            if code=="30": # Aus
                print("Aus!")
                if lamp.on:
                    print("-> Aus")
                    lamp.switch(False)
                else:
                    print("-> Dunkel Rot")
                    lamp.setRed(15)
                    lamp.dimTo(0)
                    lamp.switch(True)

            elif code=="46": # An
                print("An!")
                if lamp.on:
                    lamp.dimTo(15)
                    print("-> Vollgas")
                else:
                    print("-> Dunkelisch")
                    lamp.setRed(0)
                    lamp.dimTo(4)
                    lamp.switch(True)

        else:
            continue # dont coinsider for timing stuff

        lt = ts

    }
*/