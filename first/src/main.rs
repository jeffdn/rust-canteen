extern crate postgres;
extern crate chrono;

use postgres::{Connection, SslMode, types};
use std::io::{self, Read};

enum Align {
    Left,
    Right,
    Center,
}

#[allow(dead_code)]
enum Color {
    White,
    BoldWhite,
    BoldRed,
    BoldGreen,
    BoldBlue,
}

fn color_text(text: &str, color: Color) -> String {
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
        _ => String::from(""),
    };

    val
}

fn get_alignment(coltype: &Vec<types::Type>, colpos: usize) -> Align {
    match coltype[colpos] {
        types::Type::Int2   => Align::Right,
        types::Type::Int4   => Align::Right,
        types::Type::Int8   => Align::Right,
        types::Type::Float4 => Align::Right,
        types::Type::Float8 => Align::Right,
        _                   => Align::Left,
    }
}

fn format_field(column: &str, width: usize, align: Align) -> String {
    let mut padstr = String::from("");
    let padlen: usize = width - column.len();

    for _ in 0..padlen {
        padstr = padstr + " ";
    }

    let ret: String = match align {
        Align::Right => format!("{}{}", padstr, column),
        Align::Left  => format!("{}{}", column, padstr),
    };

    ret
}

fn print_row(coltypes: &Vec<types::Type>, colwidths: &Vec<usize>, rowdata: &Vec<String>) {
    for (i, col) in rowdata.iter().enumerate() {
        let align = get_alignment(&coltypes, i);

        if i > 0 {
            print!("|");
        }

        print!(" {} ", format_field(&col, colwidths[i], align));
    }

    println!("");
}

fn print_header(colwidths: &Vec<usize>) {
    for (i, col) in colwidths.iter().enumerate() {
        if i > 0 {
            print!("+");
        }

        print!("-{data:-<width$}-", data="-", width=col);
    }

    println!("");
}

fn main() {
    let mut buf = String::new();
    let mut colnames: Vec<String> = Vec::new();
    let mut coldata: Vec<Vec<String>> = Vec::new();
    let mut colwidths: Vec<usize> = Vec::new();
    let mut coltypes: Vec<types::Type> = Vec::new();

    io::stdin().read_to_string(&mut buf);

    let conn = Connection::connect("postgres://jeff@localhost", SslMode::None).unwrap();
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

    print_row(&coltypes, &colwidths, &colnames);
    print_header(&colwidths);
    for rowdata in coldata {
        print_row(&coltypes, &colwidths, &rowdata);
    }
}
