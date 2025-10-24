//! WASM frontend for websites and other tools

use crate::{query::Property, *};
use pracstro::time::Angle;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use wasm_bindgen::prelude::*;
extern crate web_sys;

/// This is needed for speed, to not format the catalog every query.
/// My testing showed the query speed reduced by a factor of around 20 with this.
static mut CATALOG: MaybeUninit<HashMap<&'static str, crate::value::CelObj>> =
    MaybeUninit::uninit();

/// Initializes the catalog in a private hashmap value
#[wasm_bindgen]
pub unsafe fn catalog_init() {
    CATALOG.write(crate::catalog::read());
}

pub fn parse_object(
    sm: &str,
    cat: &std::collections::HashMap<&'static str, value::CelObj>,
) -> Result<value::CelObj, String> {
    cat.get(sm.to_lowercase().as_str())
        .cloned()
        .ok_or("Unknown Object".into())
}

#[wasm_bindgen]
pub unsafe fn webephem_query(
    object: &str,
    property: Property,
    time: f64,
    lat: Option<f64>,
    long: Option<f64>,
    formatted: bool,
) -> Result<String, String> {
    let latlong = if let (Some(y), Some(x)) = (lat, long) {
        Some((Angle::from_degrees(x), Angle::from_degrees(y)))
    } else {
        None
    };
    let object = parse_object(object, CATALOG.assume_init_ref())?;
    let date = pracstro::time::Date::from_unix(time);

    let value = query::property_of(&object, property.clone(), latlong, date)?;

    if formatted {
        return Ok(format!("{}", value));
    } else {
        return Ok(format!("{:#}", value));
    }
}

/// Table Generation
#[wasm_bindgen]
pub unsafe fn webephem_batch_query(
    object: &str,
    property: Property,
    start_time: f64,
    step_time: f64,
    end_time: f64,
    lat: Option<f64>,
    long: Option<f64>,
    formatted: bool,
) -> Result<Vec<String>, String> {
    let latlong = if let (Some(x), Some(y)) = (lat, long) {
        Some((Angle::from_degrees(x), Angle::from_degrees(y)))
    } else {
        None
    };
    let object = parse_object(object, CATALOG.assume_init_ref())?;
    let mut ephem = Vec::<String>::new();

    for d in std::iter::successors(Some(start_time), |x| {
        if *x < end_time {
            Some(x + step_time)
        } else {
            None
        }
    }) {
        let date = pracstro::time::Date::from_unix(d);

        let value = query::property_of(&object, property, latlong, date)?;

        if formatted {
            ephem.push(format!("{}", value));
        } else {
            ephem.push(format!("{:#}", value));
        }
    }

    Ok(ephem)
}
