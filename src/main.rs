use pracstro::{coord, moon, sol, time};

mod value {
use pracstro::{coord, moon, sol, time};
use std::fmt;
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RefFrame {
    pub lat: time::Period,
    pub long: time::Period,
    pub date: time::Date,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PhaseView {
    Default,
    Nemoji,
    Semoji,
    Illumfrac,
    PhaseName,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CrdView {
    Equatorial,
    Horizontal(RefFrame),
    Ecliptic(time::Date),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CelObj {
    Planet(sol::Planet),
    Moon,
    Sun,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PerView {
    Angle,
    Latitude,
    Time,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    // Primatives
    Date(time::Date),
    Per(time::Period, PerView),
    Crd(coord::Coord, CrdView),
    Num(f64),
    Dist(f64),
    Phase(time::Period, PhaseView),
    // Celestial Objects
    Obj(CelObj),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const EMOJIS: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
        const SEMOJI: [&str; 8] = ["🌑", "🌘", "🌗", "🌖", "🌕", "🌔", "🌓", "🌒"];
        const PNAMES: [&str; 8] = [
            "New",
            "Waxing Crescent",
            "First Quarter",
            "Waxing Gibbous",
            "Full",
            "Waning Gibbous",
            "Last Quarter",
            "Waning Crescent",
        ];

        fn phaseidx(ilumfrac: f64, ang: time::Period) -> usize {
            match (ilumfrac, ang.degrees() > 90.0) {
                (0.00..0.04, _) => 0,
                (0.96..1.00, _) => 4,
                (0.46..0.54, true) => 6,
                (0.46..0.54, false) => 2,
                (0.54..0.96, true) => 5,
                (0.54..0.96, false) => 3,
                (_, true) => 7,
                (_, false) => 1,
            }
        }

        use chrono::DateTime;
        match self {
            Value::Date(d) => write!(
                f,
                "{}",
                DateTime::from_timestamp(d.unix() as i64, 0)
                    .expect("Invalid Date")
                    .format("%Y-%m-%dT%T")
            ),
            Value::Per(p, PerView::Angle) => {
                let (d, m, s) = p.degminsec();
                write!(f, "{:02}°{:02}′{:02.1}″", d, m, s)
            }
            Value::Per(p, PerView::Latitude) => {
                let (d, m, s) = p.to_latitude().degminsec();
                write!(f, "{:+02}°{:02}′{:02.1}″", d, m, s)
            }
            //Value::Per(p, PerView::Raw) => write!(f, "{:.5}", p.degrees()),
            Value::Per(p, PerView::Time) => {
                let (h, m, s) = p.clock();
                write!(f, "{:02}h{:02}m{:02}s", h, m, s.trunc())
            }
            Value::Dist(d) => write!(f, "{} AU", d),
            Value::Crd(c, CrdView::Equatorial) => {
                let d = c.equatorial();
                write!(
                    f,
                    "{} {}",
                    Value::Per(d.0, PerView::Time),
                    Value::Per(d.1, PerView::Latitude)
                )
            }
            Value::Crd(c, CrdView::Horizontal(rf)) => {
                let d = c.horizon(rf.date, rf.date.time(), rf.lat, rf.long);
                write!(
                    f,
                    "{} {}",
                    Value::Per(d.0, PerView::Angle),
                    Value::Per(d.1, PerView::Latitude)
                )
            }
            Value::Crd(c, CrdView::Ecliptic(d)) => {
                let d = c.ecliptic(*d);
                write!(
                    f,
                    "{} {}",
                    Value::Per(d.0, PerView::Angle),
                    Value::Per(d.1, PerView::Latitude)
                )
            }
            Value::Phase(pa, PhaseView::Default) => {
                let ilf = (1.0 - pa.cos()) / 2.0;
                let pi = phaseidx(ilf, *pa);
                write!(f, "{} {} ({:2.1}%)", EMOJIS[pi], PNAMES[pi], ilf * 100.0)
            }
            Value::Phase(pa, PhaseView::Nemoji) => {
                write!(f, "{}", EMOJIS[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
            }
            Value::Phase(pa, PhaseView::Semoji) => {
                write!(f, "{}", SEMOJI[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
            }
            Value::Phase(pa, PhaseView::Illumfrac) => {
                write!(f, "{:2.1}", 100.0 * (1.0 - pa.cos()) / 2.0)
            }
            Value::Phase(pa, PhaseView::PhaseName) => {
                write!(f, "{}", PNAMES[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
            }
            Value::Num(n) => write!(f, "{:0.2}", n),
            Value::Obj(_p) => write!(f, "Celestial Object"),
        }
    }
}
}

use value::*;


type Step = (f64, f64, f64);

fn step_date(d: time::Date, s: Step) -> time::Date {
    let (y, mon, d, t) = d.calendar();
    time::Date::from_calendar(y + s.0 as i64, mon + s.1 as u8, d + s.2 as u8, t)
}

mod parse {
    use super::*;
    use chrono::prelude::*;

    fn suffix_num(s: &str, j: &str) -> Option<f64> {
        s.strip_suffix(j)?.parse::<f64>().ok()
    }

    /// A step in time, returns (years, months, days, hours, minutes, seconds)
    pub fn step(s: &str) -> Result<Step, &'static str> {
        if let Some(n) = suffix_num(s, "y") {
            Ok((n, 0.0, 0.0))
        } else if let Some(n) = suffix_num(s, "mon") {
            Ok((0.0, n, 0.0))
        } else if let Some(n) = suffix_num(s, "w") {
            Ok((0.0, 0.0, n * 7.0))
        } else if let Some(n) = suffix_num(s, "d") {
            Ok((0.0, 0.0, n))
        } else {
            Err("Bad interval")
        }
    }

    pub fn ephemq(s: &str) -> Result<(time::Date, Step, time::Date), &'static str> {
        let mut eq = s.split(',');
        let start = eq.next().ok_or("Bad CSV")?;
        let ste = eq.next().ok_or("Bad CSV")?;
        let end = eq.next().ok_or("Bad CSV")?;
        Ok((date(start)?, step(ste)?, date(end)?))
    }

    /// The inbuilt RFC3339/ISO6901 date parser in chrono does not support subsets of the formatting.
    pub fn date(s: &str) -> Result<time::Date, &'static str> {
        if s.starts_with("@") {
            Ok(time::Date::from_unix(
                s.strip_prefix("@")
                    .ok_or("")?
                    .parse()
                    .ok()
                    .ok_or("Bad Number")?,
            ))
        } else if let Some(n) = suffix_num(s, "u") {
            Ok(time::Date::from_unix(n))
        } else if let Some(n) = suffix_num(s, "jd") {
            Ok(time::Date::from_julian(n))
        } else if let Some(n) = suffix_num(s, "j") {
            Ok(time::Date::from_julian(n))
        } else if let Ok(d) = DateTime::parse_from_rfc3339(s) {
            Ok(time::Date::from_unix(d.timestamp() as f64))
        } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M:%S") {
            Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
        } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M") {
            Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
        } else if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(time::Date::from_unix(
                NaiveDateTime::from(d).and_utc().timestamp() as f64,
            ))
        } else {
            Err("Invalid Date")
        }
    }

    pub fn angle(s: &str) -> Result<time::Period, &'static str> {
        if let Some(n) = suffix_num(s, "e") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "w") {
            Ok(time::Period::from_degrees(-n))
        } else if let Some(n) = suffix_num(s, "n") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "s") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "d") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "deg") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "°") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(s, "rad") {
            Ok(time::Period::from_radians(n))
        } else {
            Err("Invalid Angle")
        }
    }

    /// Reads anything that at its core is a number,
    /// These numbers are floating point and can have prefixes or suffixes
    ///
    /// Prefixes:
    ///
    /// | Prefix | Meaning       |
    /// |--------|---------------|
    /// | `@`    | Unix Date     |
    ///
    /// Suffixes:
    ///
    /// | Suffix | Meaning       |
    /// |--------|---------------|
    /// | `U`    | Unix Date     |
    /// | `JD`   | Julian Day    |
    /// | `J`    | Julian Day    |
    /// | `D`    | Degrees       |
    /// | `d`    | Degrees       |
    /// | `deg`  | Degrees       |
    /// | `°`    | Degrees       |
    /// | `rad`  | Radians       |
    /// | `H`    | Decimal Hours |
    /// | `h`    | Decimal Hours |
    /// | `rad`  | Radians       |
    ///
    pub fn primative(s: &str) -> Option<Value> {
        if let Ok(d) = date(s) {
            Some(Value::Date(d))
        } else if let Ok(d) = angle(s) {
            Some(Value::Per(d, PerView::Angle))
        }
        // Time
        else if s.ends_with("h") {
            Some(Value::Per(
                time::Period::from_decimal((s.strip_suffix("H"))?.parse().ok()?),
                PerView::Time,
            ))
        } else {
            Some(Value::Num(s.parse().ok()?))
        }
    }

    pub fn function(s: &str, stack: &mut Vec<Value>, rf: &mut RefFrame) -> Option<()> {
        if let Value::Obj(c) = &stack[stack.len() - 1] {
            if let Ok(v) = property_of(c, s, rf) {
                stack.push(v);
                return Some(());
            }
        }
        match s {
            ".s" => stack
                .iter()
                .enumerate()
                .rev()
                .for_each(|(n, x)| println!("#{:02}: {}", n, x)),
            "." => println!("{}", stack.pop()?),
            "between" => match (stack.pop()?, stack.pop()?) {
                (Value::Crd(a, _), Value::Crd(b, _)) => {
                    stack.push(Value::Per(a.dist(b), PerView::Angle))
                }
                _ => return None,
            },
            "latlong" => match (stack.pop()?, stack.pop()?) {
                (Value::Per(long, _), Value::Per(lat, _)) => {
                    rf.lat = lat;
                    rf.long = long;
                }
                _ => return None,
            },
            "rise" => match stack.pop()? {
                Value::Crd(c, _) => stack.push(Value::Per(
                    c.riseset(rf.date, rf.lat, rf.long).unwrap().0,
                    PerView::Time,
                )),
                _ => return None,
            },
            "set" => match stack.pop()? {
                Value::Crd(c, _) => stack.push(Value::Per(
                    c.riseset(rf.date, rf.lat, rf.long).unwrap().1,
                    PerView::Time,
                )),
                _ => return None,
            },
            "now" => stack.push(Value::Date(time::Date::now())),
            "isdate" => match stack.pop()? {
                Value::Date(d) => rf.date = d,
                _ => return None,
            },
            "to_horiz" => match stack.pop()? {
                Value::Crd(c, _) => stack.push(Value::Crd(c, CrdView::Horizontal(*rf))),
                _ => return None,
            },
            "to_equatorial" => match stack.pop()? {
                Value::Crd(c, _) => stack.push(Value::Crd(c, CrdView::Equatorial)),
                _ => return None,
            },
            _ => return None,
        };

        Some(())
    }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rdunix() {
        assert_eq!(
            primative("@86400").unwrap(),
            Value::Date(time::Date::from_calendar(
                1970,
                1,
                2,
                time::Period::default()
            ))
        );
        assert_eq!(
            primative("86400u").unwrap(),
            Value::Date(time::Date::from_calendar(
                1970,
                1,
                2,
                time::Period::default()
            ))
        );
        assert_eq!(
            primative("86400jd").unwrap(),
            Value::Date(time::Date::from_julian(86400.0))
        );
        assert_eq!(primative("@86400U"), None);

        assert_eq!(
            primative("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            primative("120.5deg").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            primative("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            primative("120.5°").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            primative("120.5°").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            primative("2000-12-25").unwrap(),
            Value::Date(time::Date::from_calendar(
                2000,
                12,
                25,
                time::Period::default()
            ))
        );
    }
}
}

fn property_of(obj: &CelObj, q: &str, rf: &RefFrame) -> Result<Value, &'static str> {
    match (q, obj.clone()) {
        ("equ", CelObj::Planet(p)) => Ok(Value::Crd(p.location(rf.date), CrdView::Equatorial)),
        ("equ", CelObj::Sun) => Ok(Value::Crd(
            sol::SUN.location(rf.date),
            CrdView::Equatorial,
        )),
        ("equ", CelObj::Moon) => Ok(Value::Crd(
            moon::MOON.location(rf.date),
            CrdView::Equatorial,
        )),
        ("horiz", _) => {
            let Value::Crd(p, _) = property_of(obj, "equ", rf)? else {
                panic!();
            };
            Ok(Value::Crd(p, CrdView::Horizontal(*rf)))
        }
        ("ecliptic", _) => {
            let Value::Crd(p, _) = property_of(obj, "equ", rf)? else {
                panic!();
            };
            Ok(Value::Crd(p, CrdView::Ecliptic(rf.date)))
        }
        ("distance", CelObj::Planet(p)) => Ok(Value::Dist(p.distance(rf.date))),
        ("distance", CelObj::Sun) => Ok(Value::Dist(sol::SUN.distance(rf.date))),
        ("distance", CelObj::Moon) => Ok(Value::Dist(moon::MOON.distance(rf.date))),
        ("magnitude", CelObj::Planet(p)) => Ok(Value::Num(p.magnitude(rf.date))),
        ("magnitude", CelObj::Sun) => Ok(Value::Num(sol::SUN.magnitude(rf.date))),
        ("magnitude", CelObj::Moon) => Ok(Value::Num(moon::MOON.magnitude(rf.date))),
        ("phase", CelObj::Planet(p)) => Ok(Value::Phase(p.phaseangle(rf.date), PhaseView::Default)),
        ("phase", CelObj::Moon) => Ok(Value::Phase(
            moon::MOON.phaseangle(rf.date),
            PhaseView::Default,
        )),
        ("phaseemoji", _) => {
            let Value::Phase(p, _) = property_of(obj, "phase", rf)? else {
                panic!();
            };
            // The default emojis for people who don't specify a latitude are the northern ones
            eprintln!("{:?}", rf.lat.to_latitude().degrees());
            if rf.lat.to_latitude().degrees() >= 0.0 {
                Ok(Value::Phase(p, PhaseView::Nemoji))
            } else {
                Ok(Value::Phase(p, PhaseView::Semoji))
            }
        }
        ("phasename", _) => {
            let Value::Phase(p, _) = property_of(obj, "phase", rf)? else {
                panic!();
            };
            Ok(Value::Phase(p, PhaseView::PhaseName))
        }
        ("illumfrac", _) => {
            let Value::Phase(p, _) = property_of(obj, "phase", rf)? else {
                panic!();
            };
            Ok(Value::Phase(p, PhaseView::Illumfrac))
        }
        ("angdia", CelObj::Planet(p)) => Ok(Value::Per(p.angdia(rf.date), PerView::Angle)),
        ("angdia", CelObj::Sun) => Ok(Value::Per(sol::SUN.angdia(rf.date), PerView::Angle)),
        ("angdia", CelObj::Moon) => Ok(Value::Per(moon::MOON.angdia(rf.date), PerView::Angle)),
        ("phase", CelObj::Sun) => Err("Can't get phase of the Sun"),
        _ => Err("No Property"),
    }
}

/// A query is anything that produces a return stack dependant on reference frame and catalog.
mod query {
    use super::*;
    use std::collections::HashMap;

    fn get_catobj(s: &str, catalog: &HashMap<&str, CelObj>) -> Option<CelObj> {
        Some(catalog.get(s)?.clone())
    }

    /// An object and a CSV list of properties. The return stack is these properties.
    pub fn basic(
        words: &[String],
        rf: RefFrame,
        catalog: &HashMap<&str, CelObj>,
    ) -> Result<Vec<(Value, String)>, &'static str> {
        let obj = get_catobj(&words[0].clone(), catalog).unwrap();
        Ok(words[1]
            .split(',')
            .map(|prop| (property_of(&obj, prop, &rf).unwrap(), prop.to_owned()))
            .collect())
    }

    pub fn rpn(
        words: &[String],
        rf: RefFrame,
        c: &HashMap<&str, CelObj>,
    ) -> Result<Vec<(Value, String)>, &'static str> {
        pub fn word(
            s: &str,
            stack: &mut Vec<Value>,
            catalog: &HashMap<&str, CelObj>,
            rf: &mut RefFrame,
        ) -> Option<()> {
            if let Some(c) = get_catobj(s, catalog) {
                stack.push(Value::Obj(c));
                Some(())
            } else if let Some(()) = parse::function(s, stack, rf) {
                Some(())
            } else {
                stack.push(parse::primative(s)?);
                Some(())
            }
        }
        let mut tmprf = rf; // For ephemeris, this value is not safe to variate between queries
        let mut stack: Vec<Value> = Vec::new();
        words
            .iter()
            .for_each(|x| word(x, &mut stack, c, &mut tmprf).expect("Failed to parse RPN query"));
        Ok(stack
            .iter()
            .map(|v| (v.clone(), "Stack Object".to_owned()))
            .collect())
    }
}

fn read_catalog() -> std::collections::HashMap<&'static str, CelObj> {
    std::collections::HashMap::from([
        ("sun", CelObj::Sun),
        ("mercury", CelObj::Planet(sol::MERCURY)),
        ("venus", CelObj::Planet(sol::VENUS)),
        ("moon", CelObj::Moon),
        ("mars", CelObj::Planet(sol::MARS)),
        ("jupiter", CelObj::Planet(sol::JUPITER)),
        ("saturn", CelObj::Planet(sol::SATURN)),
        ("uranus", CelObj::Planet(sol::URANUS)),
        ("neptune", CelObj::Planet(sol::NEPTUNE)),
        ("pluto", CelObj::Planet(sol::PLUTO)),
    ])
}

mod output {
    use super::*;

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

    fn term_proph(rs: Vec<(Value, String)>, _d: time::Date) {
        print!("{:^22}", "date");
        for x in rs.iter().map(|x| x.1.clone()) {
            print!("{:^29}", x);
        }
        print!("\n{:=<1$}\n", "", 29 * rs.len() + 22);
    }
    fn term_q(rs: Vec<(Value, String)>) {
        print!(
            "{}\n",
            rs.iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
    }
    fn term_eq(rs: Vec<(Value, String)>, d: time::Date) {
        print!("{:^22}", Value::Date(d).to_string());
        for x in rs.iter().map(|x| x.0.to_string()) {
            print!("{:<29}", x);
        }
        print!("\n");
    }
    pub const TERM: Driver = Driver {
        start: nop,
        propheader: term_proph,
        query: term_q,
        ephemq: term_eq,
        footer: nop,
    };

    fn csv_proph(rs: Vec<(Value, String)>, _d: time::Date) {
        print!(
            "date,{}\n",
            rs.iter()
                .map(|x| x.1.clone())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
    fn csv_q(rs: Vec<(Value, String)>) {
        print!(
            "{}",
            rs.iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
    }

    fn csv_eq(rs: Vec<(Value, String)>, d: time::Date) {
        print!(
            "{},{}\n",
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

    fn json_init() -> () {
        print!("{{ \"q\": [");
    }
    fn json_q(rs: Vec<(Value, String)>) {
        print!("{{");
        rs.iter().for_each(|(x, y)| print!("\"{}\": \"{}\",", y, x));
        print!("\"isq\": true }},");
    }
    fn json_eq(rs: Vec<(Value, String)>, d: time::Date) {
        print!("{{ \"timestamp\": {},", d.unix());
        rs.iter().for_each(|(x, y)| print!("\"{}\": \"{}\",", y, x));
        print!("\"isq\": true }},");
    }
    fn json_footer() -> () {
        print!("{{\"isq\": false}} ] }}");
    }
    pub const JSON: Driver = Driver {
        start: json_init,
        propheader: nop_fa,
        query: json_q,
        ephemq: json_eq,
        footer: json_footer,
    };
}

fn main() {
    use clap::{arg, command, Arg};
    let matches = command!()
        .arg(arg!(-l --lat [Angle] "Set the latitude").value_parser(parse::angle))
        .arg(arg!(-L --long [Angle] "Set the longitude").value_parser(parse::angle))
        .arg(arg!(-d --date [Date] "Set the date").value_parser(parse::date))
        .arg(
            arg!(-T --format [Format] "Output Format")
                .value_parser(["term", "csv", "json"])
                .default_value("term"),
        )
        .arg(arg!(-r --rpn "Arguments are parsed as RPN words").action(clap::ArgAction::SetTrue))
        .arg(arg!(-E --ephem [StartStepEnd] "Generates Table").value_parser(parse::ephemq))
        .arg(Arg::new("com").hide(true).action(clap::ArgAction::Append))
        .get_matches();

    let catalog = read_catalog();
    let mut myrf: RefFrame = RefFrame {
        lat: *matches.get_one("lat").unwrap_or(&time::Period::default()),
        long: *matches.get_one("long").unwrap_or(&time::Period::default()),
        date: *matches.get_one("date").unwrap_or(&time::Date::now()),
    };
    let formatter = match matches.get_one::<String>("format").unwrap().as_str() {
        "term" => output::TERM,
        "csv" => output::CSV,
        "json" => output::JSON,
        _ => todo!(),
    };

    let words: Vec<_> = matches
        .get_many::<String>("com")
        .unwrap()
        .map(|x| x.to_lowercase())
        .collect();

    let q = |myrf: RefFrame| {
        if !matches.get_flag("rpn") {
            query::basic(&words, myrf, &catalog).unwrap()
        } else {
            query::rpn(&words, myrf, &catalog).unwrap()
        }
    };

    (formatter.start)();

    if let Some((start, step, end)) =
        matches.get_one::<(time::Date, Step, time::Date)>("ephem")
    {
        myrf.date = *start;
        let first = q(myrf);
        (formatter.propheader)(first.clone(), myrf.date);
        (formatter.ephemq)(first.clone(), myrf.date);
        while myrf.date.julian() < end.julian() {
            myrf.date = step_date(myrf.date, *step);
            (formatter.ephemq)(q(myrf), myrf.date);
        }
    } else {
        (formatter.query)(q(myrf));
    }

    (formatter.footer)();
}
