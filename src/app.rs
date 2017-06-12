use clap::{App, AppSettings, Arg, SubCommand};

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
        .arg(Arg::with_name("pattern")
            .help("A regular expression to apply against all character \
                   names."));

    // The actual App.
    App::new("rucd")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .template(TEMPLATE)
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .subcommand(cmd_search)
}
