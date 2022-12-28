use audiotags::AnyTag;
use log::{info, warn};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum FormatError {
    #[error("Attribute `{0}` was requested but not found")]
    AttributeMissing(String),
    #[error("Could not compile format")]
    InvalidFormat,
}

pub(crate) fn format_name<S>(tags: &AnyTag, format: S) -> Result<String, FormatError>
where
    S: ToString,
{
    let format = format.to_string();
    let mut format = format.chars();
    let mut name = "".to_string();

    while let Some(next) = format.next() {
        let to_add = if next == '%' {
            if let Some(character) = format.next() {
                parse_part(character, tags)?
            } else {
                "&".to_string()
            }
        } else {
            next.to_string()
        };

        name.push_str(&to_add);
    }

    Ok(name)
}

fn parse_part(key: char, tags: &AnyTag) -> Result<String, FormatError> {
    let (tag_type, value) = match key {
        'a' => ("Artist", tags.artists().map(|a| a.join(", "))),
        'A' => ("Album artists", tags.album_artists().map(|a| a.join(", "))),
        't' => ("Title", tags.title().map(|t| t.to_string())),
        'b' => ("Album", tags.album_title().map(|t| t.to_string())),
        'y' => ("Date", tags.year().map(|y| y.to_string())),
        'n' => (
            "Track number",
            tags.track_number().map(|t| format!("{:02}", t)),
        ),
        'g' => ("Genre", tags.genre().map(|g| g.to_string())),
        'c' => ("Composer", tags.composer().map(|c| c.to_string())),
        'd' => ("Disc", tags.disc_number().map(|d| d.to_string())),
        ' ' => ("", Some("% ".to_string())),
        '%' => ("", Some("%".to_string())),
        c => {
            warn!("Unexpected character while parsing format: `%{}`", c);
            Err(FormatError::InvalidFormat)?
        }
    };

    match value {
        Some(value) => {
            info!("Formatting `%{}` to `{}`", key, value);
            Ok(value)
        }
        None => {
            warn!("Attribute `{}` was missing", tag_type);
            Err(FormatError::AttributeMissing(tag_type.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audiotags::{AudioTagEdit, Mp4Tag, ToAnyTag};

    #[test]
    fn title_and_track_number() {
        let mut tag = Mp4Tag::default();
        tag.set_track_number(1);
        tag.set_title("Cool music");

        let result = format_name(&tag.to_anytag(), "%n - %t").unwrap();
        assert_eq!(result, "01 - Cool music".to_string());
    }

    #[test]
    fn tag_missing_result_to_error() {
        assert_eq!(
            format_name(&Mp4Tag::default().to_anytag(), "%n - %t"),
            Err(FormatError::AttributeMissing("Track number".to_string()))
        );
    }

    #[test]
    fn trailing_percentage() {
        let mut tag = Mp4Tag::default();
        tag.set_track_number(1);
        tag.set_title("Cool music");

        let result = format_name(&tag.to_anytag(), "%n - %t &").unwrap();
        assert_eq!(result, "01 - Cool music &".to_string());
    }

    #[test]
    fn double_percentage() {
        let mut tag = Mp4Tag::default();
        tag.set_track_number(1);
        tag.set_title("Cool music");

        let result = format_name(&tag.to_anytag(), "%n %% %t").unwrap();
        assert_eq!(result, "01 % Cool music".to_string());
    }

    #[test]
    fn single_percentage_with_space() {
        let mut tag = Mp4Tag::default();
        tag.set_track_number(1);
        tag.set_title("Cool music");

        let result = format_name(&tag.to_anytag(), "%n % %t").unwrap();
        assert_eq!(result, "01 % Cool music".to_string());
    }
}
