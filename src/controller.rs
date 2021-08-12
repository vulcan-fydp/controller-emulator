use std::io::Result;
use std::path::Path;
pub mod ns_procon;

pub trait Controller {
    type C;

    fn create<P: AsRef<Path>>(path: P) -> Self::C;

    fn start_comms(&mut self) -> Result<()>;
    fn stop(&mut self);

    fn set(&mut self, index: usize, value: bool);
    fn press(&mut self, index: usize);
    fn release(&mut self, index: usize);
    fn set_axis(&mut self, index: usize, value: u16);

    fn log_state(&self);
}
