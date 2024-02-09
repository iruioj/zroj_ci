use std::collections::HashMap;

use askama::Template;
use regex::Regex;

#[derive(Debug)]
pub struct ConventionalCommitMsg {
    hash: String,
    kind: String,
    msg: String,
}

pub fn parse_commit_list(content: &str) -> Vec<ConventionalCommitMsg> {
    let re = Regex::new(r"^([0-9A-Za-z]+)\s+([a-zA-Z]+(?:\(.*?\))?):(.*)$").unwrap();
    let re_fallback = Regex::new(r"^(?<hash>[0-9A-Za-z]+)\s+(?<full_msg>.*)$").unwrap();

    let lns = content.split('\n').collect::<Vec<&str>>();
    let mut ret = Vec::new();
    for ln in lns {
        let ln = ln.trim();
        let msg = if let Some((_, [hash, kind, msg])) = re.captures(ln).map(|s| s.extract()) {
            ConventionalCommitMsg {
                hash: hash.into(),
                kind: kind.into(),
                msg: msg.into(),
            }
        } else {
            let Some((_, [hash, full_msg])) = re_fallback.captures(ln).map(|s| s.extract()) else {
                panic!("invalid git output")
            };
            ConventionalCommitMsg {
                hash: hash.into(),
                kind: "mix".into(),
                msg: full_msg.into(),
            }
        };
        ret.push(msg);
    }
    ret
}

#[derive(Template)]
#[template(
    source = r#"## {{ title }}

{% for msg in msgs %}- `{{ msg.hash }}` {{ msg.kind }}: {{ msg.msg.trim() }}
{% endfor %}
"#,
    ext = "txt"
)]
struct ChangelogSection<'a> {
    title: &'a str,
    msgs: &'a [ConventionalCommitMsg],
}

pub fn render_changelog(content: &str) -> String {
    let msgs = parse_commit_list(content);
    let types = [
        ("build", "Build"),
        ("chore", "Chore"),
        ("ci", "Continuous Integration"),
        ("docs", "Docs"),
        ("feat", "Feature"),
        ("fix", "Fix"),
        ("mix", "Mixture"),
        ("perf", "Performance"),
        ("refactor", "Refactor"),
        ("style", "Style"),
        ("test", "Test"),
    ];
    let ctx = msgs
        .into_iter()
        .fold(HashMap::<&str, Vec<_>>::new(), |mut map, msg| {
            if let Some((k, _)) = types.iter().find(|kind| msg.kind.contains(kind.0)) {
                map.entry(*k).or_default().push(msg);
            }
            map
        });
    types
        .into_iter()
        .map(|(k, title)| {
            if let Some(msgs) = ctx.get(k) {
                ChangelogSection { title, msgs }.render().unwrap()
            } else {
                String::default()
            }
        })
        .reduce(|a, b| a + &b)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::commit_parse::render_changelog;

    #[test]
    fn test_parse() {
        let r = render_changelog(
            r#"d0aec01 chore(web): bump deps
d0aec01 chore(web): bump deps
d0aec01 fix(web): bump deps
d0aec01 chore(web): bump deps"#,
        );
        eprintln!("{r}");
    }
}
