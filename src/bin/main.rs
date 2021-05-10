use std::io::{stdout, BufWriter};

fn main() {
    let stdout = stdout();
    let message = String::from("Hellow fellow Rustaceans!");
    let width = message.chars().count();
}
