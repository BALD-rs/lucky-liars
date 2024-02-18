use std::{io, thread};

use serialport::{available_ports, DataBits, SerialPort, StopBits};

const BAUD_RATE: u32 = 9600;

pub fn setup() -> (Vec<String>, Box<dyn SerialPort>) {
    let ports = available_ports().unwrap();
    let port_names = ports
        .iter()
        .map(|name| name.port_name.clone())
        .collect::<Vec<String>>();
    let builder = serialport::new(port_names.first().unwrap(), BAUD_RATE)
        .stop_bits(StopBits::Two)
        .data_bits(DataBits::Six);
    let mut port = builder.open().unwrap();
    // Clone the port
    let mut cloned_port = port.try_clone().expect("Failed to clone");
    // Read the four bytes back from the cloned port
    let mut buffer: [u8; 1] = [0; 1];
    // Send out 4 bytes every second
    thread::spawn(move || loop {
        match cloned_port.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 1 && (buffer == [48] || buffer == [49]) {
                    let button = buffer[0] - 48;
                    println!("{button}");
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    });
    return (port_names, port);
}
