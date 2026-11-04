pub mod backend;
pub mod context;
pub mod control;
pub mod output;
pub mod session;

pub use context::*;
pub use session::*;

pub fn runtime_hook() {
    evcxr::runtime_hook();
}

#[cfg(test)]
mod tests {
    use crate::output::append_mime_output;
    use mars_core::Output;
    use serde_json::{Map, Value};

    #[test]
    fn promotes_plotters_svg_html_to_a_mime_bundle() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String(
                    "<div style=\"\"><svg xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"4\" cy=\"4\" r=\"4\"/></svg></div>"
                        .to_owned(),
                ),
            )]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::MimeBundle { data, .. }]
                if data.contains_key("image/svg+xml")
                    && data.contains_key("text/plain")
                    && !data.contains_key("text/html")
        ));
        let Output::MimeBundle { data, .. } = &outputs[0] else {
            return;
        };
        assert!(
            data["image/svg+xml"]
                .as_str()
                .is_some_and(|svg| svg.starts_with("<svg"))
        );
    }

    #[test]
    fn keeps_image_mime_bundles_untouched() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([
                (
                    "image/png".to_owned(),
                    Value::String("iVBORw0KGgo=".to_owned()),
                ),
                (
                    "text/html".to_owned(),
                    Value::String("<svg><circle/></svg>".to_owned()),
                ),
            ]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::MimeBundle { data, .. }]
                if data.contains_key("image/png") && data.contains_key("text/html")
        ));
    }

    #[test]
    fn leaves_html_with_surrounding_text_as_html() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String("<p>Inline icon</p><svg><circle/></svg>".to_owned()),
            )]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::Html { html }] if html.contains("Inline icon")
        ));
    }

    #[test]
    fn still_collapses_single_text_and_html_outputs() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([("text/plain".to_owned(), Value::String("42".to_owned()))]),
        );
        assert!(matches!(outputs.as_slice(), [Output::Text { text }] if text == "42"));

        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String("<b>safe</b>".to_owned()),
            )]),
        );
        assert!(matches!(outputs.as_slice(), [Output::Html { html }] if html == "<b>safe</b>"));
    }
}
