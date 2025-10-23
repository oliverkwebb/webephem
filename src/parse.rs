use crate::{query::Property, value};
use pracstro::time;
use wasm_bindgen::prelude::*;

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

pub fn object(
    sm: &str,
    cat: &std::collections::HashMap<&'static str, value::CelObj>,
) -> Result<value::CelObj, String> {
    cat.get(sm.to_lowercase().as_str())
        .cloned()
        .ok_or("Unknown Object".into())
}

#[wasm_bindgen]
pub fn property(sm: &str) -> Result<Property, String> {
    let s = &sm.to_lowercase();
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
