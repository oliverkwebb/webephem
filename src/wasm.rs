//! WASM frontend for websites and other tools

use std::collections::HashMap;

use crate::*;
use pracstro::time::Angle;
use wasm_bindgen::prelude::*;
extern crate web_sys;

#[wasm_bindgen]
pub unsafe fn gen_catalog_ptr() -> *const () {
    let cat = catalog::read();
    let ptr = &cat as *const HashMap<&'static str, value::CelObj> as *const ();
    std::mem::forget(cat);
    ptr
}

#[wasm_bindgen]
pub unsafe fn wasm_query(
    object: &str,
    property: &str,
    time: f64,
    lat: Option<f64>,
    long: Option<f64>,
    rawdata: bool,
    catalog: usize,
) -> Result<String, String> {
    let catalog = catalog as *const HashMap<&'static str, value::CelObj>;
    let latlong = if let (Some(x), Some(y)) = (lat, long) {
        Some((Angle::from_degrees(x), Angle::from_degrees(y)))
    } else {
        None
    };
    let property = parse::property(property, &*catalog)?;
    let object = parse::object(object, &*catalog)?;
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
