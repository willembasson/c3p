use atty::{is, Stream};
use clap::Parser;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input to operate on (Use - if you pipe from StdIn)
    input: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Output to operate on (Use - to pipe to StdOut)
    output: Option<String>,
}

#[derive(Debug)]
enum InputKind {
    OrdinaryFile(PathBuf),
    StdIn,
    Url(url::Url),
    S3Bucket(String),
    ScpSource(String),
}

#[derive(Debug)]
struct Input {
    kind: InputKind,
}

fn to_input(input: String) -> Input {
    let input_url = Url::parse(input.as_str());
    if input_url.is_ok() {
        if input.starts_with("s3://") {
            // println!("S3 !!");
            Input {
                kind: InputKind::S3Bucket(input),
            }
        } else if input.starts_with("scp://") {
            Input {
                kind: InputKind::ScpSource(input),
            }
        } else {
            // println!("Probably a url");
            Input {
                kind: InputKind::Url(input_url.unwrap()),
            }
        }
    } else if input == "-" {
        if is(Stream::Stdin) {
            println!("You said StdIn but you didn't pipe or redirect anything");
        }
        Input {
            kind: InputKind::StdIn,
        }
    } else if input.contains('@') && input.contains(':') {
        Input {
            kind: InputKind::ScpSource(input),
        }
    } else {
        Input {
            kind: InputKind::OrdinaryFile(PathBuf::from(input)),
        }
    }
}

#[derive(Debug)]
enum OutputKind {
    OrdinaryFile(PathBuf),
    StdOut,
    Url(url::Url),
    S3Bucket(String),
    ScpTarget(String),
}

#[derive(Debug)]
struct Output {
    kind: OutputKind,
}

fn to_output(output: String) -> Output {
    let output_url = Url::parse(output.as_str());
    if output_url.is_ok() {
        if output.starts_with("s3://") {
            Output {
                kind: OutputKind::S3Bucket(output),
            }
        } else if output.starts_with("scp://") {
            Output {
                kind: OutputKind::ScpTarget(output),
            }
        } else {
            Output {
                kind: OutputKind::Url(output_url.unwrap()),
            }
        }
    } else if output == "-" {
        if is(Stream::Stdout) {
            println!("You said StdOut but you didn't pipe or redirect anything");
        }
        Output {
            kind: OutputKind::StdOut,
        }
    } else if output.contains('@') && output.contains(':') {
        Output {
            kind: OutputKind::ScpTarget(output),
        }
    } else {
        Output {
            kind: OutputKind::OrdinaryFile(PathBuf::from(output)),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(input) = cli.input.as_deref() {
        let input_string = input.to_string();
        let input: Input = to_input(input_string);
        println!("{:#?}", input.kind);
    }
    if let Some(output) = cli.output.as_deref() {
        let output_string = output.to_string();
        let output: Output = to_output(output_string);
        println!("{:#?}", output.kind);
    }
    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }
}
