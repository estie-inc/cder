use anyhow::Result;
use std::{collections::HashMap, env};

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// replaces embedded custom tags before deserialization
/// tags can be used to allocate dynamic values to the seed object
/// format:
/// tags must be surrounded between two consecutive braces: ${{ ... }}
/// inside, there must be a pair of 'directive' followed by a 'key' surrounded in the parenthesis.
/// so the basic form is: ${{ directive(key) }}
/// you can also add a 'default' value as follows, which can be used in case it fails to resolve
/// the specified key: ${{ directive(key:-default) }}
///
/// currently it accepts following types as directive:
///   ENV(FOO_BAR)   ... replace the tag with the environment variable 'FOO'
///   REF(some_name) ... replace the tag with an ID of an object, referred by the key named 'some_name'
/// constraints:
///   all keys must consist of alphabet or numbers.
///   default values must consist of alphanumeric, or string surrounded by double quotes "..." (the
///   string must not contain any other double quotes or control charactors)
pub fn resolve_tags(raw_text: &str, dict: &HashMap<String, String>) -> Result<String> {
    let mut index: usize = 0;
    let mut parsed_text: String = "".to_string();

    while index < raw_text.len() {
        let source_text = &raw_text[index..];

        let result = try_consume(source_text)?;

        index += match result {
            ParseResult::Nothing => {
                parsed_text.push_str(source_text);
                source_text.len()
            }

            ParseResult::Found {
                directive,
                key,
                default,
                start,
                end,
            } => {
                // finds a value (text) that has to be replaced with the directive/key.
                // ENV(<key>) ... replace it with the environment var <key>
                // REF(<key>) ... replace it with the object id referred by the <key>
                let replacement = match directive.as_str() {
                    "ENV" => resolve_env(&key, default),
                    "REF" => resolve_ref(&key, dict),
                    _ => Err(anyhow::anyhow!(
                        "the directive: ` {}` is not supported.",
                        directive
                    )),
                }?;
                if start > 0 {
                    parsed_text.push_str(&source_text[..start]);
                }
                parsed_text.push_str(&replacement);
                end
            }
        };
    }

    Ok(parsed_text)
}

fn resolve_ref(key: &str, dict: &HashMap<String, String>) -> Result<String> {
    dict.get(key)
        .map(|value| value.to_owned())
        .ok_or_else(|| anyhow::anyhow!("failed to idintify a record referred by the key: `{key}`"))
}

/// this enum is used to hold the type of the directive indicated by the tag
#[derive(PartialEq, Debug)]
enum ParseResult {
    Found {
        // contains the parse result if the string matches with any of the discriptor patterns
        directive: String,
        key: String,
        default: Option<String>,
        start: usize, // index the first charactor that matched with ${{...}}
        end: usize,   // index the last charactor that matched with ${{...}}
    },
    Nothing, // no matches
}

/// retrieve the values from the environment variable that matches the provided key
fn resolve_env(key: &str, defalut: Option<String>) -> Result<String> {
    env::var(key).or_else(|_| match defalut {
        Some(value) => Ok(value),
        None => Err(anyhow::anyhow!(
            "environment variable: `{}` is not found",
            key
        )),
    })
}

/// captures the directive and the key surrounded by ${{ }}, returns a ParseResult object
fn try_consume(source: &str) -> Result<ParseResult> {
    // matches with something like: ${{ AnyTag(some_key) }}
    let re = regex!(
        r#"\$\{\{\s*(?P<directive>[[:alnum:]]+)\(\s*(?P<key>[[:alnum:]_-]+)(\s*:-\s*(?P<default>([[:alnum:]]+|"[^"[:cntrl:]]+")))?\s*\)\s*\}\}"#
    );

    let captures = match re.captures(source) {
        Some(captures) => captures,
        None => return Ok(ParseResult::Nothing),
    };

    let directive = captures
        .name("directive")
        .map(|matched| matched.as_str().to_string());
    let key = captures
        .name("key")
        .map(|matched| matched.as_str().to_string());
    let default = captures
        .name("default")
        .map(|matched| matched.as_str().to_string());

    let base_capture = captures.get(0);
    let start = base_capture.map(|matched| matched.start());
    let end = base_capture.map(|matched| matched.end());

    match (directive, key, start, end) {
        (Some(directive), Some(key), Some(start), Some(end)) => Ok(ParseResult::Found {
            directive,
            key,
            default,
            start,
            end,
        }),
        // usually this should not happen
        _ => Err(anyhow::anyhow!(
            "match failed for unknown reasons: check that the regex has valid form"
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::resolver::*;
    use std::env;

    #[test]
    // test against embedded tags
    fn test_resolve_tags() {
        let raw_text =
            "The quick brown ${{ ENV(FOX) }} jumps over\nthe lazy ${{ REF(dog) }}".to_string();

        // when correspoinding env var is defined
        env::set_var("FOX", "🦊");
        // when the ref is successfully resolved
        let dict = HashMap::from([
            ("swan".to_string(), "🦢".to_string()),
            ("dog".to_string(), "🐕".to_string()),
        ]);
        let parsed_text = resolve_tags(&raw_text, &dict).unwrap();
        assert_eq!(parsed_text, "The quick brown 🦊 jumps over\nthe lazy 🐕");

        // when the ref is undefined
        let dict = HashMap::from([
            ("swan".to_string(), "🦢".to_string()),
            ("dolphin".to_string(), "🐬".to_string()),
        ]);
        let parsed_text = resolve_tags(&raw_text, &dict);
        assert!(parsed_text.is_err());

        // when the dict is empty
        let dict = HashMap::new();
        let parsed_text = resolve_tags(&raw_text, &dict);
        assert!(parsed_text.is_err());

        // when correspoinding env var is NOT defined
        env::remove_var("FOX");
        // when the ref is successfully resolved
        let dict = HashMap::from([
            ("swan".to_string(), "🦢".to_string()),
            ("dog".to_string(), "🐕".to_string()),
        ]);
        let parsed_text = resolve_tags(&raw_text, &dict);
        assert!(parsed_text.is_err());

        // when the tag cannot be recognized (due to incorrect format)
        let raw_text = "The quick brown ${{ENV(FOX?)}} jumps over\nthe lazy {REF(dog)}".to_string();
        let parsed_text = resolve_tags(&raw_text, &dict).unwrap();
        // it simply outputs the original text as it is
        assert_eq!(
            parsed_text,
            "The quick brown ${{ENV(FOX?)}} jumps over\nthe lazy {REF(dog)}".to_string()
        );

        // when the tag contains unsupported directive name
        let raw_text = "The quick brown ${{REFERENCE(fox_id)}} jumps over the lazy dog".to_string();
        let parsed_text = resolve_tags(&raw_text, &dict);
        assert!(parsed_text.is_err());
    }

    #[test]
    fn test_resolve_ref() {
        let dict = HashMap::from([
            ("foo".to_string(), "bar".to_string()),
            ("umi".to_string(), "yama".to_string()),
        ]);

        let value = resolve_ref("foo", &dict).unwrap();
        assert_eq!(value, "bar");

        let value = resolve_ref("BAZ", &dict);
        assert!(value.is_err());

        let dict = HashMap::new();
        let value = resolve_ref("foo", &dict);
        assert!(value.is_err());
    }

    #[test]
    fn test_resolve_env() {
        let key = "FOO";

        // when correspoinding env var is NOT defined
        env::remove_var(key);
        assert!(resolve_env(key, None).is_err());

        let value = resolve_env(key, Some("default".to_string())).unwrap();
        assert_eq!(value, "default");

        // when correspoinding env var is defined
        env::set_var(key, "SOME_VALUE");
        assert_eq!(resolve_env(key, None).unwrap(), "SOME_VALUE");

        let value = resolve_env(key, Some("default".to_string())).unwrap();
        assert_eq!(value, "SOME_VALUE");
    }

    #[test]
    fn test_try_consume() {
        let source_text = "abc${{ SomeDirective(key-is-here)  }}xyz";
        let result = try_consume(source_text).unwrap();
        // extracts the directive and the key surrounded between double braces ${{ }}
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "SomeDirective".to_string(),
                key: "key-is-here".to_string(),
                default: None,
                start: 3,
                end: 37,
            }
        );

        // when default value is provided after the key
        let source_text = r#"abc${{ SomeDirective(key-is-here:-DEFAULT1)  }}xyz"#;
        let result = try_consume(source_text).unwrap();
        // extracts the directive, the key, and the default value surrounded between double braces ${{ }}
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "SomeDirective".to_string(),
                key: "key-is-here".to_string(),
                default: Some("DEFAULT1".to_string()),
                start: 3,
                end: 47,
            }
        );

        // the default value may contain any non-control charactors surrounded by double quotes
        // (be it a non-ascii charactor or punctuation)
        let source_text = r#"abc${{ SomeDirective(key-is-here:-"See? th|s @lso fa!!s b/\ck to .. `default` value 🏡")  }}xyz"#;
        let result = try_consume(source_text).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "SomeDirective".to_string(),
                key: "key-is-here".to_string(),
                default: Some(
                    r#""See? th|s @lso fa!!s b/\ck to .. `default` value 🏡""#.to_string()
                ),
                start: 3,
                end: 94,
            }
        );

        // when there is multiple "directive-key" matches
        let source_text =
            "abc${{ SomeDirective(key-is-here)  }}xyz${{ SomeOtherDirective(key) }}pqrs${{FOO(bar)}}";
        let result = try_consume(source_text).unwrap();
        // it captures the first one
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "SomeDirective".to_string(),
                key: "key-is-here".to_string(),
                default: None,
                start: 3,
                end: 37,
            }
        );

        // spaces inside double braces are ignored
        let source_text = "${{　　　 FOOOOO( \t bar )   \t  }}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "FOOOOO".to_string(),
                key: "bar".to_string(),
                default: None,
                start: 0,
                end: 36,
            }
        );

        // when parsing the original text (without offset)
        let source_text = "123456789${{Hoge(fuga)}}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "Hoge".to_string(),
                key: "fuga".to_string(),
                default: None,
                start: 9,
                end: 24,
            }
        );
        // when parsing the text from certain offset index
        let result = try_consume(&source_text[9..]).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "Hoge".to_string(),
                key: "fuga".to_string(),
                default: None,
                start: 0,
                end: 15,
            }
        );

        // it detects the closest tag that appears after the offset
        let source_text = "${{A1(key1)}}  ${{A2(key2)}} ${{A3(key3)}}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "A1".to_string(),
                key: "key1".to_string(),
                default: None,
                start: 0,
                end: 13,
            }
        );
        let result = try_consume(&source_text[1..]).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "A2".to_string(),
                key: "key2".to_string(),
                default: None,
                start: 14,
                end: 27,
            }
        );
        let result = try_consume(&source_text[16..]).unwrap();
        assert_eq!(
            result,
            ParseResult::Found {
                directive: "A3".to_string(),
                key: "key3".to_string(),
                default: None,
                start: 13,
                end: 26,
            }
        );
        let result = try_consume(&source_text[30..]).unwrap();
        assert_eq!(result, ParseResult::Nothing);

        // does NOT capture the tag that is inside a pair of single braces
        let source_text = "foo bar baz{ hoge: fuga }";
        let result = try_consume(source_text).unwrap();
        assert_eq!(result, ParseResult::Nothing);

        // does NOT capture the tag surrounded by non-pairing braces, or non-consecutive braces
        let source_text = "{not(a-tag)}} ${{not(a-tag-too)} }";
        let result = try_consume(source_text).unwrap();
        assert_eq!(result, ParseResult::Nothing);

        // non-alphanumeric charactors are not recognized as directive
        let source_text = "${{F-O-O(Bar)}}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(result, ParseResult::Nothing);

        // does NOT capture a tag that has no keys surrounded by parenthesis
        let source_text = "${{no-directive-here}}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(result, ParseResult::Nothing);

        // does NOT capture a tag that has mal-formatted key/parenthesis
        let source_text = "${{foo(bar)(baz)}}  ${{foo(hoge}}";
        let result = try_consume(source_text).unwrap();
        assert_eq!(result, ParseResult::Nothing);
    }
}
