use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataResult<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[allow(dead_code)]
impl<T: Serialize> DataResult<T> {
    fn new() -> Self {
        DataResult {
            success: None,
            status: None,
            message: None,
            data: None,
        }
    }

    pub fn success(message: &str) -> Self {
        let mut vo = DataResult::new();
        vo.message = Some(message.to_string());
        vo.success = Some(true);
        vo.status = Some(200);
        vo
    }

    pub fn success_with_data(data: T) -> Self {
        let mut vo = DataResult::new();
        vo.data = Some(data);
        vo.success = Some(true);
        vo.status = Some(200);
        vo
    }

    pub fn success_void() -> Self {
        let mut vo = DataResult::new();
        vo.success = Some(true);
        vo.status = Some(200);
        vo
    }

    pub fn fail() -> Self {
        DataResult::fail_with_message(None)
    }

    pub fn fail_with_message(err_msg: Option<&str>) -> Self {
        let mut vo = DataResult::new();
        vo.success = Some(false);
        vo.status = Some(400 as u16);
        vo.message = err_msg.map(|s| s.to_string());
        vo
    }

    pub fn fail_with_code_message(err_code: u16, err_msg: &str) -> Self {
        let mut vo = DataResult::new();
        vo.success = Some(false);
        vo.status = Some(err_code);
        vo.message = Some(err_msg.to_string());
        vo
    }

    pub fn fail_with_http_status(http_status: u16) -> Self {
        let mut vo = DataResult::new();
        vo.success = Some(false);
        vo.status = Some(http_status);
        vo.message = None;
        vo
    }
}
