mod resolvers;
mod schema;

use open_runo_federation::sdl::parse_service_sdl;

fn main() {
    let parsed =
        parse_service_sdl("open-livekit", schema::SDL).expect("open-livekit SDL must parse");
    println!(
        "open-livekit subgraph loaded: {} types ({:?})",
        parsed.types.len(),
        parsed.types.keys().collect::<Vec<_>>()
    );
}
