use serde::Deserialize;

#[derive(Deserialize)]
pub struct Status {
    #[serde(rename = "Flags")]
    pub flags: u32,
}

impl Status {
    pub fn from_json(json: String) -> Status {
        serde_json::from_str(&json).expect("Could not parse status JSON")
    }
}
