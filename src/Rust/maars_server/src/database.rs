use crate::CONFIG;
use lazy_static::lazy_static;
use maars::MessageType;
use serde_json::Value;
use std::{
    path::PathBuf,
    string::FromUtf8Error,
};

lazy_static! {
    static ref MSG_DB: sled::Db = {
        let mut p = PathBuf::new();
        p.push(CONFIG.database.path.clone());
        p.push("message");
        sled::open(p.as_os_str()).unwrap()
    };
}

#[derive(Debug)]
pub enum Error {
    /// Occurs when a database operation failed
    Sled(sled::Error),

    /// Occurs when serde couldn't deserialize the json message string
    SerdeJson(serde_json::Error),

    /// Occurs when the [`sled::Ivec`] is not long enough to contain all the necessary
    /// data for a [`Msg`]
    IVecNotLongEnough,

    /// Occurs when the string retrieved from an [`sled::Ivec`] is not valid utf8
    InvalidUtf8String(FromUtf8Error),
}

impl From<sled::Error> for Error {
    fn from(e: sled::Error) -> Self {
        Self::Sled(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeJson(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::InvalidUtf8String(e)
    }
}

#[derive(Debug)]
pub struct Msg {
    pub time: i64,
    pub type_: MessageType,
    pub uuid: String,
    pub body: String,
}

impl Msg {
    /// Converts a [`sled::IVec`] to a [`Msg`] struct
    fn from_ivec<S: Into<String>>(uuid: S, ivec: &sled::IVec) -> Result<Self, Error> {
        if ivec.len() <= 12 {
            return Err(Error::IVecNotLongEnough);
        };

        // TAke the first 8 bytes and convert them to a unix timestamp
        let time = {
            let tmp: [u8; 8] = [
                ivec[0], ivec[1], ivec[2], ivec[3], ivec[4], ivec[5], ivec[6], ivec[7],
            ];
            i64::from_be_bytes(tmp)
        };

        // Take the 9th to 12th byte and convert them into the message type
        let type_ = {
            let tmp: [u8; 4] = [ivec[8], ivec[9], ivec[10], ivec[11]];
            let id = i32::from_be_bytes(tmp);
            MessageType::new(id)
        };

        // Take the rest of the bytes and convert them into the json based message body
        let body = String::from_utf8(ivec[12..].to_vec()).map_err(Error::from)?;
        let msg = Msg {
            time,
            uuid: uuid.into(),
            type_,
            body,
        };
        Ok(msg)
    }
}

/// Inserts a message into the appropriate tree
///
/// # Parameters
/// * `msg` - The message to insert
///
/// # Returns
/// The id of the inserted message
pub fn insert_msg(msg: &Msg) -> Result<u64, Error> {
    let uuid_tree = MSG_DB.open_tree(&msg.uuid)?;
    let id = MSG_DB.generate_id()?;
    let mut value = msg.time.to_be_bytes().to_vec();
    value.extend_from_slice(&msg.type_.to_be_bytes());
    value.extend_from_slice(msg.body.as_bytes());
    uuid_tree.insert(id.to_be_bytes(), value)?;
    Ok(id)
}

/// Gets a message with the given id from the specified tree
///
/// # Parameters
/// * `uuid` - The database tree in which the message is stored
/// * `id` - The id of the message
///
/// # Returns
/// The message with the given id; `None` if no message with the given id exists.
pub fn get_msg(uuid: &str, id: u64) -> Result<Option<Msg>, Error> {
    let uuid_tree = MSG_DB.open_tree(uuid)?;
    match uuid_tree.get(id.to_be_bytes())? {
        Some(value) => {
            let msg = Msg::from_ivec(uuid, &value)?;
            Ok(Some(msg))
        }
        None => Ok(None),
    }
}

/// Gets the ids for all trees in the database
///
/// # Returns
/// A vector of all trees currently in the database
pub fn get_all_uuid() -> Result<Vec<String>, Error> {
    let mut result = Vec::new();
    for i in MSG_DB.tree_names() {
        let s = String::from_utf8(i.to_vec()).map_err(Error::from)?;
        if s == "__sled__default" {
            continue;
        }
        result.push(s);
    }
    Ok(result)
}

/// Gets the last `n` messages that were stored in the database
///
/// # Parameters
/// * `uuid` - The database tree in which the messages are stored
/// * `nums` - The number of messages to retrieve
pub fn get_last_msg(uuid: &str, nums: usize) -> Result<Vec<Msg>, Error> {
    let uuid_tree = MSG_DB.open_tree(uuid)?;
    if uuid_tree.is_empty() {
        drop(uuid)?;
    }
    let mut iter = uuid_tree.iter();
    let mut result = Vec::new();
    for _ in 0..nums {
        match iter.next_back() {
            Some(Ok((_, v))) => {
                result.push(Msg::from_ivec(uuid, &v)?);
            }
            _ => break,
        }
    }
    Ok(result)
}

/// Drops a single tree from the database and deletes all associeated data
///
/// # Parameters
/// * `uuid` - The id of the message
pub fn drop(uuid: &str) -> Result<(), Error> {
    MSG_DB.drop_tree(uuid)?;
    Ok(())
}

/// Drops all trees from the database and therefore enmpties it
pub fn drop_all() -> Result<(), Error> {
    for uuid in get_all_uuid()? {
        MSG_DB.drop_tree(uuid)?;
    }
    Ok(())
}

/// Stores a message in the MAA database
///
/// # Parameters
/// * `msg` - The type of the message
/// * `detail_json` - The formatted as json
pub fn maa_store_callback(msg: MessageType, detail_json: &str) {
    // NOTE: Using uwrap is fine here, as any panic caused by it will be siltenly captured
    // by the catch_unwind @and ignored. That way we don't propagate errors back to the caller.
    _ = std::panic::catch_unwind(|| {
        let body = detail_json.to_string();

        let now = chrono::Local::now().timestamp_millis();

        let v = serde_json::from_str::<Value>(&body)
            .map(|v| v.get("uuid").map(Value::to_string))
            .unwrap();

        let Some(uuid) = v else {
            return;
        };

        let msg = Msg {
            time: now,
            type_: msg,
            uuid,
            body,
        };
        insert_msg(&msg).unwrap();
    });
}
