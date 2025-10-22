use crate::{query::Property, value};
use pracstro::{coord, time};

fn suffix_num(s: &str, j: &str) -> Option<f64> {
    s.strip_suffix(j)?.parse::<f64>().ok()
}

pub fn angle(s: &str) -> Result<time::Angle, String> {
    let sl = &s.to_lowercase(); // This can usually be guaranteed, except in argument parsing
    if let Some(n) = suffix_num(sl, "e") {
        Ok(time::Angle::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "w") {
        Ok(time::Angle::from_degrees(-n))
    } else if let Some(n) = suffix_num(sl, "n") {
        Ok(time::Angle::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "s") {
        Ok(time::Angle::from_degrees(-n))
    } else if let Some(n) = suffix_num(sl, "d") {
        Ok(time::Angle::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "deg") {
        Ok(time::Angle::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "°") {
        Ok(time::Angle::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "rad") {
        Ok(time::Angle::from_radians(n))
    } else {
        Err("Invalid Angle".to_string())
    }
}

pub fn latlong(s: &str) -> Result<value::Location, String> {
    fn long(s: &str) -> Result<time::Angle, String> {
        if let Ok(n) = s.parse::<f64>() {
            Ok(time::Angle::from_degrees(n))
        } else {
            angle(s)
        }
    }
    fn lat(s: &str) -> Result<time::Angle, String> {
        let unchecked_l = long(s)?;
        if unchecked_l.to_latitude().degrees() > 90.0 {
            Err("Latitude over 90 degrees".to_string())
        } else {
            Ok(unchecked_l)
        }
    }
    if s == "none" {
        return Ok(None);
    };
    let mut eq = s.split(',');
    let lats = eq.next().ok_or("Bad CSV".to_string())?;
    let longs = eq.next().ok_or("Bad CSV".to_string())?;
    Ok(Some((lat(lats)?, long(longs)?)))
}

pub fn object(
    sm: &str,
    cat: &std::collections::HashMap<&'static str, value::CelObj>,
) -> Result<value::CelObj, String> {
    let s = sm.to_lowercase();
    if s.starts_with("latlong:") {
        let ll = latlong(s.strip_prefix("latlong:").ok_or("Bad prefix")?)?
            .ok_or("Raw coordinate must not be none".to_string())?;
        return Ok(value::CelObj::Crd(coord::Coord::from_equatorial(
            ll.1, ll.0,
        )));
    };
    cat.get(s.as_str()).cloned().ok_or("Unknown Object".into())
}

pub fn property(
    sm: &str,
    cat: &std::collections::HashMap<&'static str, value::CelObj>,
) -> Result<Property, String> {
    let s = &sm.to_lowercase();
    if s.starts_with("angbetween:") {
        return Ok(Property::AngBet(object(
            s.strip_prefix("angbetween:")
                .ok_or("Bad prefix".to_string())?,
            cat,
        )?));
    };
    match s.as_str() {
        "equ" | "equa" | "equatorial" => Ok(Property::Equatorial),
        "horiz" | "horizontal" => Ok(Property::Horizontal),
        "ecl" | "ecliptic" => Ok(Property::Ecliptic),
        "dist" | "distance" => Ok(Property::Distance),
        "mag" | "magnitude" | "brightness" => Ok(Property::Magnitude),
        "phase" => Ok(Property::PhaseDefault),
        "phaseemoji" => Ok(Property::PhaseEmoji),
        "phasename" => Ok(Property::PhaseName),
        "angdia" => Ok(Property::AngDia),
        "phaseprecent" | "illumfrac" => Ok(Property::IllumFrac),
        "rise" => Ok(Property::Rise),
        "set" => Ok(Property::Set),
        _ => Err("Unknown Property".to_string()),
    }
}
