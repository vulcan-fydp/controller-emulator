use controller_emulator::usb_gadget::ns_procon::ns_procons;

fn main() {
    let procons = ns_procons();

    procons
        .create_config("procons")
        .expect("Could not create configuration");
}
