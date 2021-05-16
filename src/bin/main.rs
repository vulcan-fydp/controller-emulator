use controller_emulator::controller::ns_procon;
use controller_emulator::controller::Controller;
use controller_emulator::usb_gadget;
use controller_emulator::usb_gadget::ns_procon::ns_procons;

fn main() {
    // let procons = ns_procons();

    // procons
    //     .create_config("procons")
    //     .expect("Could not create configuration");
    // usb_gadget::activate("procons");

    let mut procon_1 = ns_procon::NsProcon::create("/dev/hidg0");
    procon_1
        .start_comms()
        .expect("Couldn't start communicating");

    procon_1.press(ns_procon::inputs::BUTTON_A);
    procon_1.set_axis(ns_procon::inputs::AXIS_LH, 1 << 15);
    println!("{:?}", procon_1);
    procon_1.release(ns_procon::inputs::BUTTON_A);

    procon_1.stop();
}
