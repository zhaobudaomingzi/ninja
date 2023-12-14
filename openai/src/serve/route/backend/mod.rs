mod result;
mod sysinfo;

use self::{result::DataResult, sysinfo::SysInfo};
use crate::context::args::Args;
use axum::{routing::get, Json, Router};

/// backend dashboard router
pub(super) fn config(router: Router, _args: &Args) -> Router {
    router.route("/backend/system/info", get(get_system_info))
}

async fn get_system_info() -> Json<DataResult<SysInfo>> {
    Json(DataResult::success_with_data(sysinfo::get_info()))
}
