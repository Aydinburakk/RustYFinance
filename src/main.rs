use reqwest::blocking::Client;
use reqwest::header::{USER_AGENT, HeaderMap};
use std::error::Error;
use serde::Deserialize;
use serde_json;
use chrono::{NaiveDate, Utc, TimeZone};
#[derive(Debug, Deserialize)]
struct ApiResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    open: Vec<Option<f64>>,
    high: Vec<Option<f64>>,
    low: Vec<Option<f64>>,
    close: Vec<Option<f64>>,
    volume: Vec<Option<u64>>,
}
#[derive(Debug)]
struct Frame {
    date: String,
    open: f64,
    high:f64,
    low:f64,
    close: f64,
    volume:u64
}

fn download(symbol:&str,start_date:&str,end_date:&str,interval:&str)-> Result<Vec<Frame>,Box<dyn Error>>{
    // date part // // // // // // // // // // // // // // // // // // // //
    let today=Utc::now().date_naive();
    let start_naive = if start_date.is_empty(){
        today
    } else {
        NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?
    };
    let end_naive = if end_date.is_empty() {
        today - chrono::Duration::days(365)
    } else {
        NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?
    };
    let start = Utc
        .from_utc_datetime(&start_naive.and_hms_opt(0, 0, 0).unwrap())
        .timestamp();
    let end = Utc
        .from_utc_datetime(&end_naive.and_hms_opt(0, 0, 0).unwrap())
        .timestamp();
    // date part // // // // // // // // // // // // // // // // // // // //
    let day_range = (start_naive - end_naive).num_days().abs(); // mutlak gün farkı
    // interval part // // // // // // // // // // // // // // // // // // //
    match interval {
        "1m" if day_range > 8 => {
            return Err(format!(
                "Interval '1m' supports a maximum of 8 days. You requested: {} days.",
                day_range
            ).into());
        }
        "2m" | "5m" | "15m" | "30m" | "90m" if day_range > 60 => {
            return Err(format!(
                "Interval '{}' supports a maximum of 60 days. You requested: {} days.",
                interval, day_range
            ).into());
        }
        "60m" if day_range > 730 => {
            return Err(format!(
                "Interval '60m' supports a maximum of 730 days. You requested: {} days.",
                day_range
            ).into());
        }
        _ => {}
    }
    let interval=if interval.is_empty(){"1d"}else{interval};

    let valid_intervals = [
    "1m", "2m", "5m", "15m", "30m",
    "60m", "90m", "1d", "5d", "1wk", "1mo", "3mo",
    ];
    if !valid_intervals.contains(&interval) {
        let valid_list = valid_intervals.join(", ");
        return Err(format!(
            "Invalid interval: '{}'. Valid intervals {}.",
            interval, valid_list
        ).into());
    }
    // interval part // // // // // // // // // // // // // // // // // // //

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval={}",
        symbol, start, end, interval
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/115.0.0.0 Safari/537.36".parse()?,
    );
    let client = Client::builder()
    .default_headers(headers)
    .build()?;

    let response = client.get(&url).send()?.text()?;
    let parsed: ApiResponse = serde_json::from_str(&response)?;

    let result = &parsed.chart.result[0];
    let quote = &result.indicators.quote[0];

    let mut frames = Vec::new();

    for i in 0..result.timestamp.len() {
        if let (
            Some(open), 
            Some(high),
            Some(low),
            Some(close),
            Some(volume)) = (
                quote.open[i], 
                quote.high[i],
                quote.low[i],
                quote.close[i],
                quote.volume[i]) {
            let dt = Utc.timestamp_opt(result.timestamp[i], 0).unwrap();
            let date = dt.format("%Y-%m-%d").to_string();

            frames.push(Frame {
                date,
                open,
                high,
                low,
                close,
                volume,
            });
        }
    }

    Ok(frames)

}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frames = download(
        "MAVI.IS",
         "2024-06-01",
          "2024-07-01",
           "1m")?;
    for frame in frames {
        println!(
            "{} | O: {:.2} H: {:.2} L: {:.2} C: {:.2} V: {}",
            frame.date, frame.open, frame.high, frame.low, frame.close, frame.volume
        );
    }
    
    Ok(())
}
