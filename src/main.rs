use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct MonthPrice {
    open: String,
    date: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(std::fs::File::open("./snp500.csv")?);
    let mut prices = Vec::new();
    for row in reader.deserialize() {
        let month_price: MonthPrice = row?;
        let month_price_float = str::parse::<f64>(&month_price.open.replace(',', ""))?;
        prices.push((month_price_float, month_price.date));
    }

    // Make sure no rows are missing.
    // Jan 1985 - Sep 2022 => (2022-1985)*12 + 9 = 453
    const EXPECTED_LEN: usize = 453;
    const TEN_YRS_IN_MONTHS: usize = 120;

    assert_eq!(prices.len(), EXPECTED_LEN);

    let mut num_ls_better = 0;
    let mut num_dca_better = 0;
    let mut worst_ls = 0f64;
    let mut best_ls = 0f64;
    let mut diff_growth_sum = 0f64;

    let mut start_idx = EXPECTED_LEN - 1; // 10yr window
    let mut end_idx = start_idx - TEN_YRS_IN_MONTHS;

    // Lump sum: growth = prices[end_idx]/prices[start_idx]
    // DCA: growth = (prices[end_idx]/12) * Σ(1/prices[start_idx-n], n=[0,11])
    loop {
        let ls_growth = prices[end_idx].0 / prices[start_idx].0;
        let dca_growth = (prices[end_idx].0 / 12f64)
            * prices[start_idx - 11..=start_idx]
                .iter()
                .map(|p| 1f64 / p.0)
                .fold(0f64, |a, p| a + p);
        let pct_diff_growth = 100f64 * (ls_growth / dca_growth - 1f64);
        if pct_diff_growth < 0f64 {
            num_dca_better += 1;
        } else {
            num_ls_better += 1;
        }

        if pct_diff_growth < worst_ls {
            worst_ls = pct_diff_growth;
        }
        if pct_diff_growth > best_ls {
            best_ls = pct_diff_growth;
        }

        let diff_growth = 100f64 * (ls_growth - dca_growth);
        diff_growth_sum += diff_growth;

        println!(
            "{}, {ls_growth}, {dca_growth}, {pct_diff_growth}, {diff_growth}",
            prices[start_idx].1.replace(',', "")
        );
        start_idx -= 1;
        end_idx = end_idx.wrapping_sub(1);
        if end_idx == usize::MAX {
            break;
        }
    }
    println!("ls better: {num_ls_better}, dca better: {num_dca_better}, best: {best_ls}, worst: {worst_ls}");
    println!("diff growth: {diff_growth_sum}");
    Ok(())
}
