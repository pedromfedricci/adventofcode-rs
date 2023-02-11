use y2022::day06::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day06_file()?;
    let source = DataSource::new(file)?;
    let windows = source.windows();
    let position = start_of_packet(windows)?;
    println!("{position}");
    Ok(())
}
