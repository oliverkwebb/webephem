use pracstro::time;
use value::*;

pub mod output;
pub mod value;

type Step = (f64, f64, f64);

mod parse {
    use crate::Step;
    use chrono::prelude::*;
    use pracstro::time;

    fn suffix_num(s: &str, j: &str) -> Option<f64> {
        s.strip_suffix(j)?.parse::<f64>().ok()
    }

    /// A step in time, returns (years, months, days, hours, minutes, seconds)
    pub fn step(sm: &str) -> Result<Step, &'static str> {
    	let s = &sm.to_lowercase(); // This can usually be guaranteed, except in argument parsing
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
    pub fn date(sm: &str) -> Result<time::Date, &'static str> {
    	let s = &sm.to_lowercase(); // This can usually be guaranteed, except in argument parsing
        if s == "now" {
            Ok(time::Date::now())
        } else if s.starts_with("@") {
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
    	let sl = &s.to_lowercase(); // This can usually be guaranteed, except in argument parsing
        if let Some(n) = suffix_num(sl, "e") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "w") {
            Ok(time::Period::from_degrees(-n))
        } else if let Some(n) = suffix_num(sl, "n") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "s") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "d") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "deg") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "°") {
            Ok(time::Period::from_degrees(n))
        } else if let Some(n) = suffix_num(sl, "rad") {
            Ok(time::Period::from_radians(n))
        } else {
            Err("Invalid Angle")
        }
    }
}

/// A query is anything that produces a return stack dependent on reference frame and catalog.
mod query {
    use crate::value::*;
    use pracstro::{moon, sol};
    use std::collections::HashMap;

    fn get_catobj(s: &str, catalog: &HashMap<&str, CelObj>) -> Option<CelObj> {
        Some(catalog.get(s)?.clone())
    }

    pub fn property_of(obj: &CelObj, q: &str, rf: &RefFrame) -> Result<Value, &'static str> {
        match (q, obj.clone()) {
            ("equ", CelObj::Planet(p)) => Ok(Value::Crd(p.location(rf.date), CrdView::Equatorial)),
            ("equ", CelObj::Sun) => Ok(Value::Crd(sol::SUN.location(rf.date), CrdView::Equatorial)),
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
            ("phase", CelObj::Planet(p)) => {
                Ok(Value::Phase(p.phaseangle(rf.date), PhaseView::Default))
            }
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

    mod rpn_parse {
        use crate::parse;
        use super::get_catobj;
    	use std::collections::HashMap;
        use crate::query::property_of;
        use crate::value::*;
        use pracstro::time;

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
        fn primative(s: &str) -> Option<Value> {
            if let Ok(d) = parse::date(s) {
                Some(Value::Date(d))
            } else if let Ok(d) = parse::angle(s) {
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

        fn function(s: &str, stack: &mut Vec<Value>, rf: &mut RefFrame) -> Option<()> {
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

        pub fn word(
            s: &str,
            stack: &mut Vec<Value>,
            catalog: &HashMap<&str, CelObj>,
            rf: &mut RefFrame,
        ) -> Option<()> {
            if let Some(c) = get_catobj(s, catalog) {
                stack.push(Value::Obj(c));
                Some(())
            } else if let Some(()) = function(s, stack, rf) {
                Some(())
            } else {
                stack.push(primative(s)?);
                Some(())
            }
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

    /// An object and a CSV list of properties. The return stack is these properties.
    pub fn basic(
        words: &[String],
        rf: RefFrame,
        catalog: &HashMap<&str, CelObj>,
    ) -> Result<Vec<(Value, String)>, &'static str> {
        let obj = get_catobj(&words[0].clone(), catalog)
            .unwrap_or_else(|| panic!("Object {} not in Catalog", &words[0]));
        Ok(words[1]
            .split(',')
            .map(|prop| {
                (
                    property_of(&obj, prop, &rf)
                        .unwrap_or_else(|e| panic!("Error on property {}: {e}", prop)),
                    prop.to_owned(),
                )
            })
            .collect())
    }

    pub fn rpn(
        words: &[String],
        rf: RefFrame,
        c: &HashMap<&str, CelObj>,
    ) -> Result<Vec<(Value, String)>, &'static str> {
        let mut tmprf = rf; // For ephemeris, this value is not safe to variate between queries
        let mut stack: Vec<Value> = Vec::new();
        words
            .iter()
            .for_each(|x| rpn_parse::word(x, &mut stack, c, &mut tmprf).expect("Failed to parse RPN query"));
        Ok(stack
            .iter()
            .map(|v| (v.clone(), "Stack Object".to_owned()))
            .collect())
    }
}

fn main() {
    fn read_catalog() -> std::collections::HashMap<&'static str, CelObj> {
        use pracstro::sol;
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
    fn step_date(d: time::Date, s: Step) -> time::Date {
        let (y, mon, d, t) = d.calendar();
        time::Date::from_calendar(y + s.0 as i64, mon + s.1 as u8, d + s.2 as u8, t)
    }

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
    let querier = if !matches.get_flag("rpn") {
    	query::basic
    } else {
    	query::rpn
    };
    let formatter = match matches.get_one::<String>("format").unwrap().as_str() {
        "term" => output::TERM,
        "csv" => output::CSV,
        "json" => output::JSON,
        _ => todo!(),
    };

    let words: Vec<_> = matches
        .get_many::<String>("com")
        .unwrap_or_else(|| panic!("Needs additional arguments"))
        .map(|x| x.to_lowercase())
        .collect();

    let q = |myrf: RefFrame| {
         querier(&words, myrf, &catalog) .unwrap_or_else(|x| panic!("Failed to parse query: {x}"))
    };

    (formatter.start)();

    if let Some((start, step, end)) = matches.get_one::<(time::Date, Step, time::Date)>("ephem") {
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
