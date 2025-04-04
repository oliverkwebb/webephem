use crate::value::*;
use pracstro::{coord, time};

#[derive(Clone, Debug, PartialEq)]
pub struct Star {
    pub loc_j2k: coord::Coord,
    pub mag: f64,
    pub pi: time::Period,
    pub pm_ra: time::Period,
    pub pm_dec: time::Period,
}

/// Creates the catalog as a hash table
///
/// This operation takes about 500 µs on my machine
pub fn read() -> std::collections::HashMap<&'static str, CelObj> {
    use pracstro::sol;

    let mut cat = std::collections::HashMap::from([
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
    ]);

    let stars = include_str!("dat/stars.csv");
    for star in stars.lines().skip(1) {
        let p: Vec<&str> = star.split(',').map(|x| x.trim()).collect();
        cat.insert(
            p[0].into(),
            CelObj::Star(Star {
                loc_j2k: coord::Coord::from_equatorial(
                    time::Period::from_degrees(p[2].parse().unwrap()),
                    time::Period::from_degrees(p[3].parse().unwrap()),
                ),
                mag: p[4].parse().unwrap(),
                pi: time::Period::from_degrees(p[5].parse::<f64>().unwrap() / 3600_000.0),
                pm_ra: time::Period::from_degrees(p[6].parse::<f64>().unwrap() / 3600_000.0),
                pm_dec: time::Period::from_degrees(p[7].parse::<f64>().unwrap() / 3600_000.0),
            }),
        );
    }

    cat
}
