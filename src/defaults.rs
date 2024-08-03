use crate::TemplateSplit;

/// Checks if a string is a template.
///
/// The default implementation considers a string starting with an apostrophe to be a template.
///
/// Returns None if the string is empty or does not contain an apostrophe as the first character.
///
/// # Examples
///
/// ```
/// use text_interpolator::defaults::is_template;
///
/// let template = "'template";
/// let not_template = "not_template";
///
/// assert!(is_template(template));
/// assert!(!is_template(not_template));
/// ```
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

    #[test]
    fn template_extration_with_prefix_and_suffix() {
        let extrated_template = extract_template("['adj.'..'.]");
        dbg!(&extrated_template);
        assert_eq!("[", extrated_template.prefix);
        assert_eq!("'..'.]", extrated_template.suffix);
        assert_eq!("adj", extrated_template.template);
    }

    #[test]
    fn template_extration_with_punctuation_and_suffix() {
        let extrated_template = extract_template("'noun's");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("s", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_no_template() {
        let extrated_template = extract_template("noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("", extrated_template.template);
    }

    #[test]
    fn template_extration_with_ending_punctuation() {
        let extrated_template = extract_template("'noun.");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!(".", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_ending_punctuation_2() {
        let extrated_template = extract_template("'noun!");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("!", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_no_suffix_or_prefix() {
        let extrated_template = extract_template("'noun");
        dbg!(&extrated_template);

        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_nested_template() {
        let extrated_template = extract_template("'noun'noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("noun", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }
}
