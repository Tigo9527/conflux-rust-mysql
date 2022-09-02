
use dotenv::dotenv;
use chrono::NaiveDateTime;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::prelude::*;
use std::env;
use primitives::{Block, TransactionOutcome, BlockReceipts};
use cfx_types::{Address, H256};
use std::sync::Arc;
use primitives::transaction::Action::{Call, Create};
use std::sync::Mutex;

type DbCon = MysqlConnection;
lazy_static! {
    static ref _POOL: Pool<ConnectionManager<DbCon>> = pool();
    static ref PREVIOUS_SAVED_EPOCH: Mutex<u64> = Mutex::new(find_config_u64(EPOCH_CONFIG).unwrap());
}
pub fn pool() -> Pool<ConnectionManager<DbCon>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<DbCon>::new(database_url);
    Pool::builder().max_size(15).build(manager).unwrap()
}
//===
const EPOCH_CONFIG: &str = "epoch";
table! {
    t_config (name) {
        name -> Text,
        content -> Text,
    }
}
#[derive(Queryable)]
pub struct ConfigPO {
    pub name: String,
    pub content: String,
}
#[derive(Insertable)]
#[table_name="t_config"]
pub struct NewConfig<'a> {
    pub name: &'a str,
    pub content: &'a str,
}
fn find_config_u64(name: &str) -> Option<u64> {
    return match find_config(name) {
        Some(cfg)=>cfg.content.parse::<u64>().ok(),
        None=> None,
    }
}
fn find_config(name_: &str) -> Option<ConfigPO> {
    use self::t_config::dsl::*;
    let conn = _POOL.get().unwrap();
    t_config.filter(name.eq(name_))
        .get_result(&conn).optional().unwrap()
}
fn save_config(name_: &str, content_: &str) {
    let conn = _POOL.get().unwrap();
    diesel::insert_into(t_config::table).values(&NewConfig{
        name: name_, content: content_
    })
        .execute(&conn).unwrap();
}

//===
table! {
    blocks (epoch, block_index) {
        epoch -> Unsigned<Bigint>,
        block_index -> Unsigned<Tinyint>,
        hash -> Text,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct BlockPO {
    pub epoch: u64,
    pub block_index: u8,
    pub hash: String,
    pub timestamp: NaiveDateTime,
}
#[derive(Insertable)]
#[table_name="blocks"]
pub struct NewBlock<'a> {
    pub epoch: u64,
    pub block_index: u8,
    pub hash: &'a str,
    pub timestamp: &'a NaiveDateTime,
}
//===
table! {
    log_data (epoch, block_index, tx_index, log_index) {
        epoch -> Unsigned<Bigint>,
        block_index -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Smallint>,
        log_index -> Unsigned<Smallint>,
        bytes -> Nullable<Blob>,
    }
}
#[derive(Queryable)]
pub struct LogDataPO<'a> {
    pub epoch: u64,
    pub block_index: u8,
    pub tx_index: u16,
    pub log_index: u16,
    pub bytes: &'a Vec<u8>,
}
#[derive(Insertable)]
#[table_name="log_data"]
pub struct NewLogData<'a> {
    pub epoch: u64,
    pub block_index: u8,
    pub tx_index: u16,
    pub log_index: u16,
    pub bytes: &'a Vec<u8>,
}
//===
table! {
    logs (epoch, block_index, tx_index, log_index) {
        epoch -> Unsigned<Bigint>,
        block_index -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Smallint>,
        log_index -> Unsigned<Smallint>,
        address_id -> Unsigned<Bigint>,
        topic0_id -> Unsigned<Bigint>,
        topic1_id -> Unsigned<Bigint>,
        topic2_id -> Unsigned<Bigint>,
        topic3_id -> Unsigned<Bigint>,
    }
}
#[derive(Queryable)]
pub struct LogPO {
    pub epoch: u64,
    pub block_index: u8,
    pub tx_index: u16,
    pub log_index: u16,
    pub address_id: u64,
    pub topic0_id: u64,
    pub topic1_id: u64,
    pub topic2_id: u64,
    pub topic3_id: u64,
}
#[derive(Insertable)]
#[table_name="logs"]
pub struct NewLog {
    pub epoch: u64,
    pub block_index: u8,
    pub tx_index: u16,
    pub log_index: u16,
    pub address_id: u64,
    pub topic0_id: u64,
    pub topic1_id: u64,
    pub topic2_id: u64,
    pub topic3_id: u64,
}
//===
table! {
    txs (epoch, block_index, tx_index) {
        epoch -> Unsigned<Bigint>,
        block_index -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Smallint>,
        hash -> Text,
        timestamp -> Timestamp,
        from_id -> Unsigned<Bigint>,
        to_id -> Unsigned<Bigint>,
        value -> Text,
        status -> Unsigned<Tinyint>,
    }
}
#[derive(Insertable)]
#[table_name="txs"]
pub struct NewTx<'a> {
    pub epoch: u64,
    pub block_index: u8,
    pub tx_index: u16,
    pub hash: String,
    pub timestamp: &'a NaiveDateTime,

    pub from_id: u64,
    pub to_id: u64,
    pub value: String,
    pub status: u8,
}
#[derive(Queryable)]
pub struct TxPO {
    pub epoch: u64,
    pub block_index: u8,
    pub index_in_block: u16,
    pub hash: String,
    pub timestamp: NaiveDateTime,
    pub from_id: u64,
    pub to_id: u64,
    pub value: String,
    pub status: u8,
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
        id -> Unsigned<Bigint>,
        hex -> Text,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct AddressPO {
    pub id: u64,
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
        id -> Unsigned<Bigint>,
        hex -> Text,
        timestamp -> Timestamp,
    }
}
#[derive(Queryable)]
pub struct Bytes32PO {
    pub id: u64,
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
pub fn prepare_epoch_relation(epoch_n: u64) {
    if epoch_n <= *PREVIOUS_SAVED_EPOCH.lock().unwrap() {
        let conn = &_POOL.get().unwrap();
        pop_log_data(epoch_n, conn);
        pop_logs(epoch_n, conn);
        pop_tx(epoch_n, conn);
        pop_block(epoch_n, conn);
    }
}
pub fn finish_epoch_relation(epoch_n: u64) {
    save_config(EPOCH_CONFIG, &(epoch_n.to_string()));
    *PREVIOUS_SAVED_EPOCH.lock().unwrap() = epoch_n;
}
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
pub fn insert_block_receipts(block: &Block, block_receipts: Arc<BlockReceipts>, epoch:u64, block_index: u8) {
    insert_block_relation(block, epoch, block_index);
    let receipts = &block_receipts.receipts;
    if receipts.is_empty() {
        return
    }
    let block_time = &build_block_timestamp(block.block_header.timestamp());
    let mut log_beans = vec![];
    let mut log_data_arr = vec![];
    let mut tx_status = Vec::new();
    for (idx, r) in receipts.iter().enumerate() {
        tx_status.push(r.outcome_status);
        if r.outcome_status != TransactionOutcome::Failure && r.outcome_status != TransactionOutcome::Success {
            continue;
        }
        for (log_index, log) in r.logs.iter().enumerate() {
            let address_id = save_address(&log.address, block_time).id;
            let mut new_log = NewLog{
                epoch, block_index, tx_index: idx as u16,
                log_index: log_index as u16, address_id,
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
            if log.data.len() > 0 {
                log_data_arr.push(NewLogData{
                    epoch,block_index, tx_index: idx as u16,
                    log_index: log_index as u16,
                    bytes: &log.data,
                })
            }
        }
    }
    insert_block_tx_relation(block, &tx_status, epoch, block_index);
    if log_beans.is_empty() {
        return;
    }
    let conn = _POOL.get().unwrap();
    use diesel::result::Error;
    conn.transaction::<_,Error,_>(||{
        diesel::replace_into(logs::table)
            .values(&log_beans)
            .execute(&conn)
            .expect("Error saving new logs_arr");
        diesel::replace_into(log_data::table)
            .values(&log_data_arr)
            .execute(&conn)
            .expect("Error saving new logs_data_arr");
        Ok(())
    }).unwrap()

}
pub fn pop_logs(epoch_n: u64, conn: &DbCon) {
    use self::logs::dsl::*;
    let pop_count = diesel::delete(logs.filter(epoch.ge(epoch_n)))
        .execute(conn).unwrap();
    info!("logs : epoch {} pop_count {}", epoch_n, pop_count);
}
pub fn pop_log_data(epoch_n: u64, conn: &DbCon) {
    use self::log_data::dsl::*;
    let pop_count = diesel::delete(log_data.filter(epoch.ge(epoch_n)))
        .execute(conn).unwrap();
    info!("log_data : epoch {} pop_count {}", epoch_n, pop_count);
}
// txs
pub fn remove_tx_relation(hash_: &H256) {
    use self::txs::dsl::*;
    let conn = _POOL.get().unwrap();
    let hash_0x = format!( "{:#x}", hash_);
    diesel::delete(txs.filter(hash.eq(hash_0x)))
        .execute(&conn).unwrap();
}
pub fn pop_tx(epoch_n: u64, conn: &DbCon) {
    use self::txs::dsl::*;
    let pop_count = diesel::delete(txs.filter(epoch.ge(epoch_n)))
        .execute(conn).unwrap();
    info!("tx : epoch {} pop_count {}", epoch_n, pop_count);
}
// the 1st epoch is 1, not 0. genesis epoch 0 is special. :<
pub fn insert_block_tx_relation(block: &Block, tx_status: &Vec<TransactionOutcome>, epoch:u64, block_index: u8) {
    // txVec: &Vec<Arc<SignedTransaction>>
    if block.transactions.is_empty(){
        return
    }
    let mut tx_arr =  Vec::new();
    let block_time = &build_block_timestamp(block.block_header.timestamp());
    for (idx, tx) in block.transactions.iter().enumerate() {
        let status = tx_status[idx];
        if status != TransactionOutcome::Failure && status != TransactionOutcome::Success {
            continue;
        }
        let from_id = save_address(&tx.sender, block_time).id;
        let to_id = match tx.action() {
            Call(addr)=>save_address(&addr, block_time).id,
            Create=>0,
        };
        let new_tx = NewTx{
            epoch,
            block_index,
            tx_index: idx as u16,
            hash: (format!( "{:#x}", tx.hash) ),
            timestamp: block_time,

            from_id,//: from_id,
            to_id,
            value: tx.value().to_string(),
            status: status as u8,
        };
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
pub fn insert_block_relation(block: &Block, epoch: u64, block_index: u8) {
    let conn = _POOL.get().unwrap();

    let hash = format!( "{:#x}", block.block_header.hash());

    let new_block = NewBlock{
        block_index, epoch,
        hash: &( hash ),
        timestamp: &build_block_timestamp(block.block_header.timestamp()),
    };
    diesel::insert_into(blocks::table)
        .values(&new_block)
        .execute(&conn)
        .expect("Error saving new block");
}
pub fn pop_block(epoch_n: u64, conn: &DbCon) {
    use self::blocks::dsl::*;
    let pop_count = diesel::delete(blocks.filter(epoch.ge(epoch_n)))
        .execute(conn).unwrap();
    info!("block : epoch {} pop_count {}", epoch_n, pop_count);
}
pub fn build_block_timestamp(mut timestamp: u64) -> NaiveDateTime {
    if timestamp == 0 {
        timestamp = 1;
    }
    NaiveDateTime::from_timestamp(timestamp as i64, 0)
}