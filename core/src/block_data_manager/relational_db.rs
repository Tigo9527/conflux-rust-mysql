
use dotenv::dotenv;
use chrono::NaiveDateTime;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::prelude::{RunQueryDsl};
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

#[derive(Insertable)]
#[table_name="blocks"]
pub struct NewBlock<'a> {
    pub hash: &'a str,
    pub height: &'a i64,
    pub timestamp: &'a NaiveDateTime,
}

pub fn insert_block(block: &Block) {
    let mut timestamp = block.block_header.timestamp();
    if timestamp == 0 {
        timestamp = 1;
    }
    let hash = format!( "{:#x}", block.block_header.hash());
    let new_block = NewBlock{
        hash: &( hash ),
        height: &(block.block_header.height() as i64),
        timestamp: &NaiveDateTime::from_timestamp(timestamp as i64, 0),
    };
    let conn = _POOL.get().unwrap();
    diesel::insert_into(blocks::table)
        .values(&new_block)
        .execute(&conn)
        .expect("Error saving new block");
}