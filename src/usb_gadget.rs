pub mod usb_gadget {
    pub enum Speed {
        LowSpeed,
        FullSpeed,
        HighSpeed,
        SuperSpeed,
    }

    pub struct Gadget {
        max_speed: Speed,
        device_class: u8,
        device_sub_class: u8,
        device_protocol: u8,
        device_max_packet_size: u8,

        device_version: u32,
        usb_version: u32,
        product_id: u32,
        vendor_id: u32,

        attributes: u8,
        max_power: u32,
    }
}
