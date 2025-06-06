pub mod file;
pub mod sled;
pub mod sqlite;

use super::models::{SerializableShortLink, ShortLink};
use super::register;
use super::Storage;
