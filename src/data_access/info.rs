use super::*;

// ----------------------------------------------------------------------------
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
    character_stats: crate::data_access::CharacterStatses,
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

// ----------------------------------------------------------------------------
#[derive(Serialize)]
struct PackItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    is_available: bool,
}

#[derive(Serialize)]
pub struct PackInfo {
    name: String,
    items: Vec<PackItem>,
    price: isize,
    orig_price: isize,
    discount_from: i64,
    discount_to: i64,
}

impl PackInfo {
    pub fn get_pack_list(conn: &DBAccessManager) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::PACK_INFO).unwrap();
        let mut item_stmt = conn.connection.prepare(sql_stmt::PACK_ITEM).unwrap();
        let packs = stmt
            .query_map(params![], |row| {
                let name = row.get(0)?;
                let price = row.get(1)?;
                let orig_price = row.get(2)?;
                let discount_from = row.get(3)?;
                let discount_to = row.get(4)?;

                let items = item_stmt
                    .query_map(params![name], |row| {
                        Ok(PackItem {
                            id: row.get(0)?,
                            item_type: row.get(1)?,
                            is_available: row.get::<usize, String>(2)? == "t",
                        })
                    })
                    .unwrap();

                Ok(PackInfo {
                    name,
                    items: items.into_iter().map(|x| x.unwrap()).collect(),
                    price,
                    orig_price,
                    discount_from,
                    discount_to,
                })
            })
            .unwrap();
        Ok(packs.into_iter().map(|x| x.unwrap()).collect())
    }
}

// ----------------------------------------------------------------------------
#[derive(Serialize)]
struct LevelStep {
    level: isize,
    level_exp: isize,
}

#[derive(Serialize)]
pub struct GameInfo {
    curr_ts: i64,
    max_stamina: i8,
    stamina_recover_tick: isize,
    core_exp: isize,
    level_steps: Vec<LevelStep>,
    world_ranking_enabled: bool,
    is_byd_chapter_unlocked: bool,
}

impl GameInfo {
    pub fn new(conn: &DBAccessManager) -> Result<Self, rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::LEVEL_STEP).unwrap();
        let mut level_steps = Vec::new();
        let steps = stmt
            .query_map(params![], |row| {
                Ok(LevelStep {
                    level: row.get(0)?,
                    level_exp: row.get(1)?,
                })
            })
            .unwrap();
        for step in steps {
            let step = step.unwrap();
            level_steps.push(step);
        }

        let (mut curr_ts, mut max_stamina, mut stamina_recover_tick) = (0, 0, 0);
        let (mut core_exp, mut world_ranking_enabled, mut is_byd_chapter_unlocked) =
            (250, false, false);
        let mut stmt = conn.connection.prepare(sql_stmt::GAME_INFO).unwrap();
        stmt.query_row(params![], |row| {
            curr_ts = row.get(0)?;
            max_stamina = row.get(1)?;
            stamina_recover_tick = row.get(2)?;
            core_exp = row.get(3)?;
            world_ranking_enabled = row.get::<usize, String>(4)? == "t";
            is_byd_chapter_unlocked = row.get::<usize, String>(5)? == "t";
            Ok(())
        })
        .unwrap();
        Ok(GameInfo {
            curr_ts,
            max_stamina,
            stamina_recover_tick,
            core_exp,
            level_steps,
            world_ranking_enabled,
            is_byd_chapter_unlocked,
        })
    }
}

// ----------------------------------------------------------------------------
#[derive(Serialize)]
struct MapReward {
    items: Vec<RewardItem>,
    position: isize,
}

#[derive(Serialize)]
struct RewardItem {
    #[serde(rename = "type", skip_serializing_if = "String::is_empty")]
    item_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(skip_serializing_if = "is_zero")]
    amount: i32,
}

#[derive(Serialize)]
struct MapInfo {
    affinity_multiplier: Vec<f64>,
    available_from: i64,
    available_to: i64,
    beyond_health: isize,
    character_affinity: Vec<i8>,
    chapter: isize,
    coordinate: String,
    curr_capture: isize,
    curr_position: isize,
    custom_bg: String,
    is_beyond: bool,
    is_legacy: bool,
    is_locked: bool,
    is_repeatable: bool,
    map_id: String,
    require_id: String,
    require_type: String,
    require_value: isize,
    stamina_cost: isize,
    step_count: isize,
    rewards: Vec<MapReward>,
}

impl MapInfo {
    fn get_map_affinity(&mut self, conn: &DBAccessManager) -> Result<(), rusqlite::Error> {
        let mut characters = Vec::new();
        let mut multiplier = Vec::new();
        let mut stmt = conn.connection.prepare(sql_stmt::MAP_AFFINITY).unwrap();
        let infoes = stmt
            .query_map(&[&self.map_id], |row| {
                Ok((row.get("part_id")?, row.get("multiplier")?))
            })
            .unwrap();

        for info in infoes {
            let info = info.unwrap();
            characters.push(info.0);
            multiplier.push(info.1);
        }

        self.character_affinity = characters;
        self.affinity_multiplier = multiplier;
        Ok(())
    }

    fn get_rewards(&mut self, conn: &DBAccessManager) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::MAP_REWARD).unwrap();
        let rewards = stmt
            .query_map(params![self.map_id], |row| {
                Ok((
                    row.get("position")?,
                    RewardItem {
                        id: row.get("reward_id")?,
                        item_type: row.get("item_type")?,
                        amount: row.get("amount")?,
                    },
                ))
            })
            .unwrap();
        for item in rewards {
            let item = item.unwrap();
            self.rewards.push(MapReward {
                items: vec![item.1],
                position: item.0,
            });
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct MapInfoList {
    user_id: isize,
    current_map: String,
    maps: Vec<MapInfo>,
}

impl MapInfoList {
    pub fn new(conn: &DBAccessManager, user_id: isize) -> Result<Self, rusqlite::Error> {
        let mut info_list = MapInfoList {
            user_id: user_id,
            current_map: String::new(),
            maps: Vec::new(),
        };

        let mut stmt = conn.connection.prepare(sql_stmt::MAP_INFO).unwrap();
        let map_infoes = stmt
            .query_map(&[&user_id], |row| {
                let map_id = row.get("map_id")?;
                Ok(MapInfo {
                    available_from: row.get("available_from")?,
                    available_to: row.get("available_to")?,
                    beyond_health: row.get("beyond_health")?,
                    chapter: row.get("chapter")?,
                    coordinate: row.get("coordinate")?,
                    custom_bg: row.get("custom_bg")?,
                    is_beyond: row.get::<&str, String>("is_beyond")? == "t",
                    is_legacy: row.get::<&str, String>("is_legacy")? == "t",
                    is_repeatable: row.get::<&str, String>("is_repeatable")? == "t",
                    map_id: map_id,
                    require_id: row.get("require_id")?,
                    require_type: row.get("require_type")?,
                    require_value: row.get("require_value")?,
                    stamina_cost: row.get("stamina_cost")?,
                    step_count: row.get("step_count")?,
                    curr_capture: row.get("curr_capture")?,
                    curr_position: row.get("curr_position")?,
                    is_locked: row.get::<&str, String>("is_locked")? == "t",
                    affinity_multiplier: Vec::new(),
                    character_affinity: Vec::new(),
                    rewards: Vec::new(),
                })
            })
            .unwrap();
        for map_info in map_infoes {
            let mut map_info = map_info.unwrap();
            map_info.get_map_affinity(conn)?;
            map_info.get_rewards(conn)?;
            info_list.maps.push(map_info);
        }

        Ok(info_list)
    }
}

