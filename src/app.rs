use std::ffi::OsStr;

use clap::{App, AppSettings, Arg, SubCommand};

use error::Result;

const TEMPLATE: &'static str = "\
{bin} {version}
{author}
{about}

USAGE:
    {usage}

SUBCOMMANDS:
{subcommands}

OPTIONS:
{unified}";

const TEMPLATE_SUB: &'static str = "\
{before-help}
USAGE:
    {usage}

ARGS:
{positionals}

OPTIONS:
{unified}";

const ABOUT: &'static str = "
rucd is a tool for searching and browser the Unicode character database.

Project home page: https://github.com/BurntSushi/rucd";

const ABOUT_SEARCH: &'static str = "\
This sub-command provides a convenient interface for searching codepoints.
";

/// Build a clap application.
pub fn app() -> App<'static, 'static> {
    // Subcommands.
    let cmd_search = SubCommand::with_name("search")
        .author(crate_authors!())
        .version(crate_version!())
        .template(TEMPLATE_SUB)
        .about("Search the Unicode character database.")
        .before_help(ABOUT_SEARCH)
        .arg(Arg::with_name("allow-large")
            .long("allow-large")
            .short("A")
            .help("Allow a large result set to be printed."))
        .arg(Arg::with_name("case-sensitive")
            .long("case-sensitive")
            .short("s")
            .help("Case sensitive search. By default, search is case \
                   insensitive."))
        .arg(Arg::with_name("pattern")
            .help("A regular expression to apply against all character \
                   names."));
    let cmd_list_properties = SubCommand::with_name("list-properties")
        .author(crate_authors!())
        .version(crate_version!())
        .template(TEMPLATE_SUB)
        .about("Print the names (and aliases) of all Unicode properties.");
    let cmd_list_property_values =
        SubCommand::with_name("list-property-values")
        .author(crate_authors!())
        .version(crate_version!())
        .template(TEMPLATE_SUB)
        .about("Print the values (and aliases) of a single Unicode property.")
        .arg(Arg::with_name("property")
            .help("The property to show.")
            .required(true));

    // The actual App.
    App::new("rucd")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .template(TEMPLATE)
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .subcommand(cmd_search)
        .subcommand(cmd_list_properties)
        .subcommand(cmd_list_property_values)
}

pub fn arg_to_str(name: &str, value: Option<&OsStr>) -> Result<String> {
    let value = match value {
        None => return err!("missing argument '{}'", name),
        Some(value) => value,
    };
    match value.to_str() {
        None => err!("argument '{}' is not valid UTF-8", name),
        Some(value) => Ok(value.to_string()),
    }
}
