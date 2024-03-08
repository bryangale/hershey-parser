use anyhow::{anyhow, Result};
use std::cmp;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HersheyFont {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
    glyphs: Vec<HersheyGlyph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HersheyGlyph {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
    pub paths: Vec<Vec<Edge>>,
}

#[derive(thiserror::Error, Debug)]
pub enum HersheyFontNewError {
    #[error("{1}")]
    ParseError(#[source] Box<dyn std::error::Error>, String),
}

#[derive(thiserror::Error, Debug)]
pub enum HersheyFontGetGlyphError {
    #[error("{0}")]
    GlyphNotFound(String),
}

impl HersheyFont {
    pub fn new(data: &str) -> Result<HersheyFont, HersheyFontNewError> {
        let glyphs = data
            .split('\n')
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
            .map(|(i, line)| match line_to_hershey_glyph(line) {
                Ok(glyph) => Ok(glyph),
                Err(e) => Err(HersheyFontNewError::ParseError(
                    e.into(),
                    format!("Error parsing line {}", i + 1),
                )),
            })
            .collect::<Result<Vec<_>, HersheyFontNewError>>()?;

        let points_iter = glyphs
            .iter()
            .flat_map(|glyph| glyph.paths.clone())
            .flatten();

        let (top, right, bottom, left) = points_iter.fold(
            (i32::MAX, i32::MIN, i32::MIN, i32::MAX),
            |(accum_top, accum_right, accum_bottom, accum_left), edge| {
                (
                    cmp::min(accum_top, edge.y),
                    cmp::max(accum_right, edge.x),
                    cmp::max(accum_bottom, edge.y),
                    cmp::min(accum_left, edge.x),
                )
            },
        );

        Ok(HersheyFont {
            top,
            right,
            bottom,
            left,
            glyphs,
        })
    }

    pub fn get_glyph(&self, glyph: char) -> Result<&HersheyGlyph, HersheyFontGetGlyphError> {
        self.glyphs
            .get((glyph as usize) - 32)
            .ok_or(HersheyFontGetGlyphError::GlyphNotFound(format!(
                "Glyph {} not found in font",
                glyph
            )))
    }
}

fn line_to_hershey_glyph(line: &str) -> Result<HersheyGlyph> {
    if line.len() < 10 {
        return Err(anyhow!("Invalid glyph data"));
    }

    let contents = &line[5..];

    let num_pairs = (&contents[..3].trim().parse::<i32>()? - 1) as usize;

    let left = char_to_int(&contents.chars().nth(3).unwrap());
    let right = char_to_int(&contents.chars().nth(4).unwrap());

    if contents.len() != 5 + num_pairs * 2 {
        return Err(anyhow!("Invalid glyph data"));
    }

    let mut top = i32::MAX;
    let mut bottom = i32::MIN;

    let mut paths = Vec::new();
    let mut path = Vec::new();

    for i in 0..num_pairs {
        let pair = &contents[5 + (i * 2)..7 + (i * 2)];

        if pair == " R" && !path.is_empty() {
            paths.push(path);
            path = Vec::new()
        } else {
            let x = char_to_int(&pair.chars().next().unwrap());
            let y = char_to_int(&pair.chars().nth(1).unwrap());

            top = cmp::min(top, y);
            bottom = cmp::max(bottom, y);

            path.push(Edge { x, y })
        }
    }

    if !path.is_empty() {
        paths.push(path);
    }

    Ok(HersheyGlyph {
        top,
        right,
        bottom,
        left,
        paths,
    })
}

fn char_to_int(char: &char) -> i32 {
    (*char as i32) - ('R' as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_error_if_data_is_invalid() {
        let result = HersheyFont::new(" ");

        assert!(matches!(result, Err(HersheyFontNewError::ParseError(_, _))));
    }

    #[test]
    fn get_glyph_works() {
        let font = HersheyFont::new("  720  3G][BIb").unwrap();
        let glyph = font.get_glyph(' ');

        assert!(matches!(glyph, Ok(_)));
    }

    #[test]
    fn get_glyph_returns_error_if_glyph_is_not_found() {
        let font = HersheyFont::new("  720  3G][BIb").unwrap();
        let result = font.get_glyph('A');

        assert!(matches!(
            result,
            Err(HersheyFontGetGlyphError::GlyphNotFound(_))
        ));
    }

    #[test]
    fn char_to_int_works() {
        assert_eq!(char_to_int(&'R'), 0);
        assert_eq!(char_to_int(&'M'), -5);
        assert_eq!(char_to_int(&'W'), 5);
        assert_eq!(char_to_int(&'Q'), -1);
    }

    #[test]
    fn line_to_hershey_glyph_works() {
        let glyph = line_to_hershey_glyph("  720  3G][BIb").unwrap();

        assert_eq!(
            glyph,
            HersheyGlyph {
                top: -16,
                right: 11,
                bottom: 16,
                left: -11,
                paths: vec![vec![Edge { x: 9, y: -16 }, Edge { x: -9, y: 16 }]]
            }
        );
    }

    #[test]
    fn line_to_hershey_glyph_returns_error_if_glyph_data_is_invalid() {
        let result = line_to_hershey_glyph("");

        assert!(result.is_err());
    }
}
