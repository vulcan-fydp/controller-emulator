use controller_emulator::controller::ns_procon;
use controller_emulator::controller::Controller;
use controller_emulator::usb_gadget;
use controller_emulator::usb_gadget::ns_procon::ns_procons;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let procons = ns_procons();

    procons
        .create_config("procons")
        .expect("Could not create configuration");
    usb_gadget::activate("procons").expect("Could not activate");

    let mut procon_1 = ns_procon::NsProcon::create("/dev/hidg0");
    procon_1
        .start_comms()
        .expect("Couldn't start communicating");

    sleep(Duration::from_secs(10));
    // procon_1.set_axis(ns_procon::inputs::AXIS_LH, 1 << 15);
    // procon_1.set_axis(ns_procon::inputs::AXIS_LV, 1 << 15);
    // procon_1.set_axis(ns_procon::inputs::AXIS_RH, 1 << 15);
    // procon_1.set_axis(ns_procon::inputs::AXIS_RV, 1 << 15);

    // for i in 0..100 {
    //     procon_1.press(ns_procon::inputs::BUTTON_A);
    //     println!("{:?}", procon_1);
    //     sleep(Duration::from_secs(1));
    //     procon_1.release(ns_procon::inputs::BUTTON_A);
    //     sleep(Duration::from_secs(1));
    // }

    procon_1.stop();
}
