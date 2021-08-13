use crate::controller::Controller;
use bitvec::prelude::*;
use rand::Rng;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, SyncSender, TryRecvError};
use std::thread;
use std::time::SystemTime;

// Button index constants
pub mod inputs {
    pub const BUTTON_Y: usize = 0;
    pub const BUTTON_X: usize = 1;
    pub const BUTTON_B: usize = 2;
    pub const BUTTON_A: usize = 3;
    pub const BUTTON_RSR: usize = 4;
    pub const BUTTON_RSL: usize = 5;
    pub const BUTTON_R: usize = 6;
    pub const BUTTON_ZR: usize = 7;
    pub const BUTTON_MINUS: usize = 8;
    pub const BUTTON_PLUS: usize = 9;
    pub const BUTTON_R_STICK: usize = 10;
    pub const BUTTON_L_STICK: usize = 11;
    pub const BUTTON_HOME: usize = 12;
    pub const BUTTON_CAPTURE: usize = 13;
    pub const BUTTON_CHARGING_GRIP: usize = 5;
    pub const BUTTON_DOWN: usize = 16;
    pub const BUTTON_UP: usize = 17;
    pub const BUTTON_RIGHT: usize = 18;
    pub const BUTTON_LEFT: usize = 19;
    pub const BUTTON_LSR: usize = 20;
    pub const BUTTON_LSL: usize = 21;
    pub const BUTTON_L: usize = 22;
    pub const BUTTON_ZL: usize = 23;

    pub const AXIS_LH: usize = 0;
    pub const AXIS_LV: usize = 1;
    pub const AXIS_RH: usize = 2;
    pub const AXIS_RV: usize = 3;
}

mod magic {
    pub const INITIAL_INPUT: [u8; 9] = [0x00, 0x80, 0x00, 0xf8, 0xd7, 0x7a, 0x22, 0xc8, 0x7b];
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
}

fn timestamp() -> u8 {
    (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        & 0xFF) as u8
}

fn response(code: u8, cmd: u8, data: &[u8], hid_tx: &SyncSender<Vec<u8>>) {
    if data.len() + 2 > 64 {
        return;
    }
    let padding = vec![0; 64 - 2 - data.len()];
    let send = [&[code, cmd], data, &padding].concat();
    let _ = hid_tx.send(send);
}

fn uart_response(code: u8, subcmd: u8, input: &[u8], data: &[u8], hid_tx: &SyncSender<Vec<u8>>) {
    response(
        0x21,
        timestamp(),
        &[&[0x81], input, &[0x0c, code, subcmd], data].concat(),
        hid_tx,
    )
}

fn spi_response(addr_lo: u8, addr_hi: u8, input: &[u8], data: &[u8], hid_tx: &SyncSender<Vec<u8>>) {
    let data_len = data.len() as u8;
    uart_response(
        0x90,
        0x10,
        input,
        &[&[addr_lo, addr_hi, 0x00, 0x00, data_len], data].concat(),
        hid_tx,
    );
}

// All credit for this function goes to:
// https://mzyy94.com/blog/2020/03/20/nintendo-switch-pro-controller-usb-gadget/
fn send_response(
    buffer: &[u8],
    input: &[u8],
    hid_tx: &SyncSender<Vec<u8>>,
    colour: &[u8],
    mac_addr: &[u8],
) {
    if buffer.len() < 2 {
        return;
    }
    if buffer[0] == 0x80 {
        match buffer[1] {
            0x01 => response(0x81, 0x01, &[&[0, 3], mac_addr].concat(), hid_tx),
            0x02 => response(0x81, 0x02, &[], hid_tx),
            0x04 => { /* Input sending now (do something?) */ }
            _ => (),
        }
    } else if buffer[0] == 0x01 && buffer.len() > 16 {
        match buffer[10] {
            0x01 => uart_response(0x81, 0x01, input, &[0x03], hid_tx),
            0x02 => uart_response(
                0x82,
                0x02,
                input,
                &[&[0x03, 0x48, 0x03, 0x02], mac_addr, &[0x03, 0x01]].concat(),
                hid_tx,
            ),
            0x03 | 0x08 | 0x30 | 0x38 | 0x40 | 0x48 => {
                uart_response(0x80, buffer[10], input, &[], hid_tx)
            }
            0x04 => uart_response(0x83, 0x04, input, &[], hid_tx),
            0x21 => uart_response(
                0xa0,
                0x21,
                input,
                &[0x01, 0x00, 0xff, 0x00, 0x03, 0x00, 0x05, 0x01],
                hid_tx,
            ),
            0x10 => match buffer[11] {
                0x00 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x00, 0x60, input, &magic::SERIAL_NUMBER, hid_tx)
                    }
                }
                0x50 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x50, 0x60, input, &colour, hid_tx)
                    }
                }
                0x80 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x80, 0x60, input, &magic::SENSOR_STICK_PARAMS, hid_tx)
                    }
                }
                0x98 => {
                    if buffer[12] == 0x60 {
                        spi_response(0x98, 0x60, input, &magic::STICK_PARAMS_2, hid_tx)
                    }
                }
                0x3d => {
                    if buffer[12] == 0x60 {
                        spi_response(0x3d, 0x60, input, &magic::CONFIG, hid_tx)
                    }
                }
                0x10 => {
                    if buffer[12] == 0x80 {
                        spi_response(0x10, 0x80, input, &magic::CALIBRATION, hid_tx)
                    }
                }
                0x28 => {
                    if buffer[12] == 0x80 {
                        spi_response(0x28, 0x80, input, &magic::SENSOR_CALIBRATION, hid_tx)
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

#[derive(Debug)]
pub struct NsProcon {
    hid_path: PathBuf,
    input_state: BitArr!(for 72, in Lsb0, u8),
    colour: Vec<u8>,
    mac_addr: [u8; 6],
    hid_thread_tx: Option<SyncSender<Vec<u8>>>,
    protocol_thread_tx: Option<SyncSender<()>>,
}

impl NsProcon {
    pub fn create<P: AsRef<Path>>(path: P, body_col: [u8; 3]) -> NsProcon {
        let mut procon = NsProcon {
            hid_path: path.as_ref().to_path_buf(),
            input_state: BitArray::zeroed(),
            colour: [body_col, &[0, 0, 0], body_col, body_col].concat(),
            mac_addr: rand::thread_rng().gen::<[u8; 6]>(),
            hid_thread_tx: None,
            protocol_thread_tx: None,
        };
        procon.press(inputs::BUTTON_CHARGING_GRIP, false);
        procon
    }

    fn send_input(&self) {
        let _ = match &self.hid_thread_tx {
            Some(hid_tx) => {
                let mut input_msg = vec![0x30, timestamp(), 0x81];
                input_msg.extend_from_slice(self.input_state.as_buffer());
                input_msg.extend_from_slice(&[0; 52]);
                hid_tx.send(input_msg)
            }
            None => Ok(()),
        };
    }
}

impl Controller for NsProcon {
    type C = NsProcon;

    fn start_comms(&mut self) -> Result<()> {
        let (hid_tx, hid_rx) = mpsc::sync_channel::<Vec<u8>>(10);
        let (protocol_tx, protocol_rx) = mpsc::sync_channel(10);
        let hid_read = OpenOptions::new().read(true).open(&self.hid_path)?;
        let hid_write = OpenOptions::new().write(true).open(&self.hid_path)?;
        let colour = self.colour.clone();
        let mac_addr = self.mac_addr.clone();

        let mut buffer = [0; 64];
        let mut reader = BufReader::new(hid_read);
        let mut writer = BufWriter::new(hid_write);

        // Thread for writing to the HID device
        thread::spawn(move || {
            for to_write in hid_rx {
                // println!("<<< {:02x?}", &to_write);
                let _ = writer.write_all(&to_write);
                let _ = writer.flush();
            }
        });

        self.hid_thread_tx = Some(hid_tx.clone());

        // Thread for responding to data from the Switch
        thread::spawn(move || loop {
            match protool_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            };

            let read = reader.read(&mut buffer).unwrap_or(0);

            if read == 0 {
                continue;
            }

            // println!(
            //     ">>> {:02x} {:02x} {:02x} {:02x} {:02x}",
            //     &buffer[0], &buffer[1], &buffer[10], &buffer[11], &buffer[12]
            // );

            let input = &magic::INITIAL_INPUT;

            if read >= 10 {
                send_response(&buffer, input, &hid_tx, &colour, &mac_addr);
            } else {
                for i in (0..read).step_by(2) {
                    send_response(&buffer[i..(i + 2)], input, &hid_tx, &colour, &mac_addr)
                }
            }
        });
        self.protocol_thread_tx = Some(protocol_tx);
        Ok(())
    }
    fn stop(&mut self) {
        let _ = match &self.protocol_thread_tx {
            Some(protocol_thread_tx) => protocol_thread_tx.send(()),
            None => Ok(()),
        };
        self.protocol_thread_tx = None;
        self.hid_thread_tx = None;
    }

    fn set(&mut self, index: usize, value: bool, flush: bool) {
        self.input_state.set(index, value);
        if flush {
            self.send_input();
        }
    }

    fn press(&mut self, index: usize, flush: bool) {
        self.set(index, true, flush);
    }

    fn release(&mut self, index: usize, flush: bool) {
        self.set(index, false, flush);
    }

    fn set_axis(&mut self, index: usize, value: u16, flush: bool) {
        match index {
            inputs::AXIS_LH => self.input_state[24..36].store(value >> 4),
            inputs::AXIS_LV => self.input_state[36..48].store(value >> 4),
            inputs::AXIS_RH => self.input_state[48..60].store(value >> 4),
            inputs::AXIS_RV => self.input_state[60..72].store(value >> 4),
            _ => (),
        };
        if flush {
            self.send_input();
        }
    }

    fn flush_input(&mut self) {
        self.send_input();
    }

    fn log_state(&self) {
        log::debug!("{:?}", self.input_state);
    }
}
