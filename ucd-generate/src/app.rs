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
ucd-generate is a tool that generates Rust source files containing various
Unicode tables.

Unicode tables are typically represented by finite state transducers, which
permits fast searching while simultaneously compressing the table.

Project home page: https://github.com/BurntSushi/rucd";

const ABOUT_JAMO_SHORT_NAME: &'static str = "\
jamo-short-name parses the UCD's Jamo.txt file and emits its contents as a
slice table. The slice consists of a sorted sequences of pairs, where each
pair corresponds to the codepoint and the Jamo_Short_Name property value.

When emitted as an FST table, the FST corresponds to a map from a Unicode
codepoint (encoded as a big-endian u32) to a u64, where the u64 contains the
Jamo_Short_Name property value. The value is encoded in the least significant
bytes (up to 3).

Since the table is so small, the slice table is faster to search.
";

const ABOUT_TEST_UNICODE_DATA: &'static str = "\
test-unicode-data parses the UCD's UnicodeData.txt file and emits its contents
on stdout. The purpose of this command is to diff the output with the input and
confirm that they are identical. This is a sanity test on the UnicodeData.txt
parser.
";

/// Build a clap application.
pub fn app() -> App<'static, 'static> {
    // Various common flags and arguments.
    let flag_name = |default| {
        Arg::with_name("name")
            .help("Set the name of the table in the emitted code.")
            .long("name")
            .takes_value(true)
            .default_value(default)
    };
    let flag_rust_fst = Arg::with_name("rust-fst")
        .long("rust-fst")
        .help("Emit the table as a FST in Rust source codeto stdout.");
    let flag_raw_fst = Arg::with_name("raw-fst")
        .long("raw-fst")
        .help("Emit the table as a raw FST to stdout.\n\
               Pro-tip: Run `cargo install fst-bin` to install the `fst`
               command line tool, which can be used to search the FST.");
    let flag_rust_slice = Arg::with_name("rust-slice")
        .long("rust-slice")
        .help("Emit the table as a static slice that can be binary searched.");
    let ucd_dir = Arg::with_name("ucd-dir")
        .required(true)
        .help("Directory containing the Unicode character database files.");

    // Subcommands.
    let cmd_jamo_short_name = SubCommand::with_name("jamo-short-name")
        .author(crate_authors!())
        .version(crate_version!())
        .template(TEMPLATE_SUB)
        .about("Create the Jamo_Short_Name property table.")
        .before_help(ABOUT_JAMO_SHORT_NAME)
        .arg(ucd_dir.clone())
        .arg(flag_name("JAMO_SHORT_NAME"))
        .arg(flag_rust_slice.clone())
        .arg(flag_rust_fst.clone())
        .arg(flag_raw_fst.clone());

    let cmd_test_unicode_data = SubCommand::with_name("test-unicode-data")
        .author(crate_authors!())
        .version(crate_version!())
        .template(TEMPLATE_SUB)
        .about("Test the UnicodeData.txt parser.")
        .before_help(ABOUT_TEST_UNICODE_DATA)
        .arg(ucd_dir.clone());

    // The actual App.
    App::new("ucd-generate")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .template(TEMPLATE)
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .subcommand(cmd_jamo_short_name)
        .subcommand(cmd_test_unicode_data)
}
