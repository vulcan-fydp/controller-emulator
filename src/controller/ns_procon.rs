use crate::controller::Controller;
use bitvec::prelude::*;
use std::fs::File;
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

#[derive(Debug)]
pub struct NsProcon {
    hid_path: PathBuf,
    input_state: BitArr!(for 72, in Lsb0, u8),
    thread_tx: Option<Sender<()>>,
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
        let hid_read = File::open(&self.hid_path)?;
        let hid_write = File::open(&self.hid_path)?;

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
