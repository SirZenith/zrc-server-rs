use rocket::State;
use strfmt::strfmt;

use super::*;

struct DLItem {
    song_id: String,
    audio_checksum: String,
    song_dl: bool,
    difficulty: String,
    chart_checksum: String,
    chart_dl: bool,
}

#[derive(Serialize)]
pub struct Checksum {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    audio: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    chart: HashMap<String, HashMap<String, String>>,
}

pub type ChecksumList = HashMap<String, Checksum>;

pub struct DLRequest {
    need_url: bool,
    song_ids: Vec<String>,
}

impl DLRequest {
    pub fn empty_request() -> Self {
        DLRequest {
            need_url: false,
            song_ids: Vec::new(),
        }
    }
}

impl<'f> FromForm<'f> for DLRequest {
    // In practice, we'd use a more descriptive error type.
    type Error = ();

    fn from_form(items: &mut FormItems<'f>, strict: bool) -> Result<Self, ()> {
        let mut need_url = true;
        let mut song_ids = Vec::new();

        for item in items {
            match item.key.as_str() {
                "url" => {
                    need_url = item.value.parse::<bool>().map_err(|_| ())?;
                }
                "sid" => {
                    let id = item.value.url_decode().map_err(|_| ())?;
                    song_ids.push(format!("'{}'", id));
                }
                _ if strict => return Err(()),
                _ => { /* allow extra value when not strict */ }
            }
        }

        Ok(DLRequest { need_url, song_ids })
    }
}

#[get("/serve/download/me/song?<requests..>")]
pub fn get_download_list(
    conn: ZrcDB,
    requests: LenientForm<DLRequest>,
    hostname: State<String>,
) -> Json<ResponseContainer<ChecksumList>> {
    let checksums = get_purchase_dl(
        &*conn,
        STATIC_USER_ID,
        requests.into_inner(),
        &hostname,
    );
    Json(ResponseContainer {
        success: true,
        value: checksums,
        error_code: 0,
    })
}

pub fn get_purchase_dl(
    conn: &rusqlite::Connection,
    user_id: isize,
    requests: DLRequest,
    hostname: &str,
) -> ChecksumList {
    let mut checksums = HashMap::new();
    let song_id_condition = if !requests.song_ids.is_empty() {
        format!("and song.song_id in ({})", requests.song_ids.join(", "))
    } else {
        String::new()
    };

    get_purchase_form_table(
        conn,
        user_id,
        &mut checksums,
        requests.need_url,
        sql_stmt::QUERY_DL,
        "pack_purchase_info as pur",
        "pur.pack_name = song.pack_name",
        &song_id_condition,
        hostname,
    );

    get_purchase_form_table(
        conn,
        user_id,
        &mut checksums,
        requests.need_url,
        sql_stmt::QUERY_DL,
        "single_purchase_info pur",
        "pur.song_id = song.song_id",
        &song_id_condition,
        hostname,
    );

    checksums
}

fn get_purchase_form_table(
    conn: &rusqlite::Connection,
    user_id: isize,
    checksums: &mut ChecksumList,
    need_url: bool,
    stmt: &str,
    table_name: &str,
    condition: &str,
    song_id_condition: &str,
    hostname: &str,
) {
    let mut var = HashMap::new();
    var.insert("table_name".to_string(), table_name);
    var.insert("query_condition".to_string(), condition);
    var.insert("song_id_condition".to_string(), song_id_condition);
    let mut stmt = conn.prepare(&strfmt(stmt, &var).unwrap()).unwrap();
    let items = stmt
        .query_map(&[&user_id], |row| DLItem {
            song_id: row.get::<&str, String>("song_id"),
            audio_checksum: row.get::<&str, String>("audio_checksum"),
            song_dl: row.get::<&str, String>("song_dl") == "t",
            difficulty: row.get::<&str, String>("difficulty"),
            chart_checksum: row.get::<&str, String>("chart_checksum"),
            chart_dl: row.get::<&str, String>("chart_dl") == "t",
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
                        hostname, FILE_SERVER_PREFIX, SONG_FILE_DIR, item.song_id, item.difficulty
                    ),
                );
            }
        }
    }
}
