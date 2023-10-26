use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    filename: String,

    #[arg(short, long, value_parser = parse_key_val, num_args = 1.., value_delimiter = ',')]
    require: Option<Vec<(String, String)>>,
}

/// Parse a list of key-value pair
fn parse_key_val(s: &str) -> Result<(String, String), &'static str> {
    let mut split = s.split('=');
    let key = split.next().ok_or("Missing Key")?.trim().to_string();
    let val = split.next().ok_or("Missing Value")?.trim().to_string();
    Ok((key, val))
}

fn main() {
    let cli = Cli::parse();
    println!("cli: {:?}", cli);

    match osm4routing::read(&cli.filename, cli.require) {
        Ok((nodes, edges)) => osm4routing::writers::csv(nodes, edges),
        Err(error) => println!("Error: {}", error),
    }
}

// Test parse_key_val
#[test]
fn test_parse_key_val() {
    assert_eq!(parse_key_val("key=value"), Ok(("key".to_string(), "value".to_string())));
    assert_eq!(parse_key_val("key = value"), Ok(("key".to_string(), "value".to_string())));
    assert_eq!(parse_key_val("key =value"), Ok(("key".to_string(), "value".to_string())));
    assert_eq!(parse_key_val("key=value "), Ok(("key".to_string(), "value".to_string())));
    assert_eq!(parse_key_val("key=value,"), Ok(("key".to_string(), "value,".to_string())));
    assert_eq!(parse_key_val("key=value, "), Ok(("key".to_string(), "value,".to_string())));
    assert_eq!(parse_key_val("key=value,  "), Ok(("key".to_string(), "value,".to_string())));
}