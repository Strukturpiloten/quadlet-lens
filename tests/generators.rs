//! Contract and opt-in execution tests for exact Podman Quadlet generators.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::FromStr;

use quadlet_lens::capability::PodmanVersion;
use serde::Deserialize;

const MATRIX: &str = include_str!("../tools/generator-matrix.toml");
const EXPECTED_IMAGE_VERSIONS: &[&str] = &[
    "5.4.0", "5.4.1", "5.4.2", "5.5.0", "5.5.1", "5.5.2", "5.6.0", "5.6.1", "5.6.2", "5.7.0", "5.7.1", "5.8.0",
    "5.8.1", "5.8.2",
];
const EXPECTED_SOURCE_VERSIONS: &[&str] = &["5.8.3", "5.8.4", "5.8.5", "6.0.0", "6.0.1", "6.0.2"];
const QUOTED_LABEL_LITERAL_SPACE: &str =
    r#"--label "io.github.strukturpiloten.quadlet-lens.metadata={\"channel\": \"stable\"}""#;
const QUOTED_LABEL_HEX_SPACE: &str =
    r#"--label "io.github.strukturpiloten.quadlet-lens.metadata={\"channel\":\x20\"stable\"}""#;
const ENTRYPOINT_SEPARATE_ARGUMENT: &str = r#"--entrypoint "[\"/usr/bin/env\",\"sh\"]""#;
const ENTRYPOINT_EQUALS_ARGUMENT: &str = r#""--entrypoint=[\"/usr/bin/env\",\"sh\"]""#;
const RUN_INIT_ARGUMENT: &str = "--init";
const RUN_INIT_FALSE_ARGUMENT: &str = "--init=false";
const NAMED_STOP_SIGNAL_ARGUMENT: &str = "--stop-signal SIGUSR1";
const NUMERIC_STOP_SIGNAL_ARGUMENT: &str = "--stop-signal 9";
const POSITIVE_STOP_TIMEOUT_ARGUMENT: &str = "--stop-timeout 37";
const ZERO_STOP_TIMEOUT_ARGUMENT: &str = "--stop-timeout 0";
const PULL_CASES: &[(&str, &str)] = &[
    ("pull-always.service", "--pull always"),
    ("pull-missing.service", "--pull missing"),
    ("pull-never.service", "--pull never"),
    ("pull-newer.service", "--pull newer"),
];
const PIDS_LIMIT_CASES: &[(&str, &str)] = &[
    ("pids-limit-finite.service", "--pids-limit 127"),
    ("pids-limit-unlimited.service", "--pids-limit -1"),
];
const HOSTNAME_SEPARATE_ARGUMENT: &str = "--hostname app.example";
const SHM_SIZE_CASES: &[(&str, &str)] = &[
    ("shm-size-container.service", "--shm-size 67108864b"),
    ("shm-size-zero.service", "--shm-size 0"),
    ("shm-size-pod.service", "--shm-size 32m"),
];
const CAP_DROP_ARGUMENTS: &[&str] = &[
    "--cap-drop cap_net_admin",
    "--cap-drop all",
    "--cap-drop cap_dac_override",
    "--cap-drop cap_ipc_owner",
];
const CAP_ADD_ARGUMENTS: &[&str] = &[
    "--cap-add cap_net_admin",
    "--cap-add all",
    "--cap-add cap_dac_override",
    "--cap-add cap_ipc_owner",
];
const CAP_DROP_ALL_ARGUMENT: &str = "--cap-drop all";
const CAP_ADD_NET_BIND_SERVICE_ARGUMENT: &str = "--cap-add cap_net_bind_service";
const TMPFS_ARGUMENT: &str = "--tmpfs /data:mode=755,uid=1009,gid=1009";
const TMPFS_PRE_RESET_PATHS: &[&str] = &["/earlier-one", "/earlier-two"];
const SYSCTL_ARGUMENT: &str = "--sysctl net.ipv4.ip_forward=1";
const SYSCTL_PRE_RESET_SETTINGS: &[&str] = &["net.ipv4.conf.all.rp_filter=2", "net.ipv4.tcp_syncookies=0"];
const ULIMIT_ARGUMENTS: &[&str] = &["--ulimit nproc=4096:8192", "--ulimit stack=-1:-1"];
const ULIMIT_PRE_RESET_LIMITS: &[&str] = &["core=0:0", "nofile=1024:2048"];
const ULIMIT_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &["--ulimit=", "--ulimit \"\"", "--ulimit ''"];
const DEVICE_ARGUMENTS: &[&str] = &[
    "--device /dev/null:/dev/final-null:r",
    "--device /dev/zero:/dev/final-zero:w",
];
const DEVICE_PRE_RESET_MAPPINGS: &[&str] = &["/dev/null:/dev/pre-null:r", "/dev/zero:/dev/pre-zero:w"];
const DEVICE_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--device=",
    "--device \"\"",
    "--device ''",
    "--device=/dev/null:/dev/final-null:r",
    "--device=/dev/zero:/dev/final-zero:w",
];
const LOG_DRIVER_ARGUMENT: &str = "--log-driver k8s-file";
const LOG_OPT_ARGUMENTS: &[&str] = &[
    "--log-opt tag=quadlet-lens-final",
    "--log-opt path=/tmp/quadlet-lens-final.log",
];
const LOGGING_PRE_RESET_VALUES: &[&str] = &["tag=quadlet-lens-pre", "path=/tmp/quadlet-lens-pre.log"];
const LOGGING_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--log-driver=",
    "--log-driver \"k8s-file\"",
    "--log-driver 'k8s-file'",
    "--log-opt=",
    "--log-opt=tag=quadlet-lens-final",
    "--log-opt=path=/tmp/quadlet-lens-final.log",
    "--log-opt \"tag=quadlet-lens-final\"",
    "--log-opt \"path=/tmp/quadlet-lens-final.log\"",
    "--log-opt 'tag=quadlet-lens-final'",
    "--log-opt 'path=/tmp/quadlet-lens-final.log'",
];
const NETWORK_IDENTITY_IP_ARGUMENT: &str = "--ip 192.0.2.40";
const NETWORK_IDENTITY_IP6_ARGUMENT: &str = "--ip6 2001:db8::40";
const NETWORK_IDENTITY_NETWORK_ARGUMENT: &str = "--network bridge";
const NETWORK_ALIAS_ARGUMENTS: &[&str] = &["--network-alias final-one", "--network-alias final-two"];
const NETWORK_ALIAS_PRE_RESET_VALUES: &[&str] = &["pre-one", "pre-two"];
const NETWORK_IDENTITY_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--ip=",
    "--ip6=",
    "--network=bridge",
    "--network-alias=",
    "--network-alias=final-one",
    "--network-alias=final-two",
    "--network-alias \"final-one\"",
    "--network-alias \"final-two\"",
    "--network-alias 'final-one'",
    "--network-alias 'final-two'",
];
const NETWORK_OPTIONS_DRIVER_ARGUMENT: &str = "--driver bridge";
const NETWORK_OPTIONS_ARGUMENTS: &[&str] = &["--opt alpha=final", "--opt zeta=last"];
const NETWORK_OPTIONS_BARE_ARGUMENT: &str = "--opt bare-token";
const NETWORK_OPTIONS_PRE_RESET_VALUES: &[&str] = &["pre-one", "pre-two"];
const NETWORK_OPTIONS_ALTERNATE_FORMS: &[&str] = &[
    "--driver=bridge",
    "--driver \"bridge\"",
    "--opt=alpha=final",
    "--opt=bare-token",
    "--opt=zeta=last",
    "--opt \"alpha=final\"",
    "--opt \"bare-token\"",
    "--opt \"zeta=last\"",
    "--opt alpha=first",
];
const NETWORK_LABEL_ARGUMENTS: &[&str] = &[
    "--label alpha=final",
    "--label embedded=a=b",
    "--label empty=",
    "--label zeta=last",
];
const NETWORK_LABEL_BARE_ARGUMENT: &str = "--label bare-token";
const NETWORK_LABEL_QUOTED_LITERAL_SPACE: &str = r#"--label "quoted=one value""#;
const NETWORK_LABEL_QUOTED_HEX_SPACE: &str = r#"--label "quoted=one\x20value""#;
const NETWORK_LABEL_PRE_RESET_VALUES: &[&str] = &["pre-one", "pre-two", "alpha=first"];
const NETWORK_LABEL_ALTERNATE_FORMS: &[&str] = &[
    "--label=alpha=final",
    "--label=embedded=a=b",
    "--label=empty=",
    "--label=bare-token",
    "--label=zeta=last",
    "--label \"alpha=final\"",
    "--label \"embedded=a=b\"",
    "--label \"empty=\"",
    "--label \"bare-token\"",
    "--label \"zeta=last\"",
];
const VOLUME_LABEL_ARGUMENTS: &[&str] = &[
    "--label alpha=final",
    "--label embedded=a=b",
    "--label empty=",
    "--label zeta=last",
];
const VOLUME_LABEL_BARE_ARGUMENT: &str = "--label bare-token";
const VOLUME_LABEL_QUOTED_LITERAL_SPACE: &str = r#"--label "quoted=one value""#;
const VOLUME_LABEL_QUOTED_HEX_SPACE: &str = r#"--label "quoted=one\x20value""#;
const VOLUME_LABEL_PRE_RESET_VALUES: &[&str] = &["pre-one", "pre-two", "alpha=first"];
const VOLUME_LABEL_ALTERNATE_FORMS: &[&str] = &[
    "--label=alpha=final",
    "--label=embedded=a=b",
    "--label=empty=",
    "--label=bare-token",
    "--label=zeta=last",
    "--label \"alpha=final\"",
    "--label \"embedded=a=b\"",
    "--label \"empty=\"",
    "--label \"bare-token\"",
    "--label \"zeta=last\"",
];
const NETWORK_IPAM_DRIVER_ARGUMENT: &str = "--ipam-driver host-local";
const NETWORK_IPAM_COLUMN_ARGUMENTS: &[&str] = &[
    "--subnet 10.212.0.0/24",
    "--gateway 10.212.0.1",
    "--ip-range 10.212.0.64/26",
    "--subnet 10.213.0.0/24",
    "--gateway 10.213.0.1",
    "--ip-range 10.213.0.64/26",
];
const NETWORK_IPAM_PRE_RESET_VALUES: &[&str] = &[
    "10.210.0.0/24",
    "10.211.0.0/24",
    "10.210.0.1",
    "10.211.0.1",
    "10.210.0.64/26",
    "10.211.0.64/26",
];
const NETWORK_IPAM_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--ipam-driver=host-local",
    "--ipam-driver \"host-local\"",
    "--ipam-driver ''",
    "--subnet=10.212.0.0/24",
    "--gateway=10.212.0.1",
    "--ip-range=10.212.0.64/26",
    "--subnet \"10.212.0.0/24\"",
    "--gateway \"10.212.0.1\"",
    "--ip-range \"10.212.0.64/26\"",
];
const NETWORK_BOOLEAN_CASES: &[(&str, &str, &str, Option<&str>)] = &[
    (
        "network-internal-omitted-network.service",
        "Internal omitted",
        "--internal",
        None,
    ),
    (
        "network-internal-true-network.service",
        "Internal=true",
        "--internal",
        Some("--internal"),
    ),
    (
        "network-internal-false-network.service",
        "Internal=false",
        "--internal",
        Some("--internal=false"),
    ),
    ("network-ipv6-omitted-network.service", "IPv6 omitted", "--ipv6", None),
    (
        "network-ipv6-true-network.service",
        "IPv6=true",
        "--ipv6",
        Some("--ipv6"),
    ),
    (
        "network-ipv6-false-network.service",
        "IPv6=false",
        "--ipv6",
        Some("--ipv6=false"),
    ),
];
const DNS_ARGUMENTS: &[&str] = &["--dns 9.9.9.9", "--dns 2001:4860:4860::8888"];
const DNS_PRE_RESET_VALUES: &[&str] = &["1.1.1.1"];
const DNS_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--dns=",
    "--dns \"\"",
    "--dns ''",
    "--dns=9.9.9.9",
    "--dns=2001:4860:4860::8888",
    "--dns \"9.9.9.9\"",
    "--dns \"2001:4860:4860::8888\"",
    "--dns '9.9.9.9'",
    "--dns '2001:4860:4860::8888'",
];
const DNS_OPTION_ARGUMENTS: &[&str] = &["--dns-option ndots:1", "--dns-option use-vc"];
const DNS_OPTION_PRE_RESET_VALUES: &[&str] = &["rotate"];
const DNS_OPTION_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--dns-option=",
    "--dns-option \"\"",
    "--dns-option ''",
    "--dns-option=ndots:1",
    "--dns-option=use-vc",
    "--dns-option \"ndots:1\"",
    "--dns-option \"use-vc\"",
    "--dns-option 'ndots:1'",
    "--dns-option 'use-vc'",
];
const DNS_SEARCH_ARGUMENTS: &[&str] = &["--dns-search dc1.example.com", "--dns-search ."];
const DNS_SEARCH_PRE_RESET_VALUES: &[&str] = &["pre.example.com"];
const DNS_SEARCH_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--dns-search=",
    "--dns-search \"\"",
    "--dns-search ''",
    "--dns-search=dc1.example.com",
    "--dns-search=.",
    "--dns-search \"dc1.example.com\"",
    "--dns-search \".\"",
    "--dns-search 'dc1.example.com'",
    "--dns-search '.'",
];
const EXPOSE_ARGUMENTS: &[&str] = &[
    "--expose 3000",
    "--expose 8080-8085",
    "--expose 9090/tcp",
    "--expose 5353/udp",
];
const EXPOSE_PRE_RESET_VALUES: &[&str] = &["1000"];
const EXPOSE_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--expose=",
    "--expose \"\"",
    "--expose ''",
    "--expose=3000",
    "--expose=8080-8085",
    "--expose=9090/tcp",
    "--expose=5353/udp",
    "--expose \"3000\"",
    "--expose \"8080-8085\"",
    "--expose \"9090/tcp\"",
    "--expose \"5353/udp\"",
    "--expose '3000'",
    "--expose '8080-8085'",
    "--expose '9090/tcp'",
    "--expose '5353/udp'",
];
const ANNOTATION_ARGUMENTS: &[&str] = &[
    "--annotation io.github.strukturpiloten.quadlet-lens.alpha=one",
    "--annotation io.github.strukturpiloten.quadlet-lens.beta=two",
];
const ANNOTATION_PRE_RESET_VALUES: &[&str] = &["org.example.pre=one"];
const ANNOTATION_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--annotation=",
    "--annotation \"\"",
    "--annotation ''",
    "--annotation key-only",
    "--annotation=io.github.strukturpiloten.quadlet-lens.alpha=one",
    "--annotation=io.github.strukturpiloten.quadlet-lens.beta=two",
    "--annotation \"io.github.strukturpiloten.quadlet-lens.alpha=one\"",
    "--annotation \"io.github.strukturpiloten.quadlet-lens.beta=two\"",
    "--annotation 'io.github.strukturpiloten.quadlet-lens.alpha=one'",
    "--annotation 'io.github.strukturpiloten.quadlet-lens.beta=two'",
];
const APPARMOR_ARGUMENT: &str = "--security-opt apparmor=quadlet-lens-profile";
const APPARMOR_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=",
    "--security-opt \"apparmor=quadlet-lens-profile\"",
    "--security-opt 'apparmor=quadlet-lens-profile'",
    "apparmor=unconfined",
    "label=",
    "seccomp=",
    "mask=",
];
const NO_NEW_PRIVILEGES_ARGUMENT: &str = "--security-opt=no-new-privileges";
const NO_NEW_PRIVILEGES_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt no-new-privileges",
    "--security-opt=\"no-new-privileges\"",
    "--security-opt='no-new-privileges'",
    "--security-opt=no-new-privileges=true",
    "--security-opt=no-new-privileges=false",
    "apparmor=",
    "label=",
    "seccomp=",
    "mask=",
];
const SECURITY_LABEL_DISABLE_ARGUMENT: &str = "--security-opt label=disable";
const SECURITY_LABEL_DISABLE_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=label=disable",
    "--security-opt \"label=disable\"",
    "--security-opt 'label=disable'",
    "--security-opt label=disable=true",
    "--security-opt label=disable=false",
    "label=nested",
    "label=type:",
    "label=level:",
    "label=filetype:",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
    "mask=",
];
const SECURITY_LABEL_FILE_TYPE_ARGUMENT: &str = "--security-opt label=filetype:container_file_t";
const SECURITY_LABEL_FILE_TYPE_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=label=filetype:container_file_t",
    "--security-opt \"label=filetype:container_file_t\"",
    "--security-opt 'label=filetype:container_file_t'",
    "--security-opt label=filetype=container_file_t",
    "label=filetype:custom_file_t",
    "label=disable",
    "label=nested",
    "label=type:",
    "label=level:",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
    "mask=",
];
const SECURITY_LABEL_LEVEL_ARGUMENT: &str = "--security-opt label=level:s0:c1,c2";
const SECURITY_LABEL_LEVEL_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=label=level:s0:c1,c2",
    "--security-opt \"label=level:s0:c1,c2\"",
    "--security-opt 'label=level:s0:c1,c2'",
    "--security-opt label=level=s0:c1,c2",
    "label=level:s0:c3,c4",
    "label=disable",
    "label=nested",
    "label=type:",
    "label=filetype:",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
    "mask=",
];
const SECURITY_LABEL_NESTED_ARGUMENT: &str = "--security-opt label=nested";
const SECURITY_LABEL_NESTED_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=label=nested",
    "--security-opt \"label=nested\"",
    "--security-opt 'label=nested'",
    "--security-opt label=nested=true",
    "--security-opt label=nested=false",
    "label=disable",
    "label=type:",
    "label=level:",
    "label=filetype:",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
    "mask=",
];
const SECURITY_LABEL_TYPE_ARGUMENT: &str = "--security-opt label=type:container_t";
const SECURITY_LABEL_TYPE_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=label=type:container_t",
    "--security-opt \"label=type:container_t\"",
    "--security-opt 'label=type:container_t'",
    "--security-opt label=type=container_t",
    "label=type:custom_t",
    "label=disable",
    "label=nested",
    "label=level:",
    "label=filetype:",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
    "mask=",
];
const MASK_ARGUMENT: &str = "--security-opt mask=/proc/acpi:/sys/firmware";
const MASK_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=mask=/proc/acpi:/sys/firmware",
    "--security-opt \"mask=/proc/acpi:/sys/firmware\"",
    "--security-opt 'mask=/proc/acpi:/sys/firmware'",
    "mask=/pre/one:/pre/two",
    "mask=/pre/three",
    "mask=\"",
    "mask='",
    "unmask=",
    "label=",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
];
const UNMASK_ARGUMENTS: &[&str] = &[
    "--security-opt unmask=ALL",
    "--security-opt unmask=/proc/acpi:/sys/firmware",
];
const UNMASK_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=unmask=ALL",
    "--security-opt=unmask=/proc/acpi:/sys/firmware",
    "--security-opt \"unmask=ALL\"",
    "--security-opt \"unmask=/proc/acpi:/sys/firmware\"",
    "--security-opt 'unmask=ALL'",
    "--security-opt 'unmask=/proc/acpi:/sys/firmware'",
    "unmask=/pre/one:/pre/two",
    "unmask=/pre/three/*",
    "unmask=\"",
    "unmask='",
    "--security-opt mask=",
    "label=",
    "apparmor=",
    "seccomp=",
    "no-new-privileges",
];
const SECCOMP_PROFILE_CASES: &[(&str, &str)] = &[
    ("seccomp-unconfined.service", "--security-opt seccomp=unconfined"),
    (
        "seccomp-json-path.service",
        "--security-opt seccomp=/tmp/quadlet-lens-profile.json",
    ),
];
const SECCOMP_PROFILE_UNRELATED_OR_ALTERNATE_FORMS: &[&str] = &[
    "--security-opt=seccomp=",
    "--security-opt \"seccomp=",
    "--security-opt 'seccomp=",
    "seccomp=\"",
    "seccomp='",
    "apparmor=",
    "label=",
    "no-new-privileges",
    "mask=",
];
const MEMORY_ARGUMENT: &str = "--memory 16777216b";
const MEMORY_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--memory=",
    "--memory \"\"",
    "--memory ''",
    "--memory \"16777216b\"",
    "--memory 32m",
];
const INTERACTIVE_ARGUMENT: &str = "--interactive";
const INTERACTIVE_IMAGE: &str = "registry.example.invalid/interactive:1";
const INTERACTIVE_ALTERNATE_FORMS: &[&str] = &[
    "--interactive=",
    "--interactive=true",
    "--interactive=false",
    "--interactive \"\"",
    "--interactive ''",
    "--interactive=\"\"",
    "--interactive=''",
    "\"--interactive\"",
    "'--interactive'",
];
const TTY_ARGUMENT: &str = "--tty";
const TTY_IMAGE: &str = "registry.example.invalid/tty:1";
const TTY_ALTERNATE_FORMS: &[&str] = &[
    "--tty=",
    "--tty=true",
    "--tty=false",
    "--tty \"\"",
    "--tty ''",
    "--tty=\"\"",
    "--tty=''",
    "\"--tty\"",
    "'--tty'",
];
const PRIVILEGED_TRUE_ARGUMENT: &str = "--privileged";
const PRIVILEGED_FALSE_ARGUMENT: &str = "--privileged=false";
const PRIVILEGED_TRUE_IMAGE: &str = "registry.example.invalid/privileged-true:1";
const PRIVILEGED_FALSE_IMAGE: &str = "registry.example.invalid/privileged-false:1";
const PRIVILEGED_ALTERNATE_FORMS: &[&str] = &[
    "--privileged=true",
    "--privileged true",
    "--privileged false",
    "--privileged=\"true\"",
    "--privileged=\"false\"",
    "--privileged='true'",
    "--privileged='false'",
    "--privileged \"true\"",
    "--privileged \"false\"",
    "--privileged 'true'",
    "--privileged 'false'",
    "\"--privileged\"",
    "'--privileged'",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorMatrix {
    schema: u32,
    support_minimum: String,
    tracked_current: String,
    checked_on: String,
    official_image_maximum: String,
    source_repository: String,
    builder_reference: String,
    image: Vec<GeneratorImage>,
    source: Vec<GeneratorSource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorImage {
    version: String,
    reference: String,
    #[serde(default)]
    smoke: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorSource {
    version: String,
    commit: String,
    #[serde(default)]
    smoke: bool,
}

struct SecurityLabelFixtures {
    disable: (PathBuf, Vec<String>),
    file_type: (PathBuf, Vec<String>),
    level: (PathBuf, Vec<String>),
    nested: (PathBuf, Vec<String>),
    process_type: (PathBuf, Vec<String>),
    mask: (PathBuf, Vec<String>),
    unmask: (PathBuf, Vec<String>),
}

struct GeneratorFixtures {
    memory: (PathBuf, Vec<String>),
    interactive: (PathBuf, Vec<String>),
    tty: (PathBuf, Vec<String>),
    privileged: (PathBuf, Vec<String>),
    logging: (PathBuf, Vec<String>),
    network_identity: (PathBuf, Vec<String>),
    network_driver_options: (PathBuf, Vec<String>),
    network_labels: (PathBuf, Vec<String>),
    volume_labels: (PathBuf, Vec<String>),
    network_booleans: (PathBuf, Vec<String>),
    network_ipam: (PathBuf, Vec<String>),
    volume_driver_options: VolumeDriverOptionsFixtures,
    volume_copy: (PathBuf, Vec<String>),
}

struct VolumeDriverOptionsFixtures {
    main: (PathBuf, Vec<String>),
    without_device: (PathBuf, Vec<String>),
    unmatched_quote: (PathBuf, Vec<String>),
    type_without_device: PathBuf,
}

#[test]
fn volume_label_quote_expectations_use_unescaped_output_text() {
    assert_eq!(VOLUME_LABEL_QUOTED_LITERAL_SPACE, NETWORK_LABEL_QUOTED_LITERAL_SPACE);
    assert_eq!(VOLUME_LABEL_QUOTED_HEX_SPACE, NETWORK_LABEL_QUOTED_HEX_SPACE);
}

#[test]
fn generator_matrix_is_exact_complete_and_digest_pinned() -> Result<(), String> {
    let matrix = parse_matrix()?;
    assert_eq!(matrix.schema, 1);
    assert_eq!(matrix.support_minimum, "5.4.0");
    assert_eq!(matrix.tracked_current, "6.0.2");
    assert_eq!(matrix.checked_on, "2026-08-06");
    assert_eq!(matrix.official_image_maximum, "5.8.2");

    assert_eq!(matrix.source_repository, "https://github.com/containers/podman.git");
    validate_digest_pinned_reference(&matrix.builder_reference, "docker.io/library/golang:")?;

    let image_versions: Vec<_> = matrix.image.iter().map(|image| image.version.as_str()).collect();
    assert_eq!(image_versions, EXPECTED_IMAGE_VERSIONS);
    assert_eq!(matrix.image.iter().filter(|image| image.smoke).count(), 2);
    assert!(matrix.image.first().is_some_and(|image| image.smoke));
    assert!(matrix.image.last().is_some_and(|image| image.smoke));

    let mut unique_references = BTreeSet::new();
    for image in &matrix.image {
        PodmanVersion::from_str(&image.version).map_err(|error| error.to_string())?;
        let prefix = format!("quay.io/podman/stable:v{}-immutable", image.version);
        validate_digest_pinned_reference(&image.reference, &prefix)?;
        if !unique_references.insert(&image.reference) {
            return Err(format!("duplicate generator image {}", image.reference));
        }
    }

    let source_versions: Vec<_> = matrix.source.iter().map(|source| source.version.as_str()).collect();
    assert_eq!(source_versions, EXPECTED_SOURCE_VERSIONS);
    assert_eq!(matrix.source.iter().filter(|source| source.smoke).count(), 1);
    assert!(matrix.source.last().is_some_and(|source| source.smoke));
    for source in &matrix.source {
        PodmanVersion::from_str(&source.version).map_err(|error| error.to_string())?;
        if source.commit.len() != 40
            || !source
                .commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(format!(
                "Podman {} source commit must be a full lowercase Git object ID",
                source.version
            ));
        }
    }

    assert_eq!(
        matrix.image.first().map(|image| image.version.as_str()),
        Some(matrix.support_minimum.as_str())
    );
    assert_eq!(
        matrix.image.last().map(|image| image.version.as_str()),
        Some(matrix.official_image_maximum.as_str())
    );
    assert_eq!(
        matrix.source.first().map(|source| source.version.as_str()),
        Some("5.8.3")
    );
    assert_eq!(
        matrix.source.last().map(|source| source.version.as_str()),
        Some(matrix.tracked_current.as_str())
    );
    let memory_versions = matrix
        .image
        .iter()
        .map(|image| image.version.as_str())
        .chain(matrix.source.iter().map(|source| source.version.as_str()))
        .filter(|version| PodmanVersion::from_str(version).is_ok_and(|version| version >= PodmanVersion::new(5, 5, 0)))
        .count();
    assert_eq!(memory_versions, 17);
    Ok(())
}

#[test]
#[ignore = "pulls or builds exact Podman releases and executes their Quadlet generators"]
fn supported_generators_match_the_first_conversion_fixture() -> Result<(), String> {
    let matrix = parse_matrix()?;
    let engine = env::var("QUADLET_LENS_CONTAINER_ENGINE").unwrap_or_else(|_| "podman".to_owned());
    let lane = env::var("QUADLET_LENS_GENERATOR_LANE").unwrap_or_else(|_| "smoke".to_owned());
    if lane != "smoke" && lane != "full" {
        return Err(format!("unknown generator lane `{lane}`; expected `smoke` or `full`"));
    }
    let version_filter = env::var("QUADLET_LENS_GENERATOR_VERSION").ok();
    let selected_images: Vec<_> = matrix
        .image
        .iter()
        .filter(|image| selected(&image.version, image.smoke, &lane, version_filter.as_deref()))
        .collect();
    let selected_sources: Vec<_> = matrix
        .source
        .iter()
        .filter(|source| selected(&source.version, source.smoke, &lane, version_filter.as_deref()))
        .collect();
    if selected_images.is_empty() && selected_sources.is_empty() {
        return Err(format!("generator selection is empty for lane `{lane}`"));
    }

    let fixture = fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    let fixtures = load_generator_fixtures()?;
    let apparmor_fixture = apparmor_fixture_directory()?;
    let apparmor_expected = expected_fragments(&apparmor_fixture)?;
    let no_new_privileges_fixture = no_new_privileges_fixture_directory()?;
    let no_new_privileges_expected = expected_fragments(&no_new_privileges_fixture)?;
    let seccomp_profile_fixture = seccomp_profile_fixture_directory()?;
    let seccomp_profile_expected = expected_fragments(&seccomp_profile_fixture)?;
    let security_labels = load_security_label_fixtures()?;
    for image in selected_images {
        eprintln!("testing Podman {} with {}", image.version, image.reference);
        verify_image_version(&engine, image)?;
        let output = run_generator(&engine, image, &fixture)?;
        verify_generator_output(&image.version, &expected, &output)?;
        verify_image_isolated_fixtures(&engine, image, &fixtures)?;
        let apparmor_output = run_generator_raw(&engine, image, &apparmor_fixture)?;
        verify_apparmor_generator_output(&image.version, &apparmor_expected, &apparmor_output)?;
        let no_new_privileges_output = run_generator(&engine, image, &no_new_privileges_fixture)?;
        verify_no_new_privileges_generator_output(
            &image.version,
            &no_new_privileges_expected,
            &no_new_privileges_output,
        )?;
        let seccomp_profile_output = run_generator(&engine, image, &seccomp_profile_fixture)?;
        verify_seccomp_profile_generator_output(&image.version, &seccomp_profile_expected, &seccomp_profile_output)?;
        verify_image_security_label_disable(&engine, image, &security_labels.disable)?;
        verify_image_security_label_file_type(&engine, image, &security_labels.file_type)?;
        verify_image_security_label_level(&engine, image, &security_labels.level)?;
        verify_image_security_label_nested(&engine, image, &security_labels.nested)?;
        verify_image_security_label_type(&engine, image, &security_labels.process_type)?;
        verify_image_path_security_options(&engine, image, &security_labels)?;
    }
    for source in selected_sources {
        eprintln!("testing Podman {} source at {}", source.version, source.commit);
        let generator = build_source_generator(&engine, &matrix, source)?;
        verify_source_version(&engine, &matrix.builder_reference, source, &generator)?;
        let output = run_source_generator(&engine, &matrix.builder_reference, source, &generator, &fixture)?;
        verify_generator_output(&source.version, &expected, &output)?;
        verify_source_isolated_fixtures(&engine, &matrix, source, &generator, &fixtures)?;
        let apparmor_output = run_source_generator(
            &engine,
            &matrix.builder_reference,
            source,
            &generator,
            &apparmor_fixture,
        )?;
        verify_apparmor_generator_output(&source.version, &apparmor_expected, &apparmor_output)?;
        let no_new_privileges_output = run_source_generator(
            &engine,
            &matrix.builder_reference,
            source,
            &generator,
            &no_new_privileges_fixture,
        )?;
        verify_no_new_privileges_generator_output(
            &source.version,
            &no_new_privileges_expected,
            &no_new_privileges_output,
        )?;
        let seccomp_profile_output = run_source_generator(
            &engine,
            &matrix.builder_reference,
            source,
            &generator,
            &seccomp_profile_fixture,
        )?;
        verify_seccomp_profile_generator_output(&source.version, &seccomp_profile_expected, &seccomp_profile_output)?;
        verify_source_security_label_disable(&engine, &matrix, source, &generator, &security_labels.disable)?;
        verify_source_security_label_file_type(&engine, &matrix, source, &generator, &security_labels.file_type)?;
        verify_source_security_label_level(&engine, &matrix, source, &generator, &security_labels.level)?;
        verify_source_security_label_nested(&engine, &matrix, source, &generator, &security_labels.nested)?;
        verify_source_security_label_type(&engine, &matrix, source, &generator, &security_labels.process_type)?;
        verify_source_path_security_options(&engine, &matrix, source, &generator, &security_labels)?;
    }
    Ok(())
}

fn validate_digest_pinned_reference(reference: &str, expected_prefix: &str) -> Result<(), String> {
    let suffix = reference
        .strip_prefix(expected_prefix)
        .ok_or_else(|| format!("container image `{reference}` must use an exact tag and sha256 digest"))?;
    let (tag, digest) = suffix
        .split_once("@sha256:")
        .ok_or_else(|| format!("container image `{reference}` must use an exact tag and sha256 digest"))?;
    let tag_is_valid = if expected_prefix.ends_with(':') {
        !tag.is_empty()
    } else {
        tag.is_empty()
    };
    if !tag_is_valid || tag.contains('@') {
        return Err(format!("container image `{reference}` has an invalid tag"));
    }
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("container image `{reference}` has an invalid digest"));
    }
    Ok(())
}

fn selected(version: &str, smoke: bool, lane: &str, version_filter: Option<&str>) -> bool {
    version_filter.map_or_else(|| lane == "full" || smoke, |filter| version == filter)
}

fn parse_matrix() -> Result<GeneratorMatrix, String> {
    toml::from_str(MATRIX).map_err(|error| format!("invalid generator matrix: {error}"))
}

fn fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/first-conversion-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn memory_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/memory-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn interactive_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/podman-args-interactive-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn tty_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/podman-args-tty-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn privileged_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/podman-args-privileged-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn logging_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/container-logging-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn network_identity_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/container-network-identity-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn network_driver_options_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/network-driver-options-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn network_labels_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/network-labels-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn network_booleans_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/network-booleans-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn network_ipam_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/network-ipam-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_driver_options_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/volume-driver-options-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_labels_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/volume-labels-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_copy_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/volume-copy-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_options_without_device_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-options-without-device-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_options_unmatched_quote_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-options-unmatched-quote-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn volume_type_without_device_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-type-without-device-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_memory_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = memory_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_interactive_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = interactive_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_tty_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = tty_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_privileged_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = privileged_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_generator_fixtures() -> Result<GeneratorFixtures, String> {
    Ok(GeneratorFixtures {
        memory: load_memory_fixture()?,
        interactive: load_interactive_fixture()?,
        tty: load_tty_fixture()?,
        privileged: load_privileged_fixture()?,
        logging: load_logging_fixture()?,
        network_identity: load_network_identity_fixture()?,
        network_driver_options: load_network_driver_options_fixture()?,
        network_labels: load_network_labels_fixture()?,
        volume_labels: load_volume_labels_fixture()?,
        network_booleans: load_network_booleans_fixture()?,
        network_ipam: load_network_ipam_fixture()?,
        volume_driver_options: load_volume_driver_options_fixtures()?,
        volume_copy: load_volume_copy_fixture()?,
    })
}

fn load_logging_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = logging_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_network_identity_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = network_identity_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_network_driver_options_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = network_driver_options_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_network_labels_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = network_labels_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_volume_labels_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_labels_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_volume_copy_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_copy_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_network_booleans_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = network_booleans_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_network_ipam_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = network_ipam_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_volume_driver_options_fixtures() -> Result<VolumeDriverOptionsFixtures, String> {
    let load = |directory: fn() -> Result<PathBuf, String>| -> Result<(PathBuf, Vec<String>), String> {
        let fixture = directory()?;
        let expected = expected_fragments(&fixture)?;
        Ok((fixture, expected))
    };
    Ok(VolumeDriverOptionsFixtures {
        main: load(volume_driver_options_fixture_directory)?,
        without_device: load(volume_options_without_device_fixture_directory)?,
        unmatched_quote: load(volume_options_unmatched_quote_fixture_directory)?,
        type_without_device: volume_type_without_device_fixture_directory()?,
    })
}

fn verify_image_memory(engine: &str, image: &GeneratorImage, fixture: &(PathBuf, Vec<String>)) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_memory_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_isolated_fixtures(
    engine: &str,
    image: &GeneratorImage,
    fixtures: &GeneratorFixtures,
) -> Result<(), String> {
    verify_image_memory(engine, image, &fixtures.memory)?;
    verify_image_interactive(engine, image, &fixtures.interactive)?;
    verify_image_tty(engine, image, &fixtures.tty)?;
    verify_image_privileged(engine, image, &fixtures.privileged)?;
    verify_image_logging(engine, image, &fixtures.logging)?;
    verify_image_network_identity(engine, image, &fixtures.network_identity)?;
    verify_image_network_driver_options(engine, image, &fixtures.network_driver_options)?;
    verify_image_network_labels(engine, image, &fixtures.network_labels)?;
    verify_image_volume_labels(engine, image, &fixtures.volume_labels)?;
    verify_image_network_booleans(engine, image, &fixtures.network_booleans)?;
    verify_image_network_ipam(engine, image, &fixtures.network_ipam)?;
    verify_image_volume_driver_options(engine, image, &fixtures.volume_driver_options)?;
    verify_image_volume_copy(engine, image, &fixtures.volume_copy)
}

fn verify_source_memory(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_memory_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_isolated_fixtures(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixtures: &GeneratorFixtures,
) -> Result<(), String> {
    verify_source_memory(engine, matrix, source, generator, &fixtures.memory)?;
    verify_source_interactive(engine, matrix, source, generator, &fixtures.interactive)?;
    verify_source_tty(engine, matrix, source, generator, &fixtures.tty)?;
    verify_source_privileged(engine, matrix, source, generator, &fixtures.privileged)?;
    verify_source_logging(engine, matrix, source, generator, &fixtures.logging)?;
    verify_source_network_identity(engine, matrix, source, generator, &fixtures.network_identity)?;
    verify_source_network_driver_options(engine, matrix, source, generator, &fixtures.network_driver_options)?;
    verify_source_network_labels(engine, matrix, source, generator, &fixtures.network_labels)?;
    verify_source_volume_labels(engine, matrix, source, generator, &fixtures.volume_labels)?;
    verify_source_network_booleans(engine, matrix, source, generator, &fixtures.network_booleans)?;
    verify_source_network_ipam(engine, matrix, source, generator, &fixtures.network_ipam)?;
    verify_source_volume_driver_options(engine, matrix, source, generator, &fixtures.volume_driver_options)?;
    verify_source_volume_copy(engine, matrix, source, generator, &fixtures.volume_copy)
}

fn verify_image_logging(engine: &str, image: &GeneratorImage, fixture: &(PathBuf, Vec<String>)) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_logging_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_interactive(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_interactive_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_interactive(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_interactive_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_tty(engine: &str, image: &GeneratorImage, fixture: &(PathBuf, Vec<String>)) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_tty_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_tty(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_tty_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_privileged(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_privileged_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_privileged(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_privileged_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_logging(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_logging_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_network_identity(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_network_identity_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_network_identity(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_network_identity_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_network_driver_options(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_network_driver_options_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_network_driver_options(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_network_driver_options_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_network_labels(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_network_labels_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_network_labels(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_network_labels_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_volume_labels(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_volume_labels_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_volume_labels(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_volume_labels_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_network_booleans(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_network_boolean_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_network_booleans(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_network_boolean_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_network_ipam(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_network_ipam_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_network_ipam(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_network_ipam_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_volume_driver_options(
    engine: &str,
    image: &GeneratorImage,
    fixtures: &VolumeDriverOptionsFixtures,
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixtures.main.0)?;
    verify_volume_driver_options_generator_output(&image.version, &fixtures.main.1, &output)?;
    let output = run_generator_raw(engine, image, &fixtures.without_device.0)?;
    verify_volume_options_without_device_generator_output(&image.version, &fixtures.without_device.1, &output)?;
    let output = run_generator_raw(engine, image, &fixtures.unmatched_quote.0)?;
    verify_volume_options_unmatched_quote_generator_output(&image.version, &fixtures.unmatched_quote.1, &output)?;
    let output = run_generator_raw(engine, image, &fixtures.type_without_device)?;
    verify_volume_type_without_device_generator_output(&image.version, &output)
}

fn verify_source_volume_driver_options(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixtures: &VolumeDriverOptionsFixtures,
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixtures.main.0)?;
    verify_volume_driver_options_generator_output(&source.version, &fixtures.main.1, &output)?;
    let output = run_source_generator_raw(
        engine,
        &matrix.builder_reference,
        source,
        generator,
        &fixtures.without_device.0,
    )?;
    verify_volume_options_without_device_generator_output(&source.version, &fixtures.without_device.1, &output)?;
    let output = run_source_generator_raw(
        engine,
        &matrix.builder_reference,
        source,
        generator,
        &fixtures.unmatched_quote.0,
    )?;
    verify_volume_options_unmatched_quote_generator_output(&source.version, &fixtures.unmatched_quote.1, &output)?;
    let output = run_source_generator_raw(
        engine,
        &matrix.builder_reference,
        source,
        generator,
        &fixtures.type_without_device,
    )?;
    verify_volume_type_without_device_generator_output(&source.version, &output)
}

fn verify_image_volume_copy(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_volume_copy_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_volume_copy(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_volume_copy_generator_output(&source.version, &fixture.1, &output)
}

fn apparmor_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/apparmor-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn no_new_privileges_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/no-new-privileges-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn seccomp_profile_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/seccomp-profile-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn security_label_disable_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/security-label-disable-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_security_label_disable_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = security_label_disable_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn security_label_file_type_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/security-label-file-type-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_security_label_file_type_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = security_label_file_type_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn security_label_level_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/security-label-level-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_security_label_level_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = security_label_level_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn security_label_nested_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/security-label-nested-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_security_label_nested_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = security_label_nested_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn security_label_type_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/security-label-type-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_security_label_type_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = security_label_type_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn mask_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/mask-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_mask_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = mask_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn unmask_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/unmask-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn load_unmask_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = unmask_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_security_label_fixtures() -> Result<SecurityLabelFixtures, String> {
    Ok(SecurityLabelFixtures {
        disable: load_security_label_disable_fixture()?,
        file_type: load_security_label_file_type_fixture()?,
        level: load_security_label_level_fixture()?,
        nested: load_security_label_nested_fixture()?,
        process_type: load_security_label_type_fixture()?,
        mask: load_mask_fixture()?,
        unmask: load_unmask_fixture()?,
    })
}

fn verify_image_security_label_disable(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_security_label_disable_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_security_label_disable(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_security_label_disable_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_security_label_file_type(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_security_label_file_type_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_security_label_file_type(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_security_label_file_type_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_security_label_level(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_security_label_level_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_security_label_level(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_security_label_level_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_security_label_nested(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_security_label_nested_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_security_label_nested(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_security_label_nested_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_security_label_type(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_security_label_type_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_security_label_type(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_security_label_type_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_mask(engine: &str, image: &GeneratorImage, fixture: &(PathBuf, Vec<String>)) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_mask_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_path_security_options(
    engine: &str,
    image: &GeneratorImage,
    fixtures: &SecurityLabelFixtures,
) -> Result<(), String> {
    verify_image_mask(engine, image, &fixtures.mask)?;
    verify_image_unmask(engine, image, &fixtures.unmask)
}

fn verify_source_mask(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_mask_generator_output(&source.version, &fixture.1, &output)
}

fn verify_image_unmask(engine: &str, image: &GeneratorImage, fixture: &(PathBuf, Vec<String>)) -> Result<(), String> {
    let output = run_generator(engine, image, &fixture.0)?;
    verify_unmask_generator_output(&image.version, &fixture.1, &output)
}

fn verify_source_unmask(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_unmask_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_path_security_options(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixtures: &SecurityLabelFixtures,
) -> Result<(), String> {
    verify_source_mask(engine, matrix, source, generator, &fixtures.mask)?;
    verify_source_unmask(engine, matrix, source, generator, &fixtures.unmask)
}

fn expected_fragments(fixture: &Path) -> Result<Vec<String>, String> {
    let path = fixture.join("expected-fragments.txt");
    let text = fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let fragments: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    if fragments.is_empty() {
        return Err(format!("{} must contain expected fragments", path.display()));
    }
    Ok(fragments)
}

fn verify_image_version(engine: &str, image: &GeneratorImage) -> Result<(), String> {
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--entrypoint",
            "/usr/bin/podman",
            &image.reference,
            "version",
            "--format",
            "{{.Client.Version}}",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&image.version, "version probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != image.version {
        return Err(format!("generator image for {} reports Podman {actual}", image.version));
    }
    Ok(())
}

fn run_generator(engine: &str, image: &GeneratorImage, fixture: &Path) -> Result<Output, String> {
    let output = run_generator_raw(engine, image, fixture)?;
    ensure_success(&image.version, "generator", &output)?;
    Ok(output)
}

fn run_generator_raw(engine: &str, image: &GeneratorImage, fixture: &Path) -> Result<Output, String> {
    let mount = format!("type=bind,src={},dst=/fixtures,ro", fixture.display());
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &mount,
            "-e",
            "QUADLET_UNIT_DIRS=/fixtures",
            "--entrypoint",
            "/usr/lib/systemd/system-generators/podman-system-generator",
            &image.reference,
            "-dryrun",
            "-no-kmsg-log",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    Ok(output)
}

fn build_source_generator(engine: &str, matrix: &GeneratorMatrix, source: &GeneratorSource) -> Result<PathBuf, String> {
    let matrix_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/generator-matrix");
    let source_directory = matrix_directory.join("source");
    let output_directory = matrix_directory.join("out").join(&source.version);
    let module_cache = matrix_directory.join("cache/go-mod");
    let build_cache = matrix_directory.join("cache/go-build");
    for directory in [&source_directory, &output_directory, &module_cache, &build_cache] {
        fs::create_dir_all(directory).map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    }

    checkout_source(&source_directory, &matrix.source_repository, source)?;
    let source_mount = bind_mount(&source_directory, "/src", true)?;
    let output_mount = bind_mount(&output_directory, "/out", false)?;
    let module_cache_mount = bind_mount(&module_cache, "/cache/mod", false)?;
    let build_cache_mount = bind_mount(&build_cache, "/cache/build", false)?;
    let user = container_user(engine)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--user",
            &user,
            "--mount",
            &source_mount,
            "--mount",
            &output_mount,
            "--mount",
            &module_cache_mount,
            "--mount",
            &build_cache_mount,
            "-e",
            "CGO_ENABLED=0",
            "-e",
            "HOME=/tmp",
            "-e",
            "GOMODCACHE=/cache/mod",
            "-e",
            "GOCACHE=/cache/build",
            "-w",
            "/src",
            "--entrypoint",
            "/usr/local/go/bin/go",
            &matrix.builder_reference,
            "build",
            "-trimpath",
            "-o",
            "/out/quadlet",
            "./cmd/quadlet",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&source.version, "source generator build", &output)?;

    let generator = output_directory.join("quadlet");
    if !generator.is_file() {
        return Err(format!(
            "Podman {} build did not create {}",
            source.version,
            generator.display()
        ));
    }
    Ok(generator)
}

fn checkout_source(directory: &Path, repository: &str, source: &GeneratorSource) -> Result<(), String> {
    if !directory.join(".git").is_dir() {
        let output = Command::new("git")
            .args(["init", "--quiet"])
            .arg(directory)
            .output()
            .map_err(|error| format!("cannot execute `git`: {error}"))?;
        ensure_success(&source.version, "source checkout initialization", &output)?;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args([
            "fetch",
            "--quiet",
            "--force",
            "--depth",
            "1",
            repository,
            &source.commit,
        ])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source fetch", &output)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["checkout", "--quiet", "--force", "--detach", "FETCH_HEAD"])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source checkout", &output)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source commit probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != source.commit {
        return Err(format!(
            "Podman {} checkout resolved to {actual}, expected {}",
            source.version, source.commit
        ));
    }
    Ok(())
}

fn container_user(engine: &str) -> Result<String, String> {
    if Path::new(engine).file_name().and_then(|name| name.to_str()) == Some("podman") {
        return Ok("0:0".to_owned());
    }
    let uid = command_text("id", &["-u"])?;
    let gid = command_text("id", &["-g"])?;
    Ok(format!("{uid}:{gid}"))
}

fn command_text(program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("cannot execute `{program}`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`{program}` failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn bind_mount(source: &Path, destination: &str, read_only: bool) -> Result<String, String> {
    let source = source
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", source.display()))?;
    let mode = if read_only { ",ro" } else { "" };
    Ok(format!("type=bind,src={},dst={destination}{mode}", source.display()))
}

fn verify_source_version(
    engine: &str,
    builder_reference: &str,
    source: &GeneratorSource,
    generator: &Path,
) -> Result<(), String> {
    let output_directory = generator
        .parent()
        .ok_or_else(|| format!("generator {} has no parent directory", generator.display()))?;
    let output_mount = bind_mount(output_directory, "/out", true)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &output_mount,
            "--entrypoint",
            "/out/quadlet",
            builder_reference,
            "-version",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&source.version, "source generator version probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != source.version {
        return Err(format!(
            "source generator for Podman {} reports `{actual}`",
            source.version
        ));
    }
    Ok(())
}

fn run_source_generator(
    engine: &str,
    builder_reference: &str,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &Path,
) -> Result<Output, String> {
    let output = run_source_generator_raw(engine, builder_reference, source, generator, fixture)?;
    ensure_success(&source.version, "source generator", &output)?;
    Ok(output)
}

fn run_source_generator_raw(
    engine: &str,
    builder_reference: &str,
    _source: &GeneratorSource,
    generator: &Path,
    fixture: &Path,
) -> Result<Output, String> {
    let output_directory = generator
        .parent()
        .ok_or_else(|| format!("generator {} has no parent directory", generator.display()))?;
    let output_mount = bind_mount(output_directory, "/out", true)?;
    let fixture_mount = bind_mount(fixture, "/fixtures", true)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &output_mount,
            "--mount",
            &fixture_mount,
            "-e",
            "QUADLET_UNIT_DIRS=/fixtures",
            "--entrypoint",
            "/out/quadlet",
            builder_reference,
            "-dryrun",
            "-no-kmsg-log",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    Ok(output)
}

fn verify_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    verify_entrypoint_encoding(version, &generated, output)?;
    verify_run_init_argument(version, &generated, output)?;
    verify_stop_lifecycle_arguments(version, &generated, output)?;
    verify_pull_arguments(version, &generated, output)?;
    verify_pids_limit_arguments(version, &generated, output)?;
    verify_hostname_argument(version, &generated, output)?;
    verify_shm_size_arguments(version, &generated, output)?;
    verify_cap_drop_arguments(version, &generated, output)?;
    verify_cap_add_arguments(version, &generated, output)?;
    verify_cap_drop_all_add_one_arguments(version, &generated, output)?;
    verify_tmpfs_argument(version, &generated, output)?;
    verify_sysctl_argument(version, &generated, output)?;
    verify_ulimit_arguments(version, &generated, output)?;
    verify_device_arguments(version, &generated, output)?;
    verify_dns_arguments(version, &generated, output)?;
    verify_dns_option_arguments(version, &generated, output)?;
    verify_dns_search_arguments(version, &generated, output)?;
    verify_expose_arguments(version, &generated, output)?;
    verify_annotation_arguments(version, &generated, output)?;
    verify_quoted_label_encoding(version, &generated, output)?;
    Ok(())
}

fn verify_memory_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);

    if parsed < PodmanVersion::new(5, 5, 0) {
        let memory_argument_count = generated.matches("--memory").count();
        let rejected_or_excluded =
            !output.status.success() || !generated.contains("---memory.service---") || diagnostics.contains("Memory");
        if memory_argument_count != 0 || !rejected_or_excluded {
            return Err(format!(
                "Podman {version} predates native Memory support and must reject or exclude the fixture without emitting --memory; found memory-arguments={memory_argument_count}, status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} Memory: unsupported key is rejected or excluded with no --memory argument");
        return Ok(());
    }

    ensure_success(version, "memory generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} memory generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "memory.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for memory.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(MEMORY_ARGUMENT).count();
    let all_memory_count = podman_run.matches("--memory").count();
    let empty_or_alternate_forms: Vec<_> = MEMORY_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1 || all_memory_count != 1 || !empty_or_alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for memory.service must contain exactly one final `{MEMORY_ARGUMENT}` and no duplicate, equals, empty, quoted, or alternate form; found expected={expected_count}, all-memory={all_memory_count}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} Memory: last effective assignment emits exactly one --memory 16777216b argument");
    Ok(())
}

fn verify_interactive_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} interactive generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} interactive generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "interactive.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for interactive.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{INTERACTIVE_ARGUMENT} {INTERACTIVE_IMAGE}");
    let expected_count = podman_run.matches(&expected_pair).count();
    let all_long_count = podman_run.matches(INTERACTIVE_ARGUMENT).count();
    let short_forms: Vec<_> = podman_run
        .split_whitespace()
        .filter(|token| {
            let unquoted = token.trim_matches(|character| character == '"' || character == '\'');
            unquoted.starts_with("-i") && !unquoted.starts_with("--")
        })
        .collect();
    let alternate_forms: Vec<_> = INTERACTIVE_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1 || all_long_count != 1 || !short_forms.is_empty() || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for interactive.service must place exactly one separate `{INTERACTIVE_ARGUMENT}` immediately before `{INTERACTIVE_IMAGE}` with no short, equals, quoted, alternate, or duplicate form; found expected-pair={expected_count}, all-long={all_long_count}, short={short_forms:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} PodmanArgs: exactly one separate --interactive argument immediately precedes the image"
    );
    Ok(())
}

fn verify_tty_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} TTY generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} TTY generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "tty.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for tty.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{TTY_ARGUMENT} {TTY_IMAGE}");
    let expected_count = podman_run.matches(&expected_pair).count();
    let all_long_count = podman_run.matches(TTY_ARGUMENT).count();
    let short_or_combined_forms: Vec<_> = podman_run
        .split_whitespace()
        .filter(|token| {
            let unquoted = token.trim_matches(|character| character == '"' || character == '\'');
            unquoted.starts_with('-') && !unquoted.starts_with("--") && unquoted.trim_start_matches('-').contains('t')
        })
        .collect();
    let alternate_forms: Vec<_> = TTY_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1 || all_long_count != 1 || !short_or_combined_forms.is_empty() || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for tty.service must place exactly one separate `{TTY_ARGUMENT}` immediately before `{TTY_IMAGE}` with no -t short, combined, equals, quoted, alternate, or duplicate form; found expected-pair={expected_count}, all-long={all_long_count}, short-or-combined={short_or_combined_forms:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} PodmanArgs: exactly one separate --tty argument immediately precedes the image");
    Ok(())
}

fn verify_privileged_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} privileged generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} privileged generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    verify_privileged_unit(
        version,
        &generated,
        &diagnostics,
        output,
        "privileged-true.service",
        PRIVILEGED_TRUE_ARGUMENT,
        PRIVILEGED_TRUE_IMAGE,
    )?;
    verify_privileged_unit(
        version,
        &generated,
        &diagnostics,
        output,
        "privileged-false.service",
        PRIVILEGED_FALSE_ARGUMENT,
        PRIVILEGED_FALSE_IMAGE,
    )?;
    eprintln!(
        "Podman {version} PodmanArgs: exactly one separate --privileged and --privileged=false argument each precede their respective images"
    );
    Ok(())
}

fn verify_privileged_unit(
    version: &str,
    generated: &str,
    diagnostics: &str,
    output: &Output,
    unit_name: &str,
    expected_argument: &str,
    image: &str,
) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, unit_name, output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for {unit_name} is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{expected_argument} {image}");
    let expected_count = podman_run.matches(&expected_pair).count();
    let all_privileged_count = podman_run.matches(PRIVILEGED_TRUE_ARGUMENT).count();
    let short_or_bundled_forms: Vec<_> = podman_run
        .split_whitespace()
        .filter(|token| {
            let unquoted = token.trim_matches(|character| character == '"' || character == '\'');
            unquoted.starts_with('-') && !unquoted.starts_with("--") && unquoted.contains('p')
        })
        .collect();
    let alternate_forms: Vec<_> = PRIVILEGED_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1
        || all_privileged_count != 1
        || !short_or_bundled_forms.is_empty()
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for {unit_name} must place exactly one separate `{expected_argument}` immediately before `{image}` with no --privileged=true, positional false, short, quoted, bundled, alternate, duplicate, or conflicting form; found expected-pair={expected_count}, all-privileged={all_privileged_count}, short-or-bundled={short_or_bundled_forms:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    Ok(())
}

fn verify_logging_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} logging generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} logging generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "logging.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for logging.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let driver_count = podman_run.matches(LOG_DRIVER_ARGUMENT).count();
    let all_driver_count = podman_run.matches("--log-driver").count();
    let mut option_positions = Vec::with_capacity(LOG_OPT_ARGUMENTS.len());
    for argument in LOG_OPT_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for logging.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        option_positions.push(matches[0]);
    }
    let all_option_count = podman_run.matches("--log-opt").count();
    let pre_reset_values: Vec<_> = LOGGING_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = LOGGING_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if driver_count != 1
        || all_driver_count != 1
        || all_option_count != LOG_OPT_ARGUMENTS.len()
        || !option_positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for logging.service must contain one log driver and exactly two ordered post-reset log options with no pre-reset, empty, duplicate, or alternate form; found driver={driver_count}, all-drivers={all_driver_count}, all-options={all_option_count}, option-positions={option_positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} container logging: one --log-driver and two ordered post-reset --log-opt arguments");
    Ok(())
}

fn verify_network_identity_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} network-identity generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} network-identity generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "network-identity.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for network-identity.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let address_counts = [
        (
            "IPv4",
            podman_run.matches(NETWORK_IDENTITY_IP_ARGUMENT).count(),
            podman_run.matches("--ip ").count() + podman_run.matches("--ip=").count(),
        ),
        (
            "IPv6",
            podman_run.matches(NETWORK_IDENTITY_IP6_ARGUMENT).count(),
            podman_run.matches("--ip6 ").count() + podman_run.matches("--ip6=").count(),
        ),
    ];
    let network_count = podman_run.matches(NETWORK_IDENTITY_NETWORK_ARGUMENT).count();
    let all_network_count = podman_run.matches("--network ").count() + podman_run.matches("--network=").count();
    let mut alias_positions = Vec::with_capacity(NETWORK_ALIAS_ARGUMENTS.len());
    for argument in NETWORK_ALIAS_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for network-identity.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        alias_positions.push(matches[0]);
    }
    let all_alias_count = podman_run.matches("--network-alias").count();
    let pre_reset_values: Vec<_> = NETWORK_ALIAS_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = NETWORK_IDENTITY_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if address_counts
        .iter()
        .any(|(_, expected, all)| *expected != 1 || *all != 1)
        || network_count != 1
        || all_network_count != 1
        || all_alias_count != NETWORK_ALIAS_ARGUMENTS.len()
        || !alias_positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for network-identity.service must contain one IP, one IP6, one Network, and exactly two ordered post-reset network aliases without alternate or duplicate forms; found addresses={address_counts:?}, network={network_count}/{all_network_count}, aliases={all_alias_count}, alias-positions={alias_positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} container network identity: one --ip, one --ip6, one --network, and two ordered post-reset --network-alias arguments"
    );
    Ok(())
}

fn verify_network_driver_options_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} network-driver-options generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} network-driver-options generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "network-driver-options-network.service", output)?;
    let podman_network = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman network create "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for network-driver-options-network.service is missing its Podman network create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let driver_count = podman_network.matches(NETWORK_OPTIONS_DRIVER_ARGUMENT).count();
    let all_driver_count = podman_network.matches("--driver ").count() + podman_network.matches("--driver=").count();
    let mut option_positions = Vec::with_capacity(NETWORK_OPTIONS_ARGUMENTS.len());
    for argument in NETWORK_OPTIONS_ARGUMENTS {
        let matches: Vec<_> = podman_network
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for network-driver-options-network.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        option_positions.push(matches[0]);
    }
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let bare_count = podman_network.matches(NETWORK_OPTIONS_BARE_ARGUMENT).count();
    let bare_expectation = if parsed == PodmanVersion::new(5, 4, 0) {
        Some(0)
    } else if parsed == PodmanVersion::new(6, 0, 2) {
        Some(1)
    } else {
        None
    };
    let pre_reset_values: Vec<_> = NETWORK_OPTIONS_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_network.contains(value))
        .collect();
    let alternate_forms: Vec<_> = NETWORK_OPTIONS_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_network.contains(form))
        .collect();
    let expected_option_count = bare_expectation.map(|bare| NETWORK_OPTIONS_ARGUMENTS.len() + bare);
    let all_option_count = podman_network.matches("--opt ").count() + podman_network.matches("--opt=").count();
    if driver_count != 1
        || all_driver_count != 1
        || !option_positions.windows(2).all(|pair| pair[0] < pair[1])
        || bare_expectation.is_some_and(|expected| bare_count != expected)
        || expected_option_count.is_some_and(|expected| all_option_count != expected)
        || !((NETWORK_OPTIONS_ARGUMENTS.len())..=(NETWORK_OPTIONS_ARGUMENTS.len() + 1)).contains(&all_option_count)
        || !pre_reset_values.is_empty()
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for network-driver-options-network.service must contain one --driver bridge, final reset-aware key-sorted --opt alpha=final then --opt zeta=last, and the version-specific bare-token behavior; found driver={driver_count}/{all_driver_count}, option-positions={option_positions:?}, bare={bare_count}/{bare_expectation:?}, all-options={all_option_count}/{expected_option_count:?}, pre-reset={pre_reset_values:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} network driver/options: one --driver, reset-aware duplicate-collapsed key-sorted --opt arguments, bare token expectation {bare_expectation:?}"
    );
    Ok(())
}

fn verify_volume_driver_options_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "volume driver/options generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} volume driver/options generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} volume driver/options generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let command = |unit| -> Result<&str, String> {
        generated_unit(version, &generated, unit, output)?.lines().find(|line| {
            line.starts_with("ExecStart=/usr/bin/podman volume create ")
        }).ok_or_else(|| format!(
            "Podman {version} generator output for {unit} is missing its Podman volume create command\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };
    let primary = command("driver-options-volume.service")?;
    let option_count = primary.matches("--opt ").count() + primary.matches("--opt=").count();
    if primary.matches("--driver local").count() != 1
        || primary.matches("--opt device=tmpfs").count() != 1
        || primary.matches("--opt type=bind").count() != 1
        || primary.matches("--opt o=final-option").count() != 1
        || option_count != 3
        || primary.contains("pre-option")
        || primary.contains("--opt final-option")
    {
        return Err(format!(
            "Podman {version} volume driver/options output must use the final singleton Driver, Device, Type, and one raw o=Options argument without general option expansion; found command `{primary}`"
        ));
    }
    let bare = command("options-bare-volume.service")?;
    if bare.matches("--opt o=bare-option").count() != 1 || bare.contains("--opt bare-option") {
        return Err(format!(
            "Podman {version} volume bare Options output must use exactly one o= argument; found command `{bare}`"
        ));
    }
    let matched = command("options-matched-quote-volume.service")?;
    if !matched.contains("matched")
        || matched.matches("--opt ").count() + matched.matches("--opt=").count() != 2
        || matched.contains("--opt matched")
    {
        return Err(format!(
            "Podman {version} volume matched-quote Options output must retain one option argument instead of general expansion; found command `{matched}`"
        ));
    }
    let empty = command("options-empty-volume.service")?;
    let empty_option_count = empty.matches("--opt ").count() + empty.matches("--opt=").count();
    if empty_option_count != 1 || empty.contains("o=pre-option") || empty.contains("--opt o=") {
        return Err(format!(
            "Podman {version} volume final empty Options output must omit the raw o= option; found command `{empty}`"
        ));
    }
    verify_volume_device_type_forms(version, &generated, output)
}

fn verify_volume_device_type_forms(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let command = |unit| -> Result<&str, String> {
        generated_unit(version, generated, unit, output)?.lines().find(|line| {
            line.starts_with("ExecStart=/usr/bin/podman volume create ")
        }).ok_or_else(|| format!(
            "Podman {version} generator output for {unit} is missing its Podman volume create command\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };
    for (unit, absent) in [
        ("device-final-blank-volume.service", "device="),
        ("type-final-blank-volume.service", "type="),
    ] {
        let command = command(unit)?;
        if command.contains(absent) {
            return Err(format!(
                "Podman {version} {unit} output must suppress the final blank singleton {absent:?}; found command `{command}`"
            ));
        }
    }
    for unit in [
        "device-matched-quote-volume.service",
        "device-specifier-volume.service",
        "device-continuation-volume.service",
    ] {
        let command = command(unit)?;
        if command.matches("--opt device=").count() != 1 || command.matches("--opt type=bind").count() != 1 {
            return Err(format!(
                "Podman {version} {unit} output must retain exactly one Device and one Type option; found command `{command}`"
            ));
        }
    }
    let unmatched = command("device-unmatched-quote-volume.service")?;
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let unmatched_expected = if parsed < PodmanVersion::new(5, 8, 2) {
        "--opt device=tmpfs"
    } else {
        "--opt \"device=\\\"tmpfs\""
    };
    if unmatched.matches(unmatched_expected).count() != 1 || unmatched.matches("--opt type=bind").count() != 1 {
        return Err(format!(
            "Podman {version} unmatched Device quote must use the {unmatched_expected:?} presentation family with one Type option; found command `{unmatched}`"
        ));
    }
    let bind_unit = generated_unit(version, generated, "bind-requires-mounts-volume.service", output)?;
    let requires_mounts: Vec<_> = bind_unit
        .lines()
        .filter(|line| *line != "RequiresMountsFor=%t/containers")
        .filter(|line| line.starts_with("RequiresMountsFor="))
        .collect();
    let expected_requires_mounts = if parsed <= PodmanVersion::new(5, 5, 2) {
        None
    } else if parsed <= PodmanVersion::new(5, 7, 1) {
        Some("RequiresMountsFor=/tmp/quadlet lens")
    } else {
        Some(r#"RequiresMountsFor="/tmp/quadlet\x20lens""#)
    };
    match expected_requires_mounts {
        None if requires_mounts.is_empty() => {}
        Some(expected) if requires_mounts == [expected] => {}
        _ => {
            return Err(format!(
                "Podman {version} Type=bind RequiresMountsFor must use the documented presentation band {expected_requires_mounts:?}; found {requires_mounts:?}\nunit:\n{bind_unit}"
            ));
        }
    }
    Ok(())
}

fn verify_volume_copy_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "volume Copy generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} volume Copy generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} volume Copy generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let command = |unit| -> Result<&str, String> {
        generated_unit(version, &generated, unit, output)?
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
            .ok_or_else(|| format!(
                "Podman {version} generator output for {unit} is missing its Podman volume create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ))
    };
    let count = |command: &str, value: &str| {
        command.matches(&format!("--opt {value}")).count() + command.matches(&format!("--opt={value}")).count()
    };
    let assert_form = |unit, expected_form: Option<&str>| -> Result<(), String> {
        let command = command(unit)?;
        let copy = count(command, "copy");
        let nocopy = count(command, "nocopy");
        let valid = match expected_form {
            Some("copy") => copy == 1 && nocopy == 0,
            Some("nocopy") => copy == 0 && nocopy == 1,
            None => copy == 0 && nocopy == 0,
            Some(_) => false,
        };
        if valid {
            Ok(())
        } else {
            Err(format!(
                "Podman {version} {unit} must emit {expected_form:?} as its only Copy form; found copy={copy}, nocopy={nocopy}; command `{command}`"
            ))
        }
    };
    assert_form("copy-omitted-volume.service", None)?;
    for unit in [
        "copy-true-volume.service",
        "copy-yes-volume.service",
        "copy-on-volume.service",
        "copy-one-volume.service",
        "copy-true-upper-volume.service",
        "copy-yes-upper-volume.service",
        "copy-on-upper-volume.service",
        "copy-true-mixed-volume.service",
        "copy-last-wins-true-volume.service",
        "copy-matched-quote-volume.service",
        "copy-continuation-volume.service",
    ] {
        assert_form(unit, Some("copy"))?;
    }
    for unit in [
        "copy-false-volume.service",
        "copy-false-upper-volume.service",
        "copy-blank-volume.service",
        "copy-invalid-volume.service",
        "copy-specifier-volume.service",
        "copy-last-wins-false-volume.service",
    ] {
        assert_form(unit, Some("nocopy"))?;
    }
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    assert_form(
        "copy-unmatched-quote-volume.service",
        Some(if parsed < PodmanVersion::new(5, 8, 2) {
            "copy"
        } else {
            "nocopy"
        }),
    )?;
    assert_form("copy-image-false-volume.service", None)?;
    eprintln!("Podman {version} Volume Copy: 20 isolated physical forms retain the recorded boolean parser boundary");
    Ok(())
}

fn verify_volume_type_without_device_generator_output(version: &str, output: &Output) -> Result<(), String> {
    if output.status.success() {
        return Err(format!(
            "Podman {version} must reject Volume Type without Device\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn verify_volume_options_without_device_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    if parsed < PodmanVersion::new(6, 0, 0) {
        if output.status.success() {
            return Err(format!(
                "Podman {version} must reject Volume Options without Device before 6.0.0\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        return Ok(());
    }
    ensure_success(version, "volume Options-without-Device generator", output)?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} volume Options-without-Device generator emitted non-UTF-8 output: {error}")
    })?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Volume Options-without-Device output is missing `{fragment}`"
            ));
        }
    }
    let unit = generated_unit(version, &generated, "options-without-device-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or_else(|| format!("Podman {version} Volume Options-without-Device output has no volume create command"))?;
    if command.matches("--opt o=without-device").count() != 1
        || command.matches("--opt ").count() + command.matches("--opt=").count() != 1
        || command.contains("--opt without-device")
    {
        return Err(format!(
            "Podman {version} Volume Options-without-Device output must emit one raw o= argument; found command `{command}`"
        ));
    }
    Ok(())
}

fn verify_volume_options_unmatched_quote_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    ensure_success(version, "volume unmatched-quote Options generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} volume unmatched-quote generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} unmatched-quote Volume Options output is missing `{fragment}`"
            ));
        }
    }
    let unit = generated_unit(version, &generated, "options-unmatched-quote-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or_else(|| {
            format!("Podman {version} unmatched-quote Volume Options output has no volume create command")
        })?;
    let expected = if parsed < PodmanVersion::new(5, 8, 2) {
        "--opt o=unmatched-option"
    } else {
        "--opt \"o=\\\"unmatched-option\""
    };
    if command.matches(expected).count() != 1 {
        return Err(format!(
            "Podman {version} unmatched Volume Options quote must use the {expected:?} presentation family; found command `{command}`"
        ));
    }
    Ok(())
}

fn verify_network_labels_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} network-labels generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} network-labels generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "network-labels-network.service", output)?;
    let podman_network = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman network create "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for network-labels-network.service is missing its Podman network create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(NETWORK_LABEL_ARGUMENTS.len());
    for argument in NETWORK_LABEL_ARGUMENTS {
        let matches: Vec<_> = podman_network
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for network-labels-network.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let bare_expected = usize::from(parsed > PodmanVersion::new(5, 5, 2));
    let bare_matches: Vec<_> = podman_network
        .match_indices(NETWORK_LABEL_BARE_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    let literal_quote_count = podman_network.matches(NETWORK_LABEL_QUOTED_LITERAL_SPACE).count();
    let hex_quote_count = podman_network.matches(NETWORK_LABEL_QUOTED_HEX_SPACE).count();
    let (quoted_count, unexpected_quote_count, quoted_name) = if parsed.major() == 5 && parsed.minor() == 4 {
        (literal_quote_count, hex_quote_count, "literal-space")
    } else {
        (hex_quote_count, literal_quote_count, "hex-space")
    };
    let quoted_position = if parsed.major() == 5 && parsed.minor() == 4 {
        podman_network.find(NETWORK_LABEL_QUOTED_LITERAL_SPACE)
    } else {
        podman_network.find(NETWORK_LABEL_QUOTED_HEX_SPACE)
    };
    let mut sorted_positions = vec![positions[0]];
    if let Some(position) = bare_matches.first() {
        sorted_positions.push(*position);
    }
    sorted_positions.extend_from_slice(&positions[1..3]);
    if let Some(position) = quoted_position {
        sorted_positions.push(position);
    }
    sorted_positions.push(positions[3]);
    let pre_reset_values: Vec<_> = NETWORK_LABEL_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_network.contains(value))
        .collect();
    let alternate_forms: Vec<_> = NETWORK_LABEL_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_network.contains(form))
        .collect();
    let all_label_count = podman_network.matches("--label ").count() + podman_network.matches("--label=").count();
    let expected_label_count = NETWORK_LABEL_ARGUMENTS.len() + 1 + bare_expected;
    if bare_matches.len() != bare_expected
        || quoted_count != 1
        || unexpected_quote_count != 0
        || sorted_positions.len() != expected_label_count
        || !sorted_positions.windows(2).all(|pair| pair[0] < pair[1])
        || all_label_count != expected_label_count
        || !pre_reset_values.is_empty()
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for network-labels-network.service must contain final reset-aware duplicate-collapsed key-sorted labels, explicit empty and embedded-equals values, one {quoted_name} quoted whitespace argument, and {bare_expected} bare-token arguments; found bare={}, quoted={quoted_count}/{unexpected_quote_count}, positions={sorted_positions:?}, labels={all_label_count}/{expected_label_count}, pre-reset={pre_reset_values:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            bare_matches.len(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} network labels: reset-aware duplicate-collapsed key-sorted labels, explicit empty and embedded-equals values, {quoted_name} quoted whitespace, bare token {bare_expected}"
    );
    Ok(())
}

fn verify_volume_labels_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "volume-labels generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} volume-labels generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} volume-labels generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "volume-labels-volume.service", output)?;
    let podman_volume = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for volume-labels-volume.service is missing its Podman volume create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(VOLUME_LABEL_ARGUMENTS.len());
    for argument in VOLUME_LABEL_ARGUMENTS {
        let matches: Vec<_> = podman_volume
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for volume-labels-volume.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let bare_expected = usize::from(parsed > PodmanVersion::new(5, 5, 2));
    let bare_matches: Vec<_> = podman_volume
        .match_indices(VOLUME_LABEL_BARE_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    let literal_quote_count = podman_volume.matches(VOLUME_LABEL_QUOTED_LITERAL_SPACE).count();
    let hex_quote_count = podman_volume.matches(VOLUME_LABEL_QUOTED_HEX_SPACE).count();
    let (quoted_count, unexpected_quote_count, quoted_name) = if parsed.major() == 5 && parsed.minor() == 4 {
        (literal_quote_count, hex_quote_count, "literal-space")
    } else {
        (hex_quote_count, literal_quote_count, "hex-space")
    };
    let quoted_position = if parsed.major() == 5 && parsed.minor() == 4 {
        podman_volume.find(VOLUME_LABEL_QUOTED_LITERAL_SPACE)
    } else {
        podman_volume.find(VOLUME_LABEL_QUOTED_HEX_SPACE)
    };
    let mut sorted_positions = vec![positions[0]];
    if let Some(position) = bare_matches.first() {
        sorted_positions.push(*position);
    }
    sorted_positions.extend_from_slice(&positions[1..3]);
    if let Some(position) = quoted_position {
        sorted_positions.push(position);
    }
    sorted_positions.push(positions[3]);
    let pre_reset_values: Vec<_> = VOLUME_LABEL_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_volume.contains(value))
        .collect();
    let alternate_forms: Vec<_> = VOLUME_LABEL_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_volume.contains(form))
        .collect();
    let all_label_count = podman_volume.matches("--label ").count() + podman_volume.matches("--label=").count();
    let expected_label_count = VOLUME_LABEL_ARGUMENTS.len() + 1 + bare_expected;
    if bare_matches.len() != bare_expected
        || quoted_count != 1
        || unexpected_quote_count != 0
        || sorted_positions.len() != expected_label_count
        || !sorted_positions.windows(2).all(|pair| pair[0] < pair[1])
        || all_label_count != expected_label_count
        || !pre_reset_values.is_empty()
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for volume-labels-volume.service must contain final reset-aware duplicate-collapsed key-sorted labels, explicit empty and embedded-equals values, one {quoted_name} quoted whitespace argument, and {bare_expected} bare-token arguments; found bare={}, quoted={quoted_count}/{unexpected_quote_count}, positions={sorted_positions:?}, labels={all_label_count}/{expected_label_count}, pre-reset={pre_reset_values:?}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            bare_matches.len(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} volume labels: reset-aware duplicate-collapsed key-sorted labels, explicit empty and embedded-equals values, {quoted_name} quoted whitespace, bare token {bare_expected}"
    );
    Ok(())
}

fn verify_network_ipam_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "network-IPAM generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} network-IPAM generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} network-IPAM generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let columns_unit = generated_unit(version, &generated, "ipam-columns-network.service", output)?;
    let columns_command = columns_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman network create "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for ipam-columns-network.service is missing its Podman network create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let driver_count = columns_command.matches(NETWORK_IPAM_DRIVER_ARGUMENT).count();
    let all_driver_count =
        columns_command.matches("--ipam-driver ").count() + columns_command.matches("--ipam-driver=").count();
    let mut positions = Vec::with_capacity(NETWORK_IPAM_COLUMN_ARGUMENTS.len());
    for argument in NETWORK_IPAM_COLUMN_ARGUMENTS {
        let matches: Vec<_> = columns_command
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for ipam-columns-network.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    let subnet_count = columns_command.matches("--subnet ").count() + columns_command.matches("--subnet=").count();
    let gateway_count = columns_command.matches("--gateway ").count() + columns_command.matches("--gateway=").count();
    let range_count = columns_command.matches("--ip-range ").count() + columns_command.matches("--ip-range=").count();
    let pre_reset_values: Vec<_> = NETWORK_IPAM_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| columns_command.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = NETWORK_IPAM_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| columns_command.contains(form))
        .collect();
    if driver_count != 1
        || all_driver_count != 1
        || subnet_count != 2
        || gateway_count != 2
        || range_count != 2
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for ipam-columns-network.service must contain one explicit IPAM driver and exactly two ordered post-reset subnet/gateway/ip-range groups; found driver={driver_count}/{all_driver_count}, subnet={subnet_count}, gateway={gateway_count}, range={range_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let blank_unit = generated_unit(version, &generated, "ipam-driver-blank-network.service", output)?;
    let blank_command = blank_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman network create "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for ipam-driver-blank-network.service is missing its Podman network create command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let blank_driver_count =
        blank_command.matches("--ipam-driver ").count() + blank_command.matches("--ipam-driver=").count();
    if blank_driver_count != 0 {
        return Err(format!(
            "Podman {version} generator output for a blank IPAMDriver must omit --ipam-driver; found {blank_driver_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} network IPAM: one explicit driver, two ordered reset-aware subnet/gateway/ip-range groups, and blank driver omission"
    );
    Ok(())
}

fn verify_network_boolean_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} network-booleans generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} network-booleans generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    for &(unit_name, case, flag, expected_form) in NETWORK_BOOLEAN_CASES {
        let generated_unit = generated_unit(version, &generated, unit_name, output)?;
        let podman_network = generated_unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman network create "))
            .ok_or_else(|| {
                format!(
                    "Podman {version} generator output for {unit_name} is missing its Podman network create command\nstdout:\n{generated}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                )
            })?;
        let intended_count = expected_form.map_or(0, |form| {
            podman_network.split_whitespace().filter(|token| *token == form).count()
        });
        let all_flag_forms = podman_network.matches(flag).count();
        let expected_count = usize::from(expected_form.is_some());
        if intended_count != expected_count || all_flag_forms != expected_count {
            return Err(format!(
                "Podman {version} generator output for {unit_name} ({case}) must contain exactly {expected_count} intended {expected_form:?} form and no alternate `{flag}` form; found intended={intended_count}, all={all_flag_forms}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!("Podman {version} network booleans: isolated omission, true, and false forms for Internal and IPv6");
    Ok(())
}

fn verify_entrypoint_encoding(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let separate_count = generated.matches(ENTRYPOINT_SEPARATE_ARGUMENT).count();
    let equals_count = generated.matches(ENTRYPOINT_EQUALS_ARGUMENT).count();
    let (expected_name, expected_count, unexpected_count) = if parsed < PodmanVersion::new(5, 8, 2) {
        ("separate-argument", separate_count, equals_count)
    } else {
        ("equals-argument", equals_count, separate_count)
    };
    if expected_count != 1 || unexpected_count != 0 {
        return Err(format!(
            "Podman {version} generator output must contain exactly one {expected_name} JSON-array entrypoint encoding and no other supported encoding; found separate-argument={separate_count}, equals-argument={equals_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} JSON-array entrypoint encoding: {expected_name}");
    Ok(())
}

fn verify_run_init_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let true_unit = generated_unit(version, generated, "app.service", output)?;
    let true_count = true_unit.matches(RUN_INIT_ARGUMENT).count();
    if true_count != 1 {
        return Err(format!(
            "Podman {version} generator output for authored `RunInit=true` must contain exactly one {RUN_INIT_ARGUMENT} argument; found {true_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let false_unit = generated_unit(version, generated, "run-init-false.service", output)?;
    let false_count = false_unit.matches(RUN_INIT_FALSE_ARGUMENT).count();
    let false_form_count = false_unit.matches(RUN_INIT_ARGUMENT).count();
    if false_count != 1 || false_form_count != 1 {
        return Err(format!(
            "Podman {version} generator output for authored `RunInit=false` must contain exactly one {RUN_INIT_FALSE_ARGUMENT} argument and no other {RUN_INIT_ARGUMENT} form; found explicit-false={false_count}, all-forms={false_form_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} RunInit: true emits one --init; false emits one --init=false");
    Ok(())
}

fn generated_unit<'a>(version: &str, generated: &'a str, unit: &str, output: &Output) -> Result<&'a str, String> {
    let marker = format!("---{unit}---");
    let (_, remainder) = generated.split_once(&marker).ok_or_else(|| {
        format!(
            "Podman {version} generator output is missing unit marker `{marker}`\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;
    Ok(remainder.split("\n---").next().unwrap_or(remainder))
}

fn verify_stop_lifecycle_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for argument in [
        NAMED_STOP_SIGNAL_ARGUMENT,
        NUMERIC_STOP_SIGNAL_ARGUMENT,
        POSITIVE_STOP_TIMEOUT_ARGUMENT,
        ZERO_STOP_TIMEOUT_ARGUMENT,
    ] {
        let count = generated.matches(argument).count();
        if count != 1 {
            return Err(format!(
                "Podman {version} generator output must contain exactly one `{argument}` observation; found {count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!(
        "Podman {version} container stop lifecycle: named and numeric signals, positive timeout, and zero timeout preserved"
    );
    Ok(())
}

fn verify_pull_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in PULL_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_pull_count = generated_unit.matches("--pull").count();
        if expected_count != 1 || all_pull_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no other --pull form; found expected={expected_count}, all-pull={all_pull_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!("Podman {version} Pull: always, missing, never, and newer each emit their matching --pull argument");
    Ok(())
}

fn verify_pids_limit_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in PIDS_LIMIT_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_limit_count = generated_unit.matches("--pids-limit").count();
        if expected_count != 1 || all_limit_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no other --pids-limit form; found expected={expected_count}, all-pids-limit={all_limit_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!("Podman {version} PidsLimit: finite 127 and unlimited -1 each emit one matching --pids-limit argument");
    Ok(())
}

fn verify_hostname_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "hostname.service", output)?;
    let separate_count = generated_unit.matches(HOSTNAME_SEPARATE_ARGUMENT).count();
    let all_hostname_count = generated_unit.matches("--hostname").count();
    if separate_count != 1 || all_hostname_count != 1 {
        return Err(format!(
            "Podman {version} generator output for hostname.service must contain exactly one `--hostname app.example` argument and no duplicate hostname form; found expected={separate_count}, all-hostname={all_hostname_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} HostName: app.example emits exactly one --hostname argument");
    Ok(())
}

fn verify_shm_size_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in SHM_SIZE_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_shm_size_count = generated_unit.matches("--shm-size").count();
        if expected_count != 1 || all_shm_size_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no duplicate --shm-size form; found expected={expected_count}, all-shm-size={all_shm_size_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let member_unit = generated_unit(version, generated, "shm-size-pod-member.service", output)?;
    let member_count = member_unit.matches("--shm-size").count();
    if member_count != 0 {
        return Err(format!(
            "Podman {version} generator output for the container joining shm-size.pod must not duplicate the pod-owned --shm-size argument; found {member_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} ShmSize: positive container, zero container, and pod-owned values each emit exactly one matching --shm-size argument"
    );
    Ok(())
}

fn verify_cap_drop_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-drop.service", output)?;
    let all_drop_count = generated_unit.matches("--cap-drop").count();
    let add_count = generated_unit.matches("--cap-add").count();
    let mut positions = Vec::with_capacity(CAP_DROP_ARGUMENTS.len());
    for argument in CAP_DROP_ARGUMENTS {
        let matches: Vec<_> = generated_unit
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for cap-drop.service must contain exactly one `{argument}`; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    if all_drop_count != CAP_DROP_ARGUMENTS.len()
        || add_count != 0
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(format!(
            "Podman {version} generator output for cap-drop.service must contain exactly four ordered lowercase separate-argument --cap-drop forms and no --cap-add form; found cap-drop={all_drop_count}, cap-add={add_count}, positions={positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} DropCapability: repeated and space-separated values emit four ordered lowercase --cap-drop arguments and no --cap-add"
    );
    Ok(())
}

fn verify_cap_add_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-add.service", output)?;
    let add_count = generated_unit.matches("--cap-add").count();
    let drop_count = generated_unit.matches("--cap-drop").count();
    let mut positions = Vec::with_capacity(CAP_ADD_ARGUMENTS.len());
    for argument in CAP_ADD_ARGUMENTS {
        let matches: Vec<_> = generated_unit
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for cap-add.service must contain exactly one `{argument}`; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    if add_count != CAP_ADD_ARGUMENTS.len() || drop_count != 0 || !positions.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(format!(
            "Podman {version} generator output for cap-add.service must contain exactly four ordered lowercase separate-argument --cap-add forms and no --cap-drop form; found cap-add={add_count}, cap-drop={drop_count}, positions={positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} AddCapability: repeated and space-separated values emit four ordered lowercase --cap-add arguments and no --cap-drop"
    );
    Ok(())
}

fn verify_cap_drop_all_add_one_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-drop-all-add-one.service", output)?;
    let all_add_count = generated_unit.matches("--cap-add").count();
    let all_drop_count = generated_unit.matches("--cap-drop").count();
    let drop_positions: Vec<_> = generated_unit
        .match_indices(CAP_DROP_ALL_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    let add_positions: Vec<_> = generated_unit
        .match_indices(CAP_ADD_NET_BIND_SERVICE_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    if all_drop_count != 1
        || all_add_count != 1
        || drop_positions.len() != 1
        || add_positions.len() != 1
        || drop_positions[0] >= add_positions[0]
    {
        return Err(format!(
            "Podman {version} generator output for cap-drop-all-add-one.service must contain exactly one `{CAP_DROP_ALL_ARGUMENT}` followed by exactly one `{CAP_ADD_NET_BIND_SERVICE_ARGUMENT}` and no other capability arguments; found cap-drop={all_drop_count}, cap-add={all_add_count}, drop-positions={drop_positions:?}, add-positions={add_positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} capability ordering: one --cap-drop all precedes one --cap-add cap_net_bind_service");
    Ok(())
}

fn verify_tmpfs_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "tmpfs.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for tmpfs.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let expected_count = podman_run.matches(TMPFS_ARGUMENT).count();
    let all_tmpfs_count = podman_run.matches("--tmpfs").count();
    let pre_reset_paths: Vec<_> = TMPFS_PRE_RESET_PATHS
        .iter()
        .copied()
        .filter(|path| podman_run.contains(path))
        .collect();
    if expected_count != 1 || all_tmpfs_count != 1 || !pre_reset_paths.is_empty() {
        return Err(format!(
            "Podman {version} generator output for tmpfs.service must contain exactly one post-reset `{TMPFS_ARGUMENT}`, no other --tmpfs form, and no pre-reset path; found expected={expected_count}, all-tmpfs={all_tmpfs_count}, pre-reset={pre_reset_paths:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} Tmpfs: LookupAll reset leaves exactly one --tmpfs /data:mode=755,uid=1009,gid=1009 argument"
    );
    Ok(())
}

fn verify_sysctl_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "sysctl.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for sysctl.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let expected_count = podman_run.matches(SYSCTL_ARGUMENT).count();
    let all_sysctl_count = podman_run.matches("--sysctl").count();
    let pre_reset_settings: Vec<_> = SYSCTL_PRE_RESET_SETTINGS
        .iter()
        .copied()
        .filter(|setting| podman_run.contains(setting))
        .collect();
    if expected_count != 1 || all_sysctl_count != 1 || !pre_reset_settings.is_empty() {
        return Err(format!(
            "Podman {version} generator output for sysctl.service must contain exactly one post-reset `{SYSCTL_ARGUMENT}`, no other --sysctl form, and neither pre-reset setting; found expected={expected_count}, all-sysctl={all_sysctl_count}, pre-reset={pre_reset_settings:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} Sysctl: LookupAllStrv reset leaves exactly one --sysctl net.ipv4.ip_forward=1 argument"
    );
    Ok(())
}

fn verify_ulimit_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "ulimit.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for ulimit.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(ULIMIT_ARGUMENTS.len());
    for argument in ULIMIT_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for ulimit.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_ulimit_count = podman_run.matches("--ulimit").count();
    let pre_reset_limits: Vec<_> = ULIMIT_PRE_RESET_LIMITS
        .iter()
        .copied()
        .filter(|limit| podman_run.contains(limit))
        .collect();
    let empty_or_alternate_forms: Vec<_> = ULIMIT_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_ulimit_count != ULIMIT_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_limits.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for ulimit.service must contain exactly two ordered post-reset Ulimit arguments, no duplicates, no pre-reset limit, and no empty/alternate form; found all-ulimit={all_ulimit_count}, positions={positions:?}, pre-reset={pre_reset_limits:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} Ulimit: LookupAll reset leaves exactly two ordered --ulimit nproc=4096:8192 and --ulimit stack=-1:-1 arguments"
    );
    Ok(())
}

fn verify_device_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "device.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for device.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(DEVICE_ARGUMENTS.len());
    for argument in DEVICE_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for device.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_device_count = podman_run.matches("--device").count();
    let pre_reset_mappings: Vec<_> = DEVICE_PRE_RESET_MAPPINGS
        .iter()
        .copied()
        .filter(|mapping| podman_run.contains(mapping))
        .collect();
    let empty_or_alternate_forms: Vec<_> = DEVICE_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_device_count != DEVICE_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_mappings.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for device.service must contain exactly two ordered post-reset AddDevice arguments, no duplicates, no pre-reset mapping, and no empty/alternate form; found all-device={all_device_count}, positions={positions:?}, pre-reset={pre_reset_mappings:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!("Podman {version} AddDevice: LookupAllStrv reset leaves exactly two ordered final --device arguments");
    Ok(())
}

fn verify_dns_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "dns.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for dns.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(DNS_ARGUMENTS.len());
    for argument in DNS_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for dns.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_dns_count = podman_run.matches("--dns").count();
    let pre_reset_values: Vec<_> = DNS_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = DNS_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_dns_count != DNS_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for dns.service must contain exactly two ordered post-reset DNS arguments, no duplicates, no pre-reset value, and no empty/equals/quoted/alternate form; found all-dns={all_dns_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} DNS: LookupAll reset leaves exactly two ordered --dns 9.9.9.9 and --dns 2001:4860:4860::8888 arguments"
    );
    Ok(())
}

fn verify_dns_option_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "dns-option.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for dns-option.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(DNS_OPTION_ARGUMENTS.len());
    for argument in DNS_OPTION_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for dns-option.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_dns_option_count = podman_run.matches("--dns-option").count();
    let pre_reset_values: Vec<_> = DNS_OPTION_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = DNS_OPTION_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_dns_option_count != DNS_OPTION_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for dns-option.service must contain exactly two ordered post-reset DNSOption arguments, no duplicates, no pre-reset value, and no empty/equals/quoted/alternate form; found all-dns-option={all_dns_option_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} DNSOption: LookupAll reset leaves exactly two ordered --dns-option ndots:1 and --dns-option use-vc arguments"
    );
    Ok(())
}

fn verify_dns_search_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "dns-search.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for dns-search.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(DNS_SEARCH_ARGUMENTS.len());
    for argument in DNS_SEARCH_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for dns-search.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_dns_search_count = podman_run.matches("--dns-search").count();
    let pre_reset_values: Vec<_> = DNS_SEARCH_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = DNS_SEARCH_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_dns_search_count != DNS_SEARCH_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for dns-search.service must contain exactly two ordered post-reset DNSSearch arguments, no duplicates, no pre-reset value, and no empty/equals/quoted/alternate form; found all-dns-search={all_dns_search_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} DNSSearch: LookupAll reset leaves exactly two ordered --dns-search dc1.example.com and --dns-search . arguments"
    );
    Ok(())
}

fn verify_expose_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "expose-host-port.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for expose-host-port.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(EXPOSE_ARGUMENTS.len());
    for argument in EXPOSE_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for expose-host-port.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_expose_count = podman_run.matches("--expose").count();
    let pre_reset_values: Vec<_> = EXPOSE_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = EXPOSE_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_expose_count != EXPOSE_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for expose-host-port.service must contain exactly four ordered post-reset ExposeHostPort arguments, no duplicate, no pre-reset value, and no empty/equals/quoted/alternate form; found all-expose={all_expose_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} ExposeHostPort: LookupAll reset leaves exactly four ordered --expose 3000, 8080-8085, 9090/tcp, and 5353/udp arguments"
    );
    Ok(())
}

fn verify_annotation_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "annotation.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for annotation.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(ANNOTATION_ARGUMENTS.len());
    for argument in ANNOTATION_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for annotation.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_annotation_count = podman_run.matches("--annotation").count();
    let pre_reset_values: Vec<_> = ANNOTATION_PRE_RESET_VALUES
        .iter()
        .copied()
        .filter(|value| podman_run.contains(value))
        .collect();
    let empty_or_alternate_forms: Vec<_> = ANNOTATION_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_annotation_count != ANNOTATION_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_values.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for annotation.service must contain exactly two ordered post-reset Annotation arguments, no duplicate, no pre-reset key, and no empty/equals/quoted/key-only/alternate form; found all-annotation={all_annotation_count}, positions={positions:?}, pre-reset={pre_reset_values:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} Annotation: environment-style reset leaves exactly two sorted --annotation key=value arguments"
    );
    Ok(())
}

fn verify_apparmor_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 8, 0) {
        let argument_count = generated.matches("apparmor=").count();
        let rejected = !output.status.success()
            && diagnostics.contains("unsupported key 'AppArmor'")
            && !generated.contains("---apparmor.service---");
        if argument_count != 0 || !rejected {
            return Err(format!(
                "Podman {version} predates native AppArmor support and must reject the fixture without emitting an AppArmor argument; found arguments={argument_count}, status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} AppArmor: unsupported key is rejected with no generated unit or argument");
        return Ok(());
    }

    ensure_success(version, "AppArmor generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} AppArmor generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "apparmor.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for apparmor.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let expected_count = podman_run.matches(APPARMOR_ARGUMENT).count();
    let all_security_opt_count = podman_run.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = APPARMOR_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1 || all_security_opt_count != 1 || !unrelated_or_alternate.is_empty() {
        return Err(format!(
            "Podman {version} generator output for apparmor.service must contain exactly one separate `{APPARMOR_ARGUMENT}` occurrence and no equals, quoted, unconfined, label, seccomp, mask, or other security-option form; found expected={expected_count}, all-security-opt={all_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} AppArmor: one profile emits exactly one separate --security-opt apparmor=profile argument"
    );
    Ok(())
}

fn verify_no_new_privileges_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "NoNewPrivileges generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} NoNewPrivileges generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let true_unit = generated_unit(version, &generated, "no-new-privileges-true.service", output)?;
    let false_unit = generated_unit(version, &generated, "no-new-privileges-false.service", output)?;
    let true_run = true_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} NoNewPrivileges true unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let false_run = false_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} NoNewPrivileges false unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;

    let expected_count = true_run.matches(NO_NEW_PRIVILEGES_ARGUMENT).count();
    let true_security_opt_count = true_run.matches("--security-opt").count();
    let false_security_opt_count = false_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(NO_NEW_PRIVILEGES_ARGUMENT).count();
    let unrelated_or_alternate: Vec<_> = NO_NEW_PRIVILEGES_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| true_run.contains(form) || false_run.contains(form))
        .collect();
    if expected_count != 1
        || true_security_opt_count != 1
        || false_security_opt_count != 0
        || total_expected_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} NoNewPrivileges output must contain exactly one equals-form argument in the true unit, no security option in the false unit, and no separate, quoted, alternate, duplicate, or unrelated security-option form; found expected-in-true={expected_count}, true-security-options={true_security_opt_count}, false-security-options={false_security_opt_count}, expected-total={total_expected_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} NoNewPrivileges: true emits exactly one equals-form security option; false emits none");
    Ok(())
}

fn verify_seccomp_profile_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "SeccompProfile generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SeccompProfile generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let mut expected_total = 0;
    let mut security_opt_total = 0;
    for (unit_name, argument) in SECCOMP_PROFILE_CASES {
        let unit = generated_unit(version, &generated, unit_name, output)?;
        let podman_run = unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
            .ok_or_else(|| {
                format!(
                    "Podman {version} SeccompProfile unit {unit_name} is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
                )
            })?;
        let expected_count = podman_run.matches(argument).count();
        let security_opt_count = podman_run.matches("--security-opt").count();
        let unrelated_or_alternate: Vec<_> = SECCOMP_PROFILE_UNRELATED_OR_ALTERNATE_FORMS
            .iter()
            .copied()
            .filter(|form| podman_run.contains(form))
            .collect();
        if expected_count != 1 || security_opt_count != 1 || !unrelated_or_alternate.is_empty() {
            return Err(format!(
                "Podman {version} SeccompProfile unit {unit_name} must contain exactly one separate `{argument}` and no equals, quoted, unrelated, alternate, or duplicate security-option form; found expected={expected_count}, all-security-opt={security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
        expected_total += expected_count;
        security_opt_total += security_opt_count;
    }

    let generated_expected_total: usize = SECCOMP_PROFILE_CASES
        .iter()
        .map(|(_, argument)| generated.matches(argument).count())
        .sum();
    let generated_security_opt_total = generated.matches("--security-opt").count();
    if expected_total != SECCOMP_PROFILE_CASES.len()
        || security_opt_total != SECCOMP_PROFILE_CASES.len()
        || generated_expected_total != SECCOMP_PROFILE_CASES.len()
        || generated_security_opt_total != SECCOMP_PROFILE_CASES.len()
    {
        return Err(format!(
            "Podman {version} SeccompProfile output must contain exactly one scoped separate security option per isolated unit and exactly two total; found scoped-expected={expected_total}, scoped-security-opt={security_opt_total}, total-expected={generated_expected_total}, total-security-opt={generated_security_opt_total}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} SeccompProfile: isolated unconfined and JSON path each emit exactly one separate security option"
    );
    Ok(())
}

fn verify_security_label_disable_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "SecurityLabelDisable generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SecurityLabelDisable generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let true_unit = generated_unit(version, &generated, "security-label-disable-true.service", output)?;
    let false_unit = generated_unit(version, &generated, "security-label-disable-false.service", output)?;
    let true_run = true_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelDisable true unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let false_run = false_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelDisable false unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;

    let expected_count = true_run.matches(SECURITY_LABEL_DISABLE_ARGUMENT).count();
    let true_security_opt_count = true_run.matches("--security-opt").count();
    let false_security_opt_count = false_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(SECURITY_LABEL_DISABLE_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = SECURITY_LABEL_DISABLE_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| true_run.contains(form) || false_run.contains(form))
        .collect();
    if expected_count != 1
        || true_security_opt_count != 1
        || false_security_opt_count != 0
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} SecurityLabelDisable output must contain exactly one separate argument in the true unit, no security option in the false unit, exactly one security option total, and no equals, quoted, alternate, duplicate, or unrelated form; found expected-in-true={expected_count}, true-security-options={true_security_opt_count}, false-security-options={false_security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} SecurityLabelDisable: true emits exactly one separate security option; false emits none"
    );
    Ok(())
}

fn verify_security_label_file_type_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "SecurityLabelFileType generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SecurityLabelFileType generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let unit = generated_unit(version, &generated, "security-label-file-type.service", output)?;
    let podman_run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelFileType unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(SECURITY_LABEL_FILE_TYPE_ARGUMENT).count();
    let security_opt_count = podman_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(SECURITY_LABEL_FILE_TYPE_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = SECURITY_LABEL_FILE_TYPE_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1
        || security_opt_count != 1
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} SecurityLabelFileType output must contain exactly one separate `{SECURITY_LABEL_FILE_TYPE_ARGUMENT}`, exactly one security option total, and no equals, quoted, alternate, unrelated, or duplicate form; found expected={expected_count}, scoped-security-options={security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} SecurityLabelFileType: exactly one separate file-type security option");
    Ok(())
}

fn verify_security_label_level_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "SecurityLabelLevel generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SecurityLabelLevel generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let unit = generated_unit(version, &generated, "security-label-level.service", output)?;
    let podman_run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelLevel unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(SECURITY_LABEL_LEVEL_ARGUMENT).count();
    let security_opt_count = podman_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(SECURITY_LABEL_LEVEL_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = SECURITY_LABEL_LEVEL_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1
        || security_opt_count != 1
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} SecurityLabelLevel output must contain exactly one separate `{SECURITY_LABEL_LEVEL_ARGUMENT}`, exactly one security option total, and no equals, quoted, alternate, unrelated, or duplicate form; found expected={expected_count}, scoped-security-options={security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} SecurityLabelLevel: exactly one separate label-level security option");
    Ok(())
}

fn verify_security_label_nested_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "SecurityLabelNested generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SecurityLabelNested generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let true_unit = generated_unit(version, &generated, "security-label-nested-true.service", output)?;
    let false_unit = generated_unit(version, &generated, "security-label-nested-false.service", output)?;
    let true_run = true_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelNested true unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let false_run = false_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelNested false unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;

    let expected_count = true_run.matches(SECURITY_LABEL_NESTED_ARGUMENT).count();
    let true_security_opt_count = true_run.matches("--security-opt").count();
    let false_security_opt_count = false_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(SECURITY_LABEL_NESTED_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = SECURITY_LABEL_NESTED_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| true_run.contains(form) || false_run.contains(form))
        .collect();
    if expected_count != 1
        || true_security_opt_count != 1
        || false_security_opt_count != 0
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} SecurityLabelNested output must contain exactly one separate argument in the true unit, no security option in the false unit, exactly one security option total, and no equals, quoted, alternate, duplicate, or unrelated form; found expected-in-true={expected_count}, true-security-options={true_security_opt_count}, false-security-options={false_security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} SecurityLabelNested: true emits exactly one separate security option; false emits none"
    );
    Ok(())
}

fn verify_security_label_type_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "SecurityLabelType generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} SecurityLabelType generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let unit = generated_unit(version, &generated, "security-label-type.service", output)?;
    let podman_run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} SecurityLabelType unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(SECURITY_LABEL_TYPE_ARGUMENT).count();
    let security_opt_count = podman_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(SECURITY_LABEL_TYPE_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = SECURITY_LABEL_TYPE_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1
        || security_opt_count != 1
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} SecurityLabelType output must contain exactly one separate `{SECURITY_LABEL_TYPE_ARGUMENT}`, exactly one security option total, and no equals, quoted, alternate, unrelated, or duplicate form; found expected={expected_count}, scoped-security-options={security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} SecurityLabelType: exactly one separate process-type security option");
    Ok(())
}

fn verify_mask_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "Mask generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Mask generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let unit = generated_unit(version, &generated, "mask.service", output)?;
    let podman_run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} Mask unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(MASK_ARGUMENT).count();
    let security_opt_count = podman_run.matches("--security-opt").count();
    let total_expected_count = generated.matches(MASK_ARGUMENT).count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = MASK_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1
        || security_opt_count != 1
        || total_expected_count != 1
        || total_security_opt_count != 1
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} Mask output must contain exactly one final separate `{MASK_ARGUMENT}`, exactly one security option total, and no pre-reset, empty, equals, quoted, alternate, unrelated, or duplicate form; found expected={expected_count}, scoped-security-options={security_opt_count}, expected-total={total_expected_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} Mask: reset leaves exactly one separate final colon-path security option");
    Ok(())
}

fn verify_unmask_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    ensure_success(version, "Unmask generator", output)?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Unmask generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }

    let unit = generated_unit(version, &generated, "unmask.service", output)?;
    let podman_run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} Unmask unit is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let argument_counts: Vec<_> = UNMASK_ARGUMENTS
        .iter()
        .map(|argument| podman_run.matches(argument).count())
        .collect();
    let total_argument_counts: Vec<_> = UNMASK_ARGUMENTS
        .iter()
        .map(|argument| generated.matches(argument).count())
        .collect();
    let first_position = podman_run.find(UNMASK_ARGUMENTS[0]);
    let second_position = podman_run.find(UNMASK_ARGUMENTS[1]);
    let security_opt_count = podman_run.matches("--security-opt").count();
    let total_security_opt_count = generated.matches("--security-opt").count();
    let unrelated_or_alternate: Vec<_> = UNMASK_UNRELATED_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if argument_counts != [1, 1]
        || total_argument_counts != [1, 1]
        || !matches!((first_position, second_position), (Some(first), Some(second)) if first < second)
        || security_opt_count != 2
        || total_security_opt_count != 2
        || !unrelated_or_alternate.is_empty()
    {
        return Err(format!(
            "Podman {version} Unmask output must contain only two ordered final separate arguments `{}` then `{}`, exactly once each, and no pre-reset, empty, equals, quoted, alternate, unrelated, or duplicate form; found scoped-counts={argument_counts:?}, total-counts={total_argument_counts:?}, positions=({first_position:?}, {second_position:?}), scoped-security-options={security_opt_count}, security-options-total={total_security_opt_count}, unrelated-or-alternate={unrelated_or_alternate:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            UNMASK_ARGUMENTS[0], UNMASK_ARGUMENTS[1]
        ));
    }
    eprintln!("Podman {version} Unmask: reset leaves exactly two separate ordered final security options");
    Ok(())
}

fn verify_quoted_label_encoding(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let literal_count = generated.matches(QUOTED_LABEL_LITERAL_SPACE).count();
    let hex_count = generated.matches(QUOTED_LABEL_HEX_SPACE).count();
    let (expected_name, expected_count, unexpected_count) = if parsed.major() == 5 && parsed.minor() == 4 {
        ("literal-space", literal_count, hex_count)
    } else {
        ("hex-space", hex_count, literal_count)
    };
    if expected_count != 1 || unexpected_count != 0 {
        return Err(format!(
            "Podman {version} generator output must contain exactly one {expected_name} quoted-label encoding and no other supported encoding; found literal-space={literal_count}, hex-space={hex_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} quoted-label encoding: {expected_name}");
    Ok(())
}

fn ensure_success(version: &str, operation: &str, output: &Output) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "Podman {version} {operation} failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}
