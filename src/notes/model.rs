//use serde::de::{Deserializer, Deserialize, Error};
use serde_yaml;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Note {
    pub identifier: String,
    pub person: Option<String>,
    pub notes: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub nsa: String,
}

#[derive(Clone)]
pub enum NSAType {
    False,
    Pending,
    Assigned(String),
}

impl Note {
    pub fn new(identifier: String) -> Self {
        Note {
            identifier,
            description: None,
            person: None,
            notes: None,
            nsa: "pending".into(),
        }
    }

    pub fn to_string(&self) -> String {
        // FIXME: the unwrap shouldn't be here
        // FIXME: to_str should be part of a trait implementation...
        serde_yaml::to_string(&self).unwrap()
    }
}

trait NSA {
    fn is_pending(&self) -> bool;
    fn is_false(&self) -> bool;
    fn is_nsa(&self) -> bool;
    fn to_nsa(&self) -> Option<NSAType>;
}

impl NSA for NSAType {
    fn is_pending(&self) -> bool {
        match self {
            NSAType::Pending => true,
            _ => false,
        }
    }

    fn is_false(&self) -> bool {
        match self {
            NSAType::False => true,
            _ => false,
        }
    }

    fn is_nsa(&self) -> bool {
        match self {
            NSAType::Assigned(_) => true,
            _ => false,
        }
    }

    fn to_nsa(&self) -> Option<NSAType> {
        Some((*self).clone())
    }
}

impl NSA for Note {
    fn is_pending(&self) -> bool {
        self.nsa.trim().to_lowercase() == "pending"
    }

    fn is_false(&self) -> bool {
        self.nsa.trim().to_lowercase() == "false"
    }

    fn is_nsa(&self) -> bool {
        self.nsa.trim().to_lowercase().starts_with("nsa-")
    }

    fn to_nsa(&self) -> Option<NSAType> {
        let s = self.nsa.trim().to_lowercase();
        match &s[..] {
            "pending" => Some(NSAType::Pending),
            "false" => Some(NSAType::False),
            x => {
                if x.len() > 4 && &x[..4] == "nsa-" {
                    Some(NSAType::Assigned(self.nsa.clone()))
                } else {
                    None
                }
            }
        }
    }
}

//impl NoteNSA {
//    pub fn as_string(&self) -> String {
//        match *self {
//            NoteNSA::Pending => "pending".into(),
//            NoteNSA::False(_) => "false".into(),
//            NoteNSA::Assigned(ref v) => v.clone(),
//        }
//    }
//}

// #[derive(Debug)]
// pub struct NotAValidNSAState;
// impl stdError for NotAValidNSAState {
//     fn description(&self) -> &str {
//         r#"The input is not a valid NSA state. Valid values are "false", "pending" or any value starting with "NSA-""#
//     }
// }
//
// impl fmt::Display for NotAValidNSAState {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.description())
//     }
// }
//
// impl FromStr for NoteNSA {
//     type Err = NotAValidNSAState;
//     fn from_str(s: &str) -> Result<Self, NotAValidNSAState> {
//         let lc = s.to_lowercase();
//         let val = match lc.as_ref() {
//             "pending" => NoteNSA::Pending,
//             "false" => NoteNSA::False(false),
//             lc => {
//                 if lc.len() < 4 || &lc[..4] != "nsa-" {
//                     return Err(NotAValidNSAState {});
//                 } else {
//                     NoteNSA::Assigned(s.into())
//                 }
//             }
//         };
//         Ok(val)
//     }
// }
//
// impl Default for NoteNSA {
//     fn default() -> Self {
//         NoteNSA::Pending
//     }
// }

// impl<'d> ::serde::de::Visitor<'d> for NoteNSA
// //        where __D: ::serde::de::Deserializer
// {
//     type Value = NoteNSA;
//
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         write!(formatter, "a string containing a valid NSA value")
//     }
//
//     fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
//         where E: ::serde::de::Error
//     {
//         FromStr::from_str(value).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
//     }
// }
//
//
// impl ::serde::ser::Serialize for NoteNSA {
//     fn serialize<__S>(&self, serializer: __S) -> Result<<__S as serde::Serializer>::Ok, __S::Error>
//         where __S: ::serde::ser::Serializer
//     {
//          serializer.serialize_str(&self.as_string())
//     }
// }

#[cfg(test)]
mod tests {

    use super::Note;
    use serde_yaml;
    use std::str::FromStr;

    const INPUT: &'static str = r#"---
identifier: CVE-20XX-XXXXXX

description: >
    A generic issue that breaks the world an everyone panics on more time.
    Next week there will be an issue even worse so don't panic (yet)!

# the person that "claims" an issue as working on it. This would be
# from a set of people that have commit access otherwise there might
# be many PRs just to claim and more to add notes to them?
# having this kind of feature would probably be helpful to avoid duplicate
# work if someone is already working on it.
person: andir

# three-ish well-defined values: false, pending, NSA-20XX-XXXXX (other optional?)
nsa: pending

# a simple multi-line string allowing for reference, notes, ...
notes: >
    Will be addressed by backporting an update (123d3124)

    Darwin & aarch64 are not affected because $randomAnswer.

    Upstream patch: https://git.acme.tld/cgit/project/commit/123123123123.diff

    ---
    Automatically imported from NVD Database on 2018-08-13 using $software, $version
"#;

    #[test]
    fn parse_example_document_pending() {
        let doc: Note = serde_yaml::from_str(INPUT).unwrap();
        assert_eq!(doc.nsa, "pending");
        assert_eq!(doc.identifier, "CVE-20XX-XXXXXX");
        assert_eq!(doc.person, Some("andir".into()));
        assert!(doc.notes.unwrap().find("123d3124").unwrap() > 0);
        assert!(doc.description.unwrap().find("panic").unwrap() > 0);
    }

    #[test]
    fn test_full_circle() {
        let doc1: Note = serde_yaml::from_str(INPUT).unwrap();
        let s = doc1.to_string();
        println!("string: {}", s);
        let doc2: Note = serde_yaml::from_str(&s).unwrap();
        assert_eq!(doc1, doc2);
    }

    //    #[test]
    //    fn test_serialize_nsa_field() {
    //        let cases = vec![
    //            ("---\npending", NoteNSA::Pending),
    //            ("---\n\"false\"", NoteNSA::False(false)),
    //            ("---\nnsa-test", NoteNSA::Assigned("nsa-test".into())),
    //        ];
    //
    //        for (output, input) in cases {
    //            let o = serde_yaml::to_string(&input).unwrap();
    //            assert_eq!(o, output);
    //        }
    //    }

    //    #[test]
    //    fn test_invalid_nsa_field() {
    //        let cases = vec!["NSA123", "foo", "123", ""];
    //
    //        for case in cases.iter() {
    //            let r: Result<NoteNSA, _> = NoteNSA::from_str(case);
    //            assert!(r.is_err());
    //        }
    //    }

    //    #[test]
    //    fn test_deserialize_nsa_field() {
    //        let cases = vec![
    //            ("pending", NoteNSA::Pending),
    //            ("false", NoteNSA::False(false)),
    //            ("\"false\"", NoteNSA::False(false)),
    //            ("NSA-123", NoteNSA::Assigned("NSA-123".into())),
    //        ];
    //
    //        for (input, result) in cases.iter() {
    //            let r = NoteNSA::from_str(input).unwrap();
    //            assert_eq!(r, *result);
    //        }
    //    }
}
