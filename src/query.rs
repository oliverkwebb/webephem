use crate::value::*;
use pracstro::{moon, sol, time};

pub fn property_of(obj: &CelObj, q: Property, rf: &RefFrame) -> Result<Value, &'static str> {
    fn hemisphere(ll: Option<(pracstro::time::Period, pracstro::time::Period)>) -> bool {
        if let Some((lat, _)) = ll {
            lat.to_latitude().degrees() >= 0.0
        } else {
            true
        }
    }
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
            if rf.latlong.is_none() {
                return Err("Need to specify a lat/long with -l");
            };
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
        (Property::Rise, _) => {
            if rf.latlong.is_none() {
                return Err("Need to specify a lat/long with -l");
            };
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, rf)? else {
                panic!();
            };
            match p.riseset(rf.date, rf.latlong.unwrap().0, rf.latlong.unwrap().1) {
                Some((x, _)) => Ok(Value::RsTime(Some(time::Date::from_time(rf.date, x)))),
                None => Ok(Value::RsTime(None)),
            }
        }
        (Property::Set, _) => {
            if rf.latlong.is_none() {
                return Err("Need to specify a lat/long with -l");
            };
            let Value::Crd(p, _) = property_of(obj, Property::Equatorial, rf)? else {
                panic!();
            };
            match p.riseset(rf.date, rf.latlong.unwrap().0, rf.latlong.unwrap().1) {
                Some((_, y)) => Ok(Value::RsTime(Some(time::Date::from_time(rf.date, y)))),
                None => Ok(Value::RsTime(None)),
            }
        }
        (Property::Distance, CelObj::Planet(p)) => Ok(Value::Dist(p.distance(rf.date))),
        (Property::Distance, CelObj::Sun) => Ok(Value::Dist(sol::SUN.distance(rf.date))),
        (Property::Distance, CelObj::Moon) => Ok(Value::Dist(moon::MOON.distance(rf.date))),
        (Property::Magnitude, CelObj::Planet(p)) => Ok(Value::Num(p.magnitude(rf.date))),
        (Property::Magnitude, CelObj::Sun) => Ok(Value::Num(sol::SUN.magnitude(rf.date))),
        (Property::Magnitude, CelObj::Moon) => Ok(Value::Num(moon::MOON.magnitude(rf.date))),
        (Property::PhaseDefault, CelObj::Planet(p)) => Ok(Value::Phase(
            p.phaseangle(rf.date),
            PhaseView::Default(hemisphere(rf.latlong)),
        )),
        (Property::PhaseDefault, CelObj::Sun) => Err("Can't get phase of the Sun"),
        (Property::PhaseDefault, CelObj::Moon) => Ok(Value::Phase(
            moon::MOON.phaseangle(rf.date),
            PhaseView::Default(hemisphere(rf.latlong)),
        )),
        (Property::PhaseEmoji, _) => {
            let Value::Phase(p, _) = property_of(obj, Property::PhaseDefault, rf)? else {
                panic!();
            };
            // The default emojis for people who don't specify a latitude are the northern ones
            if hemisphere(rf.latlong) {
                Ok(Value::Phase(p, PhaseView::Emoji(true)))
            } else {
                Ok(Value::Phase(p, PhaseView::Emoji(false)))
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
        (Property::AngDia, CelObj::Planet(p)) => Ok(Value::Per(p.angdia(rf.date), PerView::Angle)),
        (Property::AngDia, CelObj::Sun) => Ok(Value::Per(sol::SUN.angdia(rf.date), PerView::Angle)),
        (Property::AngDia, CelObj::Moon) => {
            Ok(Value::Per(moon::MOON.angdia(rf.date), PerView::Angle))
        }
    }
}

/// An object and a CSV list of properties. The return stack is these properties.
pub fn run(
    object: &CelObj,
    proplist: &[Property],
    rf: &RefFrame,
) -> Result<Vec<Value>, &'static str> {
    Ok(proplist
        .iter()
        .map(|prop| {
            property_of(object, prop.clone(), rf)
                .unwrap_or_else(|e| panic!("Error on property {prop}: {e}"))
        })
        .collect())
}
