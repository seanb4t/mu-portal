//! The verb `mu server`: the secrets, the socket directory, then `mu-server`.

use std::process::ExitCode;
use std::sync::Arc;

use clap::Args;
use mu_core::{
    EnvSecretStore, FakeTransport, ResolvedWorld, SecretRef, WorldId, WorldSetup, load_password,
};
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::config::ConfigArgs;
use crate::server_config;
use crate::socket::SocketArgs;
use crate::telemetry::{self, Providers, STOP_BOUND, SdkDisabled};

/// Arguments of `mu server`.
#[derive(Clone, Debug, Default, Args)]
pub struct ServerArgs {
    #[command(flatten)]
    socket: SocketArgs,

    #[command(flatten)]
    config: ConfigArgs,

    /// A fake world for tests: `<id>=env:<NAME>`. The flag can repeat.
    #[arg(long = "fake-world", hide = true, value_name = "ID=env:NAME", value_parser = parse_fake_world)]
    fake_worlds: Vec<FakeWorld>,
}

/// One `--fake-world` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FakeWorld {
    id: WorldId,
    secret: SecretRef,
}

/// Parses `<id>=env:<NAME>`.
///
/// # Errors
///
/// A text that names the bad part.
pub fn parse_fake_world(value: &str) -> Result<FakeWorld, String> {
    let (id, secret) = value
        .split_once('=')
        .ok_or_else(|| format!("{value:?} has no `=`: write <id>=env:<NAME>"))?;
    Ok(FakeWorld {
        id: id.parse().map_err(|error| format!("{error}"))?,
        secret: secret.parse().map_err(|error| format!("{error}"))?,
    })
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail("server", error)
}

/// Runs the server until a signal stops it.
pub fn run(args: &ServerArgs) -> ExitCode {
    // The crypto provider of the process, for reqwest and tonic. A second call finds it installed.
    let _ = rustls::crypto::ring::default_provider().install_default();
    // `server.toml` and the variables decide the export before the socket opens. The HTTP
    // exporters are built here, outside the runtime of the engine.
    let file = match server_config::read(args.config.dir_from_env().as_deref()) {
        Ok(file) => file,
        Err(error) => return fail(&error),
    };
    let export = match telemetry::export(|name| std::env::var_os(name), &file.telemetry) {
        Ok(export) => export,
        Err(error) => return fail(&error),
    };
    // With no export, the server writes no line about telemetry, so this warning too needs one.
    if export.is_some()
        && telemetry::sdk_disabled(|name| std::env::var_os(name)) == Ok(SdkDisabled::Ignored)
    {
        crate::warn(
            "server",
            &format_args!(
                "{}: the value is not true or false, so the server ignores it",
                telemetry::SDK_DISABLED
            ),
        );
    }
    let providers = match export.as_ref().map(Providers::start).transpose() {
        Ok(providers) => providers.unwrap_or_else(Providers::none),
        Err(error) => return fail(&error),
    };
    // Only the server logs: the lines would break the screen of the client.
    // `try_init` also sends the records of the `log` crate to the subscriber.
    if let Err(error) = providers.subscriber().try_init() {
        return fail(&format!("the log of the server: {error}"));
    }
    let code = serve_worlds(args);
    providers.shutdown(STOP_BOUND);
    code
}

/// Resolves the worlds, opens the socket and serves until a signal stops it.
fn serve_worlds(args: &ServerArgs) -> ExitCode {
    // The configuration resolves first, so a bad file stops the start before the socket opens.
    let configured = match args.config.worlds("server") {
        Ok(worlds) => worlds,
        Err(error) => return fail(&error),
    };
    if let Some(fake) = args
        .fake_worlds
        .iter()
        .find(|fake| configured.iter().any(|world| world.id == fake.id))
    {
        return fail(&format!(
            "{}: the id is both a fake world and a configured world",
            fake.id
        ));
    }
    let store = EnvSecretStore::new();
    let mut worlds: Vec<WorldSetup> = configured
        .into_iter()
        .map(WorldSetup::without_transport)
        .collect();
    for fake in &args.fake_worlds {
        let password = match load_password(&store, &fake.secret) {
            Ok(password) => password,
            Err(error) => return fail(&format!("{}: {error}", fake.id)),
        };
        let transport = FakeTransport::with_secret(fake.id.character(), &password);
        let world = ResolvedWorld::with_default_login(fake.id.clone(), fake.secret.clone());
        worlds.push(WorldSetup::new(world, Some(password), Arc::new(transport)));
    }

    let socket = match args.socket.private_socket() {
        Ok(socket) => socket,
        Err(error) => return fail(&error),
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => return fail(&error),
    };
    runtime.block_on(async {
        let shutdown = match mu_server::signals() {
            Ok(shutdown) => shutdown,
            Err(error) => return fail(&error),
        };
        match mu_server::serve(&socket, worlds, shutdown).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        }
    })
}
