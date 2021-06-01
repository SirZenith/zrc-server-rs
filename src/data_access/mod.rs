use thiserror::Error;

mod info;
pub mod save;
mod score;
mod sql_stmt;

mod dlc {
    use super::*;
    use std::fmt;

    pub enum ItemType {
        Pack,
        Single,
    }
    /// A download request pass to `DBAccessManager` to look up info like cheksum
    /// and download URL for a downloadable song or downloadable chart of a song.
    pub struct DLRequest {
        pub need_url: bool,
        /// A list of song id, such as `vec!["ifi", "onefr", "fractureray"]`.
        pub song_ids: Vec<String>,
    }

    impl DLRequest {
        /// Create a download request with no song id condition.
        pub fn empty_request() -> Self {
            DLRequest {
                need_url: false,
                song_ids: Vec::new(),
            }
        }

        pub fn with_id_list(song_ids: Vec<String>, need_url: bool) -> Self {
            DLRequest { need_url, song_ids }
        }
    }

    impl<'de> Deserialize<'de> for DLRequest {
        fn deserialize<D>(deserializer: D) -> Result<DLRequest, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                type Value = DLRequest;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter
                        .write_str("a query string specifying song ids and whether url is needed.")
                }

                fn visit_map<V>(self, mut map: V) -> Result<DLRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
                {
                    let mut sids: Vec<String> = Vec::default();
                    let mut need_url = false;
                    while let Some(key) = map.next_key()? {
                        match key {
                            "sid" => sids.push(format!("{}", map.next_value::<String>()?)),
                            "url" => need_url = map.next_value::<bool>()?,
                            _ => unreachable!(),
                        }
                    }
                    Ok(DLRequest {
                        need_url: need_url,
                        song_ids: sids,
                    })
                }
            }
            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    // DLC info item return from database.
    pub struct DLItem {
        pub song_id: String,
        pub audio_checksum: String,
        pub song_dl: bool,
        pub difficulty: String,
        pub chart_checksum: String,
        pub chart_dl: bool,
    }

    impl DLItem {
        pub fn song_dl_url(
            &self,
            hostname: &str,
            prefix_static_file: &str,
            songs_dirname: &str,
        ) -> String {
            format!(
                "http://{}/{}/{}/{}/{}",
                hostname, prefix_static_file, songs_dirname, self.song_id, "base.ogg"
            )
        }

        pub fn chart_dl_url(
            &self,
            hostname: &str,
            prefix_static_file: &str,
            songs_dirname: &str,
        ) -> String {
            format!(
                "http://{}/{}/{}/{}/{}.aff",
                hostname, prefix_static_file, songs_dirname, self.song_id, self.difficulty
            )
        }
    }

    // A container for checksum and download URL.
    #[derive(Serialize, Debug)]
    pub struct InfoItem {
        #[serde(skip_serializing_if = "String::is_empty")]
        pub checksum: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub url: String,
    }

    impl InfoItem {
        pub fn new() -> Self {
            InfoItem {
                checksum: String::new(),
                url: String::new(),
            }
        }

        fn is_empty(&self) -> bool {
            self.checksum.is_empty() && self.url.is_empty()
        }
    }

    /// Checksum and download URL (if needed) for a single song.
    #[derive(Serialize, Debug)]
    pub struct DlcInfo {
        #[serde(skip_serializing_if = "InfoItem::is_empty")]
        pub audio: InfoItem,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        pub chart: HashMap<String, InfoItem>,
    }

    pub type DlcInfoList = HashMap<String, DlcInfo>;
}

mod character {
    use super::*;

    const VOICE: [isize; 7] = [0, 1, 2, 3, 100, 1000, 1001];

    #[derive(Serialize)]
    struct CharacterStats {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        voice: Vec<isize>,
        is_uncapped_override: bool,
        is_uncapped: bool,
        uncap_cores: Vec<String>,
        char_type: i8,
        skill_id_uncap: Option<String>,
        skill_requires_uncap: bool,
        skill_unlock_level: i8,
        skill_id: Option<String>,
        overdrive: f64,
        prog: f64,
        frag: f64,
        level_exp: isize,
        exp: f64,
        level: i8,
        name: String,
        character_id: i8,
        #[serde(skip_serializing_if = "is_zero")]
        prog_tempest: f64,
    }

    #[derive(Serialize)]
    pub struct CharacterStatses(Vec<CharacterStats>);

    impl CharacterStatses {
        pub fn new(
            conn: &DBAccessManager,
            user_id: isize,
            part_id: Option<isize>,
        ) -> Result<CharacterStatses, rusqlite::Error> {
            let cond = match part_id {
                Some(id) => format!("{}{}", sql_stmt::COND_SINGLE_CHAR_STATS, id),
                None => sql_stmt::COND_ALL_CHAR_STATS.to_string(),
            };
            let mut stmt = conn
                .connection
                .prepare(&format!("{}{};", sql_stmt::CHAR_STATS, cond))
                .unwrap();

            // TODO: Possible error point
            let statses = stmt
                .query_map(params![user_id], |row| {
                    Ok(CharacterStats {
                        voice: if row.get::<&str, isize>("have_voice").unwrap() >= 0 {
                            VOICE.iter().map(|x| *x).collect()
                        } else {
                            Vec::new()
                        },
                        is_uncapped_override: row.get::<&str, String>("uncapped_override")? == "t",
                        is_uncapped: row.get::<&str, String>("uncapped")? == "t",
                        uncap_cores: Vec::new(),
                        char_type: row.get("char_type")?,
                        skill_id_uncap: row.get("uncap_skill")?,
                        skill_requires_uncap: row.get::<&str, String>("skill_requires_uncap")?
                            == "t",
                        skill_unlock_level: row.get("skill_unlock_level")?,
                        skill_id: row.get("skill_id")?,
                        overdrive: row.get("overdrive")?,
                        prog: row.get("prog")?,
                        frag: row.get("frag")?,
                        level_exp: row.get("level_exp")?,
                        exp: row.get("exp_val")?,
                        level: row.get("lv")?,
                        name: row.get("part_name")?,
                        character_id: row.get("part_id")?,
                        prog_tempest: row.get("prog_tempest")?,
                    })
                })
                .unwrap();
            Ok(CharacterStatses(
                statses.into_iter().map(|s| s.unwrap()).collect(),
            ))
        }

        pub fn list_char_ids(&self) -> Vec<i8> {
            self.0.iter().map(|x| x.character_id).collect()
        }
    }
}

use super::*;
pub use character::CharacterStatses;
use dlc::{DLItem, DlcInfo, DlcInfoList, InfoItem};
pub use dlc::{DLRequest, ItemType};
use info::{
    GameInfo, MapInfoList, PackInfo, PackItem, UserInfo, UserInfoForItemPurchase,
    UserInfoForScoreLookup,
};
pub use score::{LookupedScore, ScoreRecord};

pub type SqlitePool = Arc<Pool<SqliteConnectionManager>>;
pub type PooledSqlite = PooledConnection<SqliteConnectionManager>;
#[derive(Error, Debug)]
pub enum ZrcDBError {
    #[error("No data found - {0}")]
    DataNotFound(String),
    // Error that caused by sql stataments in modules
    #[error("internal error, context - {0} || error: {1}")]
    Internal(String, rusqlite::Error),
    #[error("other error, context - {0}")]
    Other(String),
    #[error("user with the same name already exists")]
    UserNameExists,
    #[error("this email is already used")]
    EmailExists,
}

impl warp::reject::Reject for ZrcDBError {}

type ZrcDBResult<T> = Result<T, ZrcDBError>;

pub fn with_db_access_manager(
    pool: SqlitePool,
) -> impl Filter<Extract = (DBAccessManager,), Error = warp::Rejection> + Clone {
    warp::any()
        .map(move || pool.clone())
        .and_then(|pool: SqlitePool| async move {
            match pool.get() {
                Ok(conn) => Ok(DBAccessManager::new(conn)),
                Err(_) => Err(warp::reject()),
            }
        })
}

pub struct DBAccessManager {
    connection: PooledSqlite,
}


impl DBAccessManager {
    pub fn new(connection: PooledSqlite) -> DBAccessManager {
        DBAccessManager { connection }
    }

    pub fn map_err(msg: &str, err: Option<rusqlite::Error>) -> ZrcDBError {
        let msg = msg.to_string();
        match err {
            None => ZrcDBError::Other(msg),
            Some(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => ZrcDBError::DataNotFound(msg),
                _ => ZrcDBError::Internal(msg, e),
            },
        }
    }
}

// ----------------------------------------------------------------------------
/// DLC service
impl DBAccessManager {
    /// Conditionally lookup data for downloadable songs. Conditions is passed
    /// by a `DLRequest` object.
    /// If song id list contained in `DLRequest` is not empty, returned result
    /// will be info for song in list, other wise, all DLC info will be contained
    /// in returned result.
    pub fn get_purchase_dl(
        &self,
        user_id: isize,
        requests: DLRequest,
        hostname: &str,
        prefix_static_file: &str,
        songs_dirname: &str,
    ) -> ZrcDBResult<dlc::DlcInfoList> {
        let mut infoes = HashMap::new();
        let song_id_condition = if !requests.song_ids.is_empty() {
            format!(
                "and song.song_id in ({})",
                requests
                    .song_ids
                    .iter()
                    .map(|id| format!("'{}'", id))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let table_name = "pack_purchase_info as pur";
        let condition = "pur.pack_name = song.pack_name";
        self.get_purchase_form_table(
            user_id,
            &mut infoes,
            requests.need_url,
            sql_stmt::QUERY_DL,
            table_name,
            condition,
            &song_id_condition,
            hostname,
            prefix_static_file,
            songs_dirname,
        )
        .map_err(|e| {
            DBAccessManager::map_err(
                &format!(
                    "while query purchase data from table '{}' with condition '{}'",
                    table_name, condition
                ),
                Some(e),
            )
        })?;

        let table_name = "single_purchase_info pur";
        let condition = "pur.song_id = song.song_id";
        self.get_purchase_form_table(
            user_id,
            &mut infoes,
            requests.need_url,
            sql_stmt::QUERY_DL,
            table_name,
            condition,
            &song_id_condition,
            hostname,
            prefix_static_file,
            songs_dirname,
        )
        .map_err(|e| {
            DBAccessManager::map_err(
                &format!(
                    "while query purchase data from table '{}' with condition '{}'",
                    table_name, condition
                ),
                Some(e),
            )
        })?;

        Ok(infoes)
    }

    /// Return all DLC info (checksum only).
    pub fn get_all_purchase_dl(&self, user_id: isize) -> ZrcDBResult<dlc::DlcInfoList> {
        self.get_purchase_dl(user_id, DLRequest::empty_request(), "", "", "")
    }

    // Look up checksum and download URL for DLC with given table name and condition.
    fn get_purchase_form_table(
        &self,
        user_id: isize,
        infoes: &mut DlcInfoList,
        need_url: bool,
        stmt: &str,
        table_name: &str,
        condition: &str,
        song_id_condition: &str,
        hostname: &str,
        prefix_static_file: &str,
        songs_dirname: &str,
    ) -> Result<(), rusqlite::Error> {
        let items = self.get_dl_items(user_id, stmt, table_name, condition, song_id_condition)?;
        for item in items.into_iter().filter(|i| i.chart_dl || i.song_dl) {
            let info = infoes.entry(item.song_id.clone()).or_insert(DlcInfo {
                audio: InfoItem::new(),
                chart: HashMap::new(),
            });
            if item.song_dl && !item.audio_checksum.is_empty() {
                info.audio.checksum = item.audio_checksum.clone();
                if need_url {
                    info.audio.url = item.song_dl_url(hostname, prefix_static_file, songs_dirname);
                }
            }
            if item.chart_dl && !item.chart_checksum.is_empty() {
                let entry = info
                    .chart
                    .entry(item.difficulty.clone())
                    .or_insert(InfoItem::new());
                entry.checksum = item.chart_checksum.clone();
                if need_url {
                    entry.url = item.chart_dl_url(hostname, prefix_static_file, songs_dirname);
                }
            }
        }
        Ok(())
    }

    fn get_dl_items(
        &self,
        user_id: isize,
        stmt: &str,
        table_name: &str,
        condition: &str,
        song_id_condition: &str,
    ) -> Result<Vec<dlc::DLItem>, rusqlite::Error> {
        use strfmt::strfmt;

        let mut var = HashMap::new();
        var.insert("table_name".to_string(), table_name);
        var.insert("query_condition".to_string(), condition);
        var.insert("song_id_condition".to_string(), song_id_condition);
        // TODO: Possible error point.
        let mut stmt = self.connection.prepare(&strfmt(stmt, &var).unwrap())?;
        let items = stmt.query_map(&[&user_id], |row| {
            Ok(DLItem {
                song_id: row.get::<&str, String>("song_id")?,
                audio_checksum: row.get::<&str, String>("audio_checksum")?,
                song_dl: row.get::<&str, String>("song_dl")? == "t",
                difficulty: row.get::<&str, String>("difficulty")?,
                chart_checksum: row.get::<&str, String>("chart_checksum")?,
                chart_dl: row.get::<&str, String>("chart_dl")? == "t",
            })
        })?;
        Ok(items.map(|i| i.unwrap()).collect())
    }

    pub fn purchase_item(
        &mut self,
        user_id: isize,
        item_id: &str,
        item_type: ItemType,
    ) -> ZrcDBResult<UserInfoForItemPurchase> {
        let tx = self.connection.transaction().map_err(|e| {
            DBAccessManager::map_err("while opening transacation for pack purchasing", Some(e))
        })?;
        {
            let stmt = match item_type {
                ItemType::Pack => sql_stmt::PURCHASE_PACK,
                ItemType::Single => sql_stmt::PURCHASE_SINGLE,
            };
            // TODO: check user possession, prevent repeatedly purchase
            // TODO: modify user ticket
            let mut stmt = tx.prepare(stmt).map_err(|e| {
                DBAccessManager::map_err("while preparing statement for pack purchasing", Some(e))
            })?;
            stmt.execute(params![user_id, item_id]).map_err(|e| {
                DBAccessManager::map_err(
                    &format!("while purchasing '{}' for user id '{}'", item_id, user_id),
                    Some(e),
                )
            })?;
        }
        tx.commit().map_err(|e| DBAccessManager::map_err("while commit purchase", Some(e)))?;
        match UserInfoForItemPurchase::new(self, user_id) {
            Ok(info) => Ok(info),
            Err(e) => Err(DBAccessManager::map_err(
                "while generate user info after pack purchasing",
                Some(e),
            )),
        }
    }
}

// ----------------------------------------------------------------------------
/// Character management.
impl DBAccessManager {
    pub fn change_character(
        &self,
        user_id: isize,
        char_id: isize,
        skill_sealed: bool,
    ) -> Result<usize, rusqlite::Error> {
        let skill_sealed = if skill_sealed { "t" } else { "f" };
        self.connection.execute(
            sql_stmt::CHANGE_CHARACTER,
            params![char_id, skill_sealed, user_id],
        )
    }

    pub fn toggle_uncap(
        &self,
        user_id: isize,
        part_id: isize,
    ) -> Result<CharacterStatses, rusqlite::Error> {
        let mut stmt = self.connection.prepare(sql_stmt::TOGGLE_UNCAP).unwrap();
        stmt.execute(params![user_id, part_id]).unwrap();
        let stats = self.get_char_statses(user_id, Some(part_id));
        stats
    }

    pub fn get_char_statses(
        &self,
        user_id: isize,
        part_id: Option<isize>,
    ) -> Result<CharacterStatses, rusqlite::Error> {
        Ok(CharacterStatses::new(&self, user_id, part_id)?)
    }
}

// ----------------------------------------------------------------------------
/// Getting and setting basic info for user log in.
impl DBAccessManager {
    fn is_user_exists(
        tx: &rusqlite::Transaction,
        user_name: &str,
        email: &str,
    ) -> Result<(), ZrcDBError> {
        match tx.query_row(sql_stmt::CHECK_USER_NAME_EXISTS, [user_name], |row| {
            Ok(row.get::<usize, usize>(0)?)
        }) {
            Ok(_) => return Err(ZrcDBError::UserNameExists),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {}
                _ => {
                    return Err(ZrcDBError::Internal(
                        "while check existance of user name".to_string(),
                        e,
                    ))
                }
            },
        }
        match tx.query_row(sql_stmt::CHECK_EMAIL_EXISTS, [email], |row| {
            Ok(row.get::<usize, usize>(0)?)
        }) {
            Ok(_) => return Err(ZrcDBError::EmailExists),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {}
                _ => {
                    return Err(ZrcDBError::Internal(
                        "while check existance of email".to_string(),
                        e,
                    ))
                }
            },
        }
        Ok(())
    }

    pub fn signup(
        &mut self,
        user_name: &str,
        pwd_hash: &str,
        email: &str,
        device_id: &str,
    ) -> ZrcDBResult<isize> {
        use rand::{thread_rng, Rng};

        let tx = self.connection.transaction().map_err(|e| {
            DBAccessManager::map_err("failed to make transication for signing up", Some(e))
        })?;

        let user_id = tx
            .query_row(sql_stmt::GET_NEW_USER_ID, [], |row| Ok(row.get("user_id")?))
            .map_err(|e| DBAccessManager::map_err("while getting new user_id", Some(e)))?;

        let mut rng = thread_rng();
        let mut user_code: u32 = rng.gen_range(0..=999_999_999);
        loop {
            match tx.query_row(sql_stmt::CHECK_USER_CODE_EXISTS, [user_code], |row| {
                Ok(row.get::<usize, usize>(0)?)
            }) {
                Ok(_) => user_code = (user_code + 1) % 1_000_000_000,
                Err(e) => match e {
                    rusqlite::Error::QueryReturnedNoRows => break,
                    _ => {
                        return Err(DBAccessManager::map_err(
                            "while generating user code",
                            Some(e),
                        ))
                    }
                },
            }
        }

        DBAccessManager::is_user_exists(&tx, user_name, email)?;

        tx.execute(
            sql_stmt::SING_UP,
            params![user_id, device_id, email, pwd_hash, user_name, user_code, user_name],
        )
        .map_err(|e| DBAccessManager::map_err("while signing up", Some(e)))?;

        tx.commit()
            .map_err(|e| DBAccessManager::map_err("while commit sing up data", Some(e)))?;

        Ok(user_id)
    }

    pub fn login(&self, name: &str, pwd_hash: &str) -> ZrcDBResult<isize> {
        self.connection
            .query_row(sql_stmt::LOGIN, params![name, pwd_hash], |row| {
                Ok(row.get("user_id")?)
            })
            .map_err(|e| DBAccessManager::map_err("while querying login id", Some(e)))
    }

    pub fn get_user_info(&self, user_id: isize) -> ZrcDBResult<UserInfo> {
        UserInfo::new(&self, user_id).map_err(|e| {
            DBAccessManager::map_err(
                &format!("while querying user info for user id '{}'", user_id),
                Some(e),
            )
        })
    }

    pub fn get_minimum_user_info(&self, user_id: isize) -> ZrcDBResult<UserInfoForScoreLookup> {
        UserInfoForScoreLookup::new(&self, user_id)
            .map_err(|e| DBAccessManager::map_err("while querying minimum user info", Some(e)))
    }

    pub fn get_game_info(&self) -> ZrcDBResult<GameInfo> {
        GameInfo::new(&self)
            .map_err(|e| DBAccessManager::map_err("while querying game info", Some(e)))
    }

    pub fn get_single_info(&self) -> ZrcDBResult<Vec<PackInfo>> {
        let mut stmt = self
            .connection
            .prepare(sql_stmt::GET_SINGLE_LIST)
            .map_err(|e| {
                DBAccessManager::map_err(
                    "while opening trasacation for getting single purchase list",
                    Some(e),
                )
            })?;
        let infoes = stmt
            .query_map([], |row| {
                let song_id: String = row.get("song_id")?;
                Ok(PackInfo {
                    name: song_id.clone(),
                    items: vec![PackItem {
                        id: song_id,
                        item_type: "single".to_string(),
                        is_available: true,
                    }],
                    orig_price: 0,
                    price: 0,
                    discount_from: 1491868801000,
                    discount_to: 1491868801000,
                })
            })
            .map_err(|e| {
                DBAccessManager::map_err("while querying single purchase list", Some(e))
            })?;
        Ok(infoes.map(|i| i.unwrap()).collect())
    }

    pub fn get_pack_info(&self) -> ZrcDBResult<Vec<PackInfo>> {
        PackInfo::get_pack_list(&self)
            .map_err(|e| DBAccessManager::map_err("while querying pack info", Some(e)))
    }

    pub fn get_map_info(&self, user_id: isize) -> ZrcDBResult<MapInfoList> {
        MapInfoList::new(&self, user_id)
            .map_err(|e| DBAccessManager::map_err("while querying map info", Some(e)))
    }

    pub fn set_favorite_character(&self, user_id: isize, char_id: isize) -> ZrcDBResult<usize> {
        self.connection
            .execute(sql_stmt::SET_FAVORITE_CHARACTER, params![char_id, user_id])
            .map_err(|e| DBAccessManager::map_err("while querying map info", Some(e)))
    }

    pub fn set_user_setting(
        &self,
        user_id: isize,
        option_name: String,
        value: bool,
    ) -> ZrcDBResult<usize> {
        use strfmt::strfmt;

        let mut var = HashMap::new();
        var.insert("option_name".to_string(), option_name);
        let value = if value { "t" } else { "" };
        let stmt = &strfmt(sql_stmt::SET_USER_SETTING, &var).map_err(|e| {
            DBAccessManager::map_err(
                &format!(
                    "while preparing statement '{}': {}",
                    sql_stmt::SET_USER_SETTING,
                    e
                ),
                None,
            )
        })?;
        self.connection
            .execute(stmt, params![value, user_id])
            .map_err(|e| DBAccessManager::map_err("while set user setting", Some(e)))
    }
}

// ----------------------------------------------------------------------------
/// Score upload and lookup service.
impl DBAccessManager {
    /// Generate token for score upload around the corner.
    pub fn gen_score_token(&self) -> String {
        "nothing".to_string()
    }

    /// Insert a score record into database.
    pub fn score_upload(
        &mut self,
        score: &ScoreRecord,
        user_id: isize,
        time: Option<&i64>,
    ) -> Result<HashMap<String, isize>, rusqlite::Error> {
        score::score_upload(self, score, user_id, time)
    }

    pub fn get_best_scores_with_iden(
        &self,
        user_id: isize,
    ) -> Result<HashMap<String, isize>, rusqlite::Error> {
        score::get_best_scores_with_iden(self, user_id)
    }

    pub fn get_all_best_scores(
        &self,
        user_id: isize,
    ) -> Result<Vec<(ScoreRecord, i64)>, rusqlite::Error> {
        score::get_all_best_scores(self, user_id)
    }

    pub fn score_lookup(&self, user_id: isize) -> Result<Vec<LookupedScore>, rusqlite::Error> {
        score::score_lookup(self, user_id)
    }

    pub fn get_r10_and_b30(&self, user_id: isize) -> ZrcDBResult<(f64, f64)> {
        self._get_r10_and_b30(user_id)
            .map_err(|e| DBAccessManager::map_err("while querying r10 and b30", Some(e)))
    }

    fn _get_r10_and_b30(&self, user_id: isize) -> Result<(f64, f64), rusqlite::Error> {
        let mut stmt = self.connection.prepare(sql_stmt::COMPUTE_R10_AND_B30)?;
        stmt.query_row(params![user_id], |row| {
            Ok((row.get::<&str, f64>("r10")?, row.get::<&str, f64>("b30")?))
        })
    }
}
