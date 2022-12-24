use y2022::day01::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day01_file()?;
    let elfs = ElvesReader::new(file);
    let cals = elfs.max_by_cal()?.cals();
    println!("{cals}");
    Ok(())
}
