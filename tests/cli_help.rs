use assert_cmd::Command;

#[test]
fn cli_help_works() {
    let mut cmd = Command::cargo_bin("gmaps-scraper-rs").unwrap();
    cmd.arg("--help").assert().success();
}
