use ai_commit::*;
use clap::Parser;

#[test]
fn test_args_default() {
    let args = args::Args::parse_from(["ai-commit"]);
    assert_eq!(args.provider, "");
    assert_eq!(args.model, "");
    assert!(!args.no_add);
    assert!(!args.push);
    assert!(args.new_tag.is_none());
    assert!(!args.show_tag);
    assert!(!args.push_branches);
}

#[test]
fn test_args_with_values() {
    let args = args::Args::parse_from([
        "ai-commit",
        "--provider",
        "deepseek",
        "--model",
        "gpt-4",
        "--no-add",
        "--push",
        "--new-tag",
        "v1.2.3",
        "--show-tag",
        "--push-branches",
    ]);
    assert_eq!(args.provider, "deepseek");
    assert_eq!(args.model, "gpt-4");
    assert!(args.no_add);
    assert!(args.push);
    assert_eq!(args.new_tag, Some("v1.2.3".to_string()));
    assert!(args.show_tag);
    assert!(args.push_branches);
}
