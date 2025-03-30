use chrono::prelude::*;
use clap::{arg, command, Arg};
use pracstro::{coord, moon, sol, time};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
enum PerView {
    Angle,
    Time,
}

#[derive(Debug, PartialEq, Clone)]
enum CoordView {
    Equatorial,
    Horizontal(RefFrame),
    //Ecliptic,
}

#[derive(Debug, PartialEq, Clone)]
enum CelObj {
    Planet(sol::Planet),
    Moon,
    Sun,
}

#[derive(Debug, PartialEq, Clone)]
struct RefFrame {
    lat: time::Period,
    long: time::Period,
    date: time::Date,
}

#[derive(Debug, PartialEq, Clone)]
enum Value {
    // Primatives
    Date(time::Date),
    Per(time::Period, PerView),
    Crd(coord::Coord, CoordView),
    Num(f64),
    Dist(f64),
    Phase(time::Period),
    // Celestial Objects
    Obj(CelObj),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Date(d) => {
                write!(
                    f,
                    "{}",
                    DateTime::from_timestamp(d.unix() as i64, 0)
                        .expect("Invalid Date")
                        .format("%Y-%m-%dT%T")
                )
            }
            Value::Per(p, PerView::Angle) => {
                let (d, m, s) = p.degminsec();
                write!(f, "{}°{}'{:.2}\"", d, m, s)
            }
            Value::Per(p, PerView::Time) => {
                let (h, m, s) = p.clock();
                write!(f, "{:02}:{:02}:{:02}", h, m, s.trunc())
            }
            Value::Dist(d) => {
                write!(f, "{} AU", d)
            }
            Value::Crd(c, CoordView::Equatorial) => {
                let (ra, de) = c.equatorial();
                let ((rh, rm, rs), (dd, dm, ds)) = (ra.clock(), de.to_latitude().degminsec());
                write!(
                    f,
                    "{:02}h{:02}m{:02.1}s {:+}°{:02}'{:.1}\"",
                    rh, rm, rs, dd, dm, ds
                )
            }
            Value::Crd(c, CoordView::Horizontal(rf)) => {
                let d = c.horizon(rf.date, rf.date.time(), rf.lat, rf.long);
                write!(
                    f,
                    "{} {}",
                    Value::Per(d.0, PerView::Angle),
                    Value::Per(d.1.to_latitude(), PerView::Angle)
                )
            }
            Value::Phase(pa) => {
                const EMOJIS: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
                //const SEMOJI: [&str; 8] = ["🌑", "🌘", "🌗", "🌖", "🌕", "🌔", "🌓", "🌒"];
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
                    match (ilumfrac, ang.degrees() < 180.0) {
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
                let ilf = (1.0 - pa.cos()) / 2.0;
                let pi = phaseidx(ilf, *pa);
                write!(f, "{} {} ({:2.2}%)", EMOJIS[pi], PNAMES[pi], ilf * 100.0)
            }
            Value::Num(n) => write!(f, "{:0.2}", n),
            Value::Obj(_p) => write!(f, "Celestial Object"),
        }
    }
}

/// The inbuilt RFC3339/ISO6901 date parser in chrono does not support subsets of the formatting.
fn parse_date(s: &str) -> Result<time::Date, &'static str> {
    if s.starts_with("@") {
        Ok(time::Date::from_unix(
            (s.strip_prefix("@"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("u") {
        Ok(time::Date::from_unix(
            (s.strip_suffix("u"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("jd") {
        Ok(time::Date::from_julian(
            (s.strip_suffix("jd"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("j") {
        Ok(time::Date::from_julian(
            (s.strip_suffix("j"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        Ok(time::Date::from_unix(d.timestamp() as f64))
    } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M:%S") {
        Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
    } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M") {
        Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
    } else if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        Ok(time::Date::from_unix(
            d.and_hms_opt(0, 0, 0).ok_or("")?.and_utc().timestamp() as f64,
        ))
    } else {
        Err("Invalid Date")
    }
}

fn parse_angle(s: &str) -> Result<time::Period, &'static str> {
    if s.ends_with("e") {
        Ok(time::Period::from_degrees(
            (s.strip_suffix("e"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("w") {
        Ok(time::Period::from_degrees(
            -(s.strip_suffix("w"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("n") {
        Ok(time::Period::from_degrees(
            (s.strip_suffix("n"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("s") {
        Ok(time::Period::from_degrees(
            -(s.strip_suffix("s"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("d") {
        Ok(time::Period::from_degrees(
            (s.strip_suffix("d"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("deg") {
        Ok(time::Period::from_degrees(
            (s.strip_suffix("deg"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("°") {
        Ok(time::Period::from_degrees(
            (s.strip_suffix("°"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if s.ends_with("rad") {
        Ok(time::Period::from_radians(
            (s.strip_suffix("rad"))
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
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
fn parse_primative(s: &str) -> Option<Value> {
    if let Ok(d) = parse_date(s) {
        Some(Value::Date(d))
    } else if let Ok(d) = parse_angle(s) {
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

fn try_parse_function(s: &str, stack: &mut Vec<Value>, rf: &mut RefFrame) -> Option<()> {
    if let Value::Obj(c) = &stack[stack.len() - 1] {
        if let Ok(v) = property_of(c.clone(), s, rf) {
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
            Value::Crd(c, _) => stack.push(Value::Crd(c, CoordView::Horizontal(rf.clone()))),
            _ => return None,
        },
        "to_equatorial" => match stack.pop()? {
            Value::Crd(c, _) => stack.push(Value::Crd(c, CoordView::Equatorial)),
            _ => return None,
        },
        _ => return None,
    };

    Some(())
}

fn get_catobj(s: &str, catalog: &HashMap<&str, CelObj>) -> Option<CelObj> {
    if catalog.contains_key(s) {
        Some(catalog.get(s)?.clone())
    } else {
    	None
    }
}

fn parse_word(
    s: &str,
    stack: &mut Vec<Value>,
    catalog: &HashMap<&str, CelObj>,
    rf: &mut RefFrame,
) -> Option<()> {
    if catalog.contains_key(s) {
        stack.push(Value::Obj(catalog.get(s)?.clone()));
        Some(())
    } else {
        match try_parse_function(s, stack, rf) {
            Some(()) => Some(()),
            _ => Some(stack.push(parse_primative(s)?)),
        }
    }
}

fn property_of(obj: CelObj, q: &str, rf: &RefFrame) -> Result<Value, &'static str> {
    match (q, obj) {
        ("eq", CelObj::Planet(p)) => Ok(Value::Crd(p.location(rf.date), CoordView::Equatorial)),
        ("eq", CelObj::Sun) => Ok(Value::Crd(
            sol::SUN.location(rf.date),
            CoordView::Equatorial,
        )),
        ("eq", CelObj::Moon) => Ok(Value::Crd(
            moon::MOON.location(rf.date),
            CoordView::Equatorial,
        )),
        ("horiz", CelObj::Planet(p)) => Ok(Value::Crd(
            p.location(rf.date),
            CoordView::Horizontal(rf.clone()),
        )),
        ("horiz", CelObj::Sun) => Ok(Value::Crd(
            sol::SUN.location(rf.date),
            CoordView::Horizontal(rf.clone()),
        )),
        ("horiz", CelObj::Moon) => Ok(Value::Crd(
            moon::MOON.location(rf.date),
            CoordView::Horizontal(rf.clone()),
        )),
        ("distance", CelObj::Planet(p)) => Ok(Value::Dist(p.distance(rf.date))),
        ("distance", CelObj::Sun) => Ok(Value::Dist(sol::SUN.distance(rf.date))),
        ("distance", CelObj::Moon) => Ok(Value::Dist(moon::MOON.distance(rf.date))),
        ("magnitude", CelObj::Planet(p)) => Ok(Value::Num(p.magnitude(rf.date))),
        ("magnitude", CelObj::Sun) => Ok(Value::Num(sol::SUN.magnitude(rf.date))),
        ("magnitude", CelObj::Moon) => Ok(Value::Num(moon::MOON.magnitude(rf.date))),
        ("phase", CelObj::Planet(p)) => Ok(Value::Phase(p.phaseangle(rf.date))),
        ("phase", CelObj::Moon) => Ok(Value::Phase(moon::MOON.phaseangle(rf.date))),
        ("angdia", CelObj::Planet(p)) => Ok(Value::Per(p.angdia(rf.date), PerView::Angle)),
        ("angdia", CelObj::Sun) => Ok(Value::Per(sol::SUN.angdia(rf.date), PerView::Angle)),
        ("angdia", CelObj::Moon) => Ok(Value::Per(moon::MOON.angdia(rf.date), PerView::Angle)),
        ("phase", CelObj::Sun) => Err("Can not get phase of sun"),
        _ => Err("No Property"),
    }
}

fn read_catalog() -> HashMap<&'static str, CelObj> {
    HashMap::from([
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

fn main() {
    let matches = command!()
        .arg(arg!(-l --lat [Angle] "Set the latitude").value_parser(parse_angle))
        .arg(arg!(-L --long [Angle] "Set the longitude").value_parser(parse_angle))
        .arg(arg!(-d --date [Date] "Set the date").value_parser(parse_date))
        .arg(arg!(-b --basic "No RPN Formatting, [object] [property]").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("com").hide(true).action(clap::ArgAction::Append))
        .get_matches();
    let mut stack: Vec<Value> = Vec::new();
    let catalog = read_catalog();
    let mut myrf: RefFrame = RefFrame {
        lat: *matches.get_one("lat").unwrap_or(&time::Period::default()),
        long: *matches.get_one("long").unwrap_or(&time::Period::default()),
        date: *matches.get_one("date").unwrap_or(&time::Date::now()),
    };

	if matches.get_flag("basic") {
		let mut iter = matches.get_many::<String>("com").unwrap();
		let obj = iter.next().unwrap();
		let prop = iter.next().unwrap();
        println!("{}", property_of(get_catobj(obj, &catalog).unwrap(), prop, &myrf).unwrap());
	} else {
    matches
        .get_many::<String>("com")
        .unwrap()
        .map(|x| x.to_lowercase())
        .enumerate()
        .for_each(|(n, x)| {
            parse_word(&x, &mut stack, &catalog, &mut myrf)
                .unwrap_or_else(|| panic!("Failed to parse word {} `{}`", n, x))
        });

    if let Some(t) = stack.pop() {
        println!("{}", t);
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rdunix() {
        assert_eq!(
            parse_primative("@86400").unwrap(),
            Value::Date(time::Date::from_calendar(
                1970,
                1,
                2,
                time::Period::default()
            ))
        );
        assert_eq!(
            parse_primative("86400u").unwrap(),
            Value::Date(time::Date::from_calendar(
                1970,
                1,
                2,
                time::Period::default()
            ))
        );
        assert_eq!(
            parse_primative("86400jd").unwrap(),
            Value::Date(time::Date::from_julian(86400.0))
        );
        assert_eq!(parse_primative("@86400U"), None);

        assert_eq!(
            parse_primative("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            parse_primative("120.5deg").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            parse_primative("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            parse_primative("120.5°").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            parse_primative("120.5°").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            parse_primative("2000-12-25").unwrap(),
            Value::Date(time::Date::from_calendar(
                2000,
                12,
                25,
                time::Period::default()
            ))
        );
    }
}
