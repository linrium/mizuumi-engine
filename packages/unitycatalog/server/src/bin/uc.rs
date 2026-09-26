use std::{fmt::Display, process::ExitCode};

use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::{Method, blocking::Client};
use serde::Serialize;
use serde_json::{Value, json};

const DEFAULT_SERVER: &str = "http://localhost:8080";
const API_PATH: &str = "/api/2.1/unity-catalog";

#[derive(Debug, Parser)]
#[command(name = "uc", about = "Unity Catalog command-line client")]
struct Cli {
    #[arg(long, global = true, env = "UNITYCATALOG_SERVER", default_value = DEFAULT_SERVER)]
    server: String,
    #[arg(
        long = "auth_token",
        alias = "auth-token",
        global = true,
        env = "UNITYCATALOG_AUTH_TOKEN"
    )]
    auth_token: Option<String>,
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::JsonPretty)]
    output: OutputFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(name = "external_location", alias = "external-location")]
    ExternalLocation(ExternalLocationArgs),
    Permission(PermissionArgs),
}

#[derive(Debug, Args)]
struct ExternalLocationArgs {
    #[command(subcommand)]
    command: ExternalLocationCommand,
}

#[derive(Debug, Subcommand)]
enum ExternalLocationCommand {
    Create(CreateExternalLocationArgs),
    List(ListExternalLocationsArgs),
    Get(NameArgs),
    Update(UpdateExternalLocationArgs),
    Delete(DeleteExternalLocationArgs),
}

#[derive(Debug, Args)]
struct CreateExternalLocationArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    url: String,
    #[arg(long = "credential_name", alias = "credential-name")]
    credential_name: String,
    #[arg(long)]
    comment: Option<String>,
}

#[derive(Debug, Args)]
struct ListExternalLocationsArgs {
    #[arg(long = "max_results", alias = "max-results")]
    max_results: Option<i32>,
    #[arg(long = "page_token", alias = "page-token")]
    page_token: Option<String>,
}

#[derive(Debug, Args)]
struct NameArgs {
    #[arg(long)]
    name: String,
}

#[derive(Debug, Args)]
struct UpdateExternalLocationArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    url: Option<String>,
    #[arg(long = "credential_name", alias = "credential-name")]
    credential_name: Option<String>,
    #[arg(long)]
    comment: Option<String>,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long = "new_name", alias = "new-name")]
    new_name: Option<String>,
}

#[derive(Debug, Args)]
struct DeleteExternalLocationArgs {
    #[arg(long)]
    name: String,
    #[arg(long, default_value_t = false)]
    force: bool,
}

#[derive(Debug, Args)]
struct PermissionArgs {
    #[command(subcommand)]
    command: PermissionCommand,
}

#[derive(Debug, Subcommand)]
enum PermissionCommand {
    Create(ChangePermissionArgs),
    Delete(ChangePermissionArgs),
    Get(GetPermissionArgs),
}

#[derive(Debug, Args)]
struct ChangePermissionArgs {
    #[arg(long = "securable_type", alias = "securable-type")]
    securable_type: SecurableType,
    #[arg(long)]
    name: String,
    #[arg(long)]
    privilege: Privilege,
    #[arg(long)]
    principal: String,
}

#[derive(Debug, Args)]
struct GetPermissionArgs {
    #[arg(long = "securable_type", alias = "securable-type")]
    securable_type: SecurableType,
    #[arg(long)]
    name: String,
    #[arg(long)]
    principal: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SecurableType {
    Metastore,
    Catalog,
    Schema,
    Table,
    Function,
    Volume,
    #[value(name = "registered_model", alias = "registered-model")]
    RegisteredModel,
    #[value(name = "external_location", alias = "external-location")]
    ExternalLocation,
    Credential,
}

impl Display for SecurableType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Metastore => "metastore",
            Self::Catalog => "catalog",
            Self::Schema => "schema",
            Self::Table => "table",
            Self::Function => "function",
            Self::Volume => "volume",
            Self::RegisteredModel => "registered_model",
            Self::ExternalLocation => "external_location",
            Self::Credential => "credential",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Privilege {
    #[value(name = "CREATE CATALOG")]
    CreateCatalog,
    #[value(name = "USE CATALOG")]
    UseCatalog,
    #[value(name = "CREATE SCHEMA")]
    CreateSchema,
    #[value(name = "USE SCHEMA")]
    UseSchema,
    #[value(name = "CREATE TABLE")]
    CreateTable,
    #[value(name = "SELECT")]
    Select,
    #[value(name = "MODIFY")]
    Modify,
    #[value(name = "CREATE FUNCTION")]
    CreateFunction,
    #[value(name = "EXECUTE")]
    Execute,
    #[value(name = "CREATE VOLUME")]
    CreateVolume,
    #[value(name = "READ VOLUME")]
    ReadVolume,
    #[value(name = "CREATE MODEL")]
    CreateModel,
    #[value(name = "CREATE EXTERNAL LOCATION")]
    CreateExternalLocation,
    #[value(name = "READ FILES")]
    ReadFiles,
    #[value(name = "WRITE FILES")]
    WriteFiles,
    #[value(name = "CREATE EXTERNAL TABLE")]
    CreateExternalTable,
    #[value(name = "CREATE EXTERNAL VOLUME")]
    CreateExternalVolume,
    #[value(name = "CREATE MANAGED STORAGE")]
    CreateManagedStorage,
    #[value(name = "CREATE STORAGE CREDENTIAL")]
    CreateStorageCredential,
}

impl Display for Privilege {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::CreateCatalog => "CREATE CATALOG",
            Self::UseCatalog => "USE CATALOG",
            Self::CreateSchema => "CREATE SCHEMA",
            Self::UseSchema => "USE SCHEMA",
            Self::CreateTable => "CREATE TABLE",
            Self::Select => "SELECT",
            Self::Modify => "MODIFY",
            Self::CreateFunction => "CREATE FUNCTION",
            Self::Execute => "EXECUTE",
            Self::CreateVolume => "CREATE VOLUME",
            Self::ReadVolume => "READ VOLUME",
            Self::CreateModel => "CREATE MODEL",
            Self::CreateExternalLocation => "CREATE EXTERNAL LOCATION",
            Self::ReadFiles => "READ FILES",
            Self::WriteFiles => "WRITE FILES",
            Self::CreateExternalTable => "CREATE EXTERNAL TABLE",
            Self::CreateExternalVolume => "CREATE EXTERNAL VOLUME",
            Self::CreateManagedStorage => "CREATE MANAGED STORAGE",
            Self::CreateStorageCredential => "CREATE STORAGE CREDENTIAL",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    #[value(name = "json")]
    Json,
    #[value(name = "jsonPretty", alias = "json-pretty")]
    JsonPretty,
}

struct ApiClient {
    base_url: String,
    auth_token: Option<String>,
    client: Client,
}

impl ApiClient {
    fn new(server: String, auth_token: Option<String>) -> Result<Self, String> {
        let base_url = format!("{}{}", server.trim_end_matches('/'), API_PATH);
        let client = Client::builder()
            .build()
            .map_err(|error| format!("failed to create HTTP client: {error}"))?;
        Ok(Self {
            base_url,
            auth_token,
            client,
        })
    }

    fn request<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
    ) -> Result<Value, String> {
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.base_url))
            .query(query);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .map_err(|error| format!("request failed: {error}"))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .map_err(|error| format!("failed to read response: {error}"))?;
        if !status.is_success() {
            let message = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|value| {
                    value
                        .get("error")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
            return Err(format!("server returned {status}: {message}"));
        }
        if bytes.is_empty() {
            return Ok(json!({}));
        }

        serde_json::from_slice(&bytes).map_err(|error| format!("invalid JSON response: {error}"))
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let client = ApiClient::new(cli.server, cli.auth_token)?;
    let value = match cli.command {
        Command::ExternalLocation(args) => execute_external_location(&client, args.command)?,
        Command::Permission(args) => execute_permission(&client, args.command)?,
    };
    let output = match cli.output {
        OutputFormat::Json => serde_json::to_string(&value),
        OutputFormat::JsonPretty => serde_json::to_string_pretty(&value),
    }
    .map_err(|error| format!("failed to format output: {error}"))?;
    println!("{output}");
    Ok(())
}

fn execute_external_location(
    client: &ApiClient,
    command: ExternalLocationCommand,
) -> Result<Value, String> {
    match command {
        ExternalLocationCommand::Create(args) => client.request(
            Method::POST,
            "/external-locations",
            &[],
            Some(&json!({
                "name": args.name,
                "url": args.url,
                "credential_name": args.credential_name,
                "comment": args.comment,
            })),
        ),
        ExternalLocationCommand::List(args) => {
            let mut query = Vec::new();
            if let Some(max_results) = args.max_results {
                query.push(("max_results", max_results.to_string()));
            }
            if let Some(page_token) = args.page_token {
                query.push(("page_token", page_token));
            }
            let response =
                client.request::<Value>(Method::GET, "/external-locations", &query, None)?;
            Ok(response
                .get("external_locations")
                .cloned()
                .unwrap_or(response))
        }
        ExternalLocationCommand::Get(args) => client.request::<Value>(
            Method::GET,
            &format!("/external-locations/{}", path_segment(&args.name)),
            &[],
            None,
        ),
        ExternalLocationCommand::Update(args) => {
            if args.url.is_none()
                && args.credential_name.is_none()
                && args.comment.is_none()
                && args.owner.is_none()
                && args.new_name.is_none()
            {
                return Err("external_location update requires at least one field".to_string());
            }
            client.request(
                Method::PATCH,
                &format!("/external-locations/{}", path_segment(&args.name)),
                &[],
                Some(&json!({
                    "url": args.url,
                    "credential_name": args.credential_name,
                    "comment": args.comment,
                    "owner": args.owner,
                    "new_name": args.new_name,
                })),
            )
        }
        ExternalLocationCommand::Delete(args) => client.request::<Value>(
            Method::DELETE,
            &format!("/external-locations/{}", path_segment(&args.name)),
            &[("force", args.force.to_string())],
            None,
        ),
    }
}

fn execute_permission(client: &ApiClient, command: PermissionCommand) -> Result<Value, String> {
    let response = match command {
        PermissionCommand::Create(args) => change_permission(client, args, true)?,
        PermissionCommand::Delete(args) => change_permission(client, args, false)?,
        PermissionCommand::Get(args) => {
            let query = args
                .principal
                .map(|principal| vec![("principal", principal)])
                .unwrap_or_default();
            client.request::<Value>(
                Method::GET,
                &permission_path(args.securable_type, &args.name),
                &query,
                None,
            )?
        }
    };
    Ok(response
        .get("privilege_assignments")
        .cloned()
        .unwrap_or(response))
}

fn change_permission(
    client: &ApiClient,
    args: ChangePermissionArgs,
    add: bool,
) -> Result<Value, String> {
    let privilege = args.privilege.to_string();
    let (add_privileges, remove_privileges) = if add {
        (vec![privilege], Vec::new())
    } else {
        (Vec::new(), vec![privilege])
    };
    client.request(
        Method::PATCH,
        &permission_path(args.securable_type, &args.name),
        &[],
        Some(&json!({
            "changes": [{
                "principal": args.principal,
                "add": add_privileges,
                "remove": remove_privileges,
            }]
        })),
    )
}

fn permission_path(securable_type: SecurableType, name: &str) -> String {
    format!("/permissions/{}/{}", securable_type, path_segment(name))
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, ExternalLocationCommand, PermissionCommand, permission_path};

    #[test]
    fn parses_upstream_external_location_command_shape() {
        let cli = Cli::try_parse_from([
            "uc",
            "external_location",
            "create",
            "--name",
            "raw",
            "--url",
            "s3://bucket/raw",
            "--credential_name",
            "storage",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Command::ExternalLocation(args)
                if matches!(args.command, ExternalLocationCommand::Create(_))
        ));
    }

    #[test]
    fn parses_upstream_permission_command_shape() {
        let cli = Cli::try_parse_from([
            "uc",
            "permission",
            "create",
            "--securable_type",
            "external_location",
            "--name",
            "raw data",
            "--privilege",
            "READ FILES",
            "--principal",
            "reader@example.com",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Command::Permission(args) if matches!(args.command, PermissionCommand::Create(_))
        ));
    }

    #[test]
    fn encodes_permission_path_segments() {
        assert_eq!(
            permission_path(super::SecurableType::ExternalLocation, "raw data/2026"),
            "/permissions/external_location/raw%20data%2F2026"
        );
    }
}
