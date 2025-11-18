use std::env;
use std::path::PathBuf;

fn print_usage() {
    eprintln!("Usage: import_storylets <db_path> <storylet_dir>");
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        print_usage();
        anyhow::bail!("invalid arguments");
    }
    let db_path = &args[1];
    let dir = PathBuf::from(&args[2]);
    if !dir.is_dir() {
        anyhow::bail!("{} is not a directory", dir.display());
    }
    let count = syn_content::import_storylets_from_dir(db_path, &dir)?;
    println!("Imported {} storylets into {}", count, db_path);
    Ok(())
}
