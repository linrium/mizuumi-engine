#[derive(Debug)]
pub struct Credential {
    pub name: String,
    pub kind: CredentialKind,
    pub comment: String,
    pub owner: String,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: Option<i64>,
    pub updated_by: String,
    pub purpose: String,
}

#[derive(Debug)]
pub enum CredentialKind {
    AwsIamRole(AwsIamRole),
    RustfsServiceAccount(RustfsServiceAccount),
}

#[derive(Debug)]
pub struct AwsIamRole {
    pub role_arn: String,
    pub external_id: String,
}

#[derive(Debug)]
pub struct RustfsServiceAccount {
    pub endpoint_url: String,
    pub region: String,
    pub access_key: String,
    pub role_arn: String,
    pub force_path_style: bool,
    pub duration_seconds: u32,
}
