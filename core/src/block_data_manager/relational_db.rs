
use dotenv::dotenv;
use chrono::NaiveDateTime;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::prelude::*;
use std::env;
use primitives::{Block, TransactionOutcome, BlockReceipts};
use cfx_types::{Address, H256};
use std::sync::Arc;

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
//===
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
//===
table! {
    logs (id) {
        id -> BigInt,
        tx_id -> BigInt,
        log_index -> Integer,
        address_id -> BigInt,
        topic0_id -> BigInt,
        topic1_id -> BigInt,
        topic2_id -> BigInt,
        topic3_id -> BigInt,
    }
}
#[derive(Queryable)]
pub struct LogPO {
    pub id: i64,
    pub tx_id: i64,
    pub log_index: i32,
    pub address_id: i64,
    pub topic0_id: i64,
    pub topic1_id: i64,
    pub topic2_id: i64,
    pub topic3_id: i64,
}
#[derive(Insertable)]
#[table_name="logs"]
pub struct NewLog {
    pub tx_id: i64,
    pub log_index: i32,
    pub address_id: i64,
    pub topic0_id: i64,
    pub topic1_id: i64,
    pub topic2_id: i64,
    pub topic3_id: i64,
}
//===
table! {
    txs (id) {
        id -> BigInt,
        hash -> Text,
        height -> BigInt,
        timestamp -> Timestamp,
        block_id -> BigInt,
        from_id -> BigInt,
        to_id -> BigInt,
        value -> Text,
        status -> Integer,
    }
}
#[derive(Insertable)]
#[table_name="txs"]
pub struct NewTx<'a, 'b> {
    pub hash: String,
    pub height: &'b i64,
    pub timestamp: &'a NaiveDateTime,

    pub block_id: &'a i64,
    pub from_id: i64,
    pub to_id: &'a i64,
    pub value: String,
    pub status: i32,
}
#[derive(Queryable)]
pub struct TxPO {
    pub id: i64,
    pub hash: String,
    pub height: i64,
    pub timestamp: NaiveDateTime,
    pub block_id: i64,
    pub from_id: i64,
    pub to_id: i64,
    pub value: String,
    pub status: i32,
}
pub fn find_tx(hash_: &String) -> Option<TxPO> {
    use self::txs::dsl::*;
    let conn = _POOL.get().unwrap();
    txs.filter(hash.eq(hash_))
        .get_result(&conn).optional().unwrap()
}
//==
table! {
    addresses (id) {
        id -> BigInt,
        hex -> Text,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct AddressPO {
    pub id: i64,
    pub hex: String,
    pub timestamp: NaiveDateTime,
}
#[derive(Insertable)]
#[table_name="addresses"]
pub struct NewAddress<'a> {
    pub hex: &'a str,
    pub timestamp: &'a NaiveDateTime,
}
//==
table! {
    bytes32s (id) {
        id -> BigInt,
        hex -> Text,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct Bytes32PO {
    pub id: i64,
    pub hex: String,
    pub timestamp: NaiveDateTime,
}
#[derive(Insertable)]
#[table_name="bytes32s"]
pub struct NewBytes32<'a> {
    pub hex: &'a str,
    pub timestamp: &'a NaiveDateTime,
}
//==
pub fn find_address(addr: &str) -> Option<AddressPO>{
    use self::addresses::dsl::*;
    let conn = _POOL.get().unwrap();
    addresses.filter(hex.eq(addr))
        .get_result(&conn).optional().unwrap()
}
pub fn find_byte32(hex_: &str) -> Option<Bytes32PO>{
    use self::bytes32s::dsl::*;
    let conn = _POOL.get().unwrap();
    bytes32s.filter(hex.eq(hex_))
        .get_result(&conn).optional().unwrap()
}
pub fn save_bytes32(bytes32: &H256, timestamp: &NaiveDateTime) -> Bytes32PO {
    let hex_str = &format!("{:#x}", bytes32);
    let bean = find_byte32(&hex_str);
    if bean.is_some() {
        bean.unwrap()
    } else {
        let new_addr = NewBytes32{
            hex: hex_str, timestamp
        };
        let conn = _POOL.get().unwrap();
        diesel::insert_into(bytes32s::table).values(&new_addr)
            .execute(&conn).unwrap();
        find_byte32(hex_str).unwrap()
    }
}
pub fn save_address(addr: &Address, timestamp: &NaiveDateTime) -> AddressPO {
    let addr_str = &format!( "{:#x}", addr);
    let bean = find_address(&addr_str);
    if bean.is_some() {
        bean.unwrap()
    } else {
        let new_addr = NewAddress{
            hex: addr_str, timestamp
        };
        let conn = _POOL.get().unwrap();
        diesel::insert_into(addresses::table).values(&new_addr)
            .execute(&conn).unwrap();
        find_address(addr_str).unwrap()
    }
}
// logs
pub fn insert_block_receipts(block: &Block, block_receipts: Arc<BlockReceipts>) {
    let receipts = &block_receipts.receipts;
    if receipts.is_empty() {
        return
    }
    let block_time = &build_block_timestamp(block.block_header.timestamp());
    let mut log_beans = vec![];
    for (idx, r) in receipts.iter().enumerate() {
        if r.outcome_status != TransactionOutcome::Failure && r.outcome_status != TransactionOutcome::Success {
            continue;
        }
        let tx = &block.transactions[idx];
        let tx_hash = format!( "{:#x}", tx.hash);
        let tx_id = find_tx(&tx_hash).unwrap().id;
        for (log_index, log) in r.logs.iter().enumerate() {
            let address_id = save_address(&log.address, block_time).id;
            let mut new_log = NewLog{
                tx_id, log_index: log_index as i32, address_id,
                topic0_id: 0, topic1_id: 0, topic2_id: 0, topic3_id: 0
            };
            let mut topic_ids = [0; 4];
            for (t_index, t) in log.topics.iter().enumerate() {
                topic_ids[t_index] = save_bytes32(&t, block_time).id;
            }
            new_log.topic0_id = topic_ids[0];
            new_log.topic1_id = topic_ids[1];
            new_log.topic2_id = topic_ids[2];
            new_log.topic3_id = topic_ids[3];
            log_beans.push(new_log);
        }
    }
    if log_beans.len() == 0 {
        return;
    }
    let conn = _POOL.get().unwrap();
    diesel::replace_into(logs::table)
        .values(&log_beans)
        .execute(&conn)
        .expect("Error saving new logs_arr");
}
// txs
pub fn remove_tx_relation(hash_: &H256) {
    use self::txs::dsl::*;
    let conn = _POOL.get().unwrap();
    let hash_0x = format!( "{:#x}", hash_);
    diesel::delete(txs.filter(hash.eq(hash_0x)))
        .execute(&conn).unwrap();
}
pub fn insert_block_tx_relation(block: &Block, tx_status: &Vec<TransactionOutcome>) {
    // txVec: &Vec<Arc<SignedTransaction>>
    if block.transactions.is_empty(){
        return
    }
    let height = block.block_header.height();
    let hash = format!( "{:#x}", block.block_header.hash());
    let block_po = query_block(&hash).unwrap();
    let mut tx_arr =  Vec::new();
    let block_time = &build_block_timestamp(block.block_header.timestamp());
    let i64height = &(height as i64);
    for (idx, tx) in block.transactions.iter().enumerate() {
        let status = tx_status[idx];
        if status != TransactionOutcome::Failure && status != TransactionOutcome::Success {
            continue;
        }
        let from_id = save_address(&tx.sender, block_time).id;
        let new_tx = NewTx{
            hash: (format!( "{:#x}", tx.hash) ),
            height: i64height,
            timestamp: block_time,

            block_id: &block_po.id,
            from_id,//: from_id,
            to_id: &0,//&save_address(&tx.sender, blockTime).id,
            value: tx.value().to_string(),
            status: status as i32,
        };
        if height == 0 {
            if find_tx(&new_tx.hash).is_some() {
                return
            }
        }
        tx_arr.push(new_tx);
    }
    let conn = _POOL.get().unwrap();
    diesel::replace_into(txs::table)
        .values(&tx_arr)
        .execute(&conn)
        .expect("Error saving new tx_arr");
}
// blocks
pub fn query_block(block_hash: &str) -> Option<BlockPO> {
    use self::blocks::dsl::*;
    let conn = _POOL.get().unwrap();
    blocks.filter(hash.eq(block_hash))
        .get_result(&conn).optional().unwrap()
}
pub fn insert_block_relation(block: &Block) {
    let conn = _POOL.get().unwrap();

    let height = block.block_header.height();
    let hash = format!( "{:#x}", block.block_header.hash());

    if height == 0 {
        let block_0 = query_block(&hash);
        if block_0.is_some() {
            return;
        }
    }
    let new_block = NewBlock{
        hash: &( hash ),
        height: &(height as i64),
        timestamp: &build_block_timestamp(block.block_header.timestamp()),
    };
    diesel::insert_into(blocks::table)
        .values(&new_block)
        .execute(&conn)
        .expect("Error saving new block");
    if height == 0 {
        insert_block_tx_relation(block, &vec![TransactionOutcome::Success; block.transactions.len()]);
    }
}

pub fn build_block_timestamp(mut timestamp: u64) -> NaiveDateTime {
    if timestamp == 0 {
        timestamp = 1;
    }
    NaiveDateTime::from_timestamp(timestamp as i64, 0)
}