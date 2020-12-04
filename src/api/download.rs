use super::*;
use std::fmt;

pub struct DLRequest {
    pub need_url: bool,
    pub song_ids: Vec<String>,
}

impl DLRequest {
    pub fn empty_request() -> Self {
        DLRequest {
            need_url: false,
            song_ids: Vec::new(),
        }
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
                formatter.write_str("a query string specifying song ids and whether url is needed.")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DLRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut sids: Vec<String> = Vec::default();
                let mut need_url = false;
                while let Some(key) = map.next_key()? {
                    match key {
                        "sid" => sids.push(format!("'{}'", map.next_value::<String>()?)),
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

// GET /serve/download/me/song?url&sid
pub async fn get_download_list(
    hostname: String,
    requests: DLRequest,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let checksums = conn.get_purchase_dl(STATIC_USER_ID, requests, hostname);
    let result = ResponseContainer {
        success: true,
        value: checksums,
        error_code: 0,
    };
    respond(result, warp::http::StatusCode::OK)
}
