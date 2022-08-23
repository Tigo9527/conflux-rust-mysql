-- Your SQL goes here
create table txs (
                     id  bigint unsigned auto_increment NOT NULL PRIMARY KEY,
                     hash CHAR(66) NOT NULL unique ,
                     height  bigint unsigned NOT NULL,
                     `timestamp` timestamp NOT NULL,

                     block_id bigint unsigned NOT NULL,
                     from_id bigint unsigned NOT NULL,
                     to_id bigint unsigned NOT NULL,
                     value decimal(65, 0)NOT NULL,
                     status int NOT NULL
);

create table addresses (
                           id  bigint unsigned auto_increment NOT NULL PRIMARY KEY,
                           hex CHAR(42) NOT NULL unique ,
                           `timestamp` timestamp NOT NULL
);