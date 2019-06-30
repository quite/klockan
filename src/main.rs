use std::thread;
use std::time::Duration;

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
const LEFT_COLON_LOW: u8 = 0x04;
const LEFT_COLON_HIGH: u8 = 0x08;
const DECIMAL_POINT: u8 = 0x10;

const SYMBOLS: [u8; 20] = [
    0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f, // 0..9
    0x77, 0x7c, 0x39, 0x5e, 0x79, 0x71, /* a b C d E F */
    0x00, 0x40, // Blank, Dash
    0b00110000, 0b00000110,
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
    _Left1,
    _Right1,
}

fn main() {
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
    i2c.write(&[
        0x00, //addr
        SYMBOLS[8 as usize],
        0x00,
        SYMBOLS[8 as usize],
        0x00,
        CENTER_COLON | LEFT_COLON_LOW | LEFT_COLON_HIGH | DECIMAL_POINT,
        0x00,
        SYMBOLS[8 as usize],
        0x00,
        SYMBOLS[8 as usize],
        0x00,
    ])
    .ok();
    thread::sleep(Duration::from_millis(500));

    let mut colon = 0;
    while running.load(Ordering::SeqCst) {
        let local = Local::now();
        let h = local.hour() as u8;
        let m = local.minute() as u8;

        // a little brighter during daytime
        let b = if h >= 8 && h <= 17 { 6 } else { 0 };
        i2c.write(&[DIGITALDIM | b as u8]).ok();

        let h10 = if h >= 10 {
            h / 10
        } else {
            Symbol::_Blank as u8
        };
        let segments = [
            SYMBOLS[h10 as usize],
            SYMBOLS[(h % 10) as usize],
            SYMBOLS[(m / 10) as usize],
            SYMBOLS[(m % 10) as usize],
        ];
        colon = colon ^ CENTER_COLON;
        i2c.write(&[
            0x00, //addr
            segments[0],
            0x00,
            segments[1],
            0x00,
            colon,
            0x00,
            segments[2],
            0x00,
            segments[3],
            0x00,
        ])
        .ok();

        thread::sleep(Duration::from_millis(1000));
    }

    // flatline on shutdown
    i2c.write(&[
        0x00, //addr
        0b1000000, 0x00, 0b1000000, 0x00, 0, 0x00, 0b1000000, 0x00, 0b1000000, 0x00,
    ])
    .ok();
    thread::sleep(Duration::from_millis(500));

    i2c.write(&[SYSTEMSET | SS_OSCILLATOR_OFF]).ok();
}
