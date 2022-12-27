use y2022::day03::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day03_file()?;
    let rucksacks = RucksackReader::new(file);
    let sum = rucksacks.common_sum()?;
    println!("{sum}");
    Ok(())
}
