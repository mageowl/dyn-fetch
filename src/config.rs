pub mod style;

use std::{env, io, process::Command};

use crate::layout::Layout;
use serde::Deserialize;
use style::{Style, Text};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub art_layout: ArtPosition,
    #[serde(default = "default_art_path")]
    pub art_path: String,
    pub layout: Layout,
    pub info: Vec<Info>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArtPosition {
    #[default]
    None,
    Left,
    Top,
}

fn default_art_path() -> String {
    String::from("art.txt")
}

#[derive(Deserialize)]
pub struct Info {
    #[serde(default)]
    pub label: Text,
    #[serde(flatten)]
    pub value: Value,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    Separator,
    Const {
        text: Text,
    },
    Command {
        cmd: String,
        #[serde(default)]
        args: Vec<String>,
        postfix: Option<Text>,
        style: Option<Style>,
    },
    Nu {
        exec: String,
        postfix: Option<Text>,
        style: Option<Style>,
    },
    Hostname {
        username: Option<Style>,
        hostname: Option<Style>,
        delimiter: Option<Text>,
    },
    EnvVariable {
        name: String,
        style: Option<Style>,
    },
}

impl TryInto<Option<Text>> for Value {
    type Error = io::Error;
    fn try_into(self) -> Result<Option<Text>, Self::Error> {
        match self {
            Self::Separator => Ok(None),
            Self::Const { text } => Ok(Some(text)),
            Self::Command {
                cmd: command,
                args,
                postfix,
                style,
            } => {
                let mut cmd_out = String::from_utf8(
                    Command::new(&command).args(args).output()?.stdout,
                )
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Invalid UTF8 was outputted by the command `{}`.", command),
                    )
                })?;
                cmd_out.truncate(cmd_out.trim_end().len());
                let cmd_out = if let Some(style) = style {
                    Text::Styled {
                        text: cmd_out,
                        style,
                    }
                } else {
                    Text::Unstyled(cmd_out)
                };
                let mut vec = vec![cmd_out];
                if let Some(postfix) = postfix {
                    vec.push(postfix);
                }
                Ok(Some(Text::Combine(vec)))
            }
            Self::Nu {
                exec,
                postfix,
                style,
            } => {
                let mut cmd_out =
                    String::from_utf8(Command::new("nu").args(["-c", &exec]).output()?.stdout)
                        .map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!(
                                    "Invalid UTF8 was outputted by the NuShell expression `{}`.",
                                    exec
                                ),
                            )
                        })?;
                cmd_out.truncate(cmd_out.trim_end().len());
                let cmd_out = if let Some(style) = style {
                    Text::Styled {
                        text: cmd_out,
                        style,
                    }
                } else {
                    Text::Unstyled(cmd_out)
                };
                let mut vec = vec![cmd_out];
                if let Some(postfix) = postfix {
                    vec.push(postfix);
                }
                Ok(Some(Text::Combine(vec)))
            }
            Self::Hostname {
                username: username_style,
                hostname: hostname_style,
                delimiter,
            } => {
                let mut hostname = String::from_utf8(Command::new("hostname").output()?.stdout)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            "Invalid UTF8 was outputted by the command `hostname`.",
                        )
                    })?;
                hostname.truncate(hostname.trim_end().len());
                let hostname = if let Some(style) = hostname_style {
                    Text::Styled {
                        text: hostname,
                        style,
                    }
                } else {
                    Text::Unstyled(hostname)
                };
                let mut username = String::from_utf8(Command::new("whoami").output()?.stdout)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            "Invalid UTF8 was outputted by the command `whoami`.",
                        )
                    })?;
                username.truncate(username.trim_end().len());
                let username = if let Some(style) = username_style {
                    Text::Styled {
                        text: username,
                        style,
                    }
                } else {
                    Text::Unstyled(username)
                };
                Ok(Some(Text::Combine(vec![
                    username,
                    delimiter.unwrap_or_else(|| Text::Unstyled(String::from("@"))),
                    hostname,
                ])))
            }
            Self::EnvVariable { name, style } => {
                let variable = env::var(&name).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Enviornment variable `{name}` was not found."),
                    )
                })?;
                if let Some(style) = style {
                    Ok(Some(Text::Styled {
                        text: variable,
                        style,
                    }))
                } else {
                    Ok(Some(Text::Unstyled(variable)))
                }
            }
        }
    }
}
