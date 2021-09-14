use std::{collections::BTreeMap, fs};

use clap::Clap;
use hashbrown::HashMap;
use image::GenericImageView;
use img_hash::HasherConfig;

#[derive(Clap, Clone, Debug)]
struct Opts {
    path: String,
}

fn main() {
    let opts = Opts::parse();

    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> anyhow::Result<()> {
    let mut by_hash = HashMap::new();
    let hasher = HasherConfig::new().to_hasher();
    let images = fs::read_dir(&opts.path)?;

    for image_entry in images {
        let path = image_entry?.path();
        if !path.is_file() {
            continue;
        }

        let image = image::open(&path)?;
        let hash = hasher.hash_image(&image);
        by_hash
            .entry(hash)
            .or_insert_with(BTreeMap::new)
            .entry(image.dimensions())
            .or_insert_with(Vec::new)
            .push(path);
    }

    for set in by_hash.values().filter(|&x| x.len() > 1) {
        println!("\ncollision:");
        for ((x, y), paths) in set {
            println!("  {} x {}", x, y);
            for path in paths {
                println!("    {}", path.display());
            }
        }
    }

    Ok(())
}
