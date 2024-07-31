use core::fmt;

const RECURSION_LIMIT: u8 = 25;

#[derive(Debug, Clone)]
pub struct RecursionLimitReachedError;

impl fmt::Display for RecursionLimitReachedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recursion limit reached")
    }
}

#[derive(Debug)]
pub struct TemplateSplit<'a> {
    pub prefix: &'a str,
    pub template: &'a str,
    pub suffix: &'a str,
}

pub struct TextInterpolator {
    pub is_template: fn(&str) -> bool,
    pub extract_template: fn(&str) -> TemplateSplit,
    times_recursed: u8,
}

impl TextInterpolator {
    pub fn new() -> Self {
        TextInterpolator {
            is_template,
            extract_template,
            times_recursed: 0,
        }
    }

    pub fn interp(
        &mut self,
        text: &str,
        map: &impl Fn(&str) -> Option<String>,
    ) -> Result<String, RecursionLimitReachedError> {
        let mut output = String::with_capacity(text.len());

        for item in text.split_whitespace() {
            let mut substitution: String;
            let template_split = (self.extract_template)(item);

            match map(template_split.template) {
                Some(substitute) => {
                    if self.times_recursed >= RECURSION_LIMIT {
                        self.times_recursed = 0;
                        return Err(RecursionLimitReachedError);
                    }
                    substitution = substitute;
                    while self.contains_template(&substitution) {
                        self.times_recursed += 1;
                        substitution = self.interp(&substitution, map)?;
                    }
                    substitution =
                        template_split.prefix.to_owned() + &substitution + template_split.suffix;
                }
                None => {
                    substitution = item.to_string();
                }
            }

            output.push_str(&substitution);
            output.push(' ');
        }

        self.times_recursed = 0;

        Ok(output.trim().to_string())
    }

    pub fn contains_template(&self, text: &str) -> bool {
        for item in text.split_whitespace() {
            if (self.is_template)(item) {
                return true;
            }
        }
        false
    }
}

pub fn is_template(text: &str) -> bool {
    match text.chars().next() {
        Some(c) => c == '\'',
        None => false,
    }
}

pub fn extract_template<'a>(embedded_template: &'a str) -> TemplateSplit<'a> {
    let prefix: &str;
    let template: &str;
    let suffix: &str;

    match embedded_template.split_once('\'') {
        Some(split) => match split.1.split_once(|ch: char| !ch.is_alphanumeric()) {
            Some(inner_split) => {
                prefix = split.0;
                template = inner_split.0;
                if inner_split.1.is_empty() {
                    suffix = &split.1[split.1.len() - 1..];
                } else {
                    suffix = inner_split.1;
                }
            }
            None => {
                prefix = split.0;
                template = split.1;
                suffix = "";
            }
        },
        None => {
            prefix = "";
            template = "";
            suffix = "";
        }
    };

    TemplateSplit {
        prefix,
        template,
        suffix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_template(template: &str) -> Option<String> {
        match template {
            "verb" => Some(["run", "fall", "fly", "swim"][0].to_string()),
            "noun" => Some(["person", "place", "thing"][1].to_string()),
            "adj" => Some(["funny", "interesting", "aggrivating"][2].to_string()),
            "sentence" => Some(
                [
                    "A 'adj 'noun should never 'verb..",
                    "I've never seen someone 'verb on a 'noun before.",
                    "You are too 'adj to be 'adj..",
                ][1]
                .to_string(),
            ),
            "paragraph" => Some(["'sentence 'sentence 'sentence"][0].to_string()),
            "infinite" => Some("'infinite".to_string()),
            _ => None,
        }
    }

    #[test]
    fn interpolate_standard_text() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from(
            "This is an example of a basic input with no templates to be substituted.",
        );
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert_eq!(&text, &interpolated_text.unwrap());
    }

    #[test]
    fn interpolate_templated_text() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("A 'adj 'noun will always 'verb in the morning.");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolate_templated_text_2() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("I'm 'verb'ing on some 'adj 'noun's right now.");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_nested_templated_text() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("'sentence");
        let interpolated_text = interpolator.interp(&text, &map_template);
        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("'paragraph");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text_with_prefix_and_suffix() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("My Story:'paragraph...");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn missing_template() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("'klsfjkaejfaeskfjl");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert_eq!("'klsfjkaejfaeskfjl", &interpolated_text.unwrap());
    }

    #[test]
    fn infinite_self_recursion() {
        let mut interpolator = TextInterpolator::new();

        let text: String = String::from("'infinite");

        let interpolated_text = interpolator.interp(&text, &map_template);

        assert!(&interpolated_text.is_err());
    }

    #[test]
    fn template_extration() {
        let extrated_template = extract_template("['adj.'..'.]");
        dbg!(&extrated_template);
        assert_eq!("[", extrated_template.prefix);
        assert_eq!("'..'.]", extrated_template.suffix);
        assert_eq!("adj", extrated_template.template);
    }

    #[test]
    fn template_extration_2() {
        let extrated_template = extract_template("'noun's");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("s", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_3() {
        let extrated_template = extract_template("noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("", extrated_template.template);
    }

    #[test]
    fn template_extration_4() {
        let extrated_template = extract_template("'noun.");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!(".", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_5() {
        let extrated_template = extract_template("'noun!");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("!", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_6() {
        let extrated_template = extract_template("'noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_7() {
        let extrated_template = extract_template("'noun'noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("noun", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }
}
