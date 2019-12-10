#[macro_use]
extern crate error_chain;
use rust_decimal::Decimal;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::io::{stdout, Write};

use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::Terminal;

// {
//   "BCH_THB": {
//     "avg24hr": "6325",
//     "baseVolume": "0.27633587786259542",
//     "high24hr": "6550",
//     "highestBid": "6150",
//     "last": "6100",
//     "low24hr": "6100",
//     "lowestAsk": "6549.99",
//     "percentChange": "-3.557312252964426877",
//     "quoteVolume": "1720.000000000000001"
//   }
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MarketCap {
    avg24hr: String,
    base_volume: Decimal,
    high24hr: String,
    highest_bid: String,
    last: Decimal,
    low24hr: String,
    lowest_ask: String,
    percent_change: Decimal,
    quote_volume: Decimal,
}

fn get() -> Result<HashMap<String, MarketCap>> {
    let req_url = "https://api.tdax.com/api/marketcap/";
    let response = reqwest::blocking::get(req_url)?;

    let mkt_cap: HashMap<String, MarketCap> = response.json()?;
    Ok(mkt_cap)
}

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
    }
}

fn run() -> Result<()> {
    // Terminal initialization
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    write!(stdout, "{}{}", termion::clear::All, termion::style::Reset,)?;

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let data = get()?;

    // to a 2 dp string, padding with whitespace
    let dec_str = |d: Decimal| -> String { format!("{: >10}", d.round_dp(2).to_string()) };
    let percent_str = |d: Decimal| -> String { format!("{: >5}", d.round_dp(2).to_string()) };

    let header = ["Pair", "Price", "%", "Vol.", "Value"];
    let mut items: Vec<(Style, Vec<String>)> = data
        .iter()
        .map(|(pair, mkt)| {
            let color = match mkt.percent_change {
                p if p < Decimal::new(-3, 0) => Color::Red,
                p if p < Decimal::new(-1, 0) => Color::LightRed,
                p if p > Decimal::new(1, 0) => Color::LightGreen,
                p if p > Decimal::new(3, 0) => Color::Green,
                _ => Color::White,
            };

            let m = match mkt.percent_change {
                p if p < Decimal::new(-3, 0) => Modifier::BOLD,
                p if p > Decimal::new(3, 0) => Modifier::BOLD,
                _ => Modifier::SLOW_BLINK,
            };

            (
                Style::default().fg(color).modifier(m),
                vec![
                    String::from(pair),
                    dec_str(mkt.last),
                    percent_str(mkt.percent_change),
                    dec_str(mkt.base_volume),
                    dec_str(mkt.quote_volume),
                ],
            )
        })
        .collect();

    // sort by pair - [0]
    items.sort_by(|(_, a), (_, b)| a.cmp(b));

    // draw
    terminal.draw(|mut f| {
        let rows = items
            .iter()
            .enumerate()
            .map(|(_, (style, item))| Row::StyledData(item.into_iter(), *style));

        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
            .split(f.size());

        Table::new(header.iter(), rows)
            .block(Block::default().borders(Borders::ALL).title("MarketCap"))
            .widths(&[
                Constraint::Percentage(25),
                Constraint::Percentage(20),
                Constraint::Length(10),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .render(&mut f, rects[0]);
    })?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        match *error.kind() {
            _ => println!("Other error: {:?}", error),
        }
    }
}
