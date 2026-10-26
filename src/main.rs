#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() -> std::io::Result<()> {
    let mut demo = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--demo" => demo = true,
            "-h" | "--help" => {
                println!(
                    "Flappy Jack, a midnight flight\n\nUsage: flappy-jack [--demo]\n\n--demo  Deterministic attract mode, never touches your best score\n\nSpace / click flap, Space start/retry, R retry, P/Esc pause, Q quit\nA native Freya window with a live procedural 3D jack-o'-lantern."
                );
                return Ok(());
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown argument {argument:?}, use --help"),
                ));
            }
        }
    }
    flappy_jack::window::launch_game(demo);
    Ok(())
}
