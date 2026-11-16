//! The verb `mu server`: the secrets, the socket directory, then `mu-server`.

use std::io::Write as _;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Args;
use mu_core::{
    EnvSecretStore, FakeTransport, ResolvedWorld, SecretRef, WorldId, WorldSetup, load_password,
};

use crate::socket::{SocketArgs, current_uid, private_dir};

/// Arguments of `mu server`.
#[derive(Clone, Debug, Default, Args)]
pub struct ServerArgs {
    #[command(flatten)]
    socket: SocketArgs,

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
    // A closed stderr leaves nothing to report to.
    let _ = writeln!(std::io::stderr(), "mu server: {error}");
    ExitCode::from(1)
}

/// Runs the server until a signal stops it.
pub fn run(args: &ServerArgs) -> ExitCode {
    let store = EnvSecretStore::new();
    let mut worlds = Vec::new();
    for fake in &args.fake_worlds {
        let password = match load_password(&store, &fake.secret) {
            Ok(password) => password,
            Err(error) => return fail(&format!("{}: {error}", fake.id)),
        };
        let transport = FakeTransport::with_secret(fake.id.character(), &password);
        let world = ResolvedWorld::with_default_login(fake.id.clone(), fake.secret.clone());
        worlds.push(WorldSetup::new(world, Some(password), Arc::new(transport)));
    }

    let socket = args.socket.resolve();
    let dir = socket.parent().unwrap_or_else(|| std::path::Path::new("/"));
    if let Err(error) = private_dir(dir, current_uid()) {
        return fail(&error);
    }

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
