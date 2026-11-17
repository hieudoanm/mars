---
title: "Sales title: "Machine Learning Project: Sales & Demand Forecasting" Demand Forecasting: Time Series Project"
description: "Forecast retail sales time series using moving averages, exponential smoothing, and ARIMA-style models in Rust."
author: "Project Mars Team"
date: "2026-11-14"
order: 41
tags: ["machine-learning", "rust", "time-series", "forecasting", "stats"]
---

# Machine Learning Project: Sales & Demand Forecasting

Demand forecasting answers: _how much of each product will we sell next
week?_ It's the ML problem behind inventory planning, staffing, and
pricing — and it looks deceptively simple until you meet real data with
seasonality, trends, and promotions.

We'll use a classic retail sales dataset (store × item × date × units)
and forecast daily sales.

In this project you will:

1. Understand **time series components**: trend, seasonality, noise.
2. Build **moving averages** and **exponential smoothing** baselines.
3. Implement a simple **ARIMA-style** model (AR + I + MA).
4. Evaluate with RMSE and MASE — the honest time-series metrics.

> Rust ML note: `ndarray` for windows, `chrono` for dates, or the
> `smartcore` time-series helpers.

## Time series anatomy

Every sales series decomposes into:

```
sales(t) = trend(t) + seasonal(t) + residual(t)
```

- **Trend**: long-term growth or decline
- **Seasonality**: repeating weekly/yearly patterns (weekend spikes,
  holiday rushes)
- **Residual**: promotions, weather, one-off events — the unpredictable part

```rust
// :dep ndarray = "0.16"
// :dep chrono = { version = "0.4", features = ["serde"] }
// :dep csv = "1"

use ndarray::{s, Array1};
use chrono::NaiveDate;

// Load daily sales: date,store,item,units
#[derive(serde::Deserialize)]
struct Sale {
    date: NaiveDate,
    #[serde(rename = "store")] _store: u32,
    #[serde(rename = "item")]  _item: u32,
    units: f64,
}

fn load_daily(path: &str) -> Vec<(NaiveDate, f64)> {
    let mut by_date: std::collections::BTreeMap<NaiveDate, f64> =
        Default::default();
    let rdr = csv::Reader::from_path(path).unwrap();
    for rec in rdr.deserialize::<Sale>() {
        let s = rec.unwrap();
        *by_date.entry(s.date).or_insert(0.0) += s.units;
    }
    by_date.into_iter().collect()
}

let series = load_daily("sales.csv");
let dates: Vec<NaiveDate> = series.iter().map(|(d,_)| *d).collect();
let y = Array1::from_vec(series.iter().map(|(_,u)| *u).collect());

println!("{} days, mean {:.0} units/day", y.len(), y.mean().unwrap());
```

## Visualize before modeling

Always plot the series first. Look for:

- **Trend**: is sales growing/declining over months?
- **Seasonality**: weekly dips (Mondays), annual peaks (December)?
- **Outliers**: promotion days, stockouts, data errors.

```rust
// ASCII sparkline of weekly totals — no plotting lib needed
fn sparkline(values: &[f64], width: usize) -> String {
    let chars = ['▁','▂','▃','▄','▅','▆','▇','█'];
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1e-9);
    values.iter().take(width).map(|&v| {
        let idx = ((v - min) / range * 7.0) as usize;
        chars[idx.min(7)]
    }).collect()
}

// Resample to weekly totals
let weekly: Vec<f64> = y
    .chunks(7)
    .map(|w| w.iter().sum())
    .collect();
println!("weekly sales: {}", sparkline(&weekly, 60));
```

## Baseline 1 — moving average

The simplest forecaster: predict tomorrow = mean of last _k_ days.

- **SMA(7)**: smooths noise, lags on trends
- **EMA(α)**: exponentially weighted — recent days count more

These are your baselines. Any fancy model must beat them.

```rust
fn moving_average(y: &Array1<f64>, window: usize) -> Vec<f64> {
    (window..y.len()).map(|i| {
        y.slice(s![i-window..i]).mean().unwrap()
    }).collect()
}

fn ema(y: &Array1<f64>, alpha: f64) -> Vec<f64> {
    let mut out = vec![y[0]];
    for i in 1..y.len() {
        out.push(alpha * y[i] + (1.0 - alpha) * out[i-1]);
    }
    out[1..].to_vec()
}

// One-step-ahead predictions on the last 30 days
let test_start = y.len() - 30;
let actual: Vec<f64> = (test_start..y.len()).map(|i| y[i]).collect();

let sma7 = moving_average(&y.slice(s![..test_start]), 7);
let sma_preds: Vec<f64> = (0..30).map(|i| sma7[sma7.len()-30+i]).collect();

let ema_preds = ema(&y.slice(s![..test_start]), 0.3);
let ema_last30: Vec<f64> = ema_preds[ema_preds.len()-30..].to_vec();

fn rmse(preds: &[f64], actual: &[f64]) -> f64 {
    (preds.iter().zip(actual).map(|(p,a)| (p-a).powi(2)).sum::<f64>()
     / actual.len() as f64).sqrt()
}
println!("SMA-7 RMSE: {:.1}", rmse(&sma_preds, &actual));
println!("EMA   RMSE: {:.1}", rmse(&ema_last30, &actual));
```

## Baseline 2 — seasonal naive

If your data has weekly seasonality, the best simple forecast is often
**"same day last week"**: predict Monday = last Monday's sales. This
captures the weekly pattern that moving averages smooth away.

```rust
// Seasonal naive: forecast t as y[t-7]
let seasonal_preds: Vec<f64> = (test_start..y.len())
    .map(|i| y[i - 7])
    .collect();

println!("seasonal-naive RMSE: {:.1}", rmse(&seasonal_preds, &actual));

// For daily retail data, seasonal-naive usually beats SMA/EMA
// because it preserves the day-of-week pattern.
```

## ARIMA-style model

**ARIMA(p,d,q)** decomposes forecasting into three parts:

- **AR(p)**: predict from _p_ past values — `y_t = c + φ₁y_{t-1} + ...`
- **I(d)**: integrate _d_ times to remove trend — work on differences
- **MA(q)**: model shocks from _q_ past forecast errors

A full ARIMA fit is complex (requires MLE for coefficients). We'll use a
simplified **AR(p) on differenced data** — good enough for weekly
seasonality removed.

```rust
// Deseasonalize: subtract weekly seasonal component
let seasonal: Vec<f64> = (0..7).map(|dow| {
    y.iter().skip(dow).step_by(7)
        .cloned().sum::<f64>() / (y.len() / 7) as f64
}).collect();

let deseasonalized: Vec<f64> = y.iter().enumerate()
    .map(|(i, &v)| v - seasonal[i % 7])
    .collect();

// AR(2) on differenced deseasonalized series:
// Δy_t = c + φ₁Δy_{t-1} + φ₂Δy_{t-2} + ε_t
// Fit by least squares — build lag matrix and solve
fn ar_fit(series: &[f64], p: usize) -> Vec<f64> {
    let n = series.len() - p;
    // X[i] = [1, y_{i+p-1}-y_{i+p-2}, ..., y_{i+1}-y_{i}]
    // target = y_{i+p} - y_{i+p-1}
    let mut x = ndarray::Array2::zeros((n, p + 1));
    let mut target = ndarray::Array1::zeros(n);
    for i in 0..n {
        x[[i, 0]] = 1.0;
        for lag in 1..=p {
            x[[i, lag]] = series[i + p - lag] - series[i + p - lag - 1];
        }
        target[i] = series[i + p] - series[i + p - 1];
    }
    // least squares: β = (X'X)⁻¹X'y
    let xt_x = x.t().dot(&x);
    let xt_y = x.t().dot(&target);
    ndarray_linalg::solve::Inverse::inv(&xt_x).unwrap().dot(&xt_y).to_vec()
}

let coefs = ar_fit(&deseasonalized, 2);
println!("AR(2) coefficients: {:?}", coefs);
```

## Forecasting with the AR model

Once you have coefficients, forecast iteratively:

```
ŷ_{t+1} = ŷ_t + c + φ₁(ŷ_t - ŷ_{t-1}) + φ₂(ŷ_{t-1} - ŷ_{t-2})
```

Then **re-seasonalize**: add back the day-of-week component.
For multi-step forecasts, errors compound — that's why horizon matters.

```rust
fn ar_forecast(history: &[f64], coefs: &[f64], horizon: usize) -> Vec<f64> {
    let p = coefs.len() - 1;
    let mut buf = history.to_vec();
    let mut preds = Vec::with_capacity(horizon);

    for _ in 0..horizon {
        let n = buf.len();
        let mut pred = coefs[0];   // intercept
        for lag in 1..=p {
            pred += coefs[lag] * (buf[n - lag] - buf[n - lag - 1]);
        }
        let next = buf[n - 1] + pred;   // integrated: add back level
        preds.push(next);
        buf.push(next);
    }
    preds
}

// Forecast next 7 days, reseasonalize
let history: Vec<f64> = deseasonalized.iter()
    .rev().take(30).cloned().collect::<Vec<_>>().into_iter().rev().collect();
let mut fc = ar_forecast(&history, &coefs, 7);
for (i, f) in fc.iter_mut().enumerate() {
    *f += seasonal[(history.len() + i) % 7];
}
println!("next-7-day forecast: {:?}", fc.iter()
    .map(|v| format!("{:.0}", v)).collect::<Vec<_>>());
```

## Evaluation — MASE, not just RMSE

For intermittent demand (many zeros), RMSE can be misleading. **MASE**
(Mean Absolute Scaled Error) compares your model to the seasonal-naive
baseline:

```
MASE = MAE(model) / MAE(seasonal_naive)
```

- MASE < 1 → your model beats the baseline
- MASE > 1 → the baseline was better; don't deploy

This is the honest metric for time series — it answers "is my model
actually adding value?

## Key takeaways

1. **Plot first, model second** — seasonality and outliers change everything.
2. **Baselines are mandatory**: seasonal-naive, SMA, EMA — beat them or
   don't ship.
3. **ARIMA is interpretable** but real-world forecasting often uses
   gradient boosting on lag features (LightGBM) or deep models (N-BEATS,
   TFT) for production accuracy.
4. **MASE > RMSE** for comparing forecasters across series.

**Next project:** sentiment analysis — text classification where the
target is an opinion, not a number.
