use crate::value::*;
use pracstro::time;

/// A set of functions that handle the formatting of queries
pub struct Driver {
    /// Starting information
    pub start: fn() -> (),
    /// Headers for columns, usually
    pub propheader: fn(Vec<(Value, String)>, time::Date) -> (),
    /// The formatting in a normal query
    pub query: fn(Vec<(Value, String)>) -> (),
    /// The formatting in a ephemeris query
    pub ephemq: fn(Vec<(Value, String)>, time::Date) -> (),
    /// Ending information
    pub footer: fn() -> (),
}

pub fn nop() {}
pub fn nop_fa(_: Vec<(Value, String)>, _: time::Date) {}

fn term_proph(rs: Vec<(Value, String)>, d: time::Date) {
    print!("{:^22}", "date");
    rs.iter().for_each(|x| print!("{:^29}", x.1));
    print!("\n{:=<1$}\n", "", 29 * rs.len() + 22);
}
fn term_q(rs: Vec<(Value, String)>) {
    println!(
        "{}",
        rs.iter()
            .map(|x| x.0.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    );
}
fn term_eq(rs: Vec<(Value, String)>, d: time::Date) {
    print!("{:^22}", Value::Date(d).to_string());
    rs.iter().for_each(|x| print!("{:<29}", x.0.to_string()));
    println!();
}
pub const TERM: Driver = Driver {
    start: nop,
    propheader: term_proph,
    query: term_q,
    ephemq: term_eq,
    footer: nop,
};

fn csv_proph(rs: Vec<(Value, String)>, _d: time::Date) {
    println!(
        "date,{}",
        rs.iter()
            .map(|x| x.1.clone())
            .collect::<Vec<String>>()
            .join(",")
    )
}
fn csv_q(rs: Vec<(Value, String)>) {
    println!(
        "{}",
        rs.iter()
            .map(|x| x.0.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
}

fn csv_eq(rs: Vec<(Value, String)>, d: time::Date) {
    println!(
        "{},{}",
        Value::Date(d),
        rs.iter()
            .map(|x| x.0.to_string())
            .collect::<Vec<String>>()
            .join(",")
    )
}
pub const CSV: Driver = Driver {
    start: nop,
    propheader: csv_proph,
    query: csv_q,
    ephemq: csv_eq,
    footer: nop,
};

fn json_init() {
    print!("{{ \"q\": [");
}
fn json_q(rs: Vec<(Value, String)>) {
    print!("{{");
    rs.iter().for_each(|(x, y)| print!("\"{}\": {:b},", y, x));
    print!("\"isq\": true }},");
}
fn json_eq(rs: Vec<(Value, String)>, d: time::Date) {
    print!("{{ \"timestamp\": {},", d.unix());
    rs.iter().for_each(|(x, y)| print!("\"{}\": {:b},", y, x));
    print!("\"isq\": true }},");
}
fn json_footer() {
    print!("{{\"isq\": false}} ] }}");
}
pub const JSON: Driver = Driver {
    start: json_init,
    propheader: nop_fa,
    query: json_q,
    ephemq: json_eq,
    footer: json_footer,
};
