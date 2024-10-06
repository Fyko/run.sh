use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub struct MatchedCode<'a> {
    pub language: &'a str,
    pub code: &'a str,
}

pub fn match_code(input: &str, no_prefix: bool) -> Option<MatchedCode<'_>> {
    let Some(codeblock) = match_codeblock(input, no_prefix) else {
        return match_inline_code(input, no_prefix);
    };

    Some(codeblock)
}

pub fn match_codeblock(input: &str, no_prefix: bool) -> Option<MatchedCode<'_>> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^\$>```(?<language>[a-zA-Z]*?)\s(?<code>[\S\s]*?)\s```$").unwrap()
    });
    static RE_NO_PREFIX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)```(?<language>[a-zA-Z]*?)\s(?<code>[\S\s]*?)\s```").unwrap()
    });

    let captures = if no_prefix {
        RE_NO_PREFIX.captures(input)?
    } else {
        RE.captures(input)?
    };
    let language = captures.name("language")?;
    let code = captures.name("code")?;

    Some(MatchedCode {
        language: language.as_str(),
        code: code.as_str(),
    })
}

pub fn match_inline_code(input: &str, no_prefix: bool) -> Option<MatchedCode<'_>> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)^\$>(?<language>[a-zA-Z]*?)`(?<code>.*?)`$").unwrap());
    static RE_NO_PREFIX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)(?<language>[a-zA-Z]*?)`(?<code>.*?)`").unwrap());

    let captures = if no_prefix {
        RE_NO_PREFIX.captures(input)?
    } else {
        RE.captures(input)?
    };
    let language = captures.name("language")?;
    let code = captures.name("code")?;

    Some(MatchedCode {
        language: language.as_str(),
        code: code.as_str(),
    })
}
