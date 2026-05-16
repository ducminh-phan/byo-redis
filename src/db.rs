use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) type Db = Arc<Mutex<HashMap<String, Bytes>>>;
