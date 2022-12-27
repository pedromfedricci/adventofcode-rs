use y2022::day04::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day04_file()?;
    let reader = SectionsPairReader::new(file);
    let count = reader.overlaped_pairs()?;
    println!("{count}");
    Ok(())
}
