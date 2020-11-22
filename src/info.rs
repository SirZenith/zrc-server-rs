extern crate serde_json;

use super::*;

#[derive(Serialize)]
pub struct LoginToken {
    access_token: String,
    token_type: String,
    success: bool,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
}

#[derive(Serialize)]
pub struct LevelStep {
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
    pub fn new(conn: &rusqlite::Connection) -> Self {
        let mut stmt = conn.prepare(sql_stmt::LEVEL_STEP).unwrap();
        let mut level_steps = Vec::new();
        let steps = stmt.query_map(&[], |row| {
            LevelStep {
                level: row.get(0),
                level_exp: row.get(1),
            }
        })
        .unwrap();
        for step in steps {
            let step = step.unwrap();
            level_steps.push(step);
        }

        let (mut curr_ts, mut max_stamina, mut stamina_recover_tick) = (0, 0, 0);
        let (mut core_exp, mut world_ranking_enabled, mut is_byd_chapter_unlocked) =
            (250, false, false);
        let mut stmt = conn.prepare(sql_stmt::GAME_INFO).unwrap();
        stmt.query_row(&[], |row| {
            curr_ts = row.get(0);
            max_stamina = row.get(1);
            stamina_recover_tick = row.get(2);
            core_exp = row.get(3);
            world_ranking_enabled = row.get::<usize, String>(4) == "t";
            is_byd_chapter_unlocked = row.get::<usize, String>(5) == "t";
        })
        .unwrap();
        GameInfo {
            curr_ts,
            max_stamina,
            stamina_recover_tick,
            core_exp,
            level_steps,
            world_ranking_enabled,
            is_byd_chapter_unlocked,
        }
    }
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
    pub fn get_pack_list(conn: &rusqlite::Connection) -> Vec<Self> {
        let mut stmt = conn.prepare(sql_stmt::PACK_INFO).unwrap();
        let mut item_stmt = conn.prepare(sql_stmt::PACK_ITEM).unwrap();
        let packs = stmt
            .query_map(&[], |row| {
                let name = row.get(0);
                let price = row.get(1);
                let orig_price = row.get(2);
                let discount_from = row.get(3);
                let discount_to = row.get(4);

                let items = item_stmt
                    .query_map(&[&name], |row| PackItem {
                        id: row.get(0),
                        item_type: row.get(1),
                        is_available: row.get::<usize, String>(2) == "t",
                    })
                    .unwrap();

                PackInfo {
                    name,
                    items: items.into_iter().map(|x| x.unwrap()).collect(),
                    price,
                    orig_price,
                    discount_from,
                    discount_to,
                }
            })
            .unwrap();
        packs.into_iter().map(|x| x.unwrap()).collect()
    }
}

#[derive(Serialize)]
pub struct PackItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    is_available: bool,
}

#[derive(Deserialize)]
struct AggregateCall {
    endpoint: String,
    id: usize,
}

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
    fn new(conn: &rusqlite::Connection, user_id: isize) -> Self {
        let mut stmt = conn.prepare(sql_stmt::USER_INFO).unwrap();
        let world_unlocks = get_item_list(&*conn, "item_name", "world_unlock", user_id);
        let world_songs = get_item_list(&*conn, "item_name", "world_song_unlock", user_id);
        let packs = get_item_list(&*conn, "pack_name", "pack_purchase_info", user_id);
        let singles = get_item_list(&*conn, "song_id", "single_purchase_info", user_id);
        let mut user_info = stmt
            .query_row(&[&user_id], |row| {
                let settings = Setting {
                    max_stamina_notification_enabled: row
                        .get::<&str, String>("stamina_notification")
                        == "t",
                    is_hide_rating: row.get::<&str, String>("hide_rating") == "t",
                    favorite_character: row.get("fav_partner"),
                };
                let character_stats = crate::character::CharacterStatses::new(&*conn, None);
                let characters = character_stats.list_char_ids();
                UserInfo {
                    is_aprilfools: row.get::<&str, String>("is_aprilfools") == "t",
                    curr_available_maps: Vec::new(),
                    character_stats,
                    friends: Vec::new(),
                    settings,
                    user_id: user_id,
                    name: row.get("user_name"),
                    display_name: row.get("display_name"),
                    user_code: format!("{:0>9}", row.get::<&str, i64>("user_code")),
                    ticket: row.get("ticket"),
                    character: row.get("partner"),
                    is_locked_name_duplicate: row.get::<&str, String>("locked") == "t",
                    is_skill_sealed: row.get::<&str, String>("skill_sealed") == "t",
                    current_map: row.get("curr_map"),
                    prog_boost: row.get("prog_boost"),
                    next_fragstam_ts: row.get("next_fragstam_ts"),
                    max_stamina_ts: row.get("max_stamina_ts"),
                    stamina: row.get("stamina"),
                    world_unlocks,
                    world_songs,
                    singles,
                    packs,
                    characters,
                    cores: Vec::new(),
                    recent_score: Vec::new(),
                    max_friend: row.get("max_friend"),
                    rating: row.get("rating"),
                    join_date: row.get("join_date"),
                }
            })
            .unwrap();
        user_info.get_most_recent_score(conn);
        user_info
    }

    fn get_most_recent_score(&mut self, conn: &rusqlite::Connection) {
        let mut stmt = conn.prepare(sql_stmt::USER_MOST_RECENT_SCORE).unwrap();
        let score = stmt
            .query_row(&[&self.user_id], |row| MostRecentScore {
                song_id: row.get("song_id"),
                difficulty: row.get("difficulty"),
                score: row.get("score"),
                shiny: row.get("shiny_pure"),
                pure: row.get("pure"),
                far: row.get("far"),
                lost: row.get("lost"),
                health: row.get("health"),
                modifier: row.get("modifier"),
                time_played: row.get("played_date"),
                clear_type: row.get("clear_type"),
                best_clear_type: row.get("best_clear_type"),
            })
            .unwrap();
        self.recent_score.push(score);
    }
}

#[get("/auth/login")]
pub fn login() -> Json<LoginToken> {
    Json(LoginToken {
        access_token: String::new(),
        token_type: String::new(),
        success: true,
        error_code: 0,
    })
}

#[get("/compose/aggregate?<calls>")]
pub fn aggregate(conn: ZrcDB, calls: String) -> String {
    let endpoints: Vec<AggregateCall> = serde_json::from_str(&calls).unwrap();
    let mut results = Vec::new();
    for call in endpoints {
        let content = match call.endpoint.as_str() {
            "/user/me" => serde_json::to_string(&UserInfo::new(&*conn, STATIC_USER_ID)).unwrap(),
            "/purchase/bundle/pack" => {
                serde_json::to_string(&PackInfo::get_pack_list(&*conn)).unwrap()
            }
            "/serve/download/me/song?url=false" => {
                serde_json::to_string(&crate::download::get_purchase_dl(
                    &*conn,
                    STATIC_USER_ID,
                    crate::download::DLRequest::empty_request(),
                    "",
                ))
                .unwrap()
            }
            "/game/info" => serde_json::to_string(&GameInfo::new(&*conn)).unwrap(),
            "/world/map/me" => serde_json::to_string(&crate::world::MapInfoList::new(&*conn, STATIC_USER_ID)).unwrap(),
            _ => "[]".to_string(),
        };
        results.push(format!(r#"{{"id":{},"value":{}}}"#, call.id, content));
    }
    format!(r#"{{"success": true,"value": [{}]}}"#, results.join(","))
}

#[get("/game/info")]
pub fn game_info(conn: ZrcDB) -> Json<GameInfo> {
    Json(GameInfo::new(&*conn))
}

#[get("/purchase/bundle/pack")]
pub fn pack_info(conn: ZrcDB) -> Json<Vec<PackInfo>> {
    Json(PackInfo::get_pack_list(&*conn))
}

#[get("/present/me")]
pub fn present_me() -> Json<Vec<()>> {
    Json(Vec::new())
}

#[get("/user/me")]
pub fn user_info(conn: ZrcDB) -> Json<UserInfo> {
    Json(UserInfo::new(&*conn, STATIC_USER_ID))
}

fn get_item_list(
    conn: &rusqlite::Connection,
    column: &str,
    table: &str,
    user_id: isize,
) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!(
            "select {} from {} where user_id = {}",
            column, table, user_id
        ))
        .unwrap();
    let items = stmt.query_map(&[], |row| row.get(0)).unwrap();

    items.into_iter().map(|x| x.unwrap()).collect()
}
