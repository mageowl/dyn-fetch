use std::mem;

use serde::{
    Deserialize,
    de::{Unexpected, Visitor},
};

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Style {
    #[serde(default)]
    fg: Color,
    #[serde(default)]
    bg: Color,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    dim: bool,
}

impl Style {
    fn codes(self) -> Vec<u8> {
        let mut codes = Vec::new();
        self.fg.append_foreground(&mut codes);
        self.bg.append_background(&mut codes);
        if self.bold {
            codes.push(1);
        }
        if self.italic {
            codes.push(3);
        }
        if self.dim {
            codes.push(2);
        }
        codes
    }

    pub fn format_start(self) -> String {
        format!(
            "\x1b[{}m",
            self.codes()
                .iter()
                .map(|c| c.to_string() + ";")
                .collect::<String>()
        )
    }
    pub fn format(self, text: &str) -> String {
        format!(
            "\x1b[{}m{text}\x1b[0m",
            self.codes()
                .iter()
                .map(|c| c.to_string() + ";")
                .collect::<String>()
        )
    }
}

pub fn option_format(style: Option<Style>, text: &str) -> String {
    if let Some(style) = style {
        style.format(text)
    } else {
        text.into()
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum Color {
    #[default]
    Default,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Black,
    Hex(u8, u8, u8),
}

impl Color {
    fn ansi_code(&self) -> u8 {
        match self {
            Color::Default => 39,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::Black => 30,
            Color::Hex(_, _, _) => panic!("hex doesn't have a numeric code."),
        }
    }
    pub fn append_foreground(self, vec: &mut Vec<u8>) {
        match self {
            Color::Default => (),
            Color::Hex(r, g, b) => {
                vec.push(38);
                vec.push(r);
                vec.push(g);
                vec.push(b);
            }
            _ => vec.push(self.ansi_code()),
        }
    }
    pub fn append_background(self, vec: &mut Vec<u8>) {
        match self {
            Color::Default => (),
            Color::Hex(r, g, b) => {
                vec.push(48);
                vec.push(r);
                vec.push(g);
                vec.push(b);
            }
            _ => vec.push(self.ansi_code() + 10),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

struct ColorVisitor;
impl Visitor<'_> for ColorVisitor {
    type Value = Color;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a color")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value.starts_with("#") {
            let Ok(r): Result<u8, _> = value[1..=2].parse() else {
                return Err(E::invalid_value(
                    Unexpected::Str(&value[1..=2]),
                    &"a hexidecimal number",
                ));
            };
            let Ok(g): Result<u8, _> = value[3..=4].parse() else {
                return Err(E::invalid_value(
                    Unexpected::Str(&value[1..=2]),
                    &"a hexidecimal number",
                ));
            };
            let Ok(b): Result<u8, _> = value[5..=6].parse() else {
                return Err(E::invalid_value(
                    Unexpected::Str(&value[1..=2]),
                    &"a hexidecimal number",
                ));
            };
            Ok(Color::Hex(r, g, b))
        } else {
            match value {
                "default" => Ok(Color::Default),
                "red" => Ok(Color::Red),
                "green" => Ok(Color::Green),
                "yellow" => Ok(Color::Yellow),
                "blue" => Ok(Color::Blue),
                "magenta" => Ok(Color::Magenta),
                "cyan" => Ok(Color::Cyan),
                "white" => Ok(Color::White),
                "black" => Ok(Color::Black),
                _ => Err(E::unknown_variant(
                    value,
                    &[
                        "red", "green", "yellow", "blue", "magenta", "cyan", "white", "black",
                        "default",
                    ],
                )),
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Text {
    Styled {
        text: String,
        #[serde(flatten)]
        style: Style,
    },
    Unstyled(String),
    Combine(Vec<Text>),
    Empty,
}

impl Text {
    pub fn append(&mut self, v: Text) {
        let vec = match self {
            Self::Combine(vec) => vec,
            _ => {
                let old_self = mem::replace(self, Self::Empty);
                let vec = vec![old_self];
                *self = Self::Combine(vec);
                let Self::Combine(vec) = self else {
                    unreachable!();
                };
                vec
            }
        };
        match v {
            Self::Combine(mut vec2) => vec.append(&mut vec2),
            Self::Empty => (),
            _ => vec.push(v),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Text::Styled { text, style: _ } => text.chars().count(),
            Text::Unstyled(text) => text.chars().count(),
            Text::Combine(vec) => vec.iter().map(Self::len).sum(),
            Text::Empty => 0,
        }
    }
}

impl Into<String> for Text {
    fn into(self) -> String {
        match self {
            Self::Styled { text, style } => style.format(&text),
            Self::Unstyled(s) => s,
            Self::Combine(v) => v.into_iter().map(Into::<String>::into).collect(),
            Self::Empty => String::new(),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::Unstyled(String::new())
    }
}
