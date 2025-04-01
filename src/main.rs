use pracstro::time;
use value::*;

pub mod output;
pub mod value;

mod catalog {
    use crate::value::*;

    pub fn read() -> std::collections::HashMap<&'static str, CelObj> {
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

    pub fn get(s: &str, catalog: &std::collections::HashMap<&str, CelObj>) -> Option<CelObj> {
        Some(catalog.get(s)?.clone())
    }
}

/// Pracstro provides a way to do this, but that isn't functional in a lot of contexts
mod timestep {
    use chrono::prelude::*;
    use pracstro::time;
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Step {
        S(f64),
        M(chrono::Months),
        Y(i32),
    }
    pub fn step_forward_date(d: time::Date, s: Step) -> time::Date {
        match s {
            Step::S(sec) => time::Date::from_julian(d.julian() + (sec.abs() / 86400.0)),
            Step::M(m) => time::Date::from_unix(
                (DateTime::from_timestamp(d.unix() as i64, 0).unwrap() + m).timestamp() as f64,
            ),
            Step::Y(m) => {
                let dt = DateTime::from_timestamp(d.unix() as i64, 0).expect("Bad time");
                time::Date::from_unix(
                    dt.with_year(dt.year() + m).expect("Bad time").timestamp() as f64
                )
            }
        }
    }
    pub fn step_back_date(d: time::Date, s: Step) -> time::Date {
        match s {
            Step::S(sec) => time::Date::from_julian(d.julian() - (sec.abs() / 86400.0)),
            Step::M(m) => time::Date::from_unix(
                (DateTime::from_timestamp(d.unix() as i64, 0).unwrap() - m).timestamp() as f64,
            ),
            Step::Y(m) => {
                let dt = DateTime::from_timestamp(d.unix() as i64, 0).expect("Bad time");
                time::Date::from_unix(
                    dt.with_year(dt.year() - m).expect("Bad time").timestamp() as f64
                )
            }
        }
    }
}

mod parse {
    use crate::catalog;
    use crate::timestep;
    use crate::Property;
    use chrono::prelude::*;
    use pracstro::time;

    fn suffix_num(s: &str, j: &str) -> Option<f64> {
        s.strip_suffix(j)?.parse::<f64>().ok()
    }

    pub fn object(s: &str) -> Result<crate::value::CelObj, &'static str> {
        catalog::get(&s.to_lowercase(), &catalog::read()).ok_or("No Catalog Object")
    }

    pub fn property(sm: &str) -> Result<Property, &'static str> {
        let s = &sm.to_lowercase();
        match s.as_str() {
            "equ" | "equa" | "equatorial" => Ok(Property::Equatorial),
            "horiz" | "horizontal" => Ok(Property::Equatorial),
            "ecl" | "ecliptic" => Ok(Property::Ecliptic),
            "dist" | "distance" => Ok(Property::Distance),
            "mag" | "magnitude" | "brightness" => Ok(Property::Magnitude),
            "phase" => Ok(Property::PhaseDefault),
            "phaseemoji" => Ok(Property::PhaseEmoji),
            "angdia" => Ok(Property::AngDia),
            "phaseprecent" | "illumfrac" => Ok(Property::IllumFrac),
            _ => Err("Unknown Property"),
        }
    }

    /// A step in time, returns (years, months, days, hours, minutes, seconds)
    pub fn step(sm: &str) -> Result<timestep::Step, &'static str> {
        let s = &sm.to_lowercase(); // This can usually be guaranteed, except in argument parsing
        if let Some(n) = suffix_num(s, "y") {
            Ok(timestep::Step::Y(n as i32))
        } else if let Some(n) = suffix_num(s, "mon") {
            Ok(timestep::Step::M(chrono::Months::new(n as u32)))
        } else if let Some(n) = suffix_num(s, "w") {
            Ok(timestep::Step::S(n * 7.0 * 86400.0))
        } else if let Some(n) = suffix_num(s, "d") {
            Ok(timestep::Step::S(n * 86400.0))
        } else if let Some(n) = suffix_num(s, "h") {
            Ok(timestep::Step::S(n * 3600.0))
        } else if let Some(n) = suffix_num(s, "min") {
            Ok(timestep::Step::S(n * 60.0))
        } else if let Some(n) = suffix_num(s, "s") {
            Ok(timestep::Step::S(n))
        } else {
            Err("Bad interval")
        }
    }

    pub fn ephemq(s: &str) -> Result<(time::Date, timestep::Step, time::Date), &'static str> {
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
        } else if s.starts_with("-") {
            Ok(timestep::step_back_date(
                time::Date::now(),
                step(s.strip_prefix("-").ok_or("Bad prefix")?)?,
            ))
        } else if s.starts_with("+") {
            Ok(timestep::step_forward_date(
                time::Date::now(),
                step(s.strip_prefix("+").ok_or("Bad prefix")?)?,
            ))
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
    use crate::Property;
    use pracstro::{moon, sol};

    pub fn property_of(obj: &CelObj, q: Property, rf: &RefFrame) -> Result<Value, &'static str> {
        match (q, obj.clone()) {
            (Property::Equatorial, CelObj::Planet(p)) => {
                Ok(Value::Crd(p.location(rf.date), CrdView::Equatorial))
            }
            (Property::Equatorial, CelObj::Sun) => {
                Ok(Value::Crd(sol::SUN.location(rf.date), CrdView::Equatorial))
            }
            (Property::Equatorial, CelObj::Moon) => Ok(Value::Crd(
                moon::MOON.location(rf.date),
                CrdView::Equatorial,
            )),
            (Property::Horizontal, _) => {
                let Value::Crd(p, _) = property_of(obj, Property::Equatorial, rf)? else {
                    panic!();
                };
                Ok(Value::Crd(p, CrdView::Horizontal(*rf)))
            }
            (Property::Ecliptic, _) => {
                let Value::Crd(p, _) = property_of(obj, Property::Equatorial, rf)? else {
                    panic!();
                };
                Ok(Value::Crd(p, CrdView::Ecliptic(rf.date)))
            }
            (Property::Distance, CelObj::Planet(p)) => Ok(Value::Dist(p.distance(rf.date))),
            (Property::Distance, CelObj::Sun) => Ok(Value::Dist(sol::SUN.distance(rf.date))),
            (Property::Distance, CelObj::Moon) => Ok(Value::Dist(moon::MOON.distance(rf.date))),
            (Property::Magnitude, CelObj::Planet(p)) => Ok(Value::Num(p.magnitude(rf.date))),
            (Property::Magnitude, CelObj::Sun) => Ok(Value::Num(sol::SUN.magnitude(rf.date))),
            (Property::Magnitude, CelObj::Moon) => Ok(Value::Num(moon::MOON.magnitude(rf.date))),
            (Property::PhaseDefault, CelObj::Planet(p)) => {
                Ok(Value::Phase(p.phaseangle(rf.date), PhaseView::Default))
            }
            (Property::PhaseDefault, CelObj::Sun) => Err("Can't get phase of the Sun"),
            (Property::PhaseDefault, CelObj::Moon) => Ok(Value::Phase(
                moon::MOON.phaseangle(rf.date),
                PhaseView::Default,
            )),
            (Property::PhaseEmoji, _) => {
                let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, rf)? else {
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
            (Property::PhaseName, _) => {
                let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, rf)? else {
                    panic!();
                };
                Ok(Value::Phase(p, PhaseView::PhaseName))
            }
            (Property::IllumFrac, _) => {
                let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, rf)? else {
                    panic!();
                };
                Ok(Value::Phase(p, PhaseView::Illumfrac))
            }
            (Property::AngDia, CelObj::Planet(p)) => {
                Ok(Value::Per(p.angdia(rf.date), PerView::Angle))
            }
            (Property::AngDia, CelObj::Sun) => {
                Ok(Value::Per(sol::SUN.angdia(rf.date), PerView::Angle))
            }
            (Property::AngDia, CelObj::Moon) => {
                Ok(Value::Per(moon::MOON.angdia(rf.date), PerView::Angle))
            }
        }
    }

    /// An object and a CSV list of properties. The return stack is these properties.
    pub fn basic(
        object: &CelObj,
        proplist: &[Property],
        rf: RefFrame,
    ) -> Result<Vec<Value>, &'static str> {
        Ok(proplist
            .iter()
            .map(|prop| {
                property_of(object, prop.clone(), &rf)
                    .unwrap_or_else(|e| panic!("Error on property {prop}: {e}"))
            })
            .collect())
    }
}

fn main() {
    use clap::{arg, command};
    let matches = command!()
    	.help_template("{before-help}{name} ({version}) - {about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}\n\nWritten by {author}")
        .arg(
            arg!(-l --lat [Angle] "Set the latitude")
                .value_parser(parse::angle)
                .default_value("0d"),
        )
        .arg(
            arg!(-L --long [Angle] "Set the longitude")
                .value_parser(parse::angle)
                .default_value("0d"),
        )
        .arg(
            arg!(-d --date [Date] "Set the date")
                .value_parser(parse::date)
                .default_value("now"),
        )
        .arg(
            arg!(-T --format [Format] "Output Format")
                .value_parser(["term", "csv", "json"])
                .default_value("term"),
        )
        .arg(arg!(-E --ephem ["Start,Step,End"] "Generates Table").value_parser(parse::ephemq))
        .arg(arg!([object] "Celestial Object").required(true).value_parser(parse::object))
        .arg(arg!([properties] ... "Properties").required(true).value_parser(parse::property))
        .get_matches();

    let mut myrf: RefFrame = RefFrame {
        lat: *matches.get_one("lat").unwrap(),
        long: *matches.get_one("long").unwrap(),
        date: *matches.get_one("date").unwrap(),
    };
    let querier = query::basic;
    let formatter = match matches.get_one::<String>("format").unwrap().as_str() {
        "term" => output::TERM,
        "csv" => output::CSV,
        "json" => output::JSON,
        _ => todo!(),
    };

    let obj = matches.get_one::<CelObj>("object").unwrap();
    let propl: Vec<Property> = matches
        .get_many::<Property>("properties")
        .unwrap()
        .cloned()
        .collect();

    let q = |myrf: RefFrame| {
        querier(obj, &propl, myrf).unwrap_or_else(|x| panic!("Failed to parse query: {x}"))
    };

    (formatter.start)();

    if let Some((start, step, end)) =
        matches.get_one::<(time::Date, timestep::Step, time::Date)>("ephem")
    {
        myrf.date = *start;
        (formatter.propheader)(&propl, myrf.date);
        while myrf.date.julian() < end.julian() {
            myrf.date = timestep::step_forward_date(myrf.date, *step);
            (formatter.ephemq)(q(myrf), &propl, myrf.date);
        }
    } else {
        (formatter.query)(q(myrf));
    }

    (formatter.footer)();
}
