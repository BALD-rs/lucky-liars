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
    let port = builder.open().unwrap();
    return (port_names, port);
}
