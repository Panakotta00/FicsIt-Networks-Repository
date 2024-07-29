use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

#[derive(Serialize)]
struct SerializeSemverReq<'a>(
    #[serde(serialize_with="serialize_semver_req")]
    &'a semver::VersionReq
);

pub fn serialize_semver_req<S: Serializer>(v: &semver::VersionReq, s: S) -> Result<S::Ok, S::Error>  {
    s.serialize_str(&v.to_string())
}
pub fn serialize_semver_req_opt<S: Serializer>(v: &Option<semver::VersionReq>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(v) => s.serialize_some(&SerializeSemverReq(v)),
        None => s.serialize_none(),
    }
}

#[derive(Deserialize)]
struct DeserializeSemverReq(
    #[serde(deserialize_with="deserialize_semver_req")]
    semver::VersionReq
);

struct SerializeSemverReqVisitor;
impl<'d> Visitor<'d> for SerializeSemverReqVisitor {
    type Value = semver::VersionReq;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an string with semver encoding (https://semver.org/)")
    }

    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
        semver::VersionReq::parse(s).map_err(serde::de::Error::custom)
    }
}

pub fn deserialize_semver_req<'d, D: Deserializer<'d>>(d: D) -> Result<semver::VersionReq, D::Error> {
    d.deserialize_str(SerializeSemverReqVisitor)
}
pub fn deserialize_semver_req_opt<'d, D: Deserializer<'d>>(d: D) -> Result<Option<semver::VersionReq>, D::Error> {
    Option::<DeserializeSemverReq>::deserialize(d).map(|o| o.map(|v| v.0))
}