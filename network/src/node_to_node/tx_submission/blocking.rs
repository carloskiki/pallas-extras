use super::reply::Ids;

type Done = crate::message::Done<4>;

crate::state!(@message crate::agency::Client | Ids<'static>, Done);
