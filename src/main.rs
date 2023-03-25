use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::LineSeries,
    style::{Color, BLACK, BLUE, GREEN, RED, WHITE},
};

use rayon::prelude::*;

fn derivative_y(x: f64, y: f64) -> f64 {
    y.cbrt() + x
}

#[derive(Debug, Default, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Into<Point> for (f64, f64) {
    fn into(self) -> Point {
        Point {
            x: self.0,
            y: self.1,
        }
    }
}

fn decide_bounds(x_range: (f64, f64), y_range: (f64, f64)) -> (f64, f64, f64, f64) {
    (
        x_range.0 - 1.0,
        x_range.1 + 1.0,
        y_range.0 - 1.0,
        y_range.1 + 1.0,
    )
}

fn is_degenerate(x: f64) -> bool {
    match x.classify() {
        std::num::FpCategory::Nan => true,
        std::num::FpCategory::Infinite => true,
        _ => false,
    }
}

struct EndCondition {
    max_x: Option<f64>,
    max_abs_y: Option<f64>,
}

impl EndCondition {
    fn has_reached(&self, current: &Point) -> bool {
        if self.max_x.map_or(false, |max_x| current.x > max_x) {
            true
        } else if self
            .max_abs_y
            .map_or(false, |max_y| current.y.abs() > max_y)
        {
            true
        } else {
            false
        }
    }
}

fn create_dataset(
    start: Point,
    step_size: f64,
    end_condition: EndCondition,
    derivative_y: impl Fn(f64, f64) -> f64,
) -> Vec<(f64, f64)> {
    let mut current = start;

    let mut points = vec![];

    while !end_condition.has_reached(&current)
        && !is_degenerate(current.x)
        && !is_degenerate(current.y)
    {
        points.push((current.x, current.y));

        current.y += derivative_y(current.x, current.y) * step_size;
        current.x += step_size;
    }

    points
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start: Point = (0.0, 5.0).into();

    let delta = 0.001;

    let start_x = 0.0;
    let y_spread = 10.0;
    let num_datasets = 10;

    let datasets: Vec<_> = (0..num_datasets)
        .into_par_iter()
        .map(|i| {
            let end_condition = EndCondition {
                max_x: Some(150.0),
                max_abs_y: Some(150.0),
            };

            let start = (start_x, 0.0 + i as f64 * y_spread).into();
            create_dataset(start, delta, end_condition, derivative_y)
        })
        .collect();

    let max_x = datasets
        .iter()
        .flatten()
        .map(|a| a.0)
        .reduce(f64::max)
        .unwrap();

    let min_y = datasets
        .iter()
        .flatten()
        .map(|a| a.1)
        .reduce(f64::min)
        .unwrap();

    let max_y = datasets
        .iter()
        .flatten()
        .map(|a| a.1)
        .reduce(f64::max)
        .unwrap();

    let (left_bound, right_bound, bottom_bound, top_bound) =
        decide_bounds((start_x, max_x), (min_y, max_y));

    let root = BitMapBackend::new("output.png", (1280, 960)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(left_bound..right_bound, bottom_bound..top_bound)?;

    chart.configure_mesh().draw()?;

    let colors = vec![&RED, &BLACK, &BLUE, &GREEN];

    for (i, points) in datasets.iter().enumerate() {
        chart
            .draw_series(LineSeries::new(
                points.iter().copied(),
                colors[i % colors.len()],
            ))?
            .label("graph");
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
