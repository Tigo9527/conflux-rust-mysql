-- Your SQL goes here
CREATE TABLE blocks (
                        epoch bigint unsigned not null,
                        block_index  tinyint unsigned ,
                        hash CHAR(66) NOT NULL unique ,
                        `timestamp` timestamp NOT NULL,
                        primary key (epoch, block_index)
);
create table txs (
                     `epoch` bigint unsigned not null,
                     block_index  tinyint unsigned ,
                     `tx_index` smallint unsigned not null,
                     hash CHAR(66) NOT NULL unique ,
                     `timestamp` timestamp NOT NULL,

                     from_id bigint unsigned NOT NULL,
                     to_id bigint unsigned NOT NULL,
                     value decimal(65, 0)NOT NULL,
                     status tinyint unsigned NOT NULL,
                     primary key (`epoch`, `block_index`, tx_index)
);

create table addresses (
                           id  bigint unsigned auto_increment NOT NULL PRIMARY KEY,
                           hex CHAR(42) NOT NULL unique ,
                           `timestamp` timestamp NOT NULL
);
create table log_data
(
    epoch bigint unsigned not null,
    `block_index`  bigint unsigned NOT NULL,
    `tx_index` smallint unsigned not null,
    log_index smallint unsigned not null, -- [0,65535]
    bytes blob,
    primary key (epoch, `block_index`, `tx_index`,`log_index`)
);
-- for topics in event
create table bytes32s (
                          id  bigint unsigned auto_increment NOT NULL PRIMARY KEY,
                          hex CHAR(66) NOT NULL unique ,
                          `timestamp` timestamp NOT NULL
);
-- use another table to save tx in evm ? for logs, save to table along with its tx ?
create table logs (
                      epoch bigint unsigned not null,
                      `block_index`  bigint unsigned NOT NULL,
                      `tx_index` smallint unsigned not null,
                      log_index smallint unsigned not null, -- [0,65535]
                      address_id bigint unsigned not null,
                      topic0_id bigint unsigned not null,
                      topic1_id bigint unsigned not null,
                      topic2_id bigint unsigned not null,
                      topic3_id bigint unsigned not null,
                      primary key (epoch, block_index, `tx_index`,`log_index`),
                index idx_epoch_topic0 (epoch, topic0_id), -- query by epoch (and topic)
                index idx_addr_epoch (address_id, epoch)  -- query by addr and epoch
);

create table t_config (
    name varchar(128) not null primary key ,
    content varchar(2048) not null
);