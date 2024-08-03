//! # Text Interpolator
//!
//! `text_interpolator` is an object that takes input text that possibly contains templates (user
//! configurable) and maps them to substitutions.
//! To do so it uses modular functions to check if a word in the text is a
//! template, extract it, and then map it to it's substitute.
//!
//! It also supports nested templates requiring recursion to reach a valid substitute.

pub mod defaults;

use core::fmt;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct NestedTemplateLoopError;

impl fmt::Display for NestedTemplateLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "detected infinitely looping nested templates")
    }
}

#[derive(Debug)]
pub struct TemplateSplit<'a> {
    pub prefix: &'a str,
    pub template: &'a str,
    pub suffix: &'a str,
}

pub type IsTemplateFn = fn(&str) -> bool;
pub type ExtractTemplateFn = fn(&str) -> TemplateSplit;

pub struct TextInterpolator {
    pub is_template: IsTemplateFn,
    pub extract_template: ExtractTemplateFn,
    template_set: HashSet<String>,
}

impl Default for TextInterpolator {
    /// Creates a TextInterpolator with a default is_template function and default extract_template
    /// function
    ///
    /// # Examples
    ///
    /// ```
    /// use text_interpolator::TextInterpolator;
    ///
    /// let text_interpolator = TextInterpolator::default();
    /// ```
    fn default() -> Self {
        TextInterpolator {
            is_template: defaults::is_template,
            extract_template: defaults::extract_template,
            template_set: HashSet::new(),
        }
    }
}

impl TextInterpolator {
    pub fn new(is_template: IsTemplateFn, extract_template: ExtractTemplateFn) -> Self {
        TextInterpolator {
            is_template,
            extract_template,
            template_set: HashSet::new(),
        }
    }

    pub fn interp(
        &mut self,
        text: &str,
        map: &impl Fn(&str) -> Option<String>,
    ) -> Result<String, NestedTemplateLoopError> {
        // String will be at least as long as input
        let mut output = String::with_capacity(text.len());

        for item in text.split_whitespace() {
            let template_split = (self.extract_template)(item);

            match map(template_split.template) {
                Some(substitute) => {
                    if !self
                        .template_set
                        .insert(template_split.template.to_string())
                    {
                        return Err(NestedTemplateLoopError);
                    }

                    let mut substitution = substitute;

                    if self.contains_template(&substitution) {
                        substitution = self.interp(&substitution, map)?;
                    }

                    self.template_set.remove(template_split.template);

                    output.push_str(template_split.prefix);
                    output.push_str(&substitution);
                    output.push_str(template_split.suffix);
                    output.push(' ');
                }
                None => {
                    output.push_str(item);
                    output.push(' ');
                }
            }
        }

        // Remove trailing space
        output.pop();

        Ok(output)
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
                    "I've never seen someone 'verb with a 'noun before.",
                    "You are too 'adj to be 'adj..",
                ][1]
                .to_string(),
            ),
            "paragraph" => Some(["'sentence 'sentence 'sentence"][0].to_string()),
            "infinite" => Some("'infinite".to_string()),
            "nonexistantnest" => Some("'nothing".to_string()),
            _ => None,
        }
    }

    #[test]
    fn interpolate_non_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from(
            "This is an example of a basic input with no templates to be substituted.",
        );
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert_eq!(&text, &interpolated_text.unwrap());
    }

    #[test]
    fn interpolate_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("A 'adj 'noun will always 'verb in the morning.");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolate_templated_text_2() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("I'm 'verb'ing on some 'adj 'noun's right now.");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_nested_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'sentence");
        let interpolated_text = interpolator.interp(&text, &map_template);
        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'paragraph");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text_with_prefix_and_suffix() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("My Story:'paragraph...");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn missing_template() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'klsfjkaejfaeskfjl");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert_eq!("'klsfjkaejfaeskfjl", &interpolated_text.unwrap());
    }

    #[test]
    fn missing_nested_template() {
        let mut interpolator = TextInterpolator::default();
        let interp_text = interpolator.interp("'nonexistantnest", &map_template);
        dbg!(interp_text.unwrap());
    }

    #[test]
    fn infinite_self_recursion() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'infinite");

        let interpolated_text = interpolator.interp(&text, &map_template);

        assert!(&interpolated_text.is_err());
    }
}
