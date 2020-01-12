use std::thread;
use std::time::Duration;

extern crate structopt;
use structopt::StructOpt;

extern crate chrono;
use chrono::prelude::*;

extern crate rppal;
use rppal::i2c::I2c;
use rppal::system::DeviceInfo;

extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const I2CADDR: u8 = 0x70;

const SYSTEMSET: u8 = 0x20;
const SS_OSCILLATOR_ON: u8 = 0x01;
const SS_OSCILLATOR_OFF: u8 = 0x00;

const DISPLAYSET: u8 = 0x80;
const DS_DISPLAY_ON: u8 = 0x01;
// const DS_DISPLAY_OFF: u8 = 0x00;
const DS_BLINK_OFF: u8 = 0x00;
// const DS_BLINK_2HZ: u8 = 0x02;
// const DS_BLINK_1HZ: u8 = 0x04;
// const DS_BLINK_HALFHZ: u8 = 0x06;

const DIGITALDIM: u8 = 0xe0;

const CENTER_COLON: u8 = 0x02;
const LEFT_COLON_HIGH: u8 = 0x04;
const LEFT_COLON_LOW: u8 = 0x08;
const DECIMAL_POINT: u8 = 0x10;

const SYMBOLS: [u8; 18] = [
    0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f, // 0..9
    0x77, 0x7c, 0x39, 0x5e, 0x79, 0x71, /* a b C d E F */
    0x00, 0x40, // Blank, Dash
];
// for indexing SYMBOLS
pub enum Symbol {
    _A = 10,
    _B,
    _C,
    _D,
    _E,
    _F,
    _Blank,
    _Dash,
}

fn display(i2c: &mut I2c, first: u8, second: u8, dots: u8, third: u8, fourth: u8) {
    i2c.write(&[
        0x00, //addr
        first, 0x00, second, 0x00, dots, 0x00, third, 0x00, fourth, 0x00,
    ])
    .ok();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "klockan")]
struct Options {
    /// flash the colon every second
    #[structopt(short = "f", long = "flash")]
    flash: bool,
}

fn nth_digit(num: u32, nth: usize) -> u32 {
    let digits: Vec<_> = num
        .to_string()
        .chars()
        .map(|d| d.to_digit(10).unwrap())
        .collect();
    return digits.into_iter().nth(nth).unwrap();
}

fn main() {
    let options = Options::from_args();

    let device_info = DeviceInfo::new().unwrap();
    println!("{}, {}", device_info.model(), device_info.soc());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("set_handler fail");

    let mut i2c = I2c::new().unwrap();

    i2c.set_slave_address(I2CADDR as u16).ok();
    i2c.write(&[SYSTEMSET | SS_OSCILLATOR_ON]).ok();

    i2c.write(&[DISPLAYSET | DS_DISPLAY_ON | DS_BLINK_OFF]).ok();

    // max out on startup!
    display(
        &mut i2c,
        SYMBOLS[8 as usize],
        SYMBOLS[8 as usize],
        CENTER_COLON | LEFT_COLON_LOW | LEFT_COLON_HIGH | DECIMAL_POINT,
        SYMBOLS[8 as usize],
        SYMBOLS[8 as usize],
    );
    thread::sleep(Duration::from_millis(500));

    let mut colon = 0;
    while running.load(Ordering::SeqCst) {
        let local = Local::now();
        let year = local.year() as u32;
        let mon = local.month() as u8;
        let dom = local.day() as u8;
        let hh = local.hour() as u8;
        let mm = local.minute() as u8;
        let ss = local.second() as u8;

        // a little brighter during daytime, and more so during brighter months
        let brightness = if hh >= 8 && hh <= 17 {
            if mon >= 3 || mon <= 10 {
                6
            } else {
                3
            }
        } else {
            0
        };
        i2c.write(&[DIGITALDIM | brightness as u8]).ok();

        let h_ = if hh >= 10 {
            hh / 10
        } else {
            Symbol::_Blank as u8
        };

        if options.flash {
            colon ^= CENTER_COLON;
        }

        match ss {
            0..=57 => display(
                &mut i2c,
                SYMBOLS[h_ as usize],
                SYMBOLS[(hh % 10) as usize],
                colon,
                SYMBOLS[(mm / 10) as usize],
                SYMBOLS[(mm % 10) as usize],
            ),
            58 => display(
                &mut i2c,
                SYMBOLS[nth_digit(year, 0) as usize],
                SYMBOLS[nth_digit(year, 1) as usize],
                LEFT_COLON_LOW,
                SYMBOLS[nth_digit(year, 2) as usize],
                SYMBOLS[nth_digit(year, 3) as usize],
            ),
            59 => display(
                &mut i2c,
                SYMBOLS[(mon / 10) as usize],
                SYMBOLS[(mon % 10) as usize],
                LEFT_COLON_LOW,
                SYMBOLS[(dom / 10) as usize],
                SYMBOLS[(dom % 10) as usize],
            ),
            _ => (),
        }

        thread::sleep(Duration::from_millis(1000));
    }

    // flatline on shutdown
    display(&mut i2c, 0b100_0000, 0b100_0000, 0, 0b100_0000, 0b100_0000);
    thread::sleep(Duration::from_millis(500));

    i2c.write(&[SYSTEMSET | SS_OSCILLATOR_OFF]).ok();
}
