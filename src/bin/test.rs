use controller_emulator::controller::ns_procon;
use controller_emulator::controller::Controller;
// use controller_emulator::usb_gadget;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut procon_1 = ns_procon::NsProcon::create_separate("test.out", "test.in", [255, 0, 0]);
    // let mut procon_2 = ns_procon::NsProcon::create("/dev/hidg1", [0, 150, 0]);
    // let mut procon_3 = ns_procon::NsProcon::create("/dev/hidg2", [255, 255, 0]);
    // let mut procon_4 = ns_procon::NsProcon::create("/dev/hidg3", [40, 40, 255]);

    println!("Starting procon 1");
    procon_1
        .start_comms()
        .expect("Couldn't start communicating");
    // println!("Starting procon 2");
    // procon_2
    //     .start_comms()
    //     .expect("Couldn't start communicating");
    // println!("Starting procon 3");
    // procon_3
    //     .start_comms()
    //     .expect("Couldn't start communicating");
    // println!("Starting procon 4");
    // procon_4
    //     .start_comms()
    //     .expect("Couldn't start communicating");

    for _i in 0..100 {
        procon_1.press(ns_procon::inputs::BUTTON_A, true);
        // procon_2.press(ns_procon::inputs::BUTTON_A, true);
        // procon_3.press(ns_procon::inputs::BUTTON_A, true);
        // procon_4.press(ns_procon::inputs::BUTTON_A, true);
        sleep(Duration::from_secs(1));
        procon_1.release(ns_procon::inputs::BUTTON_A, true);
        // procon_2.release(ns_procon::inputs::BUTTON_A, true);
        // procon_3.release(ns_procon::inputs::BUTTON_A, true);
        // procon_4.release(ns_procon::inputs::BUTTON_A, true);
        sleep(Duration::from_secs(1));
    }

    procon_1.stop();
    // procon_2.stop();
    // procon_3.stop();
    // procon_4.stop();
}
