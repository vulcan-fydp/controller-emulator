use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::prelude::*;
use std::io::Result;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

pub mod ns_procon;

pub enum Speed {
    LowSpeed,
    FullSpeed,
    HighSpeed,
    SuperSpeed,
}

impl Default for Speed {
    fn default() -> Self {
        Speed::FullSpeed
    }
}

impl Speed {
    fn to_string(&self) -> String {
        match self {
            Speed::LowSpeed => "low-speed".to_string(),
            Speed::FullSpeed => "full-speed".to_string(),
            Speed::HighSpeed => "high-speed".to_string(),
            Speed::SuperSpeed => "super-speed".to_string(),
        }
    }
}

#[derive(Default)]
pub struct Config {
    attributes: u8,
    max_power: u32,
    description: String,

    hid_functions: Vec<u32>,
}

#[derive(Default, Clone)]
pub struct HIDFunction {
    pub(in crate) protocol: u32,
    pub(in crate) report_desc: Vec<u8>,
    pub(in crate) report_length: u32,
    pub(in crate) subclass: u32,
}

#[derive(Default)]
pub struct Gadget {
    pub(in crate) max_speed: Speed,
    pub(in crate) device_class: u8,
    pub(in crate) device_sub_class: u8,
    pub(in crate) device_protocol: u8,
    pub(in crate) device_max_packet_size: u8,

    pub(in crate) device_version: u32,
    pub(in crate) usb_version: u32,
    pub(in crate) product_id: u32,
    pub(in crate) vendor_id: u32,

    pub(in crate) configs: Vec<Config>,
    pub(in crate) hid_functions: Vec<HIDFunction>,

    pub(in crate) serialnumber: String,
    pub(in crate) product: String,
    pub(in crate) manufacturer: String,
}

fn write_file(path: &Path, name: &str, contents: &[u8]) -> Result<()> {
    create_dir_all(&path)?;
    let file_name = Path::join(path, name);
    let mut file = File::create(&file_name)?;
    file.write_all(contents).expect("couldn't write");
    Ok(())
}

fn write_file_str(path: &Path, name: &str, contents: &str) -> Result<()> {
    write_file(path, name, contents.as_bytes())
}

fn write_file_byte(path: &Path, name: &str, contents: u8) -> Result<()> {
    write_file_str(path, name, &format!("{:#04x}", contents))
}

fn write_file_int(path: &Path, name: &str, contents: u32) -> Result<()> {
    write_file_str(path, name, &format!("{:#06x}", contents))
}

impl Gadget {
    pub fn create_config(&self, name: &str) -> Result<()> {
        let base_path = Path::join(Path::new("/sys/kernel/config/usb_gadget"), name);

        // Remove existing configuration
        let _ = remove_dir_all(&base_path);

        write_file_str(&base_path, "max_speed", &self.max_speed.to_string())?;
        write_file_byte(&base_path, "bDeviceClass", self.device_class)?;
        write_file_byte(&base_path, "bDeviceSubClass", self.device_sub_class)?;
        write_file_byte(&base_path, "bDeviceProtocol", self.device_protocol)?;
        write_file_byte(&base_path, "bMaxPacketSize0", self.device_max_packet_size)?;

        write_file_int(&base_path, "bcdDevice", self.device_version)?;
        write_file_int(&base_path, "bcdUSB", self.usb_version)?;
        write_file_int(&base_path, "idProduct", self.product_id)?;
        write_file_int(&base_path, "idVendor", self.vendor_id)?;

        let strings_path = Path::join(&base_path, "strings/0x409");

        write_file_str(&strings_path, "serialnumber", &self.serialnumber)?;
        write_file_str(&strings_path, "product", &self.product)?;
        write_file_str(&strings_path, "manufacturer", &self.manufacturer)?;

        for i in 0..self.hid_functions.len() {
            let hid_path = Path::join(&base_path, format!("functions/hid.usb.{}", i));
            let function = &self.hid_functions[i];
            write_file_int(&hid_path, "protocol", function.protocol)?;
            write_file(&hid_path, "report_desc", &function.report_desc)?;
            write_file_int(&hid_path, "report_length", function.report_length)?;
            write_file_int(&hid_path, "subclass", function.subclass)?;
        }

        for i in 1..=self.configs.len() {
            let config_path = Path::join(&base_path, format!("configs/c.{}", i));
            let config = &self.configs[i - 1];
            write_file_byte(&config_path, "bmAttributes", config.attributes)?;
            write_file_int(&config_path, "MaxPower", config.max_power)?;

            let string_path = Path::join(&config_path, "/strings/0x409");
            write_file_str(&string_path, "idVendor", &config.description)?;

            for hid in &config.hid_functions {
                let hid_path = Path::join(&base_path, format!("functions/hid.usb.{}", hid));
                let link_path = Path::join(&config_path, format!("hid.usb.{}", hid));
                symlink(&hid_path, &link_path)?;
            }
        }

        Ok(())
    }
}

pub fn activate(name: &str) -> Result<()> {
    let path = Path::new("/sys/kernel/config/usb_gadget")
            .join(name)
            .join("UDC");
    Command::new("sh").arg("-c").arg(format!("echo \"\" > {}", path.display())).status()?;
    let output = Command::new("ls").arg("/sys/class/udc").output()?;
    let mut udc = File::create(&path)?;
    udc.write_all(&output.stdout)?;
    Ok(())
}
