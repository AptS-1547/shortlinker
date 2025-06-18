pub mod file;
pub mod mysql;
pub mod postgres;
pub mod sled;
pub mod sqlite;

use super::models::{CachePreference, SerializableShortLink, ShortLink};
use super::Storage;
