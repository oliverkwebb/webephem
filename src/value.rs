use pracstro::{coord, sol, time};
use std::fmt;
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RefFrame {
    pub latlong: Option<(time::Period, time::Period)>,
    pub date: time::Date,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PhaseView {
    Default(bool),
    Nemoji,
    Semoji,
    Illumfrac,
    PhaseName,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CrdView {
    Equatorial,
    Horizontal(RefFrame),
    Ecliptic(time::Date),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CelObj {
    Planet(sol::Planet),
    Moon,
    Sun,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PerView {
    Angle,
    Latitude,
    Time,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    // Primatives
    Date(time::Date),
    Per(time::Period, PerView),
    Crd(coord::Coord, CrdView),
    Num(f64),
    Dist(f64),
    Phase(time::Period, PhaseView),
    // Celestial Objects
    Obj(CelObj),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const EMOJIS: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
        const SEMOJI: [&str; 8] = ["🌑", "🌘", "🌗", "🌖", "🌕", "🌔", "🌓", "🌒"];
        const PNAMES: [&str; 8] = [
            "New",
            "Waxing Crescent",
            "First Quarter",
            "Waxing Gibbous",
            "Full",
            "Waning Gibbous",
            "Last Quarter",
            "Waning Crescent",
        ];

        fn phaseidx(ilumfrac: f64, ang: time::Period) -> usize {
            match (ilumfrac, ang.degrees() > 90.0) {
                (0.00..0.04, _) => 0,
                (0.96..1.00, _) => 4,
                (0.46..0.54, true) => 6,
                (0.46..0.54, false) => 2,
                (0.54..0.96, true) => 5,
                (0.54..0.96, false) => 3,
                (_, true) => 7,
                (_, false) => 1,
            }
        }

        if !f.alternate() {
            use chrono::prelude::*;
            match self {
                Value::Date(d) => write!(
                    f,
                    "{}",
                    DateTime::<Local>::from(
                        DateTime::from_timestamp(d.unix() as i64, 0)
                            .expect("Failed to Format Date")
                    )
                    .format("%Y-%m-%dT%T")
                ),
                Value::Per(p, PerView::Angle) => {
                    let (d, m, s) = p.degminsec();
                    write!(f, "{:02}°{:02}′{:02.1}″", d, m, s)
                }
                Value::Per(p, PerView::Latitude) => {
                    let (d, m, s) = p.to_latitude().degminsec();
                    write!(f, "{:+02}°{:02}′{:02.1}″", d, m, s)
                }
                //Value::Per(p, PerView::Raw) => write!(f, "{:.5}", p.degrees()),
                Value::Per(p, PerView::Time) => {
                    let (h, m, s) = p.clock();
                    write!(f, "{:02}h{:02}m{:02}s", h, m, s.trunc())
                }
                Value::Dist(d) => match d {
                    0.0..0.003342293561 => write!(f, "{:.1} km", d * 149597870.7),
                    20000.0.. => write!(f, "{:.2} ly", d / 63241.07708),
                    _ => write!(f, "{:.2} AU", d),
                },
                Value::Crd(c, CrdView::Equatorial) => {
                    let d = c.equatorial();
                    write!(
                        f,
                        "{} {}",
                        Value::Per(d.0, PerView::Time),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Crd(c, CrdView::Horizontal(rf)) => {
                	let (lat, long) = rf.latlong.unwrap();
                    let d = c.horizon(rf.date, rf.date.time(), lat, long);
                    write!(
                        f,
                        "{} {}",
                        Value::Per(d.0, PerView::Angle),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Crd(c, CrdView::Ecliptic(d)) => {
                    let d = c.ecliptic(*d);
                    write!(
                        f,
                        "{} {}",
                        Value::Per(d.0, PerView::Angle),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Phase(pa, PhaseView::Default(n)) => {
                    let ilf = (1.0 - pa.cos()) / 2.0;
                    let pi = phaseidx(ilf, *pa);
                    write!(f, "{} {} ({:2.1}%)", if *n { EMOJIS[pi] } else { SEMOJI[pi] }, PNAMES[pi], ilf * 100.0)
                }
                Value::Phase(pa, PhaseView::Nemoji) => {
                    write!(f, "{}", EMOJIS[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Phase(pa, PhaseView::Semoji) => {
                    write!(f, "{}", SEMOJI[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Phase(pa, PhaseView::Illumfrac) => {
                    write!(f, "{:2.1}", 100.0 * (1.0 - pa.cos()) / 2.0)
                }
                Value::Phase(pa, PhaseView::PhaseName) => {
                    write!(f, "{}", PNAMES[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Num(n) => write!(f, "{:0.2}", n),
                Value::Obj(_p) => write!(f, "Celestial Object"),
            }
        } else {
            match self {
                Value::Date(d) => write!(f, "{}", d.unix()),
                Value::Per(p, PerView::Angle) => {
                    write!(f, "{:.5}", p.degrees())
                }
                Value::Per(p, PerView::Latitude) => {
                    write!(f, "{:.5}", p.to_latitude().degrees())
                }
                Value::Per(p, PerView::Time) => {
                    let (h, m, s) = p.clock();
                    write!(f, "\"{:02}h{:02}m{:02}s\"", h, m, s.trunc())
                }
                Value::Dist(d) => write!(f, "{}", d),
                Value::Crd(c, CrdView::Equatorial) => {
                    let d = c.equatorial();
                    write!(
                        f,
                        "[{:#}, {:#}]",
                        Value::Per(d.0, PerView::Time),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Crd(c, CrdView::Horizontal(rf)) => {
                	let (lat, long) = rf.latlong.unwrap();
                    let d = c.horizon(rf.date, rf.date.time(), lat, long);
                    write!(
                        f,
                        "[{:#}, {:#}]",
                        Value::Per(d.0, PerView::Angle),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Crd(c, CrdView::Ecliptic(d)) => {
                    let d = c.ecliptic(*d);
                    write!(
                        f,
                        "[{:#}, {:#}]",
                        Value::Per(d.0, PerView::Angle),
                        Value::Per(d.1, PerView::Latitude)
                    )
                }
                Value::Phase(pa, PhaseView::Default(h)) => {
                    let ilf = (1.0 - pa.cos()) / 2.0;
                    let pi = phaseidx(ilf, *pa);
                    write!(
                        f,
                        "\"{} {} ({:2.1}%)\"",
                        if *h { EMOJIS[pi] } else { SEMOJI[pi] },
                        PNAMES[pi],
                        ilf * 100.0
                    )
                }
                Value::Phase(pa, PhaseView::Nemoji) => {
                    write!(f, "\"{}\"", EMOJIS[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Phase(pa, PhaseView::Semoji) => {
                    write!(f, "\"{}\"", SEMOJI[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Phase(pa, PhaseView::Illumfrac) => {
                    write!(f, "{:2.1}", 100.0 * (1.0 - pa.cos()) / 2.0)
                }
                Value::Phase(pa, PhaseView::PhaseName) => {
                    write!(f, "\"{}\"", PNAMES[phaseidx((1.0 - pa.cos()) / 2.0, *pa)])
                }
                Value::Num(n) => write!(f, "{:0.2}", n),
                Value::Obj(_p) => write!(f, "\"Celestial Object\""),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
}
impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Property::Equatorial => "Coordinates (RA/De)",
                Property::Horizontal => "Coordinates (Azi/Alt)",
                Property::Ecliptic => "Coordinates (Ecliptic)",
                Property::Distance => "Distance",
                Property::Magnitude => "Magnitude",
                Property::PhaseDefault => "Phase",
                Property::PhaseEmoji => "Phase Emoji",
                Property::PhaseName => "Phase Name",
                Property::IllumFrac => "Illuminated Frac.",
                Property::AngDia => "Angular Diameter",
            }
        )
    }
}
