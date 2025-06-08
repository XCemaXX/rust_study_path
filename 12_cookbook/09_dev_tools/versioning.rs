use semver::{BuildMetadata, Error as SemVerError, Prerelease, Version, VersionReq};
use std::process::Command;

fn parse() -> Result<(), SemVerError> {
    let version = Version::parse("0.2.3")?;
    assert_eq!(
        version,
        Version {
            major: 0,
            minor: 2,
            patch: 3,
            pre: Prerelease::default(),
            build: BuildMetadata::default()
        }
    );
    // In semver 1.+ all functions increment_* are deleted
    Ok(())
}

fn parse_complex() -> Result<(), SemVerError> {
    let version_str = "1.0.42-123+g72ee7853";
    let version = Version::parse(version_str)?;
    assert_eq!(
        version,
        Version {
            major: 1,
            minor: 0,
            patch: 42,
            pre: Prerelease::new("123")?,
            build: BuildMetadata::new("g72ee7853")?
        }
    );
    let serialized_version = version.to_string();
    assert_eq!(&serialized_version, version_str);
    Ok(())
}

fn find_max_matching_version<'a, I>(
    version_condition: &str,
    versions: I,
) -> Result<Option<Version>, SemVerError>
where
    I: IntoIterator<Item = &'a str>,
{
    let condition = VersionReq::parse(version_condition)?;
    Ok(versions
        .into_iter()
        .filter_map(|s| Version::parse(s).ok())
        .filter(|s| condition.matches(s))
        .max())
}

fn version_conditions() -> Result<(), SemVerError> {
    assert_eq!(
        find_max_matching_version("<= 1.0.0", ["0.9.0", "1.0.0", "1.0.1"])?,
        Some(Version::parse("1.0.0")?)
    );
    assert_eq!(
        find_max_matching_version(
            ">1.2.3-alpha.3",
            [
                "1.2.3-alpha.3",
                "1.2.3-alpha.4",
                "1.2.3-alpha.10",
                "1.2.3-beta.4",
                "3.4.5-alpha.9",
            ]
        )?,
        Some(Version::parse("1.2.3-beta.4")?)
    );
    Ok(())
}

fn check_rustc_version() {
    let version_constraint = VersionReq::parse(">= 1.86.0").expect("Condition parsed");
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Cmd rustc");
    if !rustc_version.status.success() {
        panic!("Cmd 'rustc --version' returned error code");
    }
    let stdout = String::from_utf8(rustc_version.stdout).expect("Version in stdout");
    let version_str = stdout.split_whitespace().nth(1).expect("No version found");
    let version = Version::parse(version_str).expect("Invalid version");

    if !version_constraint.matches(&version) {
        println!(
            "Rustc version lower than minimum supported version (found {}, need {})",
            version, version_constraint
        );
    } else {
        println!(
            "Rustc version {} satisfies the requirement {}",
            version, version_constraint
        )
    }
}

fn main() {
    parse().unwrap();
    parse_complex().unwrap();
    version_conditions().unwrap();
    check_rustc_version();
}
