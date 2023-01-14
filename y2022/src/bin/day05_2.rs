use y2022::day05::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day05_file()?;
    let (mut platform, lifts) = drawing(file);
    platform.try_lifts(lifts)?;
    let answer = platform.collect_top_row::<String>();
    println!("{answer}");
    Ok(())
}
