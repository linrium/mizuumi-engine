use aws_credential_types::Credentials;

#[derive(Debug)]
pub struct Bucket {
    pub name: String,
}

pub struct VendedCredentials {
    pub credentials: Credentials,
    pub expiration_time: i64,
}
