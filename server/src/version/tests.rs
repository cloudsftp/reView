use crate::config::device::DeviceConfigVersion;

use super::*;

use test_log::test;

#[test]
fn parse_framebuffer_version_from_string() {
    struct TestCase<'a> {
        input: &'a str,
        expected: FirmwareVersion,
    }

    for TestCase { input, expected } in [
        TestCase {
            input: "REMARKABLE_RELEASE_VERSION=2.0.0.0",
            expected: FirmwareVersion {
                version: 2,
                major: 0,
                minor: 0,
                patch: 0,
            },
        },
        TestCase {
            input: "REMARKABLE_RELEASE_VERSION=3.7.0.1930",
            expected: FirmwareVersion {
                version: 3,
                major: 7,
                minor: 0,
                patch: 1930,
            },
        },
        TestCase {
            input: "REMARKABLE_RELEASE_VERSION=3.25.0.119",
            expected: FirmwareVersion {
                version: 3,
                major: 25,
                minor: 0,
                patch: 119,
            },
        },
    ] {
        let result = input.parse().expect("could not parse version");
        assert_eq!(expected, result, "version input '{}'", input);
    }
}

#[test]
fn parse_config_version_from_string() {
    struct TestCase {
        input: FirmwareVersion,
        expected: DeviceConfigVersion,
    }

    for TestCase { input, expected } in [
        TestCase {
            input: FirmwareVersion {
                version: 1,
                major: 0,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::Ancient,
        },
        TestCase {
            input: FirmwareVersion {
                version: 2,
                major: 0,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::Ancient,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 0,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 7,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 7,
                minor: 0,
                patch: 1929,
            },
            expected: DeviceConfigVersion::V3,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 7,
                minor: 0,
                patch: 1930,
            },
            expected: DeviceConfigVersion::V3P7,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 7,
                minor: 1,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3P7,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 10,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3P7,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 24,
                minor: 0,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3P24,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 24,
                minor: 1,
                patch: 0,
            },
            expected: DeviceConfigVersion::V3P24,
        },
        TestCase {
            input: FirmwareVersion {
                version: 3,
                major: 25,
                minor: 0,
                patch: 119,
            },
            expected: DeviceConfigVersion::V3P24,
        },
    ] {
        let result = DeviceConfigVersion::from(input);
        assert_eq!(expected, result, "version input: {:?}", input);
    }
}
