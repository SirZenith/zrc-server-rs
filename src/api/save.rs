use super::*;
use std::fmt;
use std::time::SystemTime;

#[derive(Deserialize, Serialize, Debug)]
struct SimpleIntData {
    val: isize,
}

#[derive(Deserialize, Serialize, Debug)]
struct SimpleStringData {
    val: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ScoreData {
    song_id: String,
    version: isize,
    difficulty: u8,
    score: u32,
    shiny_perfect_count: usize,
    perfect_count: usize,
    near_count: usize,
    miss_count: usize,
    health: usize,
    modifier: usize,
    time_played: i64,
    ct: isize,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClearLampData {
    song_id: String,
    difficulty: u8,
    clear_type: u8,
    ct: isize,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClearedSongData {
    song_id: String,
    difficulty: u8,
    grade: u8,
}

#[derive(Deserialize, Serialize, Debug)]
struct UnlocklistData {
    unlock_key: String,
    complete: isize,
}

#[derive(Deserialize, Serialize, Debug)]
struct StoryData {
    ma: isize,
    mi: isize,
    c: bool,
    r: bool,
}

#[derive(Serialize, Debug)]
pub struct BackupData {
    version_data: SimpleIntData,
    scores_data: HashMap<String, Vec<ScoreData>>,
    clearlamps_data: HashMap<String, Vec<ClearLampData>>,
    clearedsongs_data: HashMap<String, Vec<ClearedSongData>>,
    unlocklist_data: HashMap<String, Vec<UnlocklistData>>,
    installid_data: SimpleStringData,
    devicemodelname_data: SimpleStringData,
    story_data: HashMap<String, Vec<StoryData>>,
    checksums: HashMap<String, String>,
    #[serde(rename = "createAt")]
    created_at: i64,
}

impl BackupData {
    fn new() -> Self {
        BackupData {
            version_data: SimpleIntData { val: 1 },
            scores_data: HashMap::new(),
            clearlamps_data: HashMap::new(),
            clearedsongs_data: HashMap::new(),
            unlocklist_data: HashMap::new(),
            installid_data: SimpleStringData { val: String::new() },
            devicemodelname_data: SimpleStringData { val: String::new() },
            story_data: HashMap::new(),
            checksums: HashMap::new(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

impl<'de> Deserialize<'de> for BackupData {
    fn deserialize<D>(deserializer: D) -> Result<BackupData, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = BackupData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a query string specifying song ids and whether url is needed.")
            }

            fn visit_map<V>(self, mut map: V) -> Result<BackupData, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut data = BackupData::new();
                while let Some(key) = map.next_key()? {
                    match key {
                        "version_data" => {
                            let value = map.next_value::<String>()?;
                            data.version_data = serde_json::from_str(&value).unwrap();
                        }
                        "scores_data" => {
                            let value = map.next_value::<String>()?;
                            data.scores_data = serde_json::from_str(&value).unwrap();
                        }
                        "clearlamps_data" => {
                            let value = map.next_value::<String>()?;
                            data.clearlamps_data = serde_json::from_str(&value).unwrap();
                        }
                        "clearedsongs_data" => {
                            let value = map.next_value::<String>()?;
                            data.clearedsongs_data = serde_json::from_str(&value).unwrap();
                        }
                        "unloclist_data" => {
                            let value = map.next_value::<String>()?;
                            data.unlocklist_data = serde_json::from_str(&value).unwrap();
                        }
                        "installid_data" => {
                            let value = map.next_value::<String>()?;
                            data.installid_data = serde_json::from_str(&value).unwrap();
                        }
                        "devicemodelname_data" => {
                            let value = map.next_value::<String>()?;
                            data.devicemodelname_data = serde_json::from_str(&value).unwrap();
                        }
                        "story_data" => {
                            let value = map.next_value::<String>()?;
                            data.story_data = serde_json::from_str(&value).unwrap();
                        }
                        _ => {
                            if key.ends_with("_checksum") {
                                let key = key.trim_end_matches("_checksum");
                                let value = map.next_value::<String>()?;
                                data.checksums.insert(key.to_string(), value);
                            }
                        }
                    }
                }
                Ok(data)
            }
        }
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

pub fn upload_backup_data(data: BackupData, conn: DBAccessManager) {}
