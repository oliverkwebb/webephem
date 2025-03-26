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
    Horizontal,
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
    // Celestial Objects
    Obj(CelObj),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Date(d) => {
                let cal = d.calendar();
                write!(f, "{:04}-{:02}-{:02}", cal.0, cal.1, cal.2.trunc())
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
            Value::Crd(c, CoordView::Horizontal) => {
                let (ra, de) = c.equatorial();
                let ((rh, rm, rs), (dd, dm, ds)) = (ra.clock(), de.degminsec());
                write!(
                    f,
                    "{:02}h{:02}m{:02.1}s {:+}°{:02}'{:.1}\"",
                    rh, rm, rs, dd, dm, ds
                )
            }
            Value::Num(n) => write!(f, "{:0.2}", n),
            Value::Obj(p) => write!(f, "{:?}", p),
        }
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
fn read_numerical(s: &str) -> Option<Value> {
    // Date
    if s.starts_with("@") {
        return Some(Value::Date(time::Date::from_unix(
            (s.strip_prefix("@"))?.parse().ok()?,
        )));
    } else if s.ends_with("u") {
        return Some(Value::Date(time::Date::from_unix(
            (s.strip_suffix("u"))?.parse().ok()?,
        )));
    } else if s.ends_with("jd") {
        return Some(Value::Date(time::Date::from_julian(
            (s.strip_suffix("jd"))?.parse().ok()?,
        )));
    } else if s.ends_with("j") {
        return Some(Value::Date(time::Date::from_julian(
            (s.strip_suffix("j"))?.parse().ok()?,
        )));
    }
    // Angle
    else if s.ends_with("d") {
        return Some(Value::Per(
            time::Period::from_degrees((s.strip_suffix("d"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("deg") {
        return Some(Value::Per(
            time::Period::from_degrees((s.strip_suffix("deg"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("°") {
        return Some(Value::Per(
            time::Period::from_degrees((s.strip_suffix("°"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("rad") {
        return Some(Value::Per(
            time::Period::from_radians((s.strip_suffix("rad"))?.parse().ok()?),
            PerView::Angle,
        ));
    }
    // Time
    else if s.ends_with("h") {
        return Some(Value::Per(
            time::Period::from_decimal((s.strip_suffix("H"))?.parse().ok()?),
            PerView::Time,
        ));
    } else {
        return Some(Value::Num(s.parse().ok()?));
    }
}

fn try_parse_function(s: &str, stack: &mut Vec<Value>, rf: &mut RefFrame) -> Option<()> {
    match s {
        ".s" => stack
            .iter()
            .enumerate()
            .rev()
            .for_each(|(n, x)| println!("#{:02}: {}", n, x)),
        "." => println!("{}", stack.pop()?),
        "location" => match stack.pop()? {
            Value::Obj(CelObj::Planet(p)) => {
                stack.push(Value::Crd(p.location(rf.date), CoordView::Equatorial))
            }
            Value::Obj(CelObj::Moon) => stack.push(Value::Crd(
                moon::MOON.location(rf.date),
                CoordView::Equatorial,
            )),
            Value::Obj(CelObj::Sun) => stack.push(Value::Crd(
                sol::sun::location(rf.date),
                CoordView::Equatorial,
            )),
            _ => return None,
        },
        "between" => match (stack.pop()?, stack.pop()?) {
            (Value::Crd(a, _), Value::Crd(b, _)) => {
                stack.push(Value::Per(a.dist(b), PerView::Angle))
            }
            _ => return None,
        },
        "distance" => match stack.pop()? {
            Value::Obj(CelObj::Planet(p)) => stack.push(Value::Num(p.distance(rf.date))),
            Value::Obj(CelObj::Moon) => stack.push(Value::Num(moon::MOON.distance(rf.date))),
            _ => return None,
        },
        "phase" => match stack.pop()? {
            Value::Obj(CelObj::Planet(p)) => stack.push(Value::Num(p.illumfrac(rf.date))),
            Value::Obj(CelObj::Moon) => stack.push(Value::Num(moon::MOON.phase(rf.date).1)),
            _ => return None,
        },
        "rise" => match stack.pop()? {
            Value::Crd(c, _) => stack.push(Value::Per(
                c.riseset(rf.date, rf.lat, rf.long).unwrap().0,
                PerView::Time,
            )),
            _ => return None,
        },
        "now" => stack.push(Value::Date(time::Date::now())),
        "isdate" => match stack.pop()? {
            Value::Date(d) => rf.date = d,
            _ => return None,
        },
        "horiz" => match stack.pop()? {
            Value::Crd(c, _) => stack.push(Value::Crd(c, CoordView::Horizontal)),
            _ => return None,
        },
        _ => return None,
    };

    Some(())
}

fn parse_word(
    s: &str,
    stack: &mut Vec<Value>,
    catalog: &HashMap<&str, CelObj>,
    rf: &mut RefFrame,
) -> Option<()> {
    if catalog.contains_key(s) {
        Some(stack.push(Value::Obj(catalog.get(s)?.clone())))
    } else {
        match try_parse_function(s, stack, rf) {
            Some(()) => Some(()),
            _ => Some(stack.push(read_numerical(s)?)),
        }
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
    let mut stack: Vec<Value> = Vec::new();
    let catalog = read_catalog();
    let mut myrf: RefFrame = RefFrame {
        lat: time::Period::from_degrees(0.0),
        long: time::Period::from_degrees(0.0),
        date: time::Date::now(),
    };

    std::env::args()
        .skip(1)
        .map(|x| x.to_lowercase())
        .enumerate()
        .for_each(|(n, x)| {
            parse_word(&x, &mut stack, &catalog, &mut myrf)
                .unwrap_or_else(|| panic!("Failed to parse word {} `{}`", n, x))
        });

    if let Some(t) = stack.pop() {
        println!("{}", t);
    } else {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rdunix() {
        assert_eq!(
            read_numerical("@86400").unwrap(),
            Value::Date(time::Date::from_calendar(1970, 1, 2.0))
        );
        assert_eq!(
            read_numerical("86400u").unwrap(),
            Value::Date(time::Date::from_calendar(1970, 1, 2.0))
        );
        assert_eq!(
            read_numerical("86400jd").unwrap(),
            Value::Date(time::Date::from_julian(86400.0))
        );
        assert_eq!(read_numerical("@86400U"), None);

        assert_eq!(
            read_numerical("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5deg").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5d").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5°").unwrap(),
            Value::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
    }
}
