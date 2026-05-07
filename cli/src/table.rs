pub(crate) mod age;
pub(crate) mod all;
pub(crate) mod base;
pub(crate) mod date_format;
pub(crate) mod info;
pub(crate) mod next;

pub(crate) use age::Age;
pub(crate) use all::AllRow;
pub(crate) use base::BaseTable;
pub(crate) use info::InfoTable;
pub(crate) use next::NextRow;
use uuid::Uuid;

pub(crate) fn get_prefix(uuid: &Uuid) -> String {
    const SHORT_UUID_LEN: usize = 8;
    uuid.to_string()[..SHORT_UUID_LEN].to_owned()
}
