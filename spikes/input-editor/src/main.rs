mod app;
mod bench;
mod editor;
mod keys;
mod sample;

fn usage() -> ! {
    eprintln!("usage: input-editor bench | run [--log FILE] | sample [BYTES]");
    std::process::exit(2);
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("bench") => bench::run(),
        Some("run") => {
            let log = match args.get(1).map(String::as_str) {
                Some("--log") => Some(args.get(2).cloned().unwrap_or_else(|| usage())),
                Some(_) => usage(),
                None => None,
            };
            app::run(log)?;
        }
        Some("sample") => {
            let bytes = args
                .get(1)
                .map_or(4096, |b| b.parse().unwrap_or_else(|_| usage()));
            print!("{}", sample::pose(bytes));
        }
        _ => usage(),
    }
    Ok(())
}
