use crate::controller::Controller;
use bitvec::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;

// Button index constants
pub mod inputs {
    pub const BUTTON_Y: usize = 7;
    pub const BUTTON_X: usize = 6;
    pub const BUTTON_B: usize = 5;
    pub const BUTTON_A: usize = 4;
    pub const BUTTON_RSR: usize = 3;
    pub const BUTTON_RSL: usize = 2;
    pub const BUTTON_R: usize = 1;
    pub const BUTTON_ZR: usize = 0;
    pub const BUTTON_MINUS: usize = 15;
    pub const BUTTON_PLUS: usize = 14;
    pub const BUTTON_R_STICK: usize = 13;
    pub const BUTTON_L_STICK: usize = 12;
    pub const BUTTON_HOME: usize = 11;
    pub const BUTTON_CAPTURE: usize = 10;
    pub const BUTTON_CHARGING_GRIP: usize = 8;
    pub const BUTTON_DOWN: usize = 23;
    pub const BUTTON_UP: usize = 22;
    pub const BUTTON_RIGHT: usize = 21;
    pub const BUTTON_LEFT: usize = 20;
    pub const BUTTON_LSR: usize = 19;
    pub const BUTTON_LSL: usize = 18;
    pub const BUTTON_L: usize = 17;
    pub const BUTTON_ZL: usize = 16;

    pub const AXIS_LH: usize = 0;
    pub const AXIS_LV: usize = 1;
    pub const AXIS_RH: usize = 2;
    pub const AXIS_RV: usize = 3;
}

mod magic {
    pub const INITIAL_INPUT: [u8; 11] = [
        0x81, 0x00, 0x80, 0x00, 0xf8, 0xd7, 0x7a, 0x22, 0xc8, 0x7b, 0x0c,
    ];
    pub const SERIAL_NUMBER: [u8; 16] = [0xff; 16];
    pub const SENSOR_STICK_PARAMS: [u8; 24] = [
        0x50, 0xfd, 0x00, 0x00, 0xc6, 0x0f, 0x0f, 0x30, 0x61, 0x96, 0x30, 0xf3, 0xd4, 0x14, 0x54,
        0x41, 0x15, 0x54, 0xc7, 0x79, 0x9c, 0x33, 0x36, 0x63,
    ];
    pub const STICK_PARAMS_2: [u8; 18] = [
        0x0f, 0x30, 0x61, 0x96, 0x30, 0xf3, 0xd4, 0x14, 0x54, 0x41, 0x15, 0x54, 0xc7, 0x79, 0x9c,
        0x33, 0x36, 0x63,
    ];
    pub const CONFIG: [u8; 25] = [
        0xba, 0x15, 0x62, 0x11, 0xb8, 0x7f, 0x29, 0x06, 0x5b, 0xff, 0xe7, 0x7e, 0x0e, 0x36, 0x56,
        0x9e, 0x85, 0x60, 0xff, 0x32, 0x32, 0x32, 0xff, 0xff, 0xff,
    ];
    pub const CALIBRATION: [u8; 24] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xb2, 0xa1,
    ];
    pub const SENSOR_CALIBRATION: [u8; 24] = [
        0xbe, 0xff, 0x3e, 0x00, 0xf0, 0x01, 0x00, 0x40, 0x00, 0x40, 0x00, 0x40, 0xfe, 0xff, 0xfe,
        0xff, 0x08, 0x00, 0xe7, 0x3b, 0xe7, 0x3b, 0xe7, 0x3b,
    ];
    pub const DEFAULT_COLOUR: [u8; 12] = [
        0x03, 0x9b, 0xc5, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
}

#[derive(Debug)]
pub struct NsProcon {
    hid_path: PathBuf,
    input_state: BitArr!(for 72, in Lsb0, u8),
    thread_tx: Option<Sender<()>>,
}

fn response(code: u8, cmd: u8, data: &[u8], writer: &mut BufWriter<File>) {
    if data.len() + 2 > 64 {
        return;
    }
    let padding = vec![0; 64 - 2 - data.len()];
    let send = &[&[code, cmd], data, &padding].concat();
    let _ = writer.write_all(send);
    let _ = writer.flush();
}

fn uart_response(code: u8, subcmd: u8, input: &[u8], data: &[u8], writer: &mut BufWriter<File>) {
    // TODO timestamp?
    response(
        0x21,
        0x00,
        &[&[0x81], input, &[0x0c, code, subcmd], data].concat(),
        writer,
    )
}

fn spi_response(addr_lo: u8, addr_hi: u8, input: &[u8], data: &[u8], writer: &mut BufWriter<File>) {
    uart_response(
        0x90,
        0x10,
        input,
        &[&[addr_lo, addr_hi, 0x00, 0x00], data].concat(),
        writer,
    );
}

// All credit for this function goes to:
// https://mzyy94.com/blog/2020/03/20/nintendo-switch-pro-controller-usb-gadget/
fn send_response(buffer: &[u8], input: &[u8], writer: &mut BufWriter<File>) {
    if buffer.len() < 2 {
        return;
    }
    if buffer[0] == 0x80 {
        match buffer[1] {
            0x01 => response(0x81, 0x01, &[0, 3, 0, 0, 0, 0, 0, 0], writer),
            0x02 => response(0x81, 0x02, &[], writer),
            0x04 => {
                println!("sending input now"); // TODO
            }
            _ => (),
        }
    } else if buffer[0] == 0x01 && buffer.len() > 16 {
        match buffer[10] {
            0x01 => uart_response(0x81, 0x01, input, &[0x03], writer),
            0x02 => uart_response(
                0x82,
                0x02,
                input,
                &[
                    0x03, 0x48, 0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01,
                ],
                writer,
            ),
            0x03 | 0x08 | 0x30 | 0x38 | 0x40 | 0x48 => {
                uart_response(0x80, buffer[10], input, &[], writer)
            }
            0x04 => uart_response(0x83, 0x04, input, &[], writer),
            0x21 => uart_response(
                0xa0,
                0x21,
                input,
                &[0x01, 0x00, 0xff, 0x00, 0x03, 0x00, 0x05, 0x01],
                writer,
            ),
            0x10 => match buffer[11] {
                0x00 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x00, 0x60, input, &magic::SERIAL_NUMBER, writer)
                    }
                }
                0x50 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x50, 0x60, input, &magic::DEFAULT_COLOUR, writer)
                    }
                }
                0x80 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x80, 0x60, input, &magic::SENSOR_STICK_PARAMS, writer)
                    }
                }
                0x98 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x98, 0x60, input, &magic::STICK_PARAMS_2, writer)
                    }
                }
                0x3d => {
                    if buffer[12] == 0x60 {
                        spi_response(0x3d, 0x60, input, &magic::CONFIG, writer)
                    }
                }
                0x10 => {
                    if buffer[12] == 0x80 {
                        spi_response(0x10, 0x80, input, &magic::CALIBRATION, writer)
                    }
                }
                0x28 => {
                    if buffer[12] == 0x80 {
                        spi_response(0x28, 0x80, input, &magic::SENSOR_CALIBRATION, writer)
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl Controller for NsProcon {
    type C = NsProcon;

    fn create<P: AsRef<Path>>(path: P) -> NsProcon {
        NsProcon {
            hid_path: path.as_ref().to_path_buf(),
            input_state: BitArray::zeroed(),
            thread_tx: None,
        }
    }
    fn start_comms(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let hid_read = OpenOptions::new().read(true).open(&self.hid_path)?;
        let hid_write = OpenOptions::new().write(true).open(&self.hid_path)?;

        let mut buffer = [0; 64];
        let mut reader = BufReader::new(hid_read);
        let mut writer = BufWriter::new(hid_write);
        thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            };

            let read = match reader.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => 0,
            };

            if read == 0 {
                continue;
            }

            let input = &magic::INITIAL_INPUT[1..10];

            if read >= 10 {
                send_response(&buffer, input, &mut writer);
            } else {
                for i in (0..read).step_by(2) {
                    send_response(&buffer[i..(i + 2)], input, &mut writer)
                }
            }
        });
        self.thread_tx = Some(tx);
        Ok(())
    }
    fn stop(&mut self) {
        let _ = match &self.thread_tx {
            Some(thread_tx) => thread_tx.send(()),
            None => Ok(()),
        };
        self.thread_tx = None;
    }

    fn press(&mut self, index: usize) {
        self.input_state.set(index, true);
    }
    fn release(&mut self, index: usize) {
        self.input_state.set(index, false);
    }
    fn set_axis(&mut self, index: usize, value: u16) {
        match index {
            inputs::AXIS_LH => self.input_state[24..36].store(value >> 4),
            inputs::AXIS_LV => self.input_state[36..48].store(value >> 4),
            inputs::AXIS_RH => self.input_state[48..60].store(value >> 4),
            inputs::AXIS_RV => self.input_state[60..72].store(value >> 4),
            _ => (),
        }
    }
}
