//! WASM frontend for websites and other tools

use std::{mem::MaybeUninit, str::MatchIndices};

use crate::*;
use pracstro::time::Angle;
use wasm_bindgen::prelude::*;
extern crate web_sys;

/// This is needed for speed, to not format the catalog every query.
/// My testing showed the query speed reduced by a factor of around 20 with this.
static mut CATALOG: MaybeUninit<std::collections::HashMap<&'static str, crate::value::CelObj>> =
    MaybeUninit::uninit();

/// Initializes the catalog in a private hashmap value
#[wasm_bindgen]
pub fn catalog_init() {
    unsafe {
        CATALOG.write(crate::catalog::read());
    }
}

#[wasm_bindgen]
pub unsafe fn wasm_query(
    object: &str,
    property: &str,
    time: f64,
    lat: Option<f64>,
    long: Option<f64>,
    rawdata: bool,
) -> Result<String, String> {
    let latlong = if let (Some(x), Some(y)) = (lat, long) {
        Some((Angle::from_degrees(x), Angle::from_degrees(y)))
    } else {
        None
    };
    let property = parse::property(property, CATALOG.assume_init_ref())?;
    let object = parse::object(object, CATALOG.assume_init_ref())?;
    let date = pracstro::time::Date::from_unix(time);

    let value = query::property_of(
        &object,
        property.clone(),
        &value::RefFrame { latlong, date },
    )?;

    if rawdata {
        return Ok(format!("{:#}", value));
    } else {
        return Ok(format!("{}", value));
    }
}
