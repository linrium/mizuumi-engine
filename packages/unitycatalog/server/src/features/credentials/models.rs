#[derive(Debug)]
pub struct Credential {
    pub name: String,
    pub aws_iam_role: AwsIamRole,
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
pub struct AwsIamRole {
    pub role_arn: String,
    pub external_id: String,
}
