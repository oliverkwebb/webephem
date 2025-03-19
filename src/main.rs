use pracstro::{coord, time};

#[derive(Debug, PartialEq)]
enum PerView {
    Angle,
    Time,
}

#[derive(Debug, PartialEq)]
enum CoordView {
    Equatorial,
    Horizontal,
    Ecliptic,
}

#[derive(Debug, PartialEq)]
enum WordValue {
    Date(time::Date),
    Per(time::Period, PerView),
    Crd(coord::Coord, CoordView),
}

/// Reads anything that at its core is a number,
/// These numbers are floating point and can have prefixes or suffixes
///
/// Prefixes:
///
/// | Prefix | Meaning       |
/// |--------|---------------|
/// | `@`    | Unix Date     |
///
/// Suffixes:
///
/// | Suffix | Meaning       |
/// |--------|---------------|
/// | `U`    | Unix Date     |
/// | `JD`   | Julian Day    |
/// | `J`    | Julian Day    |
/// | `D`    | Degrees       |
/// | `d`    | Degrees       |
/// | `deg`  | Degrees       |
/// | `°`    | Degrees       |
/// | `rad`  | Radians       |
/// | `H`    | Decimal Hours |
/// | `h`    | Decimal Hours |
/// | `rad`  | Radians       |
///
fn read_numerical(s: &str) -> Option<WordValue> {
    // Date
    if s.starts_with("@") {
        return Some(WordValue::Date(time::Date::from_unix(
            (s.strip_prefix("@"))?.parse().ok()?,
        )));
    } else if s.ends_with("U") {
        return Some(WordValue::Date(time::Date::from_unix(
            (s.strip_suffix("U"))?.parse().ok()?,
        )));
    } else if s.ends_with("JD") {
        return Some(WordValue::Date(time::Date::from_julian(
            (s.strip_suffix("JD"))?.parse().ok()?,
        )));
    } else if s.ends_with("J") {
        return Some(WordValue::Date(time::Date::from_julian(
            (s.strip_suffix("J"))?.parse().ok()?,
        )));
    }
    // Angle
    else if s.ends_with("D") {
        return Some(WordValue::Per(
            time::Period::from_degrees((s.strip_suffix("D"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("d") {
        return Some(WordValue::Per(
            time::Period::from_degrees((s.strip_suffix("d"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("deg") {
        return Some(WordValue::Per(
            time::Period::from_degrees((s.strip_suffix("deg"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("°") {
        return Some(WordValue::Per(
            time::Period::from_degrees((s.strip_suffix("°"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("rad") {
        return Some(WordValue::Per(
            time::Period::from_radians((s.strip_suffix("rad"))?.parse().ok()?),
            PerView::Angle,
        ));
    }
    // Time
    else if s.ends_with("H") {
        return Some(WordValue::Per(
            time::Period::from_decimal((s.strip_suffix("H"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else if s.ends_with("h") {
        return Some(WordValue::Per(
            time::Period::from_decimal((s.strip_suffix("h"))?.parse().ok()?),
            PerView::Angle,
        ));
    } else {
        return None;
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rdunix() {
        assert_eq!(
            read_numerical("@86400").unwrap(),
            WordValue::Date(time::Date::from_calendar(1970, 1, 2.0))
        );
        assert_eq!(
            read_numerical("86400U").unwrap(),
            WordValue::Date(time::Date::from_calendar(1970, 1, 2.0))
        );
        assert_eq!(
            read_numerical("86400JD").unwrap(),
            WordValue::Date(time::Date::from_julian(86400.0))
        );
        assert_eq!(read_numerical("@86400U"), None);

        assert_eq!(
            read_numerical("120.5D").unwrap(),
            WordValue::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5deg").unwrap(),
            WordValue::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5d").unwrap(),
            WordValue::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
        assert_eq!(
            read_numerical("120.5°").unwrap(),
            WordValue::Per(time::Period::from_degrees(120.5), PerView::Angle)
        );
    }
}
