use crate::intent::*;
use crate::path_resolver::PathResolver;
use pest_derive::Parser;
use transmute_common::{Error, MediaFormat, Result};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct CommandGrammar;

/// Natural language command parser
pub struct CommandParser {
    path_resolver: PathResolver,
}

impl CommandParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            path_resolver: PathResolver::new()?,
        })
    }

    /// Parse natural language command into intent
    pub fn parse(&self, command: &str) -> Result<Intent> {
        tracing::debug!("Parsing command: {}", command);

        // Try grammar-based parsing first
        match self.parse_with_grammar(command) {
            Ok(intent) => {
                tracing::info!("Parsed intent: {:?}", intent);
                Ok(intent)
            }
            Err(e) => {
                tracing::warn!("Grammar parse failed: {}, trying regex fallback", e);
                self.parse_with_regex(command)
            }
        }
    }

    /// Parse using pest grammar
    fn parse_with_grammar(&self, command: &str) -> Result<Intent> {
        use pest::Parser;

        let pairs = CommandGrammar::parse(Rule::command, command)
            .map_err(|e| Error::ConversionError(format!("Parse error: {}", e)))?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::convert_cmd => return self.parse_convert(pair),
                Rule::compress_cmd => return self.parse_compress(pair),
                Rule::enhance_cmd => return self.parse_enhance(pair),
                Rule::batch_cmd => return self.parse_batch(pair),
                _ => {}
            }
        }

        Err(Error::ConversionError("No valid command found".into()))
    }

    fn parse_convert(&self, pair: pest::iterators::Pair<Rule>) -> Result<Intent> {
        let mut input = None;
        let mut target_format = None;
        let mut output = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    input = Some(self.path_resolver.resolve(path_str)?);
                }
                Rule::format => {
                    target_format = MediaFormat::from_extension(inner.as_str());
                }
                Rule::output_path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    output = Some(self.path_resolver.resolve(path_str)?);
                }
                _ => {}
            }
        }

        let input = input.ok_or_else(|| Error::ConversionError("Missing input path".into()))?;
        let target_format =
            target_format.ok_or_else(|| Error::ConversionError("Missing target format".into()))?;

        Ok(Intent::Convert(ConvertIntent {
            input,
            target_format,
            output,
        }))
    }

    fn parse_compress(&self, pair: pest::iterators::Pair<Rule>) -> Result<Intent> {
        let mut input = None;
        let mut target_format = None;
        let mut quality = QualitySpec::Default;
        let mut output = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    input = Some(self.path_resolver.resolve(path_str)?);
                }
                Rule::format => {
                    target_format = MediaFormat::from_extension(inner.as_str());
                }
                Rule::quality_spec => {
                    quality = self.parse_quality(inner)?;
                }
                Rule::output_path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    output = Some(self.path_resolver.resolve(path_str)?);
                }
                _ => {}
            }
        }

        let input = input.ok_or_else(|| Error::ConversionError("Missing input path".into()))?;

        Ok(Intent::Compress(CompressIntent {
            input,
            target_format,
            quality,
            output,
        }))
    }

    fn parse_enhance(&self, pair: pest::iterators::Pair<Rule>) -> Result<Intent> {
        let mut input = None;
        let mut scale_factor = 2; // Default
        let mut output = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    input = Some(self.path_resolver.resolve(path_str)?);
                }
                Rule::scale_spec => {
                    for scale_inner in inner.into_inner() {
                        if scale_inner.as_rule() == Rule::scale_factor {
                            scale_factor = scale_inner.as_str().parse().map_err(|_| {
                                Error::ConversionError("Invalid scale factor".into())
                            })?;
                        }
                    }
                }
                Rule::output_path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    output = Some(self.path_resolver.resolve(path_str)?);
                }
                _ => {}
            }
        }

        let input = input.ok_or_else(|| Error::ConversionError("Missing input path".into()))?;

        Ok(Intent::Enhance(EnhanceIntent {
            input,
            scale_factor,
            output,
        }))
    }

    fn parse_batch(&self, pair: pest::iterators::Pair<Rule>) -> Result<Intent> {
        let mut pattern = None;
        let mut target_format = None;
        let mut output = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::path_pattern => {
                    let pattern_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    pattern = Some(pattern_str.to_string());
                }
                Rule::format => {
                    target_format = MediaFormat::from_extension(inner.as_str());
                }
                Rule::output_path => {
                    let path_str = inner.as_str().trim_matches(|c| c == '"' || c == '\'');
                    output = Some(self.path_resolver.resolve(path_str)?);
                }
                _ => {}
            }
        }

        let pattern = pattern.ok_or_else(|| Error::ConversionError("Missing pattern".into()))?;
        let target_format =
            target_format.ok_or_else(|| Error::ConversionError("Missing target format".into()))?;

        Ok(Intent::Batch(BatchIntent {
            pattern,
            target_format,
            output,
        }))
    }

    fn parse_quality(&self, pair: pest::iterators::Pair<Rule>) -> Result<QualitySpec> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::percentage => {
                    let percent_str = inner.as_str().trim_end_matches('%');
                    let value = percent_str
                        .parse::<u8>()
                        .map_err(|_| Error::ConversionError("Invalid percentage".into()))?;
                    return Ok(QualitySpec::Percentage(value.clamp(1, 100)));
                }
                Rule::quality_preset => {
                    let preset_str = inner.as_str().to_lowercase();
                    let preset = match preset_str.as_str() {
                        "maximum" | "max" => QualityPreset::Maximum,
                        "high" => QualityPreset::High,
                        "medium" | "balanced" => QualityPreset::Medium,
                        "low" => QualityPreset::Low,
                        _ => {
                            return Err(Error::ConversionError(format!(
                                "Unknown quality preset: {}",
                                preset_str
                            )))
                        }
                    };
                    return Ok(QualitySpec::Preset(preset));
                }
                _ => {}
            }
        }

        Ok(QualitySpec::Default)
    }

    /// Fallback regex-based parsing for simpler commands
    fn parse_with_regex(&self, command: &str) -> Result<Intent> {
        use regex::Regex;

        let cmd_lower = command.to_lowercase();

        // Convert pattern: "convert <path> to <format>"
        let convert_re = Regex::new(
            r"(?i)convert\s+(.+?)\s+to\s+(png|jpg|jpeg|webp|pdf|tiff|bmp|gif)(?:\s+at\s+(.+))?",
        )
        .unwrap();

        if let Some(caps) = convert_re.captures(command) {
            let path_str = caps.get(1).unwrap().as_str().trim();
            let path_str = path_str.trim_matches(|c| c == '"' || c == '\'');
            let input = self.path_resolver.resolve(path_str)?;
            let format = MediaFormat::from_extension(caps.get(2).unwrap().as_str())
                .ok_or_else(|| Error::UnsupportedFormat(caps.get(2).unwrap().as_str().into()))?;
            let output = caps
                .get(3)
                .map(|m| {
                    let out_path = m.as_str().trim();
                    let out_path = out_path.trim_matches(|c| c == '"' || c == '\'');
                    self.path_resolver.resolve(out_path)
                })
                .transpose()?;

            return Ok(Intent::Convert(ConvertIntent {
                input,
                target_format: format,
                output,
            }));
        }

        // Compress pattern: "compress <path> to <quality>" (percentage or preset)
        let compress_percent_re = Regex::new(
            r"(?i)(compress|optimize|reduce)\s+(.+?)\s+to\s+(\d+)%(?:\s+at\s+(.+))?$",
        )
        .unwrap();

        if let Some(caps) = compress_percent_re.captures(command) {
            let path_str = caps.get(2).unwrap().as_str().trim();
            let path_str = path_str.trim_matches(|c| c == '"' || c == '\'');
            let input = self.path_resolver.resolve(path_str)?;
            let percent = caps.get(3).unwrap().as_str().parse::<u8>().unwrap_or(85);
            let quality = QualitySpec::Percentage(percent.clamp(1, 100));
            let output = caps
                .get(4)
                .map(|m| self.path_resolver.resolve(m.as_str().trim()))
                .transpose()?;

            return Ok(Intent::Compress(CompressIntent {
                input,
                target_format: None,
                quality,
                output,
            }));
        }

        // Compress pattern with quality preset: "compress <path> to high quality"
        let compress_preset_re = Regex::new(
            r"(?i)(compress|optimize|reduce)\s+(.+?)\s+to\s+(maximum|max|high|medium|balanced|low)\s*(?:quality)?(?:\s+at\s+(.+))?$",
        )
        .unwrap();

        if let Some(caps) = compress_preset_re.captures(command) {
            let path_str = caps.get(2).unwrap().as_str().trim();
            let path_str = path_str.trim_matches(|c| c == '"' || c == '\'');
            let input = self.path_resolver.resolve(path_str)?;
            let preset_str = caps.get(3).unwrap().as_str().to_lowercase();
            let quality = match preset_str.as_str() {
                "maximum" | "max" => QualitySpec::Preset(QualityPreset::Maximum),
                "high" => QualitySpec::Preset(QualityPreset::High),
                "medium" | "balanced" => QualitySpec::Preset(QualityPreset::Medium),
                "low" => QualitySpec::Preset(QualityPreset::Low),
                _ => QualitySpec::Default,
            };
            let output = caps
                .get(4)
                .map(|m| self.path_resolver.resolve(m.as_str().trim()))
                .transpose()?;

            return Ok(Intent::Compress(CompressIntent {
                input,
                target_format: None,
                quality,
                output,
            }));
        }

        // Simple compress without quality: "compress <path>"
        let compress_simple_re = Regex::new(
            r"(?i)(compress|optimize|reduce)\s+(.+?)(?:\s+at\s+(.+))?$",
        )
        .unwrap();

        if let Some(caps) = compress_simple_re.captures(command) {
            let path_str = caps.get(2).unwrap().as_str().trim();
            let path_str = path_str.trim_matches(|c| c == '"' || c == '\'');
            let input = self.path_resolver.resolve(path_str)?;
            let output = caps
                .get(3)
                .map(|m| self.path_resolver.resolve(m.as_str().trim()))
                .transpose()?;

            return Ok(Intent::Compress(CompressIntent {
                input,
                target_format: None,
                quality: QualitySpec::Default,
                output,
            }));
        }

        // Enhance pattern: "enhance <path> by 2x/4x"
        let enhance_re = Regex::new(
            r"(?i)(enhance|upscale|improve|enlarge)\s+(.+?)\s+(?:by\s+)?(2|4)x?(?:\s+at\s+(.+))?$",
        )
        .unwrap();

        if let Some(caps) = enhance_re.captures(command) {
            let path_str = caps.get(2).unwrap().as_str().trim();
            let path_str = path_str.trim_matches(|c| c == '"' || c == '\'');
            let input = self.path_resolver.resolve(path_str)?;
            let scale_factor = caps.get(3).unwrap().as_str().parse::<u32>().unwrap_or(2);
            let output = caps
                .get(4)
                .map(|m| {
                    let out_path = m.as_str().trim();
                    let out_path = out_path.trim_matches(|c| c == '"' || c == '\'');
                    self.path_resolver.resolve(out_path)
                })
                .transpose()?;

            return Ok(Intent::Enhance(EnhanceIntent {
                input,
                scale_factor,
                output,
            }));
        }

        // Batch pattern: "batch <pattern> convert to <format> at <output>"
        let batch_re = Regex::new(
            r"(?i)(batch|bulk)\s+(.+?)\s+convert\s+to\s+(png|jpg|jpeg|webp|pdf|tiff|bmp|gif)(?:\s+at\s+(.+))?$",
        )
        .unwrap();

        if let Some(caps) = batch_re.captures(command) {
            let pattern_str = caps.get(2).unwrap().as_str().trim();
            let pattern = pattern_str.trim_matches(|c| c == '"' || c == '\'').to_string();
            let format = MediaFormat::from_extension(caps.get(3).unwrap().as_str())
                .ok_or_else(|| Error::UnsupportedFormat(caps.get(3).unwrap().as_str().into()))?;
            let output = caps
                .get(4)
                .map(|m| self.path_resolver.resolve(m.as_str().trim()))
                .transpose()?;

            return Ok(Intent::Batch(BatchIntent {
                pattern,
                target_format: format,
                output,
            }));
        }

        Err(Error::ConversionError(format!(
            "Could not parse command: {}. Try 'convert <file> to <format>'",
            command
        )))
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new().expect("Failed to create CommandParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_convert() {
        let parser = CommandParser::new().unwrap();

        let intent = parser.parse("convert ~/image.png to jpeg").unwrap();

        match intent {
            Intent::Convert(conv) => {
                assert!(conv.input.to_string_lossy().contains("image.png"));
                assert_eq!(conv.target_format, MediaFormat::Jpeg);
            }
            _ => panic!("Wrong intent type"),
        }
    }

    #[test]
    fn test_parse_compress_with_quality() {
        let parser = CommandParser::new().unwrap();

        let intent = parser.parse("compress test.jpg to 80%").unwrap();

        match intent {
            Intent::Compress(comp) => {
                assert_eq!(comp.quality, QualitySpec::Percentage(80));
            }
            _ => panic!("Wrong intent type"),
        }
    }

    #[test]
    fn test_parse_enhance() {
        let parser = CommandParser::new().unwrap();

        let intent = parser.parse("enhance photo.png by 4x").unwrap();

        match intent {
            Intent::Enhance(enh) => {
                assert_eq!(enh.scale_factor, 4);
            }
            _ => panic!("Wrong intent type"),
        }
    }

    #[test]
    fn test_case_insensitive() {
        let parser = CommandParser::new().unwrap();

        let intent1 = parser.parse("CONVERT test.png TO jpeg").unwrap();
        let intent2 = parser.parse("convert test.png to JPEG").unwrap();

        assert!(matches!(intent1, Intent::Convert(_)));
        assert!(matches!(intent2, Intent::Convert(_)));
    }

    #[test]
    fn test_natural_language_path() {
        let parser = CommandParser::new().unwrap();

        // This will only work if desktop exists
        if directories::UserDirs::new()
            .map(|d| d.desktop_dir().is_some())
            .unwrap_or(false)
        {
            let intent = parser.parse("convert desktop/image.png to jpeg");
            assert!(intent.is_ok());
        }
    }
}
