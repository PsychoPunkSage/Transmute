// Integration test for NLP parser PDF conversion
use transmute_common::MediaFormat;
use transmute_nlp::{CommandParser, Intent};

#[test]
fn test_parse_image_to_pdf_conversion() {
    let parser = CommandParser::new().unwrap();
    let intent = parser
        .parse("convert test.jpg to pdf")
        .expect("Failed to parse command");

    match intent {
        Intent::Convert(conv) => {
            assert_eq!(conv.target_format, MediaFormat::Pdf);
            assert!(conv.input.to_string_lossy().contains("test.jpg"));
        }
        _ => panic!("Wrong intent type"),
    }
}

#[test]
fn test_parse_with_output_location_in() {
    let parser = CommandParser::new().unwrap();
    let intent = parser
        .parse("convert test.jpg to png in /tmp/output")
        .expect("Failed to parse command with 'in' keyword");

    match intent {
        Intent::Convert(conv) => {
            assert_eq!(conv.target_format, MediaFormat::Png);
            assert!(conv.output.is_some());
            assert!(conv
                .output
                .unwrap()
                .to_string_lossy()
                .contains("/tmp/output"));
        }
        _ => panic!("Wrong intent type"),
    }
}

#[test]
fn test_parse_with_output_location_at() {
    let parser = CommandParser::new().unwrap();
    let intent = parser
        .parse("convert test.jpg to png at /tmp/output")
        .expect("Failed to parse command with 'at' keyword");

    match intent {
        Intent::Convert(conv) => {
            assert_eq!(conv.target_format, MediaFormat::Png);
            assert!(conv.output.is_some());
        }
        _ => panic!("Wrong intent type"),
    }
}

#[test]
fn test_parse_compress_with_in_keyword() {
    let parser = CommandParser::new().unwrap();
    let intent = parser
        .parse("compress photo.jpg to 80% in ./compressed")
        .expect("Failed to parse compress with 'in' keyword");

    match intent {
        Intent::Compress(comp) => {
            assert!(comp.output.is_some());
            assert!(comp
                .output
                .unwrap()
                .to_string_lossy()
                .contains("compressed"));
        }
        _ => panic!("Wrong intent type"),
    }
}

#[test]
fn test_case_insensitive_pdf_conversion() {
    let parser = CommandParser::new().unwrap();

    let intent1 = parser.parse("CONVERT test.png TO pdf").unwrap();
    let intent2 = parser.parse("convert test.png to PDF").unwrap();

    match (intent1, intent2) {
        (Intent::Convert(c1), Intent::Convert(c2)) => {
            assert_eq!(c1.target_format, MediaFormat::Pdf);
            assert_eq!(c2.target_format, MediaFormat::Pdf);
        }
        _ => panic!("Wrong intent type"),
    }
}
