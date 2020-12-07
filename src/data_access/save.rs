use super::score::ScoreRecord;
use super::*;
use rusqlite::OptionalExtension;
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
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    checksums: HashMap<String, String>,
}

impl BackupData {
    pub fn new() -> Self {
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

    pub fn new_with_id(user_id: isize) -> Self {
        let mut data = BackupData::new();
        data.user_id = user_id;
        data
    }

    pub fn insert_other_data(
        &self,
        conn: &DBAccessManager,
        user_id: isize,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = conn
            .connection
            .prepare(sql_stmt::INSERT_OTHER_BACKUP)
            .unwrap();
        stmt.execute(params![
            user_id,
            self.version.val,
            serde_json::to_string(&self.unlocklist[""]).unwrap(),
            self.installid.val,
            self.devicemodelname.val,
            serde_json::to_string(&self.story[""]).unwrap(),
            self.created_at,
        ])?;

        Ok(())
    }

    pub fn update_score_on_cloud(
        &mut self,
        conn: &mut DBAccessManager,
        user_id: isize,
    ) -> Result<(), rusqlite::Error> {
        let mut recieved_record: HashMap<String, ScoreRecord> = HashMap::new();
        let mut time_played: HashMap<String, i64> = HashMap::new();
        if let Some(scores) = self.scores.get("") {
            for score in scores {
                let mut record = ScoreRecord::new();
                record.song_id = score.song_id.clone();
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
        if let Some(lamps) = self.clearlamps.get("") {
            for lamp in lamps {
                let iden = format!("{}{}", lamp.song_id, lamp.difficulty);
                if let Some(record) = recieved_record.get_mut(&iden) {
                    record.clear_type = lamp.clear_type;
                }
            }
        }
        let best_with_iden = conn.get_best_scores_with_iden(user_id)?;
        let mut score_updated = false;
        for (iden, record) in &recieved_record {
            if let Some(score) = best_with_iden.get(iden) {
                if record.score > *score {
                    conn.score_upload(record, user_id, time_played.get(iden))?;
                    score_updated = true;
                }
            } else {
                match conn.score_upload(record, user_id, time_played.get(iden)) {
                    Ok(_) => {
                        score_updated = true;
                    }
                    Err(e) => {
                        println!(
                            "Error upload local score for {}-{}, {:?}",
                            record.song_id, record.difficulty, e
                        );
                    }
                };
            }
        }
        if score_updated {
            self.installid.val = "".to_string();
        }
        Ok(())
    }
}

impl BackupData {
    pub fn get_other_data(&mut self, conn: &DBAccessManager, user_id: isize) -> bool {
        let mut stmt = conn
            .connection
            .prepare(sql_stmt::QUERY_BACKUP_DATA)
            .unwrap();
        let result = stmt
            .query_row(params![user_id], |row| {
                Ok((
                    row.get::<&str, isize>("version")?,
                    row.get::<&str, String>("unlocklist")?,
                    row.get::<&str, String>("installid")?,
                    row.get::<&str, String>("devicemodel_name")?,
                    row.get::<&str, String>("story")?,
                    row.get::<&str, i64>("create_at")?,
                ))
            })
            .optional()
            .unwrap();
        match result {
            None => false,
            Some((version, unlocklist, installid, devicemodel, story, create_at)) => {
                self.version.val = version;
                self.unlocklist
                    .insert("".to_string(), serde_json::from_str(&unlocklist).unwrap());
                self.installid.val = installid;
                self.devicemodelname.val = devicemodel;
                self.story
                    .insert("".to_string(), serde_json::from_str(&story).unwrap());
                self.created_at = create_at;
                true
            }
        }
    }

    pub fn get_score_data(&mut self, conn: &DBAccessManager, user_id: isize) {
        let score_data = self.scores.get_mut("").unwrap();
        let lamp_data = self.clearlamps.get_mut("").unwrap();
        let cleared_song_data = self.clearedsongs.get_mut("").unwrap();
        let records_and_time = conn.get_all_best_for_backup(user_id).unwrap();
        for (record, time) in records_and_time {
            score_data.push(ScoreData::from_record_time(&record, time));
            lamp_data.push(ClearLampData::from_record(&record));
            cleared_song_data.push(ClearedSongData::from_record(&record));
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
                        "unlocklist_data" => {
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
