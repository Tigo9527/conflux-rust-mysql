
use dotenv::dotenv;
use chrono::NaiveDateTime;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::prelude::*;
use std::env;
// use primitives::block::BlockHeight;
// use cfx_types::H256;
use primitives::Block;

type DbCon = MysqlConnection;
lazy_static! {
    static ref _POOL: Pool<ConnectionManager<DbCon>> = pool();
}
pub fn pool() -> Pool<ConnectionManager<DbCon>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<DbCon>::new(database_url);
    Pool::builder().max_size(15).build(manager).unwrap()
}

table! {
    blocks (id) {
        id -> BigInt,
        hash -> Text,
        height -> BigInt,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct BlockPO {
    pub id: i64,
    pub hash: String,
    pub height: i64,
    pub timestamp: NaiveDateTime,
}
#[derive(Insertable)]
#[table_name="blocks"]
pub struct NewBlock<'a> {
    pub hash: &'a str,
    pub height: &'a i64,
    pub timestamp: &'a NaiveDateTime,
}
pub fn query_block(block_hash: &str) -> Option<BlockPO> {
    use self::blocks::dsl::*;
    let conn = _POOL.get().unwrap();
    blocks.filter(hash.eq(block_hash))
        .get_result(&conn).optional().unwrap()
        //.load::<BlockPO>(&conn)?//.expect("Error querying one block");
    // if list.length
}
pub fn insert_block(block: &Block) {
    let conn = _POOL.get().unwrap();

    let height = block.block_header.height();
    let hash = format!( "{:#x}", block.block_header.hash());

    if height == 0 {
        let block_0 = query_block(&hash);
        if block_0.is_some() {
            return;
        }
    }
    let mut timestamp = block.block_header.timestamp();
    if timestamp == 0 {
        timestamp = 1;
    }
    let new_block = NewBlock{
        hash: &( hash ),
        height: &(height as i64),
        timestamp: &NaiveDateTime::from_timestamp(timestamp as i64, 0),
    };
    diesel::insert_into(blocks::table)
        .values(&new_block)
        .execute(&conn)
        .expect("Error saving new block");
}