extern crate postgres;

use TableDesc;
use super::*;
use postgres::types;

#[test]
fn test_color() {
    assert_eq!("\x1B[33m\x1B[1m\x1B[33m\x1B[37mfoo\x1B[33m\x1B[0m", TableDesc::color_text("foo", Color::BoldWhite));
}

#[test]
fn test_alignment() {
    assert_eq!(TableDesc::get_alignment(&types::Type::Int4), Align::Right);
}

#[test]
fn test_padding() {
    assert_eq!("     ", TableDesc::pad_gen(5, " "));
    assert_eq!("-----", TableDesc::pad_gen(5, "-"));
}

#[test]
fn test_formatting() {
    assert_eq!("foo   ", TableDesc::format_field("foo", 6, Align::Left));
    assert_eq!("   foo", TableDesc::format_field("foo", 6, Align::Right));
    assert_eq!(" foo  ", TableDesc::format_field("foo", 6, Align::Center));
}
