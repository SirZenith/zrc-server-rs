use super::*;

pub struct DLItem {
    pub song_id: String,
    pub audio_checksum: String,
    pub song_dl: bool,
    pub difficulty: String,
    pub chart_checksum: String,
    pub chart_dl: bool,
}

#[derive(Serialize, Debug)]
pub struct Checksum {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub audio: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub chart: HashMap<String, HashMap<String, String>>,
}

pub type ChecksumList = HashMap<String, Checksum>;