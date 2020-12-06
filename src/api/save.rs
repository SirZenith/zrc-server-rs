use super::*;
use data_access::score::ScoreRecord;
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
    difficulty: i8,
    score: isize,
    #[serde(rename = "shiny_perfect_count")]
    shiny: isize,
    #[serde(rename = "perfect_count")]
    pure: isize,
    #[serde(rename = "near_count")]
    far: isize,
    #[serde(rename = "miss_count")]
    lost: isize,
    health: i8,
    modifier: isize,
    time_played: i64,
    ct: isize,
}

impl ScoreData {
    fn from_record_time(record: &data_access::score::ScoreRecord, time: i64) -> Self {
        ScoreData {
            song_id: record.song_id.clone(),
            version: 1,
            difficulty: record.difficulty,
            score: record.score,
            shiny: record.shiny,
            pure: record.pure,
            far: record.far,
            lost: record.lost,
            health: record.health,
            modifier: record.modifier,
            time_played: time,
            ct: 0,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ClearLampData {
    song_id: String,
    difficulty: i8,
    clear_type: i8,
    ct: isize,
}

impl ClearLampData {
    fn from_record(record: &data_access::score::ScoreRecord) -> Self {
        ClearLampData {
            song_id: record.song_id.clone(),
            difficulty: record.difficulty,
            clear_type: record.clear_type,
            ct: 0,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ClearedSongData {
    song_id: String,
    difficulty: i8,
    grade: u8,
}

impl ClearedSongData {
    fn from_record(record: &data_access::score::ScoreRecord) -> Self {
        ClearedSongData {
            song_id: record.song_id.clone(),
            difficulty: record.difficulty,
            grade: record.score2grade(),
        }
    }
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
    user_id: isize,
    version: SimpleIntData,
    scores: HashMap<String, Vec<ScoreData>>,
    clearlamps: HashMap<String, Vec<ClearLampData>>,
    clearedsongs: HashMap<String, Vec<ClearedSongData>>,
    unlocklist: HashMap<String, Vec<UnlocklistData>>,
    installid: SimpleStringData,
    devicemodelname: SimpleStringData,
    story: HashMap<String, Vec<StoryData>>,
    #[serde(rename = "createAt")]
    created_at: i64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    checksums: HashMap<String, String>,
}

impl BackupData {
    fn new() -> Self {
        let mut data = BackupData {
            user_id: 0,
            version: SimpleIntData { val: 1 },
            scores: HashMap::new(),
            clearlamps: HashMap::new(),
            clearedsongs: HashMap::new(),
            unlocklist: HashMap::new(),
            installid: SimpleStringData { val: String::new() },
            devicemodelname: SimpleStringData { val: String::new() },
            story: HashMap::new(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            checksums: HashMap::new(),
        };
        data.scores.insert("".to_string(), Vec::new());
        data.clearlamps.insert("".to_string(), Vec::new());
        data.clearedsongs.insert("".to_string(), Vec::new());
        data.unlocklist.insert("".to_string(), Vec::new());
        data.story.insert("".to_string(), Vec::new());
        data
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
                            data.version = serde_json::from_str(&value).unwrap();
                        }
                        "scores_data" => {
                            let value = map.next_value::<String>()?;
                            data.scores = serde_json::from_str(&value).unwrap();
                        }
                        "clearlamps_data" => {
                            let value = map.next_value::<String>()?;
                            data.clearlamps = serde_json::from_str(&value).unwrap();
                        }
                        "clearedsongs_data" => {
                            let value = map.next_value::<String>()?;
                            data.clearedsongs = serde_json::from_str(&value).unwrap();
                        }
                        "unloclist_data" => {
                            let value = map.next_value::<String>()?;
                            data.unlocklist = serde_json::from_str(&value).unwrap();
                        }
                        "installid_data" => {
                            let value = map.next_value::<String>()?;
                            data.installid = serde_json::from_str(&value).unwrap();
                        }
                        "devicemodelname_data" => {
                            let value = map.next_value::<String>()?;
                            data.devicemodelname = serde_json::from_str(&value).unwrap();
                        }
                        "story_data" => {
                            let value = map.next_value::<String>()?;
                            data.story = serde_json::from_str(&value).unwrap();
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

// POST /user/me/save
pub async fn upload_backup_data(
    data: BackupData,
    mut conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_id = STATIC_USER_ID;
    update_scores_by_data(&data, &mut conn, user_id).unwrap();

    let mut result = HashMap::new();
    result.insert("user_id", user_id);
    respond(
        ResponseContainer {
            success: true,
            value: result,
            error_code: 0,
        },
        warp::http::StatusCode::OK,
    )
}

fn update_scores_by_data(
    data: &BackupData,
    conn: &mut DBAccessManager,
    user_id: isize,
) -> Result<(), rusqlite::Error> {
    let mut recieved_record: HashMap<String, ScoreRecord> = HashMap::new();
    let mut time_played: HashMap<String, i64> = HashMap::new();
    // TODO: Add error handling
    if let Some(scores) = data.scores.get("") {
        for score in scores {
            let mut record = ScoreRecord::new();
            record.difficulty = score.difficulty;
            record.score = score.score;
            record.shiny = score.shiny;
            record.pure = score.pure;
            record.far = score.far;
            record.lost = score.lost;
            record.health = score.health;
            record.modifier = score.modifier;

            recieved_record.insert(format!("{}{}", score.song_id, score.difficulty), record);
            time_played.insert(
                format!("{}{}", score.song_id, score.difficulty),
                score.time_played,
            );
        }
    }
    if let Some(lamps) = data.clearlamps.get("") {
        for lamp in lamps {
            let iden = format!("{}{}", lamp.song_id, lamp.difficulty);
            if let Some(record) = recieved_record.get_mut(&iden) {
                record.clear_type = lamp.clear_type;
            }
        }
    }
    let best_with_iden = conn.get_best_scores_with_iden(user_id)?;
    for (iden, record) in &recieved_record {
        if let Some(score) = best_with_iden.get(iden) {
            if record.score > *score {
                conn.score_upload(record, user_id, time_played.get(iden))?;
            }
        } else {
            conn.score_upload(record, user_id, time_played.get(iden))?;
        }
    }
    Ok(())
}

// GET /user/me/save
pub async fn download_backup_data(conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    let user_id = STATIC_USER_ID;
    // TODO: When there is no backup data on cloud.
    let mut data = BackupData::new();
    data.user_id = user_id;

    let score_data = data.scores.get_mut("").unwrap();
    let lamp_data = data.clearlamps.get_mut("").unwrap();
    let cleared_song_data = data.clearedsongs.get_mut("").unwrap();
    let records_and_time = conn.get_all_best_for_backup(user_id).unwrap();
    for (record, time) in records_and_time {
        score_data.push(ScoreData::from_record_time(&record, time));
        lamp_data.push(ClearLampData::from_record(&record));
        cleared_song_data.push(ClearedSongData::from_record(&record));
    }

    respond(data, warp::http::StatusCode::OK)
}
