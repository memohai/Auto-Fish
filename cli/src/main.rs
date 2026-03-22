use std::error::Error;
use std::fs;
use std::net::IpAddr;
use std::process::exit;
use std::time::Duration;

use chrono::Utc;
use clap::{Parser, ValueEnum};
use crossbeam_channel::{Receiver, bounded, select};
use reqwest::Method;
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ErrorCode {
    Ok,
    Interrupted,
    InvalidParams,
    AuthError,
    NetworkError,
    ServerError,
    AssertionFailed,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ProxyMode {
    System,
    Direct,
    Auto,
}

impl ProxyMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Direct => "direct",
            Self::Auto => "auto",
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "amc", about = "Deterministic executor for amctl REST API")]
struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8081")]
    base_url: String,

    #[arg(long)]
    token: Option<String>,

    #[arg(long, default_value_t = 10000)]
    timeout_ms: u64,

    #[arg(long, value_enum, default_value_t = ProxyMode::Auto)]
    proxy: ProxyMode,

    #[arg(long, default_value = "default")]
    session: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    #[command(name = "preflight")]
    Preflight,
    #[command(name = "act")]
    Act {
        #[command(subcommand)]
        command: ActCommands,
    },
    #[command(name = "observe")]
    Observe {
        #[command(subcommand)]
        command: ObserveCommands,
    },
    #[command(name = "verify")]
    Verify {
        #[command(subcommand)]
        command: VerifyCommands,
    },
    #[command(name = "recover")]
    Recover {
        #[command(subcommand)]
        command: RecoverCommands,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ActCommands {
    #[command(name = "tap")]
    Tap {
        #[arg(long)]
        x: f32,
        #[arg(long)]
        y: f32,
    },
    #[command(name = "swipe")]
    Swipe {
        #[arg(long)]
        x1: f32,
        #[arg(long)]
        y1: f32,
        #[arg(long)]
        x2: f32,
        #[arg(long)]
        y2: f32,
        #[arg(long, default_value_t = 300)]
        duration: i64,
    },
    #[command(name = "back")]
    Back,
    #[command(name = "home")]
    Home,
    #[command(name = "text")]
    Text {
        #[arg(long)]
        text: String,
    },
    #[command(name = "launch")]
    Launch {
        #[arg(long = "package")]
        package_name: String,
    },
    #[command(name = "stop")]
    Stop {
        #[arg(long = "package")]
        package_name: String,
    },
    #[command(name = "key")]
    Key {
        #[arg(long = "key-code")]
        key_code: i32,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ObserveCommands {
    #[command(name = "screen")]
    Screen,
    #[command(name = "screenshot")]
    Screenshot {
        #[arg(long = "max-dim", default_value_t = 700)]
        max_dim: i64,
        #[arg(long, default_value_t = 80)]
        quality: i64,
    },
    #[command(name = "top")]
    Top,
}

#[derive(clap::Subcommand, Debug)]
enum VerifyCommands {
    #[command(name = "text-contains")]
    TextContains {
        #[arg(long)]
        text: String,
        #[arg(long, default_value_t = true)]
        ignore_case: bool,
    },
    #[command(name = "top-activity")]
    TopActivity {
        #[arg(long)]
        expected: String,
        #[arg(long, default_value = "contains")]
        mode: String,
    },
    #[command(name = "node-exists")]
    NodeExists {
        #[arg(long)]
        by: String,
        #[arg(long)]
        value: String,
        #[arg(long, default_value_t = false)]
        exact_match: bool,
    },
}

#[derive(clap::Subcommand, Debug)]
enum RecoverCommands {
    #[command(name = "back")]
    Back {
        #[arg(long, default_value_t = 1)]
        times: u32,
    },
    #[command(name = "home")]
    Home,
    #[command(name = "relaunch")]
    Relaunch {
        #[arg(long = "package")]
        package_name: String,
    },
}

#[derive(Deserialize)]
struct ApiEnvelope {
    ok: bool,
    data: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct CliResult {
    ok: bool,
    code: ErrorCode,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    timestamp: String,
}

#[derive(Debug)]
struct CliError {
    code: ErrorCode,
    message: String,
}

impl CliError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

struct Runtime {
    client_system: Client,
    client_direct: Client,
    base_url: String,
    token: Option<String>,
    timeout_ms: u64,
    proxy_mode: ProxyMode,
    session: String,
    ctrl_c_events: Receiver<()>,
}

fn main() {
    let ctrl_c_events = match ctrl_channel() {
        Ok(rx) => rx,
        Err(e) => {
            let err = CliError::new(
                ErrorCode::InternalError,
                format!("failed to set signal handler: {e}"),
            );
            emit_and_exit("parse", Err(err), None);
        }
    };

    let cli = Cli::parse();
    let runtime = match build_runtime(&cli, ctrl_c_events) {
        Ok(r) => r,
        Err(e) => emit_and_exit("parse", Err(e), None),
    };

    let (command_name, command_args) = command_meta(&cli.command);
    let input_payload = json!({
        "command": command_name,
        "args": command_args,
        "options": {
            "baseUrl": runtime.base_url,
            "timeoutMs": runtime.timeout_ms,
            "proxyMode": runtime.proxy_mode.as_str(),
            "sessionId": runtime.session,
            "token": if runtime.token.is_some() { Value::String("***".to_string()) } else { Value::Null }
        },
        "timestamp": now_iso()
    });
    let _ = session_write(&runtime.session, command_name, "input", &input_payload);

    let result = run_command(&runtime, &cli.command);
    emit_and_exit(command_name, result, Some(&runtime.session));
}

fn build_runtime(cli: &Cli, ctrl_c_events: Receiver<()>) -> Result<Runtime, CliError> {
    if cli.base_url.trim().is_empty() {
        return Err(CliError::new(
            ErrorCode::InvalidParams,
            "base-url is required",
        ));
    }

    let client_system = Client::builder()
        .timeout(Duration::from_millis(cli.timeout_ms))
        .build()
        .map_err(|e| {
            CliError::new(
                ErrorCode::InternalError,
                format!("failed to build system-proxy http client: {e}"),
            )
        })?;

    let client_direct = Client::builder()
        .no_proxy()
        .timeout(Duration::from_millis(cli.timeout_ms))
        .build()
        .map_err(|e| {
            CliError::new(
                ErrorCode::InternalError,
                format!("failed to build direct http client: {e}"),
            )
        })?;

    Ok(Runtime {
        client_system,
        client_direct,
        base_url: cli.base_url.trim_end_matches('/').to_string(),
        token: cli.token.clone(),
        timeout_ms: cli.timeout_ms,
        proxy_mode: cli.proxy,
        session: cli.session.clone(),
        ctrl_c_events,
    })
}

fn run_command(runtime: &Runtime, cmd: &Commands) -> Result<Value, CliError> {
    match cmd {
        Commands::Preflight => cmd_preflight(runtime),
        Commands::Act { command } => match command {
            ActCommands::Tap { x, y } => {
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/tap",
                    Some(json!({"x": x, "y": y})),
                )?;
                Ok(json!({"action": "tap", "result": msg}))
            }
            ActCommands::Swipe {
                x1,
                y1,
                x2,
                y2,
                duration,
            } => {
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/swipe",
                    Some(json!({"x1": x1, "y1": y1, "x2": x2, "y2": y2, "duration": duration})),
                )?;
                Ok(json!({"action": "swipe", "result": msg}))
            }
            ActCommands::Back => {
                let msg = api_action(runtime, Method::POST, "/api/press/back", Some(json!({})))?;
                Ok(json!({"action": "back", "result": msg}))
            }
            ActCommands::Home => {
                let msg = api_action(runtime, Method::POST, "/api/press/home", Some(json!({})))?;
                Ok(json!({"action": "home", "result": msg}))
            }
            ActCommands::Text { text } => {
                if text.is_empty() {
                    return Err(CliError::new(
                        ErrorCode::InvalidParams,
                        "text must not be empty",
                    ));
                }
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/text",
                    Some(json!({"text": text})),
                )?;
                Ok(json!({"action": "text", "result": msg}))
            }
            ActCommands::Launch { package_name } => {
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/app/launch",
                    Some(json!({"package_name": package_name})),
                )?;
                Ok(json!({"action": "launch", "result": msg}))
            }
            ActCommands::Stop { package_name } => {
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/app/stop",
                    Some(json!({"package_name": package_name})),
                )?;
                Ok(json!({"action": "stop", "result": msg}))
            }
            ActCommands::Key { key_code } => {
                let msg = api_action(
                    runtime,
                    Method::POST,
                    "/api/press/key",
                    Some(json!({"key_code": key_code})),
                )?;
                Ok(json!({"action": "key", "keyCode": key_code, "result": msg}))
            }
        },
        Commands::Observe { command } => match command {
            ObserveCommands::Screen => {
                let screen = api_get_data(runtime, "/api/screen")?;
                Ok(json!({"observation": "screen", "screen": screen}))
            }
            ObserveCommands::Screenshot { max_dim, quality } => {
                let path = format!("/api/screenshot?max_dim={max_dim}&quality={quality}");
                let shot = api_get_data(runtime, &path)?;
                Ok(
                    json!({"observation": "screenshot", "screenshotBase64": shot, "maxDim": max_dim, "quality": quality}),
                )
            }
            ObserveCommands::Top => {
                let top = api_get_data(runtime, "/api/app/top")?;
                Ok(json!({"observation": "top", "topActivity": top}))
            }
        },
        Commands::Verify { command } => match command {
            VerifyCommands::TextContains { text, ignore_case } => {
                let screen = api_get_data(runtime, "/api/screen")?;
                let matched = if *ignore_case {
                    screen.to_lowercase().contains(&text.to_lowercase())
                } else {
                    screen.contains(text)
                };
                if !matched {
                    return Err(CliError::new(
                        ErrorCode::AssertionFailed,
                        format!("text not found in screen: {text}"),
                    ));
                }
                Ok(json!({"verify": "text-contains", "matched": matched, "text": text}))
            }
            VerifyCommands::TopActivity { expected, mode } => {
                if mode != "contains" && mode != "equals" {
                    return Err(CliError::new(
                        ErrorCode::InvalidParams,
                        "mode must be contains or equals",
                    ));
                }
                let top = api_get_data(runtime, "/api/app/top")?;
                let matched = if mode == "equals" {
                    top == *expected
                } else {
                    top.contains(expected)
                };
                if !matched {
                    return Err(CliError::new(
                        ErrorCode::AssertionFailed,
                        format!("top activity mismatch: expected {mode} {expected}, got {top}"),
                    ));
                }
                Ok(
                    json!({"verify": "top-activity", "matched": matched, "expected": expected, "actual": top, "mode": mode}),
                )
            }
            VerifyCommands::NodeExists {
                by,
                value,
                exact_match,
            } => {
                let by_norm = by.to_lowercase();
                let valid = ["id", "text", "desc", "class", "resource_id"];
                if !valid.contains(&by_norm.as_str()) {
                    return Err(CliError::new(
                        ErrorCode::InvalidParams,
                        "by must be one of: id,text,desc,class,resource_id",
                    ));
                }
                let found = api_action(
                    runtime,
                    Method::POST,
                    "/api/nodes/find",
                    Some(json!({"by": by_norm, "value": value, "exact_match": exact_match})),
                )?;
                if found.starts_with("No nodes found") {
                    return Err(CliError::new(
                        ErrorCode::AssertionFailed,
                        format!("node not found: by={by}, value={value}"),
                    ));
                }
                Ok(
                    json!({"verify": "node-exists", "matched": true, "by": by, "value": value, "exactMatch": exact_match, "resultText": found}),
                )
            }
        },
        Commands::Recover { command } => match command {
            RecoverCommands::Back { times } => {
                if *times == 0 {
                    return Err(CliError::new(
                        ErrorCode::InvalidParams,
                        "times must be >= 1",
                    ));
                }
                for _ in 0..*times {
                    let _ = api_action(runtime, Method::POST, "/api/press/back", Some(json!({})))?;
                }
                Ok(json!({"recover": "back", "times": times}))
            }
            RecoverCommands::Home => {
                let _ = api_action(runtime, Method::POST, "/api/press/home", Some(json!({})))?;
                Ok(json!({"recover": "home"}))
            }
            RecoverCommands::Relaunch { package_name } => {
                let _ = api_action(runtime, Method::POST, "/api/press/home", Some(json!({})))?;
                let launch = api_action(
                    runtime,
                    Method::POST,
                    "/api/app/launch",
                    Some(json!({"package_name": package_name})),
                )?;
                Ok(
                    json!({"recover": "relaunch", "packageName": package_name, "launchResult": launch}),
                )
            }
        },
    }
}

fn cmd_preflight(runtime: &Runtime) -> Result<Value, CliError> {
    let health = http_json(runtime, Method::GET, "/health", None, false)?;

    let mut auth = "skipped".to_string();
    let mut top_activity: Value = Value::Null;

    if runtime.token.is_some() {
        match api_get_data(runtime, "/api/app/top") {
            Ok(top) => {
                auth = "ok".to_string();
                top_activity = Value::String(top);
            }
            Err(e) if e.code == ErrorCode::AuthError => {
                auth = "unauthorized".to_string();
            }
            Err(e) => return Err(e),
        }
    }

    Ok(json!({
        "baseUrl": runtime.base_url,
        "timeoutMs": runtime.timeout_ms,
        "tokenProvided": runtime.token.is_some(),
        "health": health,
        "auth": auth,
        "topActivity": top_activity
    }))
}

fn api_get_data(runtime: &Runtime, path: &str) -> Result<String, CliError> {
    let val = http_json(runtime, Method::GET, path, None, true)?;
    unwrap_envelope(path, val)
}

fn api_action(
    runtime: &Runtime,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<String, CliError> {
    let val = http_json(runtime, method, path, body, true)?;
    unwrap_envelope(path, val)
}

fn http_json(
    runtime: &Runtime,
    method: Method,
    path: &str,
    body: Option<Value>,
    require_auth: bool,
) -> Result<Value, CliError> {
    if require_auth && runtime.token.is_none() {
        return Err(CliError::new(
            ErrorCode::InvalidParams,
            "token is required for this command",
        ));
    }

    let url = format!("{}{}", runtime.base_url, path);
    let request_url = url.clone();
    let method_name = method.as_str().to_string();
    let proxy_mode = runtime.proxy_mode;
    let client = select_client(runtime, &request_url);
    let mut req = client
        .request(method, url)
        .header(CONTENT_TYPE, "application/json");

    if let Some(token) = &runtime.token {
        req = req.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    if let Some(b) = body {
        req = req.json(&b);
    }

    run_with_interrupt(&runtime.ctrl_c_events, move || {
        let resp = req.send().map_err(|e| {
            CliError::new(
                ErrorCode::NetworkError,
                format_reqwest_error(&e, &method_name, &request_url, proxy_mode),
            )
        })?;

        let status = resp.status();
        let text = resp.text().map_err(|e| {
            CliError::new(
                ErrorCode::NetworkError,
                format_reqwest_error(&e, &method_name, &request_url, proxy_mode),
            )
        })?;

        if status.as_u16() == 401 {
            return Err(CliError::new(
                ErrorCode::AuthError,
                "Unauthorized: invalid or missing bearer token",
            ));
        }

        if !status.is_success() {
            let msg = extract_error_message(&text).unwrap_or_else(|| {
                format!(
                    "{} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                )
            });
            return Err(CliError::new(ErrorCode::ServerError, msg));
        }

        Ok(parse_json_or_string(&text))
    })
}

fn select_client<'a>(runtime: &'a Runtime, request_url: &str) -> &'a Client {
    match runtime.proxy_mode {
        ProxyMode::Direct => &runtime.client_direct,
        ProxyMode::System => &runtime.client_system,
        ProxyMode::Auto => {
            if should_bypass_proxy(request_url) {
                &runtime.client_direct
            } else {
                &runtime.client_system
            }
        }
    }
}

fn should_bypass_proxy(request_url: &str) -> bool {
    let Ok(url) = Url::parse(request_url) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local()
        }
        Err(_) => false,
    }
}

fn has_proxy_env() -> bool {
    [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ]
    .iter()
    .any(|k| std::env::var_os(k).is_some())
}

fn format_reqwest_error(
    err: &reqwest::Error,
    method: &str,
    url: &str,
    proxy_mode: ProxyMode,
) -> String {
    let kind = if err.is_timeout() {
        "timeout"
    } else if err.is_connect() {
        "connect"
    } else if err.is_request() {
        "request"
    } else if err.is_body() {
        "body"
    } else if err.is_decode() {
        "decode"
    } else {
        "unknown"
    };

    let mut chain = Vec::new();
    let mut cur: Option<&(dyn std::error::Error + 'static)> = err.source();
    while let Some(e) = cur {
        chain.push(e.to_string());
        cur = e.source();
    }

    let mut msg = format!("{method} {url} failed ({kind}): {err}");
    if !chain.is_empty() {
        msg.push_str(" | caused by: ");
        msg.push_str(&chain.join(" -> "));
    }
    if proxy_mode == ProxyMode::System && should_bypass_proxy(url) && has_proxy_env() {
        msg.push_str(" | hint: local/private target detected; try --proxy auto/direct or set NO_PROXY for this host");
    }
    msg
}

fn run_with_interrupt<T, F>(ctrl_c_events: &Receiver<()>, work: F) -> Result<T, CliError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, CliError> + Send + 'static,
{
    let (done_tx, done_rx) = bounded::<Result<T, CliError>>(1);
    std::thread::spawn(move || {
        let _ = done_tx.send(work());
    });

    select! {
        recv(ctrl_c_events) -> _ => Err(CliError::new(
            ErrorCode::Interrupted,
            "Interrupted by SIGINT (Ctrl+C)",
        )),
        recv(done_rx) -> msg => {
            match msg {
                Ok(res) => res,
                Err(_) => Err(CliError::new(
                    ErrorCode::InternalError,
                    "worker channel closed unexpectedly",
                )),
            }
        }
    }
}

fn extract_error_message(text: &str) -> Option<String> {
    let v: Value = serde_json::from_str(text).ok()?;
    if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
        return Some(err.to_string());
    }
    if let Some(msg) = v.get("message").and_then(|x| x.as_str()) {
        return Some(msg.to_string());
    }
    None
}

fn unwrap_envelope(op: &str, input: Value) -> Result<String, CliError> {
    let env: ApiEnvelope = serde_json::from_value(input).map_err(|_| {
        CliError::new(
            ErrorCode::ServerError,
            format!("Unexpected {op} response format"),
        )
    })?;

    if !env.ok {
        return Err(CliError::new(
            ErrorCode::ServerError,
            env.error.unwrap_or_else(|| format!("{op} failed")),
        ));
    }

    Ok(env.data.unwrap_or_default())
}

fn parse_json_or_string(text: &str) -> Value {
    if text.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str::<Value>(text).unwrap_or_else(|_| Value::String(text.to_string()))
    }
}

fn command_meta(cmd: &Commands) -> (&'static str, Value) {
    match cmd {
        Commands::Preflight => ("preflight", json!({})),
        Commands::Act { command } => match command {
            ActCommands::Tap { x, y } => ("act:tap", json!({"x": x, "y": y})),
            ActCommands::Swipe {
                x1,
                y1,
                x2,
                y2,
                duration,
            } => (
                "act:swipe",
                json!({"x1": x1, "y1": y1, "x2": x2, "y2": y2, "duration": duration}),
            ),
            ActCommands::Back => ("act:back", json!({})),
            ActCommands::Home => ("act:home", json!({})),
            ActCommands::Text { text } => ("act:text", json!({"text": text})),
            ActCommands::Launch { package_name } => {
                ("act:launch", json!({"package": package_name}))
            }
            ActCommands::Stop { package_name } => ("act:stop", json!({"package": package_name})),
            ActCommands::Key { key_code } => ("act:key", json!({"keyCode": key_code})),
        },
        Commands::Observe { command } => match command {
            ObserveCommands::Screen => ("observe:screen", json!({})),
            ObserveCommands::Screenshot { max_dim, quality } => (
                "observe:screenshot",
                json!({"maxDim": max_dim, "quality": quality}),
            ),
            ObserveCommands::Top => ("observe:top", json!({})),
        },
        Commands::Verify { command } => match command {
            VerifyCommands::TextContains { text, ignore_case } => (
                "verify:text-contains",
                json!({"text": text, "ignoreCase": ignore_case}),
            ),
            VerifyCommands::TopActivity { expected, mode } => (
                "verify:top-activity",
                json!({"expected": expected, "mode": mode}),
            ),
            VerifyCommands::NodeExists {
                by,
                value,
                exact_match,
            } => (
                "verify:node-exists",
                json!({"by": by, "value": value, "exactMatch": exact_match}),
            ),
        },
        Commands::Recover { command } => match command {
            RecoverCommands::Back { times } => ("recover:back", json!({"times": times})),
            RecoverCommands::Home => ("recover:home", json!({})),
            RecoverCommands::Relaunch { package_name } => {
                ("recover:relaunch", json!({"package": package_name}))
            }
        },
    }
}

fn emit_and_exit(command: &str, result: Result<Value, CliError>, session_id: Option<&str>) -> ! {
    match result {
        Ok(data) => {
            if let Some(session) = session_id {
                let payload = json!({"command": command, "result": data, "timestamp": now_iso()});
                let _ = session_write(session, command, "output", &payload);
            }

            let output = CliResult {
                ok: true,
                code: ErrorCode::Ok,
                command: command.to_string(),
                data: Some(data),
                error: None,
                timestamp: now_iso(),
            };
            println!(
                "{}",
                serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
            );
            exit(0);
        }
        Err(err) => {
            if let Some(session) = session_id {
                let payload = json!({"command": command, "error": err.message, "code": err.code, "timestamp": now_iso()});
                let _ = session_write(session, command, "error", &payload);
            }

            let output = CliResult {
                ok: false,
                code: err.code,
                command: command.to_string(),
                data: None,
                error: Some(err.message.clone()),
                timestamp: now_iso(),
            };
            println!(
                "{}",
                serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
            );
            exit(exit_code(err.code));
        }
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn exit_code(code: ErrorCode) -> i32 {
    match code {
        ErrorCode::Interrupted => 130,
        ErrorCode::InvalidParams => 2,
        ErrorCode::AuthError => 3,
        ErrorCode::NetworkError => 4,
        ErrorCode::ServerError => 5,
        ErrorCode::AssertionFailed => 6,
        ErrorCode::InternalError => 10,
        ErrorCode::Ok => 0,
    }
}

fn session_write(session: &str, command: &str, phase: &str, payload: &Value) -> Result<(), ()> {
    let ts = now_iso().replace(':', "-").replace('.', "-");
    let safe = sanitize(command);
    let dir = format!(".amctl-sessions/{session}/commands");
    if fs::create_dir_all(&dir).is_err() {
        return Err(());
    }
    let file = format!("{dir}/{ts}_{safe}_{phase}.json");
    if fs::write(file, serde_json::to_vec_pretty(payload).unwrap_or_default()).is_err() {
        return Err(());
    }
    Ok(())
}

fn sanitize(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ':' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;
    Ok(receiver)
}

#[cfg(test)]
mod tests {
    use super::should_bypass_proxy;

    #[test]
    fn bypasses_local_and_private_targets() {
        assert!(should_bypass_proxy("http://127.0.0.1:8081/health"));
        assert!(should_bypass_proxy("http://localhost:8081/health"));
        assert!(should_bypass_proxy("http://192.168.50.214:9998/health"));
    }

    #[test]
    fn does_not_bypass_public_targets() {
        assert!(!should_bypass_proxy("http://8.8.8.8:80/health"));
        assert!(!should_bypass_proxy("https://example.com/health"));
    }
}
