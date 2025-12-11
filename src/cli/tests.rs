use super::*;

macro_rules! read_args {
    ($a: expr) => {
        read_args($a.into_iter())
    };
}

#[test]
fn args() {
    let file_path = "--/file/path";

    let args = [
        "--details",
        "--mode",
        "stats",
        "--verbose",
        "--output",
        "quiet",
        "/wrong/path",
        "",
        "--",
        file_path,
        "--details",
        "--verbose",
    ];

    let config = read_args!(args).unwrap().unwrap();
    assert_eq!(config.file_path(), Some(file_path));
    assert_eq!(config.mode(), Some(Mode::Stats));
    assert_eq!(config.output(), Some(Output::Quiet));
}

macro_rules! test_args {
    ($a: expr, $r: expr) => {
        assert_eq!(read_args!($a).unwrap(), $r)
    };
}

#[test]
fn no_args() {
    test_args!([""; 0], Some(Config::default()));
}

#[test]
fn help() {
    test_args!(["--details", "foo", "-h", "--foo"], None);
}

macro_rules! test_error {
    ($a: expr, $r: pat) => {
        assert!(matches!(read_args!($a), Err($r)))
    };
}

#[test]
fn no_value() {
    test_error!(["--mode"], CliError::NoValue(_));
}

#[test]
fn invalid_value() {
    test_error!(["--mode", "foo"], CliError::InvalidValue(..));
}

#[test]
fn unknown_arg() {
    test_error!(["--foo"], CliError::Unknown(_));
}
