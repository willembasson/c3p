use atty::{is, Stream};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(name = "C3P(No O)")]
#[command(bin_name = "c3p")]
#[command(author = "Willem B. <willem.basson@gmail.com>")]
#[command(about = "
All your copies R mine
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
pub enum InputKind {
    OrdinaryFile,
    StdIn,
    Url(url::Url),
    S3Bucket,
    ScpSource,
}

#[derive(Debug)]
struct Input {
    kind: InputKind,
    reference: String,
}

fn to_input(input: String) -> Input {
    let input_url_result = Url::parse(input.as_str());
    if let Ok(input_url) = input_url_result {
        if input.starts_with("s3://") {
            Input {
                kind: InputKind::S3Bucket,
                reference: input,
            }
        } else if input.starts_with("scp://") {
            Input {
                kind: InputKind::ScpSource,
                reference: input,
            }
        } else {
            Input {
                kind: InputKind::Url(input_url),
                reference: input,
            }
        }
    } else if input == "-" {
        if is(Stream::Stdin) {
            println!("You said StdIn but you didn't pipe or redirect anything");
        }
        Input {
            kind: InputKind::StdIn,
            reference: input,
        }
    } else if input.contains('@') && input.contains(':') {
        Input {
            kind: InputKind::ScpSource,
            reference: input,
        }
    } else {
        Input {
            kind: InputKind::OrdinaryFile,
            reference: input,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OutputKind {
    OrdinaryFile,
    StdOut,
    Url(url::Url),
    S3Bucket,
    ScpTarget,
}

#[derive(Debug)]
struct Output {
    kind: OutputKind,
    reference: String,
}

fn to_output(output: String) -> Output {
    let output_url_result = Url::parse(output.as_str());
    if let Ok(output_url) = output_url_result {
        if output.starts_with("s3://") {
            Output {
                kind: OutputKind::S3Bucket,
                reference: output,
            }
        } else if output.starts_with("scp://") {
            Output {
                kind: OutputKind::ScpTarget,
                reference: output,
            }
        } else {
            Output {
                kind: OutputKind::Url(output_url),
                reference: output,
            }
        }
    } else if output == "-" {
        if is(Stream::Stdout) {
            println!("You said StdOut but you didn't pipe or redirect anything");
        }
        Output {
            kind: OutputKind::StdOut,
            reference: output,
        }
    } else if output.contains('@') && output.contains(':') {
        Output {
            kind: OutputKind::ScpTarget,
            reference: output,
        }
    } else {
        Output {
            kind: OutputKind::OrdinaryFile,
            reference: output,
        }
    }
}

fn copy(input: Input, output: Output) {
    match input.kind {
        Input::OrdinaryFile => {
            match output.kind {
                // Normal file to file copy
                // Lets do std::fs::copy for now
                Output::OrdinaryFile => {
                    fs::copy(input.reference, output.reference);
                }
                _ => {
                    todo!()
                }
            };
        }
        _ => {
            todo!()
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

        if let Some(output) = cli.output.as_deref() {
            let output_string = output.to_string();
            let output: Output = to_output(output_string);
            println!("{:#?}", output.kind);
            copy(input, output);
        } else {
            println!("No output defined");
        }
    } else {
        println!("No input defined");
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
        assert_eq!(InputKind::S3Bucket, input.kind);
        assert_eq!(bucket, input.reference);
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
        assert_eq!(InputKind::ScpSource, input.kind);
        assert_eq!(source, input.reference);
    }

    #[test]
    fn test_to_input_file() {
        let source = "/some/path/file.txt";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::OrdinaryFile, input.kind);
        assert_eq!(source, input.reference);
    }

    #[test]
    fn test_to_input_url() {
        let source = "http://some_site.com/some/path/";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::Url(Url::parse(source).unwrap()), input.kind);
        assert_eq!(source, input.reference);
    }

    #[test]
    fn test_to_output_s3() {
        let bucket = "s3://some_bucket";
        let output = to_output(bucket.to_string());
        assert_eq!(OutputKind::S3Bucket, output.kind);
        assert_eq!(bucket, output.reference);
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
        assert_eq!(OutputKind::ScpTarget, output.kind);
        assert_eq!(target, output.reference);
    }

    #[test]
    fn test_to_output_file() {
        let target = "/some/path/file.txt";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::OrdinaryFile, output.kind);
        assert_eq!(target, output.reference);
    }

    #[test]
    fn test_to_output_url() {
        let target = "http://some_site.com/some/path/";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::Url(Url::parse(target).unwrap()), output.kind);
        assert_eq!(target, output.reference);
    }
}
