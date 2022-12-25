use y2022::day02::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day02_file()?;
    let game = Game::new(file);
    let stats = game.tournament2()?;
    let player = stats.protagonist();
    let score = player.score();
    println!("{score}");
    Ok(())
}
