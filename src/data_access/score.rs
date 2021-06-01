use super::*;
use std::time::SystemTime;

const GRADE_STEPS: [isize; 7] = [
    0, 8_600_000, 8_900_000, 9_200_000, 9_500_000, 9_800_000, 9_900_000,
];

#[derive(Debug)]
pub struct LookupedScore {
    pub title: String,
    pub difficulty: &'static str,
    pub score: isize,
    pub shiny: isize,
    pub pure: isize,
    pub far: isize,
    pub lost: isize,
    pub clear_type: &'static str,
    pub rating: f64,
    pub base_rating: f64,
    pub played_date: i64,
}

impl LookupedScore {
    pub fn get_clear_type(clear_type: i8) -> &'static str {
        lazy_static! {
            static ref CLEAR_TYPES: [&'static str; 6] = [
                "track-lost",
                "normal-clear",
                "full-recall",
                "pure-memory",
                "easy-clear",
                "hard-clear",
            ];
        };
        CLEAR_TYPES[clear_type as usize]
    }

    pub fn get_diff_str(difficulty: i8) -> &'static str {
        lazy_static! {
            static ref DIFFS: [&'static str; 4] = ["PST", "PRS", "FTR", "BYD"];
        }
        DIFFS[difficulty as usize]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScoreRecord {
    song_token: String,
    song_hash: String,
    pub song_id: String,
    pub difficulty: i8,
    pub score: isize,
    #[serde(rename = "shiny_perfect_count")]
    pub shiny: isize,
    #[serde(rename = "perfect_count")]
    pub pure: isize,
    #[serde(rename = "near_count")]
    pub far: isize,
    #[serde(rename = "miss_count")]
    pub lost: isize,
    pub health: i8,
    pub modifier: isize,
    #[serde(skip_serializing_if = "is_zero")]
    beyond_gauge: i32,
    pub clear_type: i8,
}

impl ScoreRecord {
    pub fn new() -> Self {
        ScoreRecord {
            song_hash: String::new(),
            song_token: String::new(),
            song_id: String::new(),
            difficulty: 0,
            score: 0,
            shiny: 0,
            pure: 0,
            far: 0,
            lost: 0,
            health: 0,
            modifier: 0,
            beyond_gauge: 0,
            clear_type: 0,
        }
    }

    pub fn score2grade(&self) -> u8 {
        let mut grade = -1;
        for step in GRADE_STEPS.iter() {
            if self.score >= *step {
                grade += 1
            } else {
                break;
            }
        }
        grade as u8
    }

    fn score2rating(&self, tx: &PooledSqlite) -> Result<f64, rusqlite::Error> {
        let base_rating: f64 = tx.query_row(
            sql_stmt::BASE_RATING,
            params![self.song_id, self.difficulty],
            |row| row.get(0),
        )?;
        if base_rating == 0.0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let mut rating: f64;
        if self.score >= 10_000_000 {
            rating = base_rating + 2.
        } else if self.score >= 9_800_000 {
            rating = base_rating + 1. + (self.score - 9_800_000) as f64 / 200_000.;
        } else {
            rating = base_rating + (self.score - 9_500_000) as f64 / 300_000.;
            if rating < 0. {
                rating = 0.;
            }
        }
        Ok(rating)
    }

    fn insert_score_record(
        &self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        time_played: i64,
        rating: f64,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = tx.prepare(sql_stmt::INSERT_SCORE)?;
        stmt.execute(params![
            user_id,
            time_played,
            self.song_id,
            self.difficulty,
            self.score,
            self.shiny,
            self.pure,
            self.far,
            self.lost,
            rating,
            self.health,
            self.clear_type,
        ])?;
        Ok(())
    }

    fn update_best_score(
        &self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        time_played: i64,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = tx.prepare(sql_stmt::QUERY_BEST_SCORE)?;
        let result = stmt.query_row(params![user_id, self.song_id, self.difficulty], |row| {
            Ok((row.get::<usize, isize>(0)?, row.get::<usize, i64>(1)?))
        });
        match result {
            Ok((score, played_date)) => {
                if score < self.score {
                    tx.execute(
                        sql_stmt::UPDATE_BEST_SCORE,
                        params![time_played, played_date],
                    )?;
                }
            }
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    tx.execute(sql_stmt::INSERT_BEST_SCORE, params![user_id, time_played])?;
                }
                _ => return Err(e),
            },
        }
        Ok(())
    }

    fn update_recent_score(
        &self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        time_played: i64,
        rating: f64,
    ) -> Result<(), rusqlite::Error> {
        let new_item = RecentScoreItem {
            played_date: time_played,
            rating,
        };
        let new_identifier = format!("{}{}", self.song_id, self.difficulty);
        let mut inserter = RecentScoreInserter::new();
        let mut stmt = tx.prepare(sql_stmt::QUERY_RECENT_SCORE)?;
        let items = stmt.query_map(params![user_id], |row| {
            Ok((
                RecentScoreItem {
                    played_date: row.get("played_date")?,
                    rating: row.get("rating")?,
                },
                row.get("iden")?,
                row.get::<&str, String>("is_recent_10")? == "t",
            ))
        })?;
        for item in items {
            let item = item.unwrap();
            if item.2 {
                inserter.r10.insert(item.1, item.0);
            } else {
                inserter.normal_item.push(item.0);
            }
        }
        inserter.insert(
            tx,
            user_id,
            new_item,
            new_identifier,
            self.score,
            self.clear_type,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
struct RecentScoreItem {
    played_date: i64,
    rating: f64,
}

struct RecentScoreInserter {
    r10: HashMap<String, RecentScoreItem>,
    normal_item: Vec<RecentScoreItem>,
}

impl RecentScoreInserter {
    fn new() -> Self {
        RecentScoreInserter {
            r10: HashMap::new(),
            normal_item: Vec::new(),
        }
    }

    fn insert(
        &self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        new_item: RecentScoreItem,
        new_identifier: String,
        score: isize,
        clear_type: i8,
    ) -> Result<(), rusqlite::Error> {
        let target = &new_item;
        let (target, replacement, is_r10, need_new_r10) =
            self.insert_into_r10(tx, user_id, &new_identifier, target, score, clear_type)?;
        self.insert_into_normal_item(tx, user_id, target, replacement, is_r10, need_new_r10)?;
        Ok(())
    }

    fn insert_into_r10<'a>(
        &'a self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        identifier: &String,
        target: &'a RecentScoreItem,
        score: isize,
        clear_type: i8,
    ) -> Result<
        (
            Option<&'a RecentScoreItem>,
            Option<&'a RecentScoreItem>,
            bool,
            bool,
        ),
        rusqlite::Error,
    > {
        // target may change during trying to insert it into r10, ret_target is
        // the final target in this process, and the starting target for next
        // process (insert into normat item).
        let mut ret_target = None;
        // candidate record that current target will possiblely replace.
        let mut replacement = None;
        // wheather current target record should be marked as an r10.
        let mut is_r10 = false;
        // need_new_r10, if true, record with highest rating among normal item
        // will become a new r10 item.
        let mut need_new_r10 = false;
        match self.r10.get(identifier) {
            // r10 group does not allow repeated chart record
            Some(record) => {
                if record.rating <= target.rating {
                    tx.execute(
                        sql_stmt::REPLACE_RECENT_SCORE,
                        params![target.played_date, "t", user_id, record.played_date],
                    )?;
                    ret_target = Some(record);
                } else {
                    ret_target = Some(target);
                }
            }
            None => {
                let r30_not_full = self.r10.len() + self.normal_item.len() < 30;
                if self.r10.len() < 10 {
                    if r30_not_full {
                        tx.execute(
                            sql_stmt::INSERT_RECENT_SCORE,
                            params![user_id, target.played_date, "t"],
                        )?;
                        // no need for further process, no target any more
                        ret_target = None;
                    } else {
                        // number of r10 records is less than 10,
                        // newly insterted record must be an r10. will look for
                        // replacement among normal items.
                        is_r10 = true;
                    }
                } else {
                    let is_ex = score >= 9_800_000;
                    let is_hard_clear = clear_type == 5;
                    for item in self.r10.values() {
                        if (is_ex || is_hard_clear || r30_not_full) && target.rating < item.rating {
                            continue;
                        }
                        if item.rating <= target.rating {
                            is_r10 = true;
                        }
                        match replacement {
                            None => replacement = Some(item),
                            Some(ref r) => {
                                if item.played_date < r.played_date {
                                    replacement = Some(item)
                                }
                            }
                        }
                    }
                    if is_r10 {
                        let record = replacement.take().unwrap();
                        tx.execute(
                            sql_stmt::REPLACE_RECENT_SCORE,
                            params![target.played_date, "t", user_id, record.played_date],
                        )?;
                        ret_target = Some(record);
                        is_r10 = false;
                    } else {
                        need_new_r10 = true;
                    }
                }
            }
        }
        Ok((ret_target, replacement, is_r10, need_new_r10))
        // Possible return values:
        // None, None, false, false. When both r10 and r30 is not full.
        // Some, None,  true, false. When r10 is not full but r30 is full.
        // Some, Some, false,  true. When new record can't be insert into r10.
        // Some, None, false, false. When new record's identifier collides, or
        //                           new record insert into r10 and take a old
        //                           record out of r10 group
    }

    fn insert_into_normal_item<'a>(
        &'a self,
        tx: &rusqlite::Transaction,
        user_id: isize,
        target: Option<&'a RecentScoreItem>,
        mut replacement: Option<&'a RecentScoreItem>,
        is_r10: bool,
        mut need_new_r10: bool,
    ) -> Result<(), rusqlite::Error> {
        if target.is_none() {
            return Ok(());
        }
        let mut target = target.unwrap();
        if is_r10 {
            // is_r10 will be true only when r10 is not full but r30 is.
            tx.execute(
                sql_stmt::REPLACE_RECENT_SCORE,
                params![
                    target.played_date,
                    "t",
                    user_id,
                    self.normal_item[0].played_date,
                ],
            )?;
            target = &self.normal_item[0];
        }
        if self.r10.len() + self.normal_item.len() < 30 {
            tx.execute(
                sql_stmt::INSERT_RECENT_SCORE,
                params![user_id, target.played_date, ""],
            )?;
            return Ok(());
        }
        for item in &self.normal_item {
            match replacement {
                None => replacement = Some(item),
                Some(ref r) => {
                    if item.played_date < r.played_date {
                        replacement = Some(item);
                        need_new_r10 = false;
                    }
                }
            }
        }
        let r = replacement.unwrap();
        if r.played_date != target.played_date {
            tx.execute(
                sql_stmt::REPLACE_RECENT_SCORE,
                params![target.played_date, "", user_id, r.played_date],
            )?;
        }
        if need_new_r10 {
            // if need_new_r10 is true, record being replaced is in r10, so it's
            // safe to directly take highest rating record from normal item group
            // as new a r10 record.
            let new_r10 = self.normal_item[0].played_date;
            tx.execute(
                sql_stmt::INSERT_RECENT_SCORE,
                params![new_r10, "t", user_id, new_r10],
            )?;
        }
        Ok(())
    }
}

pub fn score_upload(
    conn: &mut DBAccessManager,
    score_record: &ScoreRecord,
    user_id: isize,
    time: Option<&i64>,
) -> Result<HashMap<String, isize>, rusqlite::Error> {
    let mut result = HashMap::new();
    let rating = score_record.score2rating(&conn.connection)?;
    let tx = conn.connection.transaction()?;
    let time_played: i64;
    match time {
        Some(t) => time_played = *t,
        None => {
            time_played = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
        }
    }
    score_record.insert_score_record(&tx, user_id, time_played, rating)?;
    score_record.update_best_score(&tx, user_id, time_played)?;
    score_record.update_recent_score(&tx, user_id, time_played, rating)?;
    let rating = update_player_rating(&tx, user_id)?;
    tx.commit()?;
    result.insert("user_rating".to_string(), rating);
    Ok(result)
}

fn update_player_rating(
    tx: &rusqlite::Transaction,
    user_id: isize,
) -> Result<isize, rusqlite::Error> {
    let mut stmt = tx.prepare(sql_stmt::COMPUTE_RATING)?;
    let rating: f64 = stmt.query_row(params![user_id], |row| row.get(0))?;
    stmt = tx.prepare(sql_stmt::UPDATE_RATING)?;
    stmt.execute(params![rating, user_id])?;

    Ok(rating as isize)
}

pub fn get_best_scores_with_iden(
    conn: &DBAccessManager,
    user_id: isize,
) -> Result<HashMap<String, isize>, rusqlite::Error> {
    let mut scores = HashMap::new();
    let mut stmt = conn
        .connection
        .prepare(sql_stmt::QUERY_BEST_SCORE_WITH_IDEN)?;
    let results = stmt
        .query_map(params![user_id], |row| {
            Ok((row.get("iden")?, row.get("score")?))
        })?;

    for score in results {
        let score = score.unwrap();
        scores.insert(score.0, score.1);
    }
    Ok(scores)
}

pub fn get_all_best_scores(
    conn: &DBAccessManager,
    user_id: isize,
) -> Result<Vec<(ScoreRecord, i64)>, rusqlite::Error> {
    let mut stmt = conn
        .connection
        .prepare(sql_stmt::QUERY_BEST_SCORE_FOR_BACKUP)?;
    let results = stmt
        .query_map(params![user_id], |row| {
            let mut record = ScoreRecord::new();
            record.song_id = row.get("song_id")?;
            record.difficulty = row.get("difficulty")?;
            record.score = row.get("score")?;
            record.shiny = row.get("shiny_pure")?;
            record.pure = row.get("pure")?;
            record.far = row.get("far")?;
            record.lost = row.get("lost")?;
            record.health = row.get("health")?;
            record.modifier = row.get("modifier")?;
            record.clear_type = row.get("clear_type")?;
            Ok((record, row.get("played_date")?))
        })?;
    Ok(results.into_iter().map(|x| x.unwrap()).collect())
}

pub fn score_lookup(
    conn: &DBAccessManager,
    user_id: isize,
) -> Result<Vec<LookupedScore>, rusqlite::Error> {
    let mut stmt = conn
        .connection
        .prepare(sql_stmt::QUERY_BEST_SCORE_FOR_LOOKUP)?;
    let results = stmt
        .query_map(params![user_id], |row| {
            let record = LookupedScore {
                title: row.get("title")?,
                difficulty: LookupedScore::get_diff_str(row.get("difficulty")?),
                score: row.get("score")?,
                shiny: row.get("shiny_pure")?,
                pure: row.get("pure")?,
                far: row.get("far")?,
                lost: row.get("lost")?,
                clear_type: LookupedScore::get_clear_type(row.get("clear_type")?),
                played_date: row.get("played_date")?,
                rating: row.get("rating")?,
                base_rating: row.get("base_rating")?,
            };
            Ok(record)
        })?;
    Ok(results.into_iter().map(|x| x.unwrap()).collect())
}
