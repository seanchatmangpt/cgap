//! journal-export CLI: sealed journal dir -> verified IPC windows ->
//! parquet block export. Fail-closed; typed refusals to stderr, exit 1.

use journal_export::parquet_sink;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: journal-export <sealed-dir> <out.parquet>");
        std::process::exit(2);
    }
    let dir = std::path::Path::new(&args[1]);
    let out = std::path::Path::new(&args[2]);
    match parquet_sink::export_segment_dir(dir, out) {
        Ok(x) => {
            println!(
                "exported rows={} bytes={} to {}",
                x.rows,
                x.bytes,
                out.display()
            );
        }
        Err(e) => {
            eprintln!("REFUSED: {e:?}");
            std::process::exit(1);
        }
    }
}
