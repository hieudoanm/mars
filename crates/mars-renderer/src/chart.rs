use crate::util::escape_html;

pub fn render_chart_svg(spec: &serde_json::Value) -> String {
    let values = spec
        .get("data")
        .and_then(|data| data.get("values"))
        .and_then(serde_json::Value::as_array);
    let Some(values) = values else {
        return "<pre>Chart data is unavailable</pre>".to_owned();
    };
    let mark = spec
        .get("mark")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("point");
    let encoding = spec.get("encoding").and_then(serde_json::Value::as_object);
    let x_field = encoding
        .and_then(|value| value.get("x"))
        .and_then(|value| value.get("field"))
        .and_then(serde_json::Value::as_str);
    let y_field = encoding
        .and_then(|value| value.get("y"))
        .and_then(|value| value.get("field"))
        .and_then(serde_json::Value::as_str);
    let Some(y_field) = y_field else {
        return "<pre>Chart requires an encoded y field</pre>".to_owned();
    };
    let y_values = values
        .iter()
        .filter_map(|value| value.get(y_field).and_then(serde_json::Value::as_f64))
        .collect::<Vec<_>>();
    if y_values.is_empty() {
        return "<pre>Chart data has no numeric y values</pre>".to_owned();
    }
    let max = y_values.iter().copied().fold(0.0_f64, f64::max).max(1.0);
    let width = 720.0;
    let height = 240.0;
    let left = 42.0;
    let bottom = 24.0;
    let plot_width = width - left - 12.0;
    let plot_height = height - bottom - 12.0;
    let points = y_values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = left + (index as f64 / y_values.len().max(1) as f64) * plot_width;
            let y = 12.0 + plot_height - (value / max) * plot_height;
            (x, y, value)
        })
        .collect::<Vec<_>>();
    let shapes = match mark {
        "bar" => points
            .iter()
            .map(|(x, y, value)| {
                let bar_width = (plot_width / y_values.len().max(1) as f64 * 0.7).max(2.0);
                format!(
                    "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bar_width:.1}\" height=\"{height:.1}\" fill=\"#f97316\"><title>{value}</title></rect>",
                    height = 12.0 + plot_height - y,
                )
            })
            .collect::<String>(),
        "line" | "point" => {
            let path = points
                .iter()
                .map(|(x, y, _)| format!("{x:.1},{y:.1}"))
                .collect::<Vec<_>>()
                .join(" ");
            let line = if mark == "line" {
                format!("<polyline points=\"{path}\" fill=\"none\" stroke=\"#fb923c\" stroke-width=\"3\"/>")
            } else {
                String::new()
            };
            let circles = points
                .iter()
                .map(|(x, y, value)| {
                    format!("<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"#f97316\"><title>{value}</title></circle>")
                })
                .collect::<String>();
            format!("{line}{circles}")
        }
        _ => "<text x=\"42\" y=\"30\" fill=\"#fca5a5\">Unsupported chart mark</text>".to_owned(),
    };
    let x_label = x_field.map_or(String::new(), |field| {
        format!(
            "<text x=\"{left}\" y=\"{height}\" fill=\"#a3a09b\">{}</text>",
            escape_html(field)
        )
    });
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"chart-svg\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Offline chart\"><line x1=\"{left}\" y1=\"12\" x2=\"{left}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/><line x1=\"{left}\" y1=\"{plot_bottom}\" x2=\"{width}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/>{shapes}{x_label}</svg>",
        plot_bottom = 12.0 + plot_height,
    )
}
