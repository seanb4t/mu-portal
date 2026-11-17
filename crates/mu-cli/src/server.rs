//! The verb `mu server`: the secrets, the socket directory, then `mu-server`.

use std::process::ExitCode;
use std::sync::Arc;

use clap::Args;
use mu_core::{
    EnvSecretStore, FakeTransport, ResolvedWorld, SecretRef, WorldId, WorldSetup, load_password,
};

use crate::config::ConfigArgs;
use crate::socket::SocketArgs;

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
    // Only the server logs: the lines would break the screen of the client.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::WARN)
        .try_init();
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
