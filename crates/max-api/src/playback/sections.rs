use serde_json::Value;

const SKIP: &str = "skip";
const END_CREDITS: &str = "end-credits";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// An opening or a recap, which the service offers to skip under the section's own label.
    Skippable,
    Credits,
}

/// A stretch of a title the service points out, timed from the start of the title itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub kind: SectionKind,
    pub label: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
}

impl Section {
    pub fn contains(&self, position_seconds: f64) -> bool {
        (self.start_seconds..self.end_seconds).contains(&position_seconds)
    }
}

/// The answer times its annotations from the start of everything it lists, the clips before
/// the title included, so the title's own start is taken off.
pub(super) fn sections_of(main_video: &Value) -> Vec<Section> {
    let title_start = main_video["start"].as_f64().unwrap_or(0.0);
    let section = |annotation: &Value| {
        let kind = match annotation["type"].as_str()? {
            SKIP => SectionKind::Skippable,
            END_CREDITS => SectionKind::Credits,
            _ => return None,
        };
        Some(Section {
            kind,
            label: annotation["label"].as_str().unwrap_or_default().to_string(),
            start_seconds: (annotation["start"].as_f64()? - title_start).max(0.0),
            end_seconds: annotation["end"].as_f64()? - title_start,
        })
    };
    let mut sections: Vec<Section> = main_video["annotations"].as_array().into_iter().flatten().filter_map(section).collect();
    sections.sort_by(|earlier, later| earlier.start_seconds.total_cmp(&later.start_seconds));
    sections
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn times_sections_from_the_start_of_the_title_and_puts_them_in_order() {
        let main = json!({"start": 43.0, "annotations": [
            {"type": "end-credits", "start": 3377.5, "end": 3437.5, "label": "Next Episode", "secondaryType": ""},
            {"type": "skip", "start": 48.0, "end": 123.0, "label": "Skip Intro", "secondaryType": "intro"},
            {"type": "something-new", "start": 1.0, "end": 2.0}
        ]});
        let sections = sections_of(&main);
        assert_eq!(sections.len(), 2);
        assert_eq!(
            (sections[0].kind, sections[0].label.as_str(), sections[0].start_seconds, sections[0].end_seconds),
            (SectionKind::Skippable, "Skip Intro", 5.0, 80.0)
        );
        assert_eq!((sections[1].kind, sections[1].start_seconds), (SectionKind::Credits, 3334.5));
        assert!(sections[0].contains(5.0) && !sections[0].contains(80.0));
    }

    #[test]
    fn a_title_without_annotations_has_no_sections() {
        assert!(sections_of(&json!({"start": 0})).is_empty());
    }
}
