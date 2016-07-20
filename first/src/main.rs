extern crate postgres;
extern crate chrono;

#[cfg(test)]
mod tests;

use postgres::{Connection, SslMode, types};
use std::io::{self, Read};

#[derive(PartialEq, Debug)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum Color {
    White,
    BoldWhite,
    BoldRed,
    BoldGreen,
    BoldBlue,
}

pub fn color_text(text: &str, color: Color) -> String {
    let clrstr = match color {
        Color::White       => { format!("{}{}\x1B[33m\x1B[0m", "\x1B[33m\x1B[37m", text)        },
        Color::BoldWhite   => { format!("{}{}\x1B[33m\x1B[0m", "\x1B[33m\x1B[1m\x1B[33m\x1B[37m", text) },
        Color::BoldRed     => { format!("{}{}\x1B[33m\x1B[0m", "\x1B[33m\x1B[1m\x1B[33m\x1B[31m", text) },
        Color::BoldGreen   => { format!("{}{}\x1B[33m\x1B[0m", "\x1B[33m\x1B[1m\x1B[33m\x1B[32m", text) },
        Color::BoldBlue    => { format!("{}{}\x1B[33m\x1B[0m", "\x1B[33m\x1B[1m\x1B[33m\x1B[34m", text) },
    };

    clrstr
}

fn parse_result(column: &postgres::rows::Row, coltype: &Vec<types::Type>, colpos: usize) -> String {
    let val: String = match coltype[colpos] {
        types::Type::Text | types::Type::Varchar => column.get(colpos),
        types::Type::Bool => {
            let tmpbool: bool = column.get(colpos);
            match tmpbool {
                true  => String::from("true"),
                false => String::from("false"),
            }
        },
        types::Type::Int2 | types::Type::Int4 => {
            let tmpint: i32 = column.get(colpos);
            tmpint.to_string()
        },
        types::Type::Int8 => {
            let tmpint: i64 = column.get(colpos);
            tmpint.to_string()
        },
        types::Type::Float4 => {
            let tmpflt: f32 = column.get(colpos);
            tmpflt.to_string()
        },
        types::Type::Float8 => {
            let tmpflt: f64 = column.get(colpos);
            tmpflt.to_string()
        },
        types::Type::Timestamp => {
            let tmpdate: chrono::NaiveDateTime = column.get(colpos);
            tmpdate.format("%Y-%m-%d %H:%M:%s").to_string()
        },
        types::Type::TimestampTZ => {
            let tmpdate: chrono::DateTime<chrono::UTC> = column.get(colpos);
            tmpdate.format("%Y-%m-%d %H:%M:%s").to_string()
        },
        types::Type::Date => {
            let tmpdate: chrono::NaiveDate = column.get(colpos);
            tmpdate.format("%Y-%m-%d").to_string()
        },
        _ => String::from(""),
    };

    val
}

pub fn get_alignment(coltype: &types::Type) -> Align {
    match coltype {
        &types::Type::Int2          => Align::Right,
        &types::Type::Int4          => Align::Right,
        &types::Type::Int8          => Align::Right,
        &types::Type::Float4        => Align::Right,
        &types::Type::Float8        => Align::Right,
        &types::Type::Date          => Align::Right,
        &types::Type::Timestamp     => Align::Right,
        &types::Type::TimestampTZ   => Align::Right,
        _                           => Align::Left,
    }
}

pub fn pad_gen(len: usize, pad: &str) -> String {
    let mut padstr = String::from("");

    for _ in 0..len {
        padstr = padstr + pad;
    }

    padstr
}

pub fn format_field(column: &str, width: usize, align: Align) -> String {
    let padlen: usize = width - column.len();
    let extra: usize = padlen % 2;

    let ret: String = match align {
        Align::Center   => { format!("{}{}{}", pad_gen(padlen/2, " "), column, pad_gen((padlen/2)+extra, " ")) },
        Align::Right    => { format!("{}{}", pad_gen(padlen, " "), column) },
        Align::Left     => { format!("{}{}", column, pad_gen(padlen, " ")) },
    };

    ret
}

fn print_row(coltypes: &Vec<types::Type>, colwidths: &Vec<usize>, rowdata: &Vec<String>) {
    for (i, col) in rowdata.iter().enumerate() {
        if i > 0 {
            print!("{}", color_text("|", Color::BoldWhite));
        }

        print!(" {} ", format_field(&col, colwidths[i], get_alignment(&coltypes[i])));
    }

    println!("");
}

fn print_header(colwidths: &Vec<usize>, colnames: &Vec<String>) {
    for (i, name) in colnames.iter().enumerate() {
        if i > 0 {
            print!("{}", color_text("|", Color::BoldWhite));
        }

        print!(" {} ", color_text(&format_field(&name, colwidths[i], Align::Center), Color::BoldWhite));
    }


    println!("");
    for (i, col) in colwidths.iter().enumerate() {
        if i > 0 {
            print!("{}", color_text("|", Color::BoldWhite));
        }

        print!("{}", color_text(&pad_gen(col+2, "-"), Color::BoldWhite));
    }

    println!("");
}

fn main() {
    let mut buf = String::new();
    let mut colnames: Vec<String> = Vec::new();
    let mut coldata: Vec<Vec<String>> = Vec::new();
    let mut colwidths: Vec<usize> = Vec::new();
    let mut coltypes: Vec<types::Type> = Vec::new();
    let mut args = std::env::args();

    if args.len() == 1 {
        println!("{} {}", color_text("error:", Color::BoldRed), color_text("a DBURI is required!", Color::BoldWhite));
        std::process::exit(1);
    }

    let connstr: String = args.nth(1).unwrap();
    io::stdin().read_to_string(&mut buf);

    let conn = Connection::connect(&connstr as &str, SslMode::None).unwrap();
    let res = &conn.query(&mut buf, &[]).unwrap();
    let numcols = res.columns().len();

    for col in res.columns() {
        let colname = String::from(col.name());
        colwidths.push(colname.len());
        colnames.push(colname);
        coltypes.push(col.type_().clone());
    }

    for row in res.iter() {
        let mut colvals: Vec<String> = Vec::new();

        for i in 0..numcols {
            let ret = row.get_bytes(i);
            let col = if ret.is_some() { parse_result(&row, &coltypes, i) } else { String::from("") };

            if colwidths[i] < col.len() {
                colwidths[i] = col.len();
            }

            colvals.push(col);
        }

        coldata.push(colvals);
    }

    print_header(&colwidths, &colnames);
    for rowdata in coldata {
        print_row(&coltypes, &colwidths, &rowdata);
    }
}
