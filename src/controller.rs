use std::io::Result;
pub mod ns_procon;

pub trait Controller {
    type C;

    fn start_comms(&mut self) -> Result<()>;
    fn stop(&mut self);

    fn set(&mut self, index: usize, value: bool, flush: bool);
    fn press(&mut self, index: usize, flush: bool);
    fn release(&mut self, index: usize, flush: bool);
    fn set_axis(&mut self, index: usize, value: u16, flush: bool);
    fn flush_input(&mut self);

    fn log_state(&self);
}
