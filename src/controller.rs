use anyhow::Result;
pub mod ns_procon;

pub trait Controller {
    type C;

    fn start_comms(&mut self) -> Result<()>;
    fn stop(&mut self);

    fn set(&mut self, index: usize, value: bool, flush: bool) -> Result<()>;
    fn press(&mut self, index: usize, flush: bool) -> Result<()>;
    fn release(&mut self, index: usize, flush: bool) -> Result<()>;
    fn set_axis(&mut self, index: usize, value: u16, flush: bool) -> Result<()>;
    fn flush_input(&mut self) -> Result<()>;

    fn log_state(&self);
}
