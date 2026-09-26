use aws_credential_types::Credentials;

pub struct VendedCredentials {
    pub credentials: Credentials,
    pub expiration_time: i64,
}
