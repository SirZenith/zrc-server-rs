use super::*;

#[derive(Serialize)]
struct Setting {
    max_stamina_notification_enabled: bool,
    is_hide_rating: bool,
    favorite_character: i8,
}

#[derive(Serialize)]
struct CoreInfo {
    core_type: String,
    amount: i8,
    #[serde(rename = "_id")]
    id: String,
}

#[derive(Serialize)]
struct MostRecentScore {
    song_id: String,
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
    clear_type: i8,
    best_clear_type: i8,
}

#[derive(Serialize)]
pub struct UserInfo {
    is_aprilfools: bool,
    curr_available_maps: Vec<String>,
    character_stats: crate::character::CharacterStatses,
    friends: Vec<String>,
    settings: Setting,
    user_id: isize,
    name: String,
    display_name: String,
    user_code: String,
    ticket: isize,
    character: i8,
    is_locked_name_duplicate: bool,
    is_skill_sealed: bool,
    current_map: String,
    prog_boost: i8,
    next_fragstam_ts: i64,
    max_stamina_ts: i64,
    stamina: i8,
    world_unlocks: Vec<String>,
    world_songs: Vec<String>,
    singles: Vec<String>,
    packs: Vec<String>,
    characters: Vec<i8>,
    cores: Vec<CoreInfo>,
    recent_score: Vec<MostRecentScore>,
    max_friend: i8,
    rating: isize,
    join_date: i64,
}

impl UserInfo {
    pub fn new(conn: &DBAccessManager, user_id: isize) -> Result<Self, rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::USER_INFO).unwrap();
        let world_unlocks = get_item_list(conn, "item_name", "world_unlock", user_id);
        let world_songs = get_item_list(conn, "item_name", "world_song_unlock", user_id);
        let packs = get_item_list(conn, "pack_name", "pack_purchase_info", user_id);
        let singles = get_item_list(conn, "song_id", "single_purchase_info", user_id);
        let mut user_info = stmt
            .query_row(params![user_id], |row| {
                let settings = Setting {
                    max_stamina_notification_enabled: row
                        .get::<&str, String>("stamina_notification")?
                        == "t",
                    is_hide_rating: row.get::<&str, String>("hide_rating")? == "t",
                    favorite_character: row.get("fav_partner")?,
                };
                let character_stats = super::character::CharacterStatses::new(conn, user_id, None)?;
                let characters = character_stats.list_char_ids();
                Ok(UserInfo {
                    is_aprilfools: row.get::<&str, String>("is_aprilfools")? == "t",
                    curr_available_maps: Vec::new(),
                    character_stats,
                    friends: Vec::new(),
                    settings,
                    user_id: user_id,
                    name: row.get("user_name")?,
                    display_name: row.get("display_name")?,
                    user_code: format!("{:0>9}", row.get::<&str, i64>("user_code")?),
                    ticket: row.get("ticket")?,
                    character: row.get("partner")?,
                    is_locked_name_duplicate: row.get::<&str, String>("locked")? == "t",
                    is_skill_sealed: row.get::<&str, String>("skill_sealed")? == "t",
                    current_map: row.get("curr_map")?,
                    prog_boost: row.get("prog_boost")?,
                    next_fragstam_ts: row.get("next_fragstam_ts")?,
                    max_stamina_ts: row.get("max_stamina_ts")?,
                    stamina: row.get("stamina")?,
                    world_unlocks,
                    world_songs,
                    singles,
                    packs,
                    characters,
                    cores: Vec::new(),
                    recent_score: Vec::new(),
                    max_friend: row.get("max_friend")?,
                    rating: row.get("rating")?,
                    join_date: row.get("join_date")?,
                })
            })
            .unwrap();
        user_info.get_most_recent_score(conn);
        Ok(user_info)
    }

    fn get_most_recent_score(&mut self, conn: &DBAccessManager) {
        let mut stmt = conn
            .connection
            .prepare(sql_stmt::USER_MOST_RECENT_SCORE)
            .unwrap();
        let score = stmt
            .query_row(&[&self.user_id], |row| {
                Ok(MostRecentScore {
                    song_id: row.get("song_id")?,
                    difficulty: row.get("difficulty")?,
                    score: row.get("score")?,
                    shiny: row.get("shiny_pure")?,
                    pure: row.get("pure")?,
                    far: row.get("far")?,
                    lost: row.get("lost")?,
                    health: row.get("health")?,
                    modifier: row.get("modifier")?,
                    time_played: row.get("played_date")?,
                    clear_type: row.get("clear_type")?,
                    best_clear_type: row.get("best_clear_type")?,
                })
            })
            .unwrap();
        self.recent_score.push(score);
    }
}

fn get_item_list(conn: &DBAccessManager, column: &str, table: &str, user_id: isize) -> Vec<String> {
    let mut stmt = conn
        .connection
        .prepare(&format!(
            "select {} from {} where user_id = {}",
            column, table, user_id
        ))
        .unwrap();
    let items = stmt
        .query_map(rusqlite::NO_PARAMS, |row| row.get::<usize, String>(0))
        .unwrap();

    items.into_iter().map(|x| x.unwrap()).collect()
}
