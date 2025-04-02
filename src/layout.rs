use std::io::{self, Write, stdout};

use serde::Deserialize;

use crate::config::{
    Config, Info,
    style::{Style, Text, option_format},
};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Layout {
    Rectangle {
        round_corners: bool,
        border_style: Option<Style>,
    },
    Table {
        round_corners: bool,
        border_style: Option<Style>,
    },
}

pub fn display(config: Config) -> Result<(), std::io::Error> {
    match config.layout {
        Layout::Rectangle {
            round_corners,
            border_style,
        } => display_rectangle(config.info, round_corners, border_style),
        Layout::Table {
            round_corners,
            border_style,
        } => display_table(config.info, round_corners, border_style),
    }
}

fn display_rectangle(
    info: Vec<Info>,
    round_corners: bool,
    border_style: Option<Style>,
) -> io::Result<()> {
    let lines: Vec<_> = info
        .into_iter()
        .map(|mut info| -> io::Result<_> {
            let value: Option<Text> = info.value.try_into()?;
            if let Some(value) = value {
                info.label.append(value);
                Ok(Some(info.label))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<_, _>>()?;
    let max_len = lines
        .iter()
        .map(|l| l.as_ref().map_or(0, |l| l.len()))
        .max()
        .unwrap_or(0);
    let mut stdout = stdout();
    write!(
        stdout,
        "{}{}{}{}\x1b[0m\n",
        border_style.map_or_else(String::new, Style::format_start),
        if round_corners { "╭" } else { "┌" },
        "─".repeat(max_len + 2),
        if round_corners { "╮" } else { "┐" }
    )?;
    for line in lines {
        if let Some(line) = line {
            write!(stdout, "{} ", option_format(border_style, "│"))?;
            let len = line.len();
            let string: String = line.into();
            stdout.write(string.as_bytes())?;
            write!(
                stdout,
                "{} {}\n",
                " ".repeat(max_len - len),
                option_format(border_style, "│")
            )?;
        } else {
            write!(
                stdout,
                "{}├{}┤\x1b[0m\n",
                border_style.map_or_else(String::new, Style::format_start),
                "─".repeat(max_len + 2)
            )?;
        }
    }
    write!(
        stdout,
        "{}{}{}{}\x1b[0m\n",
        border_style.map_or_else(String::new, Style::format_start),
        if round_corners { "╰" } else { "└" },
        "─".repeat(max_len + 2),
        if round_corners { "╯" } else { "┘" }
    )?;
    Ok(())
}

fn display_table(
    info: Vec<Info>,
    round_corners: bool,
    border_style: Option<Style>,
) -> io::Result<()> {
    let lines: Vec<_> = info
        .into_iter()
        .map(|info| -> io::Result<_> {
            let value: Option<Text> = info.value.try_into()?;
            if let Some(value) = value {
                Ok(Some((info.label, value)))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<_, _>>()?;
    let max_len0 = lines
        .iter()
        .map(|l| l.as_ref().map_or(0, |l| l.0.len()))
        .max()
        .unwrap_or(0);
    let max_len1 = lines
        .iter()
        .map(|l| l.as_ref().map_or(0, |l| l.1.len()))
        .max()
        .unwrap_or(0);
    let mut stdout = stdout();
    write!(
        stdout,
        "{}{}{}┬{}{}\x1b[0m\n",
        border_style.map_or_else(String::new, Style::format_start),
        if round_corners { "╭" } else { "┌" },
        "─".repeat(max_len0 + 2),
        "─".repeat(max_len1 + 2),
        if round_corners { "╮" } else { "┐" }
    )?;
    for line in lines {
        if let Some((label, value)) = line {
            write!(stdout, "{} ", option_format(border_style, "│"))?;
            let len0 = label.len();
            let label: String = label.into();
            stdout.write(label.as_bytes())?;
            write!(
                stdout,
                "{} {} ",
                " ".repeat(max_len0 - len0),
                option_format(border_style, "│")
            )?;
            let len1 = value.len();
            let value: String = value.into();
            stdout.write(value.as_bytes())?;
            write!(
                stdout,
                "{} {}\n",
                " ".repeat(max_len1 - len1),
                option_format(border_style, "│")
            )?;
        } else {
            write!(
                stdout,
                "{}├{}┼{}┤\x1b[0m\n",
                border_style.map_or_else(String::new, Style::format_start),
                "─".repeat(max_len0 + 2),
                "─".repeat(max_len1 + 2)
            )?;
        }
    }
    write!(
        stdout,
        "{}{}{}┴{}{}\x1b[0m\n",
        border_style.map_or_else(String::new, Style::format_start),
        if round_corners { "╰" } else { "└" },
        "─".repeat(max_len0 + 2),
        "─".repeat(max_len1 + 2),
        if round_corners { "╯" } else { "┘" }
    )?;
    Ok(())
}
