pub mod character;
pub mod download;
mod game_info;
mod pack_info;
mod user;
mod world;
mod sql_stmt;
pub mod score;

use super::*;
use character::*;
use download::*;
use game_info::*;
use pack_info::*;
use user::*;
use world::*;
use score::*;

pub type SqlitePool = Arc<Pool<SqliteConnectionManager>>;
pub type PooledSqlite = PooledConnection<SqliteConnectionManager>;

pub struct DBAccessManager {
    connection: PooledSqlite,
}

impl DBAccessManager {
    pub fn new(connection: PooledSqlite) -> DBAccessManager {
        DBAccessManager { connection }
    }

    pub fn get_purchase_dl(
        &self,
        user_id: isize,
        requests: api::download::DLRequest,
        hostname: String,
    ) -> ChecksumList {
        let mut checksums = HashMap::new();
        let song_id_condition = if !requests.song_ids.is_empty() {
            format!("and song.song_id in ({})", requests.song_ids.join(", "))
        } else {
            String::new()
        };
        self.get_purchase_form_table(
            user_id,
            &mut checksums,
            requests.need_url,
            sql_stmt::QUERY_DL,
            "pack_purchase_info as pur",
            "pur.pack_name = song.pack_name",
            &song_id_condition,
            &hostname,
        );
        self.get_purchase_form_table(
            user_id,
            &mut checksums,
            requests.need_url,
            sql_stmt::QUERY_DL,
            "single_purchase_info pur",
            "pur.song_id = song.song_id",
            &song_id_condition,
            &hostname,
        );
        checksums
    }

    pub fn get_all_purchase_dl(&self, user_id: isize) -> ChecksumList {
        self.get_purchase_dl(
            user_id,
            api::download::DLRequest::empty_request(),
            String::new(),
        )
    }

    fn get_purchase_form_table(
        &self,
        user_id: isize,
        checksums: &mut ChecksumList,
        need_url: bool,
        stmt: &str,
        table_name: &str,
        condition: &str,
        song_id_condition: &str,
        hostname: &str,
    ) {
        use strfmt::strfmt;

        let mut var = HashMap::new();
        var.insert("table_name".to_string(), table_name);
        var.insert("query_condition".to_string(), condition);
        var.insert("song_id_condition".to_string(), song_id_condition);
        let mut stmt = self
            .connection
            .prepare(&strfmt(stmt, &var).unwrap())
            .unwrap();
        let items = stmt
            .query_map(&[&user_id], |row| {
                Ok(DLItem {
                    song_id: row.get::<&str, String>("song_id")?,
                    audio_checksum: row.get::<&str, String>("audio_checksum")?,
                    song_dl: row.get::<&str, String>("song_dl")? == "t",
                    difficulty: row.get::<&str, String>("difficulty")?,
                    chart_checksum: row.get::<&str, String>("chart_checksum")?,
                    chart_dl: row.get::<&str, String>("chart_dl")? == "t",
                })
            })
            .unwrap();
        for item in items
            .map(|i| i.unwrap())
            .filter(|i| i.chart_dl || i.song_dl)
        {
            let checksum = checksums.entry(item.song_id.clone()).or_insert(Checksum {
                audio: HashMap::new(),
                chart: HashMap::new(),
            });
            if item.song_dl {
                checksum
                    .audio
                    .insert("checksum".to_string(), item.audio_checksum);
                if need_url {
                    checksum.audio.insert(
                        "url".to_string(),
                        format!(
                            "http://{}{}/{}/{}/{}",
                            hostname, FILE_SERVER_PREFIX, SONG_FILE_DIR, item.song_id, "base.ogg"
                        ),
                    );
                }
            }
            if item.chart_dl {
                let entry = checksum
                    .chart
                    .entry(item.difficulty.clone())
                    .or_insert(HashMap::new());
                entry.insert("checksum".to_string(), item.chart_checksum);
                if need_url {
                    entry.insert(
                        "url".to_string(),
                        format!(
                            "http://{}{}/{}/{}/{}.aff",
                            hostname,
                            FILE_SERVER_PREFIX,
                            SONG_FILE_DIR,
                            item.song_id,
                            item.difficulty
                        ),
                    );
                }
            }
        }
    }

    pub fn change_character(
        &self,
        user_id: isize,
        char_id: isize,
        skill_sealed: bool,
    ) -> Result<usize, rusqlite::Error> {
        self.connection.execute(
            sql_stmt::CHANGE_CHARACTER,
            params![char_id, skill_sealed, user_id],
        )
    }

    pub fn toggle_uncap(&self, user_id: isize, part_id: isize) -> Result<CharacterStatses, rusqlite::Error> {
        let mut stmt = self.connection.prepare(sql_stmt::TOGGLE_UNCAP).unwrap();
        stmt.execute(params![STATIC_USER_ID, part_id]).unwrap();
        let stats = self.get_char_statuses(user_id, Some(part_id));
        stats
    }

    pub fn get_char_statuses(
        &self,
        user_id: isize,
        part_id: Option<isize>,
    ) -> Result<CharacterStatses, rusqlite::Error> {
        Ok(CharacterStatses::new(&self, user_id, part_id)?)
    }

    pub fn get_user_info(&self, user_id: isize) -> Result<UserInfo, rusqlite::Error> {
        UserInfo::new(&self, user_id)
    }

    pub fn get_game_info(&self) -> Result<GameInfo, rusqlite::Error> {
        GameInfo::new(&self)
    }

    pub fn get_pack_info(&self) -> Result<Vec<PackInfo>, rusqlite::Error> {
        PackInfo::get_pack_list(&self)
    }

    pub fn get_map_info(&self, user_id: isize) -> Result<MapInfoList, rusqlite::Error> {
        Ok(MapInfoList::new(&self, user_id)?)
    }

    pub fn gen_score_token(&self) -> String {
        "nothing".to_string()
    }

    pub fn score_upload(&mut self, score: &ScoreRecord, user_id: isize, time: Option<&i64>) -> Result<ResponseContainer<HashMap<String, isize>>, rusqlite::Error> {
        score::score_upload(self, score, user_id, time)
    }

    pub fn get_best_scores_with_iden(&self, user_id: isize) -> Result<HashMap<String, isize>, rusqlite::Error> {
        score::get_best_scores_with_iden(self, user_id)
    }

    pub fn get_all_best_for_backup(&self, user_id: isize) -> Result<Vec<(ScoreRecord, i64)>, rusqlite::Error> {
        score::get_all_best_for_backup(self, user_id)
    }
}

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
