use super::*;

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
