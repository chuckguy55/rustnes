extern crate rustnes;

use rustnes::Nes;

use std::os;

fn main() {
    let args: Vec<String> = os::args();

    let filename = 
        if args.len() > 1 { 
            args[0].as_slice() 
        } else {
            "mario.nes"
        };

    let path = Path::new(filename);
    let mut nes = Nes::new(path);
    nes.reset();

    nes.run();
}
