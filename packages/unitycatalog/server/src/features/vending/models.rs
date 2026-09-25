use serde::Deserialize;

#[derive(Debug)]
pub struct Bucket {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AssumeRoleResponse {
    #[serde(rename = "AssumeRoleResult")]
    pub result: AssumeRoleResult,
}

#[derive(Debug, Deserialize)]
pub struct AssumeRoleResult {
    #[serde(rename = "Credentials")]
    pub credentials: TemporaryCredentials,
}

#[derive(Debug, Deserialize)]
pub struct TemporaryCredentials {
    #[serde(rename = "AccessKeyId")]
    pub access_key_id: String,
    #[serde(rename = "SecretAccessKey")]
    pub secret_access_key: String,
    #[serde(rename = "SessionToken")]
    pub session_token: String,
}
