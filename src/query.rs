use crate::value::*;
use pracstro::{moon, sol, time};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Property {
    Equatorial,
    Horizontal,
    Ecliptic,
    Distance,
    Magnitude,
    PhaseDefault,
    PhaseName,
    PhaseEmoji,
    AngDia,
    IllumFrac,
    Rise,
    Set,
}

pub fn property_of(
    obj: &CelObj,
    q: Property,
    latlong: Location,
    date: time::Date,
) -> Result<Value, String> {
    fn hemisphere(ll: Option<(pracstro::time::Angle, pracstro::time::Angle)>) -> bool {
        if let Some((lat, _)) = ll {
            lat.to_latitude().degrees() <= 0.0
        } else {
            false
        }
    }
    match (q, obj.clone()) {
        (Property::Equatorial, CelObj::Planet(p)) => {
            Ok(Value::Crd(p.location(date), CrdView::Equatorial))
        }
        (Property::Equatorial, CelObj::Sun) => Ok(Value::Crd(
            sol::SUN
                .location(date)
                .precess(time::Date::from_julian(2451545.0), date),
            CrdView::Equatorial,
        )),
        (Property::Equatorial, CelObj::Moon) => Ok(Value::Crd(
            moon::MOON
                .location(date)
                .precess(time::Date::from_julian(2451545.0), date),
            CrdView::Equatorial,
        )),
        (Property::Equatorial, CelObj::Star(s)) => Ok(Value::Crd(
            s.loc_j2k.precess(time::Date::from_julian(2451545.0), date),
            CrdView::Equatorial,
        )),
        (Property::Equatorial, CelObj::Crd(s)) => Ok(Value::Crd(s, CrdView::Equatorial)),
        (Property::Horizontal, _) => {
            if latlong.is_none() {
                return Err("Need to specify a location".to_string());
            };
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, latlong, date)? else {
                unreachable!();
            };
            Ok(Value::Crd(
                p,
                CrdView::Horizontal(RefFrame { latlong, date }),
            ))
        }
        (Property::Ecliptic, _) => {
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, latlong, date)? else {
                unreachable!();
            };
            Ok(Value::Crd(p, CrdView::Ecliptic(date)))
        }
        (Property::Rise, _) => {
            if latlong.is_none() {
                return Err("Need to specify a location".to_string());
            };
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, latlong, date)? else {
                unreachable!();
            };
            match p.riseset(date, latlong.unwrap().0, latlong.unwrap().1) {
                Some((x, _)) => Ok(Value::RsTime(Some(time::Date::from_time(date, x)))),
                None => Ok(Value::RsTime(None)),
            }
        }
        (Property::Set, _) => {
            if latlong.is_none() {
                return Err("Need to specify a location".to_string());
            };
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, latlong, date)? else {
                unreachable!();
            };
            match p.riseset(date, latlong.unwrap().0, latlong.unwrap().1) {
                Some((_, y)) => Ok(Value::RsTime(Some(time::Date::from_time(date, y)))),
                None => Ok(Value::RsTime(None)),
            }
        }
        (Property::Distance, CelObj::Planet(p)) => Ok(Value::Dist(p.distance(date))),
        (Property::Distance, CelObj::Sun) => Ok(Value::Dist(sol::SUN.distance(date))),
        (Property::Distance, CelObj::Moon) => Ok(Value::Dist(moon::MOON.distance(date))),
        (Property::Distance, CelObj::Star(s)) => {
            Ok(Value::Dist((1.0 / (s.pi.degrees() * 3600.0)) * 206_265.0))
        }
        (Property::Magnitude, CelObj::Planet(p)) => Ok(Value::Num(p.magnitude(date))),
        (Property::Magnitude, CelObj::Star(s)) => Ok(Value::Num(s.mag)),
        (Property::Magnitude, CelObj::Sun) => Ok(Value::Num(sol::SUN.magnitude(date))),
        (Property::Magnitude, CelObj::Moon) => Ok(Value::Num(moon::MOON.magnitude(date))),
        (Property::PhaseDefault, CelObj::Planet(p)) => Ok(Value::Phase(
            p.phaseangle(date),
            PhaseView::Default(hemisphere(latlong)),
        )),
        (Property::PhaseDefault, CelObj::Moon) => Ok(Value::Phase(
            moon::MOON.phaseangle(date),
            PhaseView::Default(hemisphere(latlong)),
        )),
        (Property::PhaseEmoji, _) => {
            let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, latlong, date)?
            else {
                unreachable!();
            };
            // The default emojis for people who don't specify a latitude are the northern ones
            if hemisphere(latlong) {
                Ok(Value::Phase(p, PhaseView::Emoji(true)))
            } else {
                Ok(Value::Phase(p, PhaseView::Emoji(false)))
            }
        }
        (Property::PhaseName, _) => {
            let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, latlong, date)?
            else {
                unreachable!();
            };
            Ok(Value::Phase(p, PhaseView::PhaseName))
        }
        (Property::IllumFrac, _) => {
            let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, latlong, date)?
            else {
                unreachable!();
            };
            Ok(Value::Phase(p, PhaseView::Illumfrac))
        }
        (Property::AngDia, CelObj::Planet(p)) => Ok(Value::Ang(p.angdia(date), AngView::Angle)),
        (Property::AngDia, CelObj::Sun) => Ok(Value::Ang(sol::SUN.angdia(date), AngView::Angle)),
        (Property::AngDia, CelObj::Moon) => Ok(Value::Ang(moon::MOON.angdia(date), AngView::Angle)),
        (Property::PhaseDefault, _) => Err("Can't get phase of a star".to_string()),
        (_, CelObj::Crd(_)) => Err("Can't get that property for a raw coordinate".to_string()),
        (Property::AngDia, CelObj::Star(_)) => {
            Err("Angular diameter of star not known".to_string())
        }
    }
}
