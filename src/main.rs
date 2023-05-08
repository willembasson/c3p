use atty::{is, Stream};
use clap::Parser;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(name = "C3P(No O)")]
#[command(bin_name = "c3p")]
#[command(author = "Willem B. <willem.basson@gmail.com>")]
#[command(about = "
 🤖　 　　,,''´ ￣ ヽ
　　 　　| |__　 _　|
　 　 　 {{‐'(👁 )Y(👁 )}
  　 　　 !l_l__V^`r'/
　 　　　 ~lr､i_ﾆ_l,'
　　,. r-‐‐]l===l[‐--,r- ､
　 〉､l!　　　｀Y´o　　l!ﾞ,
. //　〉､＿＿L＿＿/il〈.　ﾍ
//　/ }　,'´￣｀ヽ＿{ ﾊV_,ﾍ


  ___  ____  ____
 / __)( __ \\(  _ \\
( (__  (__ ( ) __/
 \\___)(____/(__)")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input to operate on (Use - if you pipe from StdIn)
    input: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output to operate on (Use - to pipe to StdOut)
    output: Option<String>,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
    } else {
        println!("No input defined");
    }

    if let Some(output) = cli.output.as_deref() {
        let output_string = output.to_string();
        let output: Output = to_output(output_string);
        println!("{:#?}", output.kind);
    } else {
        println!("No output defined");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_input_s3() {
        let bucket = "s3://some_bucket";
        let input = to_input(bucket.to_string());
        assert_eq!(InputKind::S3Bucket(bucket.to_string()), input.kind);
    }

    #[test]
    fn test_to_input_stdin() {
        let input = to_input("-".to_string());
        assert_eq!(InputKind::StdIn, input.kind);
    }

    #[test]
    fn test_to_input_scp() {
        let source = "some_user@some_host:~/";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::ScpSource(source.to_string()), input.kind);
    }

    #[test]
    fn test_to_input_file() {
        let source = "/some/path/file.txt";
        let input = to_input(source.to_string());
        assert_eq!(
            InputKind::OrdinaryFile(PathBuf::from(source.to_string())),
            input.kind
        );
    }

    #[test]
    fn test_to_input_url() {
        let source = "http://some_site.com/some/path/";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::Url(Url::parse(source).unwrap()), input.kind);
    }

    #[test]
    fn test_to_output_s3() {
        let bucket = "s3://some_bucket";
        let output = to_output(bucket.to_string());
        assert_eq!(OutputKind::S3Bucket(bucket.to_string()), output.kind);
    }

    #[test]
    fn test_to_output_stdout() {
        let output = to_output("-".to_string());
        assert_eq!(OutputKind::StdOut, output.kind);
    }

    #[test]
    fn test_to_output_scp() {
        let target = "some_user@some_host:~/";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::ScpTarget(target.to_string()), output.kind);
    }

    #[test]
    fn test_to_output_file() {
        let target = "/some/path/file.txt";
        let output = to_output(target.to_string());
        assert_eq!(
            OutputKind::OrdinaryFile(PathBuf::from(target.to_string())),
            output.kind
        );
    }

    #[test]
    fn test_to_output_url() {
        let target = "http://some_site.com/some/path/";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::Url(Url::parse(target).unwrap()), output.kind);
    }
}
