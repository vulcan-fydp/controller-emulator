use anyhow::Result;
use std::sync::mpsc::Receiver;
pub mod ns_procon;

pub enum ControllerEvent {
    InputActive,
    PlayerLights(u8),
}

pub trait Controller {
    type C;

    fn start_comms(&mut self) -> Result<()>;
    fn stop(&mut self);

    fn set(&mut self, index: usize, value: bool, flush: bool) -> Result<()>;
    fn press(&mut self, index: usize, flush: bool) -> Result<()>;
    fn release(&mut self, index: usize, flush: bool) -> Result<()>;
    fn set_axis(&mut self, index: usize, value: u16, flush: bool) -> Result<()>;
    fn flush_input(&mut self) -> Result<()>;

    fn listen_for_events(&mut self) -> &Receiver<ControllerEvent>;

    fn log_state(&self);
}
