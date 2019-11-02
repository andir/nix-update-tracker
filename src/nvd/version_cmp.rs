
fn get_components<'t>(s: &'t str) -> Vec<&'t str> {
    lazy_static! {
        static ref SPLIT_EXPRESSSION : regex::Regex = regex::Regex::new(r"[\.-]+") // ([0-9]+|[^-\.0-9]+)")
                                                            .expect("get_components split regex compilation failed");
        static ref COMPONENT_EXPRESSION : regex::Regex = regex::Regex::new(r"([0-9]+|[a-z]+)")
                                                            .expect("get_components component regex compilation failed");

    }

    SPLIT_EXPRESSSION
        .split(s)
        .map(|s| COMPONENT_EXPRESSION.find_iter(s))
        .fold(vec![], |mut a: Vec<_>, b| {
            for c in b {
                a.push(c.as_str());
            }
            a
        })
}

use itertools::EitherOrBoth;
use itertools::Itertools;

pub trait VersionCmp<T> {
    fn version_le(&self, other: T) -> bool;
    fn version_eq(&self, other: T) -> bool;
}

impl<T: AsRef<str>> VersionCmp<T> for &str {
    fn version_le(&self, other: T) -> bool {
        let other = other.as_ref();

        let components1 = get_components(&self);
        let components2 = get_components(other);

        #[cfg(test)]
        println!("c1: {:?}, c2: {:?}", components1, components2);

        for comp in components1.iter().zip_longest(components2.iter()) {
            #[cfg(test)]
            println!("comp: {:?}", comp);

            match comp {
                EitherOrBoth::Right(_s) => return false,
                EitherOrBoth::Left(p1) => {
                    if p1 != &"pre" {
                        return true;
                    } else {
                        continue;
                    }
                }
                EitherOrBoth::Both(p1, p2) if p1 == p2 => continue,
                EitherOrBoth::Both(p1, p2)
                    if alphanumeric_sort::compare_str(p1, p2) == std::cmp::Ordering::Greater =>
                {
                    return false
                }
                _ => continue,
            }
        }
        return true;
    }

    fn version_eq(&self, other: T) -> bool {
        // compares for "equality". We allow trailing zeros since those might just be missing minor/patch version numbers
        let other = other.as_ref();

        let components1 = get_components(&self);
        let components2 = get_components(other);

        #[cfg(test)]
        println!("c1: {:?}, c2: {:?}", components1, components2);

        for comp in components1.iter().zip_longest(components2.iter()) {
            #[cfg(test)]
            println!("comp: {:?}", comp);

            match comp {
                EitherOrBoth::Right(s) if s != &"0" => return false,
                EitherOrBoth::Left(p1) if p1 != &"0" => return false,
                EitherOrBoth::Both(p1, p2) if p1 == p2 => continue,
                EitherOrBoth::Both(p1, p2) if p1 != p2 => return false,
                _ => continue,
            }
        }
        return true;
    }
}

#[test]
fn test_version_cmp_eq() {
    let version_data = VersionData {
        version_value: "1.0.2".to_owned(),
        version_affected: Some("=".to_owned()),
    };
    assert!(version_data.matches("1.0.2"));
    assert!(version_data.matches("1.0.2.0"));
    assert!(!version_data.matches("1.0.2.5"));
    assert!(!version_data.matches("1.3.4pre123"));

    let version_data = VersionData {
        version_value: "1.0".to_owned(),
        version_affected: Some("=".to_owned()),
    };

    assert!(version_data.matches("1.0"));
    assert!(version_data.matches("1"));
    assert!(version_data.matches("1.0.0.0.0.0"));
}

#[test]
fn test_version_cmp_none() {
    let version_data = VersionData {
        version_value: "1.3.4".to_owned(),
        version_affected: None,
    };

    assert!(version_data.matches("1.3.4"));
    assert!(!version_data.matches("1.3.5"));
}

#[test]
fn test_version_cmp_le() {
    let version_data = VersionData {
        version_value: "1.3.4".to_owned(),
        version_affected: Some("<=".to_owned()),
    };

    assert!(version_data.matches("1.0.0"));
    assert!(version_data.matches("1.3.4pre123"));
    assert!(!version_data.matches("1.3.5pre123"));
    assert!(!version_data.matches("2.0.0"));

    let version_data = VersionData {
        version_value: "0.84".to_owned(),
        version_affected: Some("<=".to_owned()),
    };
    assert!(!version_data.matches("0.100.2"));
}


