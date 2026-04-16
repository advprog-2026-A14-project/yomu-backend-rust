use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: T) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn success_without_data(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn success_response_serializes_correctly() {
        let response = ApiResponse::success("OK", TestData { value: 42 });
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""message":"OK""#));
        assert!(json.contains(r#""value":42"#));
        assert!(json.contains(r#""data":"#));
    }

    #[test]
    fn success_without_data_excludes_data_field() {
        let response = ApiResponse::<i32>::success_without_data("Created");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""message":"Created""#));
        assert!(!json.contains(r#""data""#));
    }

    #[test]
    fn error_response_has_success_false() {
        let response: ApiResponse<()> = ApiResponse::error("Not found");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""success":false"#));
    }

    #[test]
    fn error_response_includes_message() {
        let response: ApiResponse<()> = ApiResponse::error("Something went wrong");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""message":"Something went wrong""#));
    }

    #[test]
    fn success_with_null_data() {
        let response = ApiResponse::<Option<i32>>::success("OK", None);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""data":null"#));
    }
}
