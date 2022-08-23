-- Your SQL goes here
CREATE TABLE blocks (
                        id  bigint unsigned auto_increment NOT NULL PRIMARY KEY,
                        hash CHAR(66) NOT NULL unique ,
                        height  bigint unsigned NOT NULL,
                        `timestamp` timestamp NOT NULL
)