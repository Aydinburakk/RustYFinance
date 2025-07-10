use rustyfinance::download;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frames = download("MAVI.IS", "2024-06-01", "2024-07-01", "1d")?;

    for frame in frames {
        println!(
            "{} | O: {:.2} H: {:.2} L: {:.2} C: {:.2} V: {}",
            frame.date, frame.open, frame.high, frame.low, frame.close, frame.volume
        );
    }

    Ok(())
}
