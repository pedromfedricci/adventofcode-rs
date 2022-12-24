use y2022::day01::*;

fn main() {
    if let Err(err) = try_main() {
        println!("{err}");
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let file = day01_file()?;
    let elfs = ElvesReader::new(file);
    let sum = elfs.sum_top(3)?;
    println!("{sum}");
    Ok(())
}
