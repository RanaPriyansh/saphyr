use std::fs::{self, File};
use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::time::Instant;

use saphyr::{LoadableYamlNode, Scalar, Yaml};

fn write_len(writer: &mut impl Write, value: usize) -> Result<(), String> {
    let value = u64::try_from(value).map_err(|_| "length exceeds u64".to_owned())?;
    writer.write_all(&value.to_le_bytes()).map_err(io_error)
}

fn write_bytes(writer: &mut impl Write, value: &[u8]) -> Result<(), String> {
    write_len(writer, value.len())?;
    writer.write_all(value).map_err(io_error)
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}

fn write_tag(writer: &mut impl Write, tag: &saphyr::Tag) -> Result<(), String> {
    write_bytes(writer, tag.handle.as_bytes())?;
    write_bytes(writer, tag.suffix.as_bytes())
}

fn write_node(writer: &mut impl Write, node: &Yaml<'_>) -> Result<(), String> {
    match node {
        Yaml::Representation(value, style, tag) => {
            writer.write_all(&[1]).map_err(io_error)?;
            write_bytes(writer, value.as_bytes())?;
            let style_code = match style {
                saphyr::ScalarStyle::Plain => 0,
                saphyr::ScalarStyle::SingleQuoted => 1,
                saphyr::ScalarStyle::DoubleQuoted => 2,
                saphyr::ScalarStyle::Literal => 3,
                saphyr::ScalarStyle::Folded => 4,
            };
            writer.write_all(&[style_code]).map_err(io_error)?;
            match tag {
                Some(tag) => {
                    writer.write_all(&[1]).map_err(io_error)?;
                    write_tag(writer, tag)?;
                }
                None => writer.write_all(&[0]).map_err(io_error)?,
            }
        }
        Yaml::Value(value) => match value {
            Scalar::Null => writer.write_all(&[2]).map_err(io_error)?,
            Scalar::Boolean(value) => {
                writer.write_all(&[3, u8::from(*value)]).map_err(io_error)?;
            }
            Scalar::Integer(value) => {
                writer.write_all(&[4]).map_err(io_error)?;
                writer.write_all(&value.to_le_bytes()).map_err(io_error)?;
            }
            #[cfg(saphyr_string_integer)]
            Scalar::StringInteger(value) => {
                writer.write_all(&[5]).map_err(io_error)?;
                write_bytes(writer, value.as_str().as_bytes())?;
            }
            Scalar::FloatingPoint(value) => {
                writer.write_all(&[6]).map_err(io_error)?;
                writer
                    .write_all(&value.into_inner().to_bits().to_le_bytes())
                    .map_err(io_error)?;
            }
            Scalar::String(value) => {
                writer.write_all(&[7]).map_err(io_error)?;
                write_bytes(writer, value.as_bytes())?;
            }
        },
        Yaml::Sequence(values) => {
            writer.write_all(&[8]).map_err(io_error)?;
            write_len(writer, values.len())?;
            for value in values {
                write_node(writer, value)?;
            }
        }
        Yaml::Mapping(values) => {
            writer.write_all(&[9]).map_err(io_error)?;
            write_len(writer, values.len())?;
            for (key, value) in values {
                write_node(writer, key)?;
                write_node(writer, value)?;
            }
        }
        Yaml::Tagged(tag, value) => {
            writer.write_all(&[10]).map_err(io_error)?;
            write_tag(writer, tag)?;
            write_node(writer, value)?;
        }
        Yaml::Alias(value) => {
            writer.write_all(&[11]).map_err(io_error)?;
            writer
                .write_all(
                    &u64::try_from(*value)
                        .map_err(|_| "alias exceeds u64")?
                        .to_le_bytes(),
                )
                .map_err(io_error)?;
        }
        Yaml::BadValue => writer.write_all(&[12]).map_err(io_error)?,
    }
    Ok(())
}

fn load(input: &str) -> Result<Vec<Yaml<'_>>, String> {
    Yaml::load_from_str(input).map_err(|error| error.to_string())
}

fn require_one_document(docs: &[Yaml<'_>]) -> Result<(), String> {
    if docs.len() != 1 {
        return Err(format!("expected one document, received {}", docs.len()));
    }
    Ok(())
}

fn witness(input_path: &str, output_path: &str) -> Result<(), String> {
    let input = fs::read_to_string(input_path).map_err(io_error)?;
    let docs = load(&input)?;
    require_one_document(&docs)?;
    let mut output = BufWriter::new(File::create(output_path).map_err(io_error)?);
    output
        .write_all(b"SAPHYR_TYPED_TREE\x01")
        .map_err(io_error)?;
    write_len(&mut output, docs.len())?;
    for doc in &docs {
        write_node(&mut output, doc)?;
    }
    output.flush().map_err(io_error)?;
    black_box(&docs);
    println!("witness\tdocuments={}", docs.len());
    Ok(())
}

fn timing(revision: &str, corpus: &str, input_path: &str) -> Result<(), String> {
    let input = fs::read_to_string(input_path).map_err(io_error)?;
    let input = black_box(input.as_str());
    for index in 0..2 {
        let start = Instant::now();
        let parsed = Yaml::load_from_str(input);
        let elapsed = start.elapsed().as_nanos();
        let docs = parsed.map_err(|error| error.to_string())?;
        require_one_document(&docs)?;
        black_box(&docs);
        println!("warmup\t{revision}\t{corpus}\t{index}\tparse_ns={elapsed}");
        std::io::stdout().flush().map_err(io_error)?;
        drop(docs);
    }
    let start = Instant::now();
    let parsed = Yaml::load_from_str(input);
    let elapsed = start.elapsed().as_nanos();
    let docs = parsed.map_err(|error| error.to_string())?;
    require_one_document(&docs)?;
    black_box(&docs);
    println!("sample\t{revision}\t{corpus}\tparse_ns={elapsed}");
    std::io::stdout().flush().map_err(io_error)?;
    drop(docs);
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("witness") if args.len() == 4 => witness(&args[2], &args[3]),
        Some("timing") if args.len() == 5 => timing(&args[2], &args[3], &args[4]),
        _ => Err("usage: loader_time witness INPUT OUTPUT | timing REVISION CORPUS INPUT".into()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("loader_time: {error}");
        std::process::exit(1);
    }
}
