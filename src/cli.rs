use clap::{App, AppSettings, Arg, SubCommand};

static ABOUT: &'static str =
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
