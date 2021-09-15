use std::{collections::BTreeMap, fs};

use clap::Clap;
use hashbrown::HashMap;
use image::GenericImageView;
use img_hash::HasherConfig;
use rayon::prelude::*;

#[derive(Clap, Clone, Debug)]
struct Opts {
    path: String,

    /// deactivate dct
    #[clap(long)]
    no_dct: bool,

    /// override resolution (default 10)
    #[clap(short, long)]
    resolution: Option<u32>,
}

fn main() {
    let opts = Opts::parse();

    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> anyhow::Result<()> {
    let images: Vec<_> = fs::read_dir(&opts.path)?
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

    let candidates: Vec<_> = images
        .par_iter()
        .map_init(
            || {
                let r = opts.resolution.unwrap_or(10);
                if opts.no_dct {
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
