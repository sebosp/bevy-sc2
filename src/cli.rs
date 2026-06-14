use bevy::prelude::*;
use clap::Parser;

#[derive(Default, Resource, bevy::reflect::Reflect)]
pub struct CliParams {
    pub path: String,
    pub ids: String,
}

/// Parse cli args.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path where the caches have been downloaded to.
    /// A user should have already clicked on download caches.
    /// This is set in the Config section of Swarmy.
    #[arg(short, long)]
    pub path: String,
    /// A comma-separated list of caches to inspect for the map data.
    /// When a replay is loaded it contains multiple ids for different purposes.
    /// I assume there's only one t3HeightMap and only one MapInfo sector.
    #[arg(short, long)]
    pub ids: String,
}
