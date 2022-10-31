use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use clap::Parser;
use hashbrown::{HashMap, HashSet};
use image::GenericImageView;
use rayon::prelude::*;
use visual_hash::HasherConfig;

#[derive(Clone, Debug, Parser)]
struct Args {
    path: Option<String>,

    /// paths to ignore
    ///
    /// Store your ignore list in any old text file.
    #[clap(short, long)]
    ignore: Option<String>,

    /// deactivate dct
    #[clap(long)]
    no_dct: bool,

    /// override resolution (default 10)
    #[clap(short, long)]
    resolution: Option<u32>,
}

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> anyhow::Result<()> {
    let path = match args.path.as_ref() {
        Some(path) => Cow::from(Path::new(path)),
        None => Cow::from(std::env::current_dir()?),
    };

    let mut images: Vec<_> = fs::read_dir(&path)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if let Some(ignore) = &args.ignore {
        apply_ignore(&mut images, ignore)?;
    }

    let candidates: Vec<_> = images
        .par_iter()
        .map_init(
            || {
                let r = args.resolution.unwrap_or(10);
                if args.no_dct {
                    HasherConfig::new().hash_size(r, r).to_hasher()
                } else {
                    HasherConfig::new()
                        .hash_size(r, r)
                        .preproc_dct()
                        .to_hasher()
                }
            },
            |hasher, path| {
                image::open(&path).map(|image| {
                    let hash = hasher.hash_image(&image);
                    (image.dimensions(), path, hash)
                })
            },
        )
        .flatten()
        .collect();

    let mut by_hash = HashMap::new();
    candidates.into_iter().for_each(|(dimensions, path, hash)| {
        by_hash
            .entry(hash)
            .or_insert_with(BTreeMap::new)
            .entry(dimensions)
            .or_insert_with(Vec::new)
            .push(path);
    });

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

fn apply_ignore(images: &mut Vec<PathBuf>, ignore: &str) -> io::Result<()> {
    let text = fs::read_to_string(ignore)?;
    let ignored: HashSet<_> = text.lines().map(Path::new).collect();
    images.retain(|entry| !ignored.contains(&**entry));
    Ok(())
}
