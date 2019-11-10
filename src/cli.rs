use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg};

static ABOUT: &str =
    "A tool for finding and selecting VCS revision hashes in your terminal.";

pub fn cli() -> App<'static, 'static> {
    App::new(crate_name!())
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about(ABOUT)
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .help("File to read from. Pass \"-\" to capture stdin."),
        )
}
