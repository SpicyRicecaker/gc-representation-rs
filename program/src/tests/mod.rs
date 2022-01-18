use crate::mark_compact::*;
use crate::shared::*;
use crate::stop_copy::StopAndCopyHeap;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod actual;
mod sanity;
mod collection;
