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
const BUILD_IMAGE_TAG_ARGUMENTS: &[&str] = &[
    "--tag localhost/quadlet-lens-build:primary",
    "--tag localhost/quadlet-lens-build:secondary",
];
const BUILD_NETWORK_ARGUMENTS: &[&str] = &["--network host", "--network none", "--network quadlet-lens-network"];
const BUILD_LABEL_ARGUMENTS: &[&str] = &["--label build.label=one", "--label empty="];
const BUILD_NETWORK_DEPENDENCY: &str = "Requires=app-network.service";
const BUILD_NETWORK_ORDERING: &str = "After=app-network.service";
const BUILD_FILE_FINAL_ARGUMENT: &str = "--file Containerfile.final";
const BUILD_FILE_EARLIER_ARGUMENT: &str = "--file Containerfile.first";
const BUILD_TARGET_ARGUMENT: &str = "--target build-stage";
const BUILD_WORKING_DIRECTORY: &str = "WorkingDirectory=/fixtures";
const BUILD_ARG_ARGUMENTS: &[&str] = &["--build-arg key=value", "--build-arg empty="];
const BUILD_SECRET_ARGUMENTS: &[&str] = &[
    "--secret id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one",
    "--secret id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two",
];
const BUILD_ARCH_ARGUMENT: &str = "--arch arm64";
const BUILD_VARIANT_ARGUMENT: &str = "--variant v8";
const BUILD_PULL_ARGUMENT: &str = "--pull=always";
const BUILD_PODMAN_ARGS_ARGUMENT: &str = "--build-context extra=container-image://alpine:3.15";
const BUILD_PODMAN_ARGS_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_ALTERNATE_FORMS: &[&str] = &[
    "--build-context=extra=container-image://alpine:3.15",
    "--build-context \"extra=container-image://alpine:3.15\"",
    "--build-context 'extra=container-image://alpine:3.15'",
    "\"--build-context extra=container-image://alpine:3.15\"",
    "'--build-context extra=container-image://alpine:3.15'",
];
const BUILD_PODMAN_ARGS_NO_CACHE_ARGUMENT: &str = "--no-cache";
const BUILD_PODMAN_ARGS_NO_CACHE_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_NO_CACHE_ALTERNATE_FORMS: &[&str] = &[
    "--no-cache=",
    "--no-cache=true",
    "--no-cache=false",
    "--no-cache \"\"",
    "--no-cache ''",
    "\"--no-cache\"",
    "'--no-cache'",
];
const BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ARGUMENT: &str = "--isolation=chroot";
const BUILD_PODMAN_ARGS_ISOLATION_CHROOT_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ALTERNATE_FORMS: &[&str] = &[
    "--isolation chroot",
    "--isolation=\"chroot\"",
    "--isolation='chroot'",
    "--isolation \"chroot\"",
    "--isolation 'chroot'",
    "\"--isolation=chroot\"",
    "'--isolation=chroot'",
    "--isolation=oci",
    "--isolation=rootless",
];
const BUILD_PODMAN_ARGS_SSH_DEFAULT_ARGUMENT: &str = "--ssh=default";
const BUILD_PODMAN_ARGS_SSH_DEFAULT_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_SSH_DEFAULT_ALTERNATE_FORMS: &[&str] = &[
    "--ssh default",
    "--ssh=default=",
    "--ssh=\"default\"",
    "--ssh='default'",
    "--ssh \"default\"",
    "--ssh 'default'",
    "\"--ssh=default\"",
    "'--ssh=default'",
    "--ssh=custom",
];
const BUILD_PODMAN_ARGS_SHM_SIZE_32M_ARGUMENT: &str = "--shm-size=32m";
const BUILD_PODMAN_ARGS_SHM_SIZE_32M_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_SHM_SIZE_32M_ALTERNATE_FORMS: &[&str] = &[
    "--shm-size 32m",
    "--shm-size=\"32m\"",
    "--shm-size='32m'",
    "--shm-size \"32m\"",
    "--shm-size '32m'",
    "\"--shm-size=32m\"",
    "'--shm-size=32m'",
    "--shm-size=0",
    "--shm-size=64m",
];
const BUILD_PODMAN_ARGS_ULIMIT_NPROC_ARGUMENT: &str = "--ulimit=nproc=4096:8192";
const BUILD_PODMAN_ARGS_ULIMIT_NPROC_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_ULIMIT_NPROC_ALTERNATE_FORMS: &[&str] = &[
    "--ulimit nproc=4096:8192",
    "--ulimit=\"nproc=4096:8192\"",
    "--ulimit='nproc=4096:8192'",
    "--ulimit \"nproc=4096:8192\"",
    "--ulimit 'nproc=4096:8192'",
    "\"--ulimit=nproc=4096:8192\"",
    "'--ulimit=nproc=4096:8192'",
    "--ulimit=nproc=2048:4096",
    "--ulimit=nproc=8192:8192",
];
const BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_ARGUMENT: &str = "--add-host=buildhost:192.0.2.10";
const BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_ALTERNATE_FORMS: &[&str] = &[
    "--add-host buildhost:192.0.2.10",
    "--add-host=\"buildhost:192.0.2.10\"",
    "--add-host='buildhost:192.0.2.10'",
    "--add-host \"buildhost:192.0.2.10\"",
    "--add-host 'buildhost:192.0.2.10'",
    "\"--add-host=buildhost:192.0.2.10\"",
    "'--add-host=buildhost:192.0.2.10'",
    "--add-host=buildhost:192.0.2.11",
    "--add-host=otherhost:192.0.2.10",
    "--add-host=buildhost:host-gateway",
];
const BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_ARGUMENT: &str = "--cap-add=CAP_SYS_ADMIN";
const BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_ALTERNATE_FORMS: &[&str] = &[
    "--cap-add CAP_SYS_ADMIN",
    "--cap-add=\"CAP_SYS_ADMIN\"",
    "--cap-add='CAP_SYS_ADMIN'",
    "--cap-add \"CAP_SYS_ADMIN\"",
    "--cap-add 'CAP_SYS_ADMIN'",
    "\"--cap-add=CAP_SYS_ADMIN\"",
    "'--cap-add=CAP_SYS_ADMIN'",
    "--cap-add=CAP_NET_ADMIN",
    "--cap-add=ALL",
];
const BUILD_PODMAN_ARGS_CACHE_FROM_ARGUMENT: &str = "--cache-from registry.invalid/quadlet-lens/cache-from";
const BUILD_PODMAN_ARGS_CACHE_TO_ARGUMENT: &str = "--cache-to registry.invalid/quadlet-lens/cache-to";
const BUILD_PODMAN_ARGS_CACHE_LOCATIONS_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_CACHE_LOCATIONS_ALTERNATE_FORMS: &[&str] = &[
    "--cache-from=registry.invalid/quadlet-lens/cache-from",
    "--cache-to=registry.invalid/quadlet-lens/cache-to",
    "--cache-from \"registry.invalid/quadlet-lens/cache-from\"",
    "--cache-to \"registry.invalid/quadlet-lens/cache-to\"",
    "--cache-from 'registry.invalid/quadlet-lens/cache-from'",
    "--cache-to 'registry.invalid/quadlet-lens/cache-to'",
    "\"--cache-from registry.invalid/quadlet-lens/cache-from\"",
    "\"--cache-to registry.invalid/quadlet-lens/cache-to\"",
    "'--cache-from registry.invalid/quadlet-lens/cache-from'",
    "'--cache-to registry.invalid/quadlet-lens/cache-to'",
];
const BUILD_PODMAN_ARGS_SBOM_PRESET_ARGUMENT: &str = "--sbom=syft";
const BUILD_PODMAN_ARGS_SBOM_OUTPUT_ARGUMENT: &str = "--sbom-output=/tmp/quadlet-lens-sbom.json";
const BUILD_PODMAN_ARGS_SBOM_CONTEXT: &str = ".";
const BUILD_PODMAN_ARGS_SBOM_ALTERNATE_FORMS: &[&str] = &[
    "--sbom syft",
    "--sbom=spdx",
    "--sbom=syft=",
    "--sbom=\"syft\"",
    "--sbom='syft'",
    "--sbom \"syft\"",
    "--sbom 'syft'",
    "\"--sbom=syft\"",
    "'--sbom=syft'",
    "--sbom-output /tmp/quadlet-lens-sbom.json",
    "--sbom-output=/tmp/quadlet-lens-sbom.json=",
    "--sbom-output=\"/tmp/quadlet-lens-sbom.json\"",
    "--sbom-output='/tmp/quadlet-lens-sbom.json'",
    "--sbom-output \"/tmp/quadlet-lens-sbom.json\"",
    "--sbom-output '/tmp/quadlet-lens-sbom.json'",
    "\"--sbom-output=/tmp/quadlet-lens-sbom.json\"",
    "'--sbom-output=/tmp/quadlet-lens-sbom.json'",
    "--sbom-output=/tmp/quadlet-lens-sbom.spdx.json",
];
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
const BUILD_RETRY_FLAG: &str = "--retry";
const BUILD_RETRY_VALUE: &str = "4";
const BUILD_RETRY_PAIR: &str = "--retry 4";
const BUILD_RETRY_DELAY_FLAG: &str = "--retry-delay";
const BUILD_RETRY_DELAY_VALUE: &str = "7s";
const BUILD_RETRY_DELAY_PAIR: &str = "--retry-delay 7s";
const BUILD_RETRY_CONTEXT: &str = ".";
const BUILD_RETRY_ALTERNATE_FORMS: &[&str] = &[
    "--retry=4",
    "--retry=\"4\"",
    "--retry='4'",
    "--retry \"4\"",
    "--retry '4'",
    "--retry=5",
    "--retry 5",
];
const BUILD_RETRY_DELAY_ALTERNATE_FORMS: &[&str] = &[
    "--retry-delay=7s",
    "--retry-delay=\"7s\"",
    "--retry-delay='7s'",
    "--retry-delay \"7s\"",
    "--retry-delay '7s'",
    "--retry-delay=8s",
    "--retry-delay 8s",
];
const BUILD_TLS_VERIFY_ARGUMENT: &str = "--tls-verify";
const BUILD_TLS_VERIFY_FALSE_ARGUMENT: &str = "--tls-verify=false";
const BUILD_TLS_VERIFY_CONTEXT: &str = ".";
const BUILD_TLS_VERIFY_TRUE_ALTERNATE_FORMS: &[&str] = &[
    "--tls-verify=true",
    "--tls-verify=\"true\"",
    "--tls-verify='true'",
    "--tls-verify \"true\"",
    "--tls-verify 'true'",
    "--tls-verify false",
];
const BUILD_TLS_VERIFY_FALSE_ALTERNATE_FORMS: &[&str] = &[
    "--tls-verify \"false\"",
    "--tls-verify 'false'",
    "--tls-verify=\"false\"",
    "--tls-verify='false'",
    "--tls-verify=true",
];
const BUILD_FORCE_RM_ARGUMENT: &str = "--force-rm";
const BUILD_FORCE_RM_FALSE_ARGUMENT: &str = "--force-rm=false";
const BUILD_FORCE_RM_CONTEXT: &str = ".";
const BUILD_FORCE_RM_TRUE_ALTERNATE_FORMS: &[&str] = &[
    "--force-rm=true",
    "--force-rm=\"true\"",
    "--force-rm='true'",
    "--force-rm \"true\"",
    "--force-rm 'true'",
    "--force-rm false",
];
const BUILD_FORCE_RM_FALSE_ALTERNATE_FORMS: &[&str] = &[
    "--force-rm \"false\"",
    "--force-rm 'false'",
    "--force-rm=\"false\"",
    "--force-rm='false'",
    "--force-rm=true",
];
const BUILD_GROUP_ADD_FLAG: &str = "--group-add";
const BUILD_GROUP_ADD_FIRST_PAIR: &str = "--group-add 1234";
const BUILD_GROUP_ADD_SECOND_PAIR: &str = "--group-add 5678";
const BUILD_GROUP_ADD_CONTEXT: &str = ".";
const BUILD_GROUP_ADD_ALTERNATE_FORMS: &[&str] = &[
    "--group-add=1234",
    "--group-add=5678",
    "--group-add=\"1234\"",
    "--group-add=\"5678\"",
    "--group-add='1234'",
    "--group-add='5678'",
    "--group-add \"1234\"",
    "--group-add \"5678\"",
    "--group-add '1234'",
    "--group-add '5678'",
    "--group-add 1234,5678",
    "--group-add=1234,5678",
    "--group-add 1234 5678",
];
const BUILD_DNS_FLAG: &str = "--dns";
const BUILD_DNS_FIRST_PAIR: &str = "--dns 9.9.9.9";
const BUILD_DNS_SECOND_PAIR: &str = "--dns 2001:4860:4860::8888";
const BUILD_DNS_CONTEXT: &str = ".";
const BUILD_DNS_ALTERNATE_FORMS: &[&str] = &[
    "--dns=9.9.9.9",
    "--dns=2001:4860:4860::8888",
    "--dns=\"9.9.9.9\"",
    "--dns=\"2001:4860:4860::8888\"",
    "--dns='9.9.9.9'",
    "--dns='2001:4860:4860::8888'",
    "--dns \"9.9.9.9\"",
    "--dns \"2001:4860:4860::8888\"",
    "--dns '9.9.9.9'",
    "--dns '2001:4860:4860::8888'",
    "--dns=",
    "--dns \"\"",
    "--dns ''",
    "--dns 9.9.9.9,2001:4860:4860::8888",
    "--dns=9.9.9.9,2001:4860:4860::8888",
    "--dns 9.9.9.9 2001:4860:4860::8888",
];
const BUILD_DNS_OPTION_FLAG: &str = "--dns-option";
const BUILD_DNS_OPTION_FIRST: &str = "--dns-option ndots:1";
const BUILD_DNS_OPTION_SECOND: &str = "--dns-option use-vc";
const BUILD_DNS_SEARCH_FLAG: &str = "--dns-search";
const BUILD_DNS_SEARCH_FIRST: &str = "--dns-search corp.example";
const BUILD_DNS_SEARCH_SECOND: &str = "--dns-search .";
const BUILD_AUTH_FILE_FLAG: &str = "--authfile";
const BUILD_AUTH_FILE_SINGLE: &str = "--authfile /run/quadlet-lens/single-auth.json";
const BUILD_AUTH_FILE_LAST: &str = "--authfile /run/quadlet-lens/last-auth.json";
const BUILD_IGNORE_FILE_FLAG: &str = "--ignorefile";
const BUILD_IGNORE_FILE_SINGLE: &str = "--ignorefile /run/quadlet-lens/single.ignore";
const BUILD_IGNORE_FILE_LAST: &str = "--ignorefile /run/quadlet-lens/last.ignore";
const BUILD_ANNOTATION_PRE_RESET_OR_REPLACED: &[&str] = &[
    "org.example.pre=one",
    "org.example.pre=two",
    "org.example.alpha=earlier",
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
    container_batch: (PathBuf, Vec<String>),
    container_direct_maps: (PathBuf, Vec<String>),
    container_sub_maps: (PathBuf, Vec<String>),
    container_retry: (PathBuf, Vec<String>),
    container_http_proxy: (PathBuf, Vec<String>),
    container_start_with_pod: (PathBuf, Vec<String>),
    memory: (PathBuf, Vec<String>),
    build_retry: (PathBuf, Vec<String>),
    build_tls_verify: (PathBuf, Vec<String>),
    build_force_rm: (PathBuf, Vec<String>),
    build_group_add: (PathBuf, Vec<String>),
    build_dns: (PathBuf, Vec<String>),
    build_dns_option: (PathBuf, Vec<String>),
    build_dns_search: (PathBuf, Vec<String>),
    build_auth_file: (PathBuf, Vec<String>),
    build_ignore_file: (PathBuf, Vec<String>),
    build_annotation: (PathBuf, Vec<String>),
    build_environment: (PathBuf, Vec<String>),
    build_containers_conf_module: (PathBuf, Vec<String>),
    build_global_args: (PathBuf, Vec<String>),
    build_service_name: (PathBuf, Vec<String>),
    build_volume: (PathBuf, Vec<String>),
    build_arg: (PathBuf, Vec<String>),
    build_secret: (PathBuf, Vec<String>),
    build_platform: (PathBuf, Vec<String>),
    build_pull: (PathBuf, Vec<String>),
    build_podman_args: (PathBuf, Vec<String>),
    build_podman_args_no_cache: (PathBuf, Vec<String>),
    build_podman_args_isolation_chroot: (PathBuf, Vec<String>),
    build_podman_args_ssh_default: (PathBuf, Vec<String>),
    build_podman_args_shm_size_32m: (PathBuf, Vec<String>),
    build_podman_args_ulimit_nproc: (PathBuf, Vec<String>),
    build_podman_args_add_host_buildhost: (PathBuf, Vec<String>),
    build_podman_args_cap_add_cap_sys_admin: (PathBuf, Vec<String>),
    build_podman_args_cache_locations: (PathBuf, Vec<String>),
    build_podman_args_sbom: (PathBuf, Vec<String>),
    interactive: (PathBuf, Vec<String>),
    tty: (PathBuf, Vec<String>),
    privileged: (PathBuf, Vec<String>),
    logging: (PathBuf, Vec<String>),
    network_identity: (PathBuf, Vec<String>),
    network_driver_options: (PathBuf, Vec<String>),
    network_labels: (PathBuf, Vec<String>),
    volume_labels: (PathBuf, Vec<String>),
    volume_containers_conf_module: (PathBuf, Vec<String>),
    volume_global_args: (PathBuf, Vec<String>),
    volume_podman_args: (PathBuf, Vec<String>),
    volume_user: (PathBuf, Vec<String>),
    volume_group: (PathBuf, Vec<String>),
    volume_uid: (PathBuf, Vec<String>),
    volume_gid: (PathBuf, Vec<String>),
    volume_service_name: (PathBuf, Vec<String>),
    volume_image: (PathBuf, Vec<String>),
    image_core: (PathBuf, Vec<String>),
    image_image_tag: (PathBuf, Vec<String>),
    image_service_name: (PathBuf, Vec<String>),
    image_all_tags: (PathBuf, Vec<String>),
    image_arch: (PathBuf, Vec<String>),
    image_auth_file: (PathBuf, Vec<String>),
    image_creds: (PathBuf, Vec<String>),
    image_decryption_key: (PathBuf, Vec<String>),
    image_global_args: (PathBuf, Vec<String>),
    image_os: (PathBuf, Vec<String>),
    image_cert_dir: (PathBuf, Vec<String>),
    image_containers_conf_module: (PathBuf, Vec<String>),
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

    assert_eq!(
        matrix.source_repository,
        "https://github.com/podman-container-tools/podman.git"
    );
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
#[allow(clippy::too_many_lines)] // Ordered full-matrix fixture contract.
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
    let reload_fixture = reload_fixture_directory()?;
    let reload_expected = expected_fragments(&reload_fixture)?;
    let reload_conflict_fixture = reload_conflict_fixture_directory()?;
    let exit_policy_fixture = exit_policy_fixture_directory()?;
    let exit_policy_expected = expected_fragments(&exit_policy_fixture)?;
    let stop_timeout_fixture = stop_timeout_fixture_directory()?;
    let stop_timeout_expected = expected_fragments(&stop_timeout_fixture)?;
    let service_name_fixture = pod_service_name_fixture_directory()?;
    let service_name_expected = expected_fragments(&service_name_fixture)?;
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
        verify_reload_generator_output(
            &image.version,
            &reload_expected,
            &run_generator_raw(&engine, image, &reload_fixture)?,
        )?;
        verify_reload_conflict_generator_output(
            &image.version,
            &run_generator_raw(&engine, image, &reload_conflict_fixture)?,
        )?;
        verify_exit_policy_generator_output(
            &image.version,
            &exit_policy_expected,
            &run_generator_raw(&engine, image, &exit_policy_fixture)?,
        )?;
        verify_stop_timeout_generator_output(
            &image.version,
            &stop_timeout_expected,
            &run_generator_raw(&engine, image, &stop_timeout_fixture)?,
        )?;
        verify_pod_service_name_generator_output(
            &image.version,
            &service_name_expected,
            &run_generator_raw(&engine, image, &service_name_fixture)?,
        )?;
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
        verify_reload_generator_output(
            &source.version,
            &reload_expected,
            &run_source_generator_raw(&engine, &matrix.builder_reference, source, &generator, &reload_fixture)?,
        )?;
        verify_reload_conflict_generator_output(
            &source.version,
            &run_source_generator_raw(
                &engine,
                &matrix.builder_reference,
                source,
                &generator,
                &reload_conflict_fixture,
            )?,
        )?;
        verify_exit_policy_generator_output(
            &source.version,
            &exit_policy_expected,
            &run_source_generator_raw(
                &engine,
                &matrix.builder_reference,
                source,
                &generator,
                &exit_policy_fixture,
            )?,
        )?;
        verify_stop_timeout_generator_output(
            &source.version,
            &stop_timeout_expected,
            &run_source_generator_raw(
                &engine,
                &matrix.builder_reference,
                source,
                &generator,
                &stop_timeout_fixture,
            )?,
        )?;
        verify_pod_service_name_generator_output(
            &source.version,
            &service_name_expected,
            &run_source_generator_raw(
                &engine,
                &matrix.builder_reference,
                source,
                &generator,
                &service_name_fixture,
            )?,
        )?;
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

fn reload_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/container-reload-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn reload_conflict_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/container-reload-conflict-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn exit_policy_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/pod-exit-policy-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn stop_timeout_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/pod-stop-timeout-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn pod_service_name_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/pod-service-name-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_retry_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-retry-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_tls_verify_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-tls-verify-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_force_rm_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-force-rm-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_group_add_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-group-add-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_dns_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-dns-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_dns_option_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-dns-option-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_dns_search_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-dns-search-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_auth_file_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-auth-file-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_ignore_file_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-ignore-file-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_annotation_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-annotation-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_environment_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-environment-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn build_containers_conf_module_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-containers-conf-module-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn build_global_args_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-global-args-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn build_service_name_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-service-name-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn build_volume_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-volume-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_containers_conf_module_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-containers-conf-module-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_global_args_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-global-args-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_podman_args_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-podman-args-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_user_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-user-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_group_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-group-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_uid_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-uid-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_gid_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-gid-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_service_name_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-service-name-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn volume_image_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/volume-image-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_core_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-core-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_image_tag_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-image-tag-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_service_name_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-service-name-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_all_tags_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-all-tags-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_arch_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-arch-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_auth_file_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-auth-file-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_creds_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-creds-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_decryption_key_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-decryption-key-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_global_args_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-global-args-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_os_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-os-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_cert_dir_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-cert-dir-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn image_containers_conf_module_fixture_directory() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/image-containers-conf-module-supported-range")
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn build_arg_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-arg-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_secret_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-secret-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_platform_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-platform-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_pull_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-pull-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-podman-args-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_no_cache_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-no-cache-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_isolation_chroot_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-isolation-chroot-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_ssh_default_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-ssh-default-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_shm_size_32m_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-shm-size-32m-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_ulimit_nproc_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-ulimit-nproc-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_add_host_buildhost_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-add-host-buildhost-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_cap_add_cap_sys_admin_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-cap-add-cap-sys-admin-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_cache_locations_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators/build-podman-args-cache-locations-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn build_podman_args_sbom_fixture_directory() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/build-podman-args-sbom-supported-range");
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

fn load_build_retry_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_retry_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_tls_verify_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_tls_verify_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_force_rm_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_force_rm_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_group_add_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_group_add_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_dns_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_dns_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_dns_option_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_dns_option_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_dns_search_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_dns_search_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_auth_file_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_auth_file_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_ignore_file_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_ignore_file_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_annotation_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_annotation_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}
fn load_build_environment_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_environment_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_containers_conf_module_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_containers_conf_module_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_global_args_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_global_args_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_service_name_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_service_name_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_volume_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_volume_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_containers_conf_module_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_containers_conf_module_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_global_args_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_global_args_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_podman_args_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_podman_args_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_user_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_user_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_group_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_group_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_uid_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_uid_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_gid_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_gid_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_service_name_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_service_name_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_volume_image_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = volume_image_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_core_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_core_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_image_image_tag_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_image_tag_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_service_name_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_service_name_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_all_tags_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_all_tags_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_arch_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_arch_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_auth_file_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_auth_file_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_creds_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_creds_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_decryption_key_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_decryption_key_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_global_args_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_global_args_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_os_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_os_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_cert_dir_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_cert_dir_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_image_containers_conf_module_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = image_containers_conf_module_fixture_directory()?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
}

fn load_build_arg_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_arg_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_secret_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_secret_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_platform_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_platform_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_pull_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_pull_fixture_directory()?;
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

fn load_build_podman_args_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_no_cache_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_no_cache_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_isolation_chroot_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_isolation_chroot_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_ssh_default_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_ssh_default_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_shm_size_32m_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_shm_size_32m_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_ulimit_nproc_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_ulimit_nproc_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_add_host_buildhost_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_add_host_buildhost_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_cap_add_cap_sys_admin_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_cap_add_cap_sys_admin_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_cache_locations_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_cache_locations_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_build_podman_args_sbom_fixture() -> Result<(PathBuf, Vec<String>), String> {
    let fixture = build_podman_args_sbom_fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    Ok((fixture, expected))
}

fn load_generator_fixtures() -> Result<GeneratorFixtures, String> {
    Ok(GeneratorFixtures {
        container_batch: load_container_batch_fixture()?,
        container_direct_maps: load_named_container_fixture("container-direct-maps-supported-range")?,
        container_sub_maps: load_named_container_fixture("container-sub-maps-supported-range")?,
        container_retry: load_named_container_fixture("container-retry-supported-range")?,
        container_http_proxy: load_named_container_fixture("container-http-proxy-supported-range")?,
        container_start_with_pod: load_named_container_fixture("container-start-with-pod-supported-range")?,
        memory: load_memory_fixture()?,
        build_retry: load_build_retry_fixture()?,
        build_tls_verify: load_build_tls_verify_fixture()?,
        build_force_rm: load_build_force_rm_fixture()?,
        build_group_add: load_build_group_add_fixture()?,
        build_dns: load_build_dns_fixture()?,
        build_dns_option: load_build_dns_option_fixture()?,
        build_dns_search: load_build_dns_search_fixture()?,
        build_auth_file: load_build_auth_file_fixture()?,
        build_ignore_file: load_build_ignore_file_fixture()?,
        build_annotation: load_build_annotation_fixture()?,
        build_environment: load_build_environment_fixture()?,
        build_containers_conf_module: load_build_containers_conf_module_fixture()?,
        build_global_args: load_build_global_args_fixture()?,
        build_service_name: load_build_service_name_fixture()?,
        build_volume: load_build_volume_fixture()?,
        volume_containers_conf_module: load_volume_containers_conf_module_fixture()?,
        volume_global_args: load_volume_global_args_fixture()?,
        volume_podman_args: load_volume_podman_args_fixture()?,
        volume_user: load_volume_user_fixture()?,
        volume_group: load_volume_group_fixture()?,
        volume_uid: load_volume_uid_fixture()?,
        volume_gid: load_volume_gid_fixture()?,
        volume_service_name: load_volume_service_name_fixture()?,
        volume_image: load_volume_image_fixture()?,
        image_core: load_image_core_fixture()?,
        image_image_tag: load_image_image_tag_fixture()?,
        image_service_name: load_image_service_name_fixture()?,
        image_all_tags: load_image_all_tags_fixture()?,
        image_arch: load_image_arch_fixture()?,
        image_auth_file: load_image_auth_file_fixture()?,
        image_creds: load_image_creds_fixture()?,
        image_decryption_key: load_image_decryption_key_fixture()?,
        image_global_args: load_image_global_args_fixture()?,
        image_os: load_image_os_fixture()?,
        image_cert_dir: load_image_cert_dir_fixture()?,
        image_containers_conf_module: load_image_containers_conf_module_fixture()?,
        build_arg: load_build_arg_fixture()?,
        build_secret: load_build_secret_fixture()?,
        build_platform: load_build_platform_fixture()?,
        build_pull: load_build_pull_fixture()?,
        build_podman_args: load_build_podman_args_fixture()?,
        build_podman_args_no_cache: load_build_podman_args_no_cache_fixture()?,
        build_podman_args_isolation_chroot: load_build_podman_args_isolation_chroot_fixture()?,
        build_podman_args_ssh_default: load_build_podman_args_ssh_default_fixture()?,
        build_podman_args_shm_size_32m: load_build_podman_args_shm_size_32m_fixture()?,
        build_podman_args_ulimit_nproc: load_build_podman_args_ulimit_nproc_fixture()?,
        build_podman_args_add_host_buildhost: load_build_podman_args_add_host_buildhost_fixture()?,
        build_podman_args_cap_add_cap_sys_admin: load_build_podman_args_cap_add_cap_sys_admin_fixture()?,
        build_podman_args_cache_locations: load_build_podman_args_cache_locations_fixture()?,
        build_podman_args_sbom: load_build_podman_args_sbom_fixture()?,
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

fn load_container_batch_fixture() -> Result<(PathBuf, Vec<String>), String> {
    load_named_container_fixture("container-batch-supported-range")
}

fn load_named_container_fixture(name: &str) -> Result<(PathBuf, Vec<String>), String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators")
        .join(name);
    let fixture = path
        .canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))?;
    Ok((fixture.clone(), expected_fragments(&fixture)?))
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

fn verify_image_build_retry(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_retry_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_tls_verify(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_tls_verify_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_force_rm(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_force_rm_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_group_add(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_group_add_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_dns(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_dns_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_dns_option(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_dns_option_generator_output(
        &image.version,
        &fixture.1,
        &run_generator_raw(engine, image, &fixture.0)?,
    )
}

fn verify_image_build_dns_search(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_dns_search_generator_output(
        &image.version,
        &fixture.1,
        &run_generator_raw(engine, image, &fixture.0)?,
    )
}

fn verify_image_build_auth_file(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_auth_file_generator_output(
        &image.version,
        &fixture.1,
        &run_generator_raw(engine, image, &fixture.0)?,
    )
}

fn verify_image_build_ignore_file(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_ignore_file_generator_output(
        &image.version,
        &fixture.1,
        &run_generator_raw(engine, image, &fixture.0)?,
    )
}

fn verify_image_build_annotation(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_annotation_generator_output(
        &image.version,
        &fixture.1,
        &run_generator_raw(engine, image, &fixture.0)?,
    )
}

fn verify_image_build_arg(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_arg_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_secret(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_secret_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_platform(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_platform_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_pull(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_pull_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_no_cache(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_no_cache_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_isolation_chroot(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_isolation_chroot_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_ssh_default(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_ssh_default_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_shm_size_32m(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_shm_size_32m_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_ulimit_nproc(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_ulimit_nproc_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_add_host_buildhost(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_add_host_buildhost_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_cap_add_cap_sys_admin(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_cap_add_cap_sys_admin_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_cache_locations(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_cache_locations_generator_output(&image.version, &fixture.1, &output)
}

fn verify_image_build_podman_args_sbom(
    engine: &str,
    image: &GeneratorImage,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_generator_raw(engine, image, &fixture.0)?;
    verify_build_podman_args_sbom_generator_output(&image.version, &fixture.1, &output)
}

#[allow(clippy::too_many_lines)] // Ordered fixture contract.
fn verify_image_isolated_fixtures(
    engine: &str,
    image: &GeneratorImage,
    fixtures: &GeneratorFixtures,
) -> Result<(), String> {
    verify_container_batch_generator_output(
        &image.version,
        &fixtures.container_batch.1,
        &run_generator_raw(engine, image, &fixtures.container_batch.0)?,
    )?;
    verify_container_direct_maps_generator_output(
        &image.version,
        &fixtures.container_direct_maps.1,
        &run_generator_raw(engine, image, &fixtures.container_direct_maps.0)?,
    )?;
    verify_container_sub_maps_generator_output(
        &image.version,
        &fixtures.container_sub_maps.1,
        &run_generator_raw(engine, image, &fixtures.container_sub_maps.0)?,
    )?;
    verify_container_retry_generator_output(
        &image.version,
        &fixtures.container_retry.1,
        &run_generator_raw(engine, image, &fixtures.container_retry.0)?,
    )?;
    verify_container_http_proxy_generator_output(
        &image.version,
        &fixtures.container_http_proxy.1,
        &run_generator_raw(engine, image, &fixtures.container_http_proxy.0)?,
    )?;
    verify_container_start_with_pod_generator_output(
        &image.version,
        &fixtures.container_start_with_pod.1,
        &run_generator_raw(engine, image, &fixtures.container_start_with_pod.0)?,
    )?;
    verify_image_memory(engine, image, &fixtures.memory)?;
    verify_image_build_retry(engine, image, &fixtures.build_retry)?;
    verify_image_build_tls_verify(engine, image, &fixtures.build_tls_verify)?;
    verify_image_build_force_rm(engine, image, &fixtures.build_force_rm)?;
    verify_image_build_group_add(engine, image, &fixtures.build_group_add)?;
    verify_image_build_dns(engine, image, &fixtures.build_dns)?;
    verify_image_build_dns_option(engine, image, &fixtures.build_dns_option)?;
    verify_image_build_dns_search(engine, image, &fixtures.build_dns_search)?;
    verify_image_build_auth_file(engine, image, &fixtures.build_auth_file)?;
    verify_image_build_ignore_file(engine, image, &fixtures.build_ignore_file)?;
    verify_image_build_annotation(engine, image, &fixtures.build_annotation)?;
    verify_build_environment_generator_output(
        &image.version,
        &fixtures.build_environment.1,
        &run_generator_raw(engine, image, &fixtures.build_environment.0)?,
    )?;
    verify_build_containers_conf_module_generator_output(
        &image.version,
        &fixtures.build_containers_conf_module.1,
        &run_generator_raw(engine, image, &fixtures.build_containers_conf_module.0)?,
    )?;
    verify_build_global_args_generator_output(
        &image.version,
        &fixtures.build_global_args.1,
        &run_generator_raw(engine, image, &fixtures.build_global_args.0)?,
    )?;
    verify_build_service_name_generator_output(
        &image.version,
        &fixtures.build_service_name.1,
        &run_generator_raw(engine, image, &fixtures.build_service_name.0)?,
    )?;
    verify_build_volume_generator_output(
        &image.version,
        &fixtures.build_volume.1,
        &run_generator_raw(engine, image, &fixtures.build_volume.0)?,
    )?;
    verify_volume_containers_conf_module_generator_output(
        &image.version,
        &fixtures.volume_containers_conf_module.1,
        &run_generator_raw(engine, image, &fixtures.volume_containers_conf_module.0)?,
    )?;
    verify_volume_global_args_generator_output(
        &image.version,
        &fixtures.volume_global_args.1,
        &run_generator_raw(engine, image, &fixtures.volume_global_args.0)?,
    )?;
    verify_volume_podman_args_generator_output(
        &image.version,
        &fixtures.volume_podman_args.1,
        &run_generator_raw(engine, image, &fixtures.volume_podman_args.0)?,
    )?;
    verify_volume_user_generator_output(
        &image.version,
        &fixtures.volume_user.1,
        &run_generator_raw(engine, image, &fixtures.volume_user.0)?,
    )?;
    verify_volume_group_generator_output(
        &image.version,
        &fixtures.volume_group.1,
        &run_generator_raw(engine, image, &fixtures.volume_group.0)?,
    )?;
    verify_volume_uid_generator_output(
        &image.version,
        &fixtures.volume_uid.1,
        &run_generator_raw(engine, image, &fixtures.volume_uid.0)?,
    )?;
    verify_volume_gid_generator_output(
        &image.version,
        &fixtures.volume_gid.1,
        &run_generator_raw(engine, image, &fixtures.volume_gid.0)?,
    )?;
    verify_volume_service_name_generator_output(
        &image.version,
        &fixtures.volume_service_name.1,
        &run_generator_raw(engine, image, &fixtures.volume_service_name.0)?,
    )?;
    verify_volume_image_generator_output(
        &image.version,
        &fixtures.volume_image.1,
        &run_generator_raw(engine, image, &fixtures.volume_image.0)?,
    )?;
    verify_image_core_generator_output(
        &image.version,
        &fixtures.image_core.1,
        &run_generator_raw(engine, image, &fixtures.image_core.0)?,
    )?;
    verify_image_image_tag_generator_output(
        &image.version,
        &fixtures.image_image_tag.1,
        &run_generator_raw(engine, image, &fixtures.image_image_tag.0)?,
    )?;
    verify_image_service_name_generator_output(
        &image.version,
        &fixtures.image_service_name.1,
        &run_generator_raw(engine, image, &fixtures.image_service_name.0)?,
    )?;
    verify_image_all_tags_generator_output(
        &image.version,
        &fixtures.image_all_tags.1,
        &run_generator_raw(engine, image, &fixtures.image_all_tags.0)?,
    )?;
    verify_image_arch_generator_output(
        &image.version,
        &fixtures.image_arch.1,
        &run_generator_raw(engine, image, &fixtures.image_arch.0)?,
    )?;
    verify_image_auth_file_generator_output(
        &image.version,
        &fixtures.image_auth_file.1,
        &run_generator_raw(engine, image, &fixtures.image_auth_file.0)?,
    )?;
    verify_image_creds_generator_output(
        &image.version,
        &fixtures.image_creds.1,
        &run_generator_raw(engine, image, &fixtures.image_creds.0)?,
    )?;
    verify_image_decryption_key_generator_output(
        &image.version,
        &fixtures.image_decryption_key.1,
        &run_generator_raw(engine, image, &fixtures.image_decryption_key.0)?,
    )?;
    verify_image_global_args_generator_output(
        &image.version,
        &fixtures.image_global_args.1,
        &run_generator_raw(engine, image, &fixtures.image_global_args.0)?,
    )?;
    verify_image_os_generator_output(
        &image.version,
        &fixtures.image_os.1,
        &run_generator_raw(engine, image, &fixtures.image_os.0)?,
    )?;
    verify_image_cert_dir_generator_output(
        &image.version,
        &fixtures.image_cert_dir.1,
        &run_generator_raw(engine, image, &fixtures.image_cert_dir.0)?,
    )?;
    verify_image_containers_conf_module_generator_output(
        &image.version,
        &fixtures.image_containers_conf_module.1,
        &run_generator_raw(engine, image, &fixtures.image_containers_conf_module.0)?,
    )?;
    verify_image_build_arg(engine, image, &fixtures.build_arg)?;
    verify_image_build_secret(engine, image, &fixtures.build_secret)?;
    verify_image_build_platform(engine, image, &fixtures.build_platform)?;
    verify_image_build_pull(engine, image, &fixtures.build_pull)?;
    verify_image_build_podman_args(engine, image, &fixtures.build_podman_args)?;
    verify_image_build_podman_args_no_cache(engine, image, &fixtures.build_podman_args_no_cache)?;
    verify_image_build_podman_args_isolation_chroot(engine, image, &fixtures.build_podman_args_isolation_chroot)?;
    verify_image_build_podman_args_ssh_default(engine, image, &fixtures.build_podman_args_ssh_default)?;
    verify_image_build_podman_args_shm_size_32m(engine, image, &fixtures.build_podman_args_shm_size_32m)?;
    verify_image_build_podman_args_ulimit_nproc(engine, image, &fixtures.build_podman_args_ulimit_nproc)?;
    verify_image_build_podman_args_add_host_buildhost(engine, image, &fixtures.build_podman_args_add_host_buildhost)?;
    verify_image_build_podman_args_cap_add_cap_sys_admin(
        engine,
        image,
        &fixtures.build_podman_args_cap_add_cap_sys_admin,
    )?;
    verify_image_build_podman_args_cache_locations(engine, image, &fixtures.build_podman_args_cache_locations)?;
    verify_image_build_podman_args_sbom(engine, image, &fixtures.build_podman_args_sbom)?;
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

fn verify_source_build_retry(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_retry_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_tls_verify(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_tls_verify_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_force_rm(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_force_rm_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_group_add(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_group_add_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_dns(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_dns_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_dns_option(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_dns_option_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_dns_search(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_dns_search_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_auth_file(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_auth_file_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_ignore_file(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_ignore_file_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_annotation(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_annotation_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_containers_conf_module(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_containers_conf_module_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_global_args(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_global_args_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_service_name(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_service_name_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_volume(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_build_volume_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_containers_conf_module(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_containers_conf_module_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_global_args(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_global_args_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_podman_args(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_podman_args_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_user(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_user_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_group(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_group_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_uid(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_uid_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_gid(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_gid_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_service_name(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_service_name_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_volume_image(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_volume_image_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_core(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_core_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_image_tag(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_image_tag_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_service_name(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_service_name_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_all_tags(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_all_tags_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_arch(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_arch_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_auth_file(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_auth_file_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_creds(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_creds_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_decryption_key(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_decryption_key_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_global_args(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_global_args_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_os(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_os_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_cert_dir(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_cert_dir_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_image_containers_conf_module(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    verify_image_containers_conf_module_generator_output(
        &source.version,
        &fixture.1,
        &run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?,
    )
}

fn verify_source_build_arg(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_arg_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_secret(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_secret_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_platform(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_platform_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_pull(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_pull_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_no_cache(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_no_cache_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_isolation_chroot(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_isolation_chroot_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_ssh_default(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_ssh_default_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_shm_size_32m(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_shm_size_32m_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_ulimit_nproc(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_ulimit_nproc_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_add_host_buildhost(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_add_host_buildhost_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_cap_add_cap_sys_admin(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_cap_add_cap_sys_admin_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_cache_locations(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_cache_locations_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_build_podman_args_sbom(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &(PathBuf, Vec<String>),
) -> Result<(), String> {
    let output = run_source_generator_raw(engine, &matrix.builder_reference, source, generator, &fixture.0)?;
    verify_build_podman_args_sbom_generator_output(&source.version, &fixture.1, &output)
}

fn verify_source_network_and_volume_fixtures(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixtures: &GeneratorFixtures,
) -> Result<(), String> {
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

#[allow(clippy::too_many_lines)] // Ordered fixture contract.
fn verify_source_isolated_fixtures(
    engine: &str,
    matrix: &GeneratorMatrix,
    source: &GeneratorSource,
    generator: &Path,
    fixtures: &GeneratorFixtures,
) -> Result<(), String> {
    verify_container_batch_generator_output(
        &source.version,
        &fixtures.container_batch.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_batch.0,
        )?,
    )?;
    verify_container_direct_maps_generator_output(
        &source.version,
        &fixtures.container_direct_maps.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_direct_maps.0,
        )?,
    )?;
    verify_container_sub_maps_generator_output(
        &source.version,
        &fixtures.container_sub_maps.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_sub_maps.0,
        )?,
    )?;
    verify_container_retry_generator_output(
        &source.version,
        &fixtures.container_retry.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_retry.0,
        )?,
    )?;
    verify_container_http_proxy_generator_output(
        &source.version,
        &fixtures.container_http_proxy.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_http_proxy.0,
        )?,
    )?;
    verify_container_start_with_pod_generator_output(
        &source.version,
        &fixtures.container_start_with_pod.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.container_start_with_pod.0,
        )?,
    )?;
    verify_source_memory(engine, matrix, source, generator, &fixtures.memory)?;
    verify_source_build_retry(engine, matrix, source, generator, &fixtures.build_retry)?;
    verify_source_build_tls_verify(engine, matrix, source, generator, &fixtures.build_tls_verify)?;
    verify_source_build_force_rm(engine, matrix, source, generator, &fixtures.build_force_rm)?;
    verify_source_build_group_add(engine, matrix, source, generator, &fixtures.build_group_add)?;
    verify_source_build_dns(engine, matrix, source, generator, &fixtures.build_dns)?;
    verify_source_build_dns_option(engine, matrix, source, generator, &fixtures.build_dns_option)?;
    verify_source_build_dns_search(engine, matrix, source, generator, &fixtures.build_dns_search)?;
    verify_source_build_auth_file(engine, matrix, source, generator, &fixtures.build_auth_file)?;
    verify_source_build_ignore_file(engine, matrix, source, generator, &fixtures.build_ignore_file)?;
    verify_source_build_annotation(engine, matrix, source, generator, &fixtures.build_annotation)?;
    verify_build_environment_generator_output(
        &source.version,
        &fixtures.build_environment.1,
        &run_source_generator_raw(
            engine,
            &matrix.builder_reference,
            source,
            generator,
            &fixtures.build_environment.0,
        )?,
    )?;
    verify_source_build_containers_conf_module(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_containers_conf_module,
    )?;
    verify_source_build_global_args(engine, matrix, source, generator, &fixtures.build_global_args)?;
    verify_source_build_service_name(engine, matrix, source, generator, &fixtures.build_service_name)?;
    verify_source_build_volume(engine, matrix, source, generator, &fixtures.build_volume)?;
    verify_source_volume_containers_conf_module(
        engine,
        matrix,
        source,
        generator,
        &fixtures.volume_containers_conf_module,
    )?;
    verify_source_volume_global_args(engine, matrix, source, generator, &fixtures.volume_global_args)?;
    verify_source_volume_podman_args(engine, matrix, source, generator, &fixtures.volume_podman_args)?;
    verify_source_volume_user(engine, matrix, source, generator, &fixtures.volume_user)?;
    verify_source_volume_group(engine, matrix, source, generator, &fixtures.volume_group)?;
    verify_source_volume_uid(engine, matrix, source, generator, &fixtures.volume_uid)?;
    verify_source_volume_gid(engine, matrix, source, generator, &fixtures.volume_gid)?;
    verify_source_volume_service_name(engine, matrix, source, generator, &fixtures.volume_service_name)?;
    verify_source_volume_image(engine, matrix, source, generator, &fixtures.volume_image)?;
    verify_source_image_core(engine, matrix, source, generator, &fixtures.image_core)?;
    verify_source_image_image_tag(engine, matrix, source, generator, &fixtures.image_image_tag)?;
    verify_source_image_service_name(engine, matrix, source, generator, &fixtures.image_service_name)?;
    verify_source_image_all_tags(engine, matrix, source, generator, &fixtures.image_all_tags)?;
    verify_source_image_arch(engine, matrix, source, generator, &fixtures.image_arch)?;
    verify_source_image_auth_file(engine, matrix, source, generator, &fixtures.image_auth_file)?;
    verify_source_image_creds(engine, matrix, source, generator, &fixtures.image_creds)?;
    verify_source_image_decryption_key(engine, matrix, source, generator, &fixtures.image_decryption_key)?;
    verify_source_image_global_args(engine, matrix, source, generator, &fixtures.image_global_args)?;
    verify_source_image_os(engine, matrix, source, generator, &fixtures.image_os)?;
    verify_source_image_cert_dir(engine, matrix, source, generator, &fixtures.image_cert_dir)?;
    verify_source_image_containers_conf_module(
        engine,
        matrix,
        source,
        generator,
        &fixtures.image_containers_conf_module,
    )?;
    verify_source_build_arg(engine, matrix, source, generator, &fixtures.build_arg)?;
    verify_source_build_secret(engine, matrix, source, generator, &fixtures.build_secret)?;
    verify_source_build_platform(engine, matrix, source, generator, &fixtures.build_platform)?;
    verify_source_build_pull(engine, matrix, source, generator, &fixtures.build_pull)?;
    verify_source_build_podman_args(engine, matrix, source, generator, &fixtures.build_podman_args)?;
    verify_source_build_podman_args_no_cache(engine, matrix, source, generator, &fixtures.build_podman_args_no_cache)?;
    verify_source_build_podman_args_isolation_chroot(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_isolation_chroot,
    )?;
    verify_source_build_podman_args_ssh_default(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_ssh_default,
    )?;
    verify_source_build_podman_args_shm_size_32m(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_shm_size_32m,
    )?;
    verify_source_build_podman_args_ulimit_nproc(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_ulimit_nproc,
    )?;
    verify_source_build_podman_args_add_host_buildhost(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_add_host_buildhost,
    )?;
    verify_source_build_podman_args_cap_add_cap_sys_admin(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_cap_add_cap_sys_admin,
    )?;
    verify_source_build_podman_args_cache_locations(
        engine,
        matrix,
        source,
        generator,
        &fixtures.build_podman_args_cache_locations,
    )?;
    verify_source_build_podman_args_sbom(engine, matrix, source, generator, &fixtures.build_podman_args_sbom)?;
    verify_source_network_and_volume_fixtures(engine, matrix, source, generator, fixtures)
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
    verify_build_core_arguments(version, &generated, output)?;
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

fn verify_build_core_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "application-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for application-build.service is missing its Podman build command\\nstdout:\\n{generated}\\nstderr:\\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let mut positions = Vec::with_capacity(BUILD_IMAGE_TAG_ARGUMENTS.len());
    for argument in BUILD_IMAGE_TAG_ARGUMENTS {
        let matches: Vec<_> = podman_build
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for application-build.service must contain `{argument}` exactly once; found {}\\nstdout:\\n{generated}\\nstderr:\\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    let all_tag_count = podman_build.matches("--tag").count();
    let mut network_positions = Vec::with_capacity(BUILD_NETWORK_ARGUMENTS.len());
    for argument in BUILD_NETWORK_ARGUMENTS {
        let matches: Vec<_> = podman_build
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for application-build.service must contain `{argument}` exactly once; found {}\\nstdout:\\n{generated}\\nstderr:\\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        network_positions.push(matches[0]);
    }
    let all_network_count = podman_build.matches("--network").count();
    for argument in BUILD_LABEL_ARGUMENTS {
        let matches: Vec<_> = podman_build.match_indices(argument).collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for application-build.service must contain `{argument}` exactly once; found {}\\nstdout:\\n{generated}\\nstderr:\\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let all_label_count = podman_build.matches("--label").count();
    let file_count = podman_build.matches("--file").count();
    let target_count = podman_build.matches("--target").count();
    if all_tag_count != BUILD_IMAGE_TAG_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || all_network_count != BUILD_NETWORK_ARGUMENTS.len()
        || !network_positions.windows(2).all(|pair| pair[0] < pair[1])
        || all_label_count != BUILD_LABEL_ARGUMENTS.len()
        || podman_build.contains("--label=")
        || !generated_unit.contains(BUILD_NETWORK_DEPENDENCY)
        || !generated_unit.contains(BUILD_NETWORK_ORDERING)
        || file_count != 1
        || !podman_build.contains(BUILD_FILE_FINAL_ARGUMENT)
        || podman_build.contains(BUILD_FILE_EARLIER_ARGUMENT)
        || target_count != 1
        || !podman_build.contains(BUILD_TARGET_ARGUMENT)
        || !generated_unit.contains(BUILD_WORKING_DIRECTORY)
    {
        return Err(format!(
            "Podman {version} generator output for application-build.service must contain exactly two ordered tags, three ordered networks, exactly two portable labels without --label= forms, `{BUILD_NETWORK_DEPENDENCY}`, `{BUILD_NETWORK_ORDERING}`, only final `{BUILD_FILE_FINAL_ARGUMENT}`, one `{BUILD_TARGET_ARGUMENT}`, and `{BUILD_WORKING_DIRECTORY}`; found tags={all_tag_count}, tag-positions={positions:?}, networks={all_network_count}, network-positions={network_positions:?}, labels={all_label_count}, equals-label={}, dependency={}, ordering={}, files={file_count}, final-file={}, earlier-file={}, targets={target_count}, target={}, working-directory={}\\nstdout:\\n{generated}\\nstderr:\\n{}",
            podman_build.contains("--label="),
            generated_unit.contains(BUILD_NETWORK_DEPENDENCY),
            generated_unit.contains(BUILD_NETWORK_ORDERING),
            podman_build.contains(BUILD_FILE_FINAL_ARGUMENT),
            podman_build.contains(BUILD_FILE_EARLIER_ARGUMENT),
            podman_build.contains(BUILD_TARGET_ARGUMENT),
            generated_unit.contains(BUILD_WORKING_DIRECTORY),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} build core: two ordered --tag arguments, three ordered --network arguments with a .network dependency, two portable --label arguments, final File only, one Target, and file working directory"
    );
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

fn verify_reload_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} reload generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 5, 0) {
        if output.status.success() || generated.contains("ExecReload=") {
            return Err(format!(
                "Podman {version} must reject unsupported ReloadCmd and ReloadSignal without emitting ExecReload; status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} reload: unsupported keys are rejected with no ExecReload");
        return Ok(());
    }

    ensure_success(version, "reload generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} reload generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let command = generated_unit(version, &generated, "reload-command.service", output)?;
    let signal = generated_unit(version, &generated, "reload-signal.service", output)?;
    let command_blank = generated_unit(version, &generated, "reload-command-final-blank.service", output)?;
    let command_malformed = generated_unit(version, &generated, "reload-command-malformed.service", output)?;
    let (command_reload, signal_reload, malformed_reload) = if parsed < PodmanVersion::new(5, 6, 0) {
        (
            "ExecReload=/usr/bin/podman exec --cidfile=%t/%N.cid /usr/bin/reload --final",
            "ExecReload=/usr/bin/podman kill --cidfile=%t/%N.cid --signal SIGUSR1",
            "ExecReload=/usr/bin/podman exec --cidfile=%t/%N.cid /usr/bin/reload unterminated",
        )
    } else {
        (
            "ExecReload=/usr/bin/podman exec systemd-%N /usr/bin/reload --final",
            "ExecReload=/usr/bin/podman kill --signal SIGUSR1 systemd-%N",
            "ExecReload=/usr/bin/podman exec systemd-%N /usr/bin/reload unterminated",
        )
    };
    if !command.contains(command_reload)
        || !signal.contains(signal_reload)
        || command_blank.contains("ExecReload=")
        || !command_malformed.contains(malformed_reload)
    {
        return Err(format!(
            "Podman {version} must retain the recorded ReloadCmd/ReloadSignal ExecReload presentation, omit a final-blank ReloadCmd action, and retain the malformed command's target tokenization; expected-command={command_reload:?}, expected-signal={signal_reload:?}, expected-malformed={malformed_reload:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} reload: expected command and signal ExecReload forms with final-blank omission and malformed command tokenization"
    );
    Ok(())
}

fn verify_reload_conflict_generator_output(version: &str, output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} reload-conflict generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if output.status.success() || generated.contains("ExecReload=") {
        return Err(format!(
            "Podman {version} must reject the ReloadCmd/ReloadSignal conflict without emitting ExecReload; status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            output.status
        ));
    }
    eprintln!("Podman {version} reload: mutually exclusive key pair is rejected");
    Ok(())
}

fn verify_exit_policy_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} ExitPolicy generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 6, 0) {
        if output.status.success() || generated.contains("--exit-policy") {
            return Err(format!(
                "Podman {version} must reject unsupported Pod ExitPolicy without emitting --exit-policy; status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} Pod ExitPolicy: unsupported key is rejected with no --exit-policy argument");
        return Ok(());
    }

    ensure_success(version, "Pod ExitPolicy generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} ExitPolicy generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    for (unit, value) in [
        ("exit-policy-continue-pod.service", "continue"),
        ("exit-policy-stop-pod.service", "stop"),
        ("exit-policy-duplicate-final-stop-pod.service", "stop"),
    ] {
        let unit = generated_unit(version, &generated, unit, output)?;
        let command = unit
            .lines()
            .find(|line| line.starts_with("ExecStartPre=/usr/bin/podman pod create "))
            .ok_or_else(|| format!("Podman {version} generator output for {unit:?} is missing pod create\nstdout:\n{generated}\nstderr:\n{diagnostics}"))?;
        let expected_argument = format!("--exit-policy {value}");
        if !command.contains("--replace")
            || !command.contains(&expected_argument)
            || command.matches("--exit-policy").count() != 1
            || command.find("--replace") > command.find(&expected_argument)
        {
            return Err(format!(
                "Podman {version} {unit:?} must contain exactly one `{expected_argument}` after --replace\ncommand: {command}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let blank = generated_unit(version, &generated, "exit-policy-final-blank-pod.service", output)?;
    let command = blank
        .lines()
        .find(|line| line.starts_with("ExecStartPre=/usr/bin/podman pod create "))
        .ok_or_else(|| format!("Podman {version} generator output for final blank ExitPolicy is missing pod create\nstdout:\n{generated}\nstderr:\n{diagnostics}"))?;
    if command.matches("--exit-policy").count() != 1 || !command.contains("--exit-policy ") {
        return Err(format!(
            "Podman {version} final blank ExitPolicy presentation differs from the recorded one-flag form\ncommand: {command}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Pod ExitPolicy: one post---replace flag for continue, stop, duplicate-final-stop, and final-blank presentations"
    );
    Ok(())
}

fn verify_stop_timeout_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Pod StopTimeout generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 7, 0) {
        if output.status.success() || generated.contains("--time=") {
            return Err(format!(
                "Podman {version} must reject unsupported Pod StopTimeout without emitting --time=; status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} Pod StopTimeout: unsupported key is rejected with no --time= argument");
        return Ok(());
    }

    ensure_success(version, "Pod StopTimeout generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} StopTimeout generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    for (name, value) in [
        ("stop-timeout-normal37-pod.service", "37"),
        ("stop-timeout-zero-pod.service", "0"),
        ("stop-timeout-negative-one-pod.service", "-1"),
        ("stop-timeout-duplicate-final37-pod.service", "37"),
        ("stop-timeout-final-blank-pod.service", ""),
    ] {
        let unit = generated_unit(version, &generated, name, output)?;
        let command = unit
            .lines()
            .find(|line| line.starts_with("ExecStop=/usr/bin/podman pod stop "))
            .ok_or_else(|| format!("Podman {version} generator output for {name} is missing pod stop\nstdout:\n{generated}\nstderr:\n{diagnostics}"))?;
        let expected_argument = format!("--time={value}");
        if command.matches("--time=").count() != 1 || !command.contains(&expected_argument) {
            return Err(format!(
                "Podman {version} {name} must contain exactly one final `{expected_argument}`\ncommand: {command}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    eprintln!(
        "Podman {version} Pod StopTimeout: exact final --time= forms for 37, 0, -1, duplicate-final-37, and final blank"
    );
    Ok(())
}

fn verify_pod_service_name_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Pod ServiceName generator emitted non-UTF-8 output: {error}"))?;
    ensure_success(version, "Pod ServiceName generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Pod ServiceName generator is missing fixture fragment\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let template = if parsed < PodmanVersion::new(5, 7, 0) {
        "---service-name-template@-pod.service---"
    } else {
        "---service-name-template-pod@.service---"
    };
    let unmatched = if parsed < PodmanVersion::new(5, 8, 2) {
        "---unmatched.service---"
    } else {
        "---\"unmatched.service---"
    };
    let headers = [
        "---service-name-default-pod.service---",
        "---chosen-override.service---",
        template,
        unmatched,
        "---.service---",
        "---extension-bearing.service.service---",
    ];
    if headers.iter().any(|header| generated.matches(header).count() != 1)
        || generated.contains("ignored-first.service")
        || generated.contains("---service-name-template@-pod.service---") && parsed >= PodmanVersion::new(5, 7, 0)
        || generated.contains("---service-name-template-pod@.service---") && parsed < PodmanVersion::new(5, 7, 0)
        || generated.contains("---unmatched.service---") && parsed >= PodmanVersion::new(5, 8, 2)
        || generated.contains("---\"unmatched.service---") && parsed < PodmanVersion::new(5, 8, 2)
    {
        return Err(format!(
            "Podman {version} Pod ServiceName must select the last physical value, append .service, retain the recorded ordinary/template and unmatched-quote boundaries, and retain final-blank and extension-bearing presentations\n{generated}"
        ));
    }
    eprintln!(
        "Podman {version} Pod ServiceName: default, duplicate-last, .service, template, unmatched-quote, final-blank, and extension-bearing naming observations"
    );
    Ok(())
}

fn verify_build_retry_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build Retry generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 5, 0) {
        let retry_argument_count = generated.matches("--retry").count();
        let rejected_or_excluded = !output.status.success()
            || !generated.contains("---build-retry-build.service---")
            || diagnostics.contains("Retry");
        if retry_argument_count != 0 || !rejected_or_excluded {
            return Err(format!(
                "Podman {version} predates native Build Retry support and must reject or exclude the fixture without emitting retry arguments; found retry-arguments={retry_argument_count}, status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} Build Retry: unsupported keys are rejected or excluded with no retry argument");
        return Ok(());
    }

    ensure_success(version, "Build Retry generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build Retry generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-retry-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-retry-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let retry_positions: Vec<_> = podman_build
        .match_indices(BUILD_RETRY_PAIR)
        .map(|(position, _)| position)
        .collect();
    let retry_delay_positions: Vec<_> = podman_build
        .match_indices(BUILD_RETRY_DELAY_PAIR)
        .map(|(position, _)| position)
        .collect();
    let retry_flag_count = podman_build.matches("--retry ").count();
    let retry_delay_flag_count = podman_build.matches("--retry-delay ").count();
    let retry_equals_count = podman_build.matches("--retry=").count();
    let retry_delay_equals_count = podman_build.matches("--retry-delay=").count();
    let retry_alternates: Vec<_> = BUILD_RETRY_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    let retry_delay_alternates: Vec<_> = BUILD_RETRY_DELAY_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    let terminal_context = format!(" {BUILD_RETRY_CONTEXT}");
    let context_position = podman_build.rfind(&terminal_context);
    let pairs_precede_context = context_position.is_some_and(|context_position| {
        retry_positions
            .first()
            .is_some_and(|position| *position < context_position)
            && retry_delay_positions
                .first()
                .is_some_and(|position| *position < context_position)
    });
    if retry_positions.len() != 1
        || retry_delay_positions.len() != 1
        || retry_flag_count != 1
        || retry_delay_flag_count != 1
        || retry_equals_count != 0
        || retry_delay_equals_count != 0
        || !retry_alternates.is_empty()
        || !retry_delay_alternates.is_empty()
        || !podman_build.ends_with(&terminal_context)
        || !pairs_precede_context
    {
        return Err(format!(
            "Podman {version} generator output for build-retry-build.service must contain exactly one separate `{BUILD_RETRY_FLAG}` `{BUILD_RETRY_VALUE}` pair and one separate `{BUILD_RETRY_DELAY_FLAG}` `{BUILD_RETRY_DELAY_VALUE}` pair before final positional `{BUILD_RETRY_CONTEXT}`, with no required relative order between pairs and no equals, quoted, alternate, duplicate, or post-context form; found retry-pairs={retry_positions:?}, retry-delay-pairs={retry_delay_positions:?}, retry-flags={retry_flag_count}, retry-delay-flags={retry_delay_flag_count}, retry-equals={retry_equals_count}, retry-delay-equals={retry_delay_equals_count}, retry-alternates={retry_alternates:?}, retry-delay-alternates={retry_delay_alternates:?}, terminal={}, pairs-precede-context={pairs_precede_context}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.ends_with(&terminal_context),
        ));
    }
    eprintln!(
        "Podman {version} Build Retry: one separate --retry 4 pair and one separate --retry-delay 7s pair precede the final positional context without a relative-order requirement"
    );
    Ok(())
}

fn verify_build_tls_verify_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build TLSVerify generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build TLSVerify generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build TLSVerify generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let terminal_context = format!(" {BUILD_TLS_VERIFY_CONTEXT}");
    for (unit, expected_argument, alternate_forms) in [
        (
            "build-tls-verify-true-build.service",
            BUILD_TLS_VERIFY_ARGUMENT,
            BUILD_TLS_VERIFY_TRUE_ALTERNATE_FORMS,
        ),
        (
            "build-tls-verify-false-build.service",
            BUILD_TLS_VERIFY_FALSE_ARGUMENT,
            BUILD_TLS_VERIFY_FALSE_ALTERNATE_FORMS,
        ),
    ] {
        let generated_unit = generated_unit(version, &generated, unit, output)?;
        let podman_build = generated_unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
            .ok_or_else(|| {
                format!(
                    "Podman {version} generator output for {unit} is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
                )
            })?;
        let argument_positions: Vec<_> = podman_build
            .match_indices(expected_argument)
            .map(|(position, _)| position)
            .collect();
        let all_tls_verify_count = podman_build.matches(BUILD_TLS_VERIFY_ARGUMENT).count();
        let alternate_forms: Vec<_> = alternate_forms
            .iter()
            .copied()
            .filter(|form| podman_build.contains(form))
            .collect();
        let context_position = podman_build.rfind(&terminal_context);
        let argument_precedes_context = context_position.is_some_and(|context_position| {
            argument_positions
                .first()
                .is_some_and(|position| *position < context_position)
        });
        if argument_positions.len() != 1
            || all_tls_verify_count != 1
            || !alternate_forms.is_empty()
            || !podman_build.ends_with(&terminal_context)
            || !argument_precedes_context
        {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{expected_argument}` before final positional `{BUILD_TLS_VERIFY_CONTEXT}`, with no bare/equals/quoted/alternate/duplicate or post-context form; found expected={argument_positions:?}, all-tls-verify={all_tls_verify_count}, alternates={alternate_forms:?}, terminal={}, precedes-context={argument_precedes_context}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                podman_build.ends_with(&terminal_context),
            ));
        }
    }
    eprintln!(
        "Podman {version} Build TLSVerify: true emits one bare --tls-verify and false one --tls-verify=false before the final positional context"
    );
    Ok(())
}

fn verify_build_force_rm_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build ForceRM generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build ForceRM generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build ForceRM generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let terminal_context = format!(" {BUILD_FORCE_RM_CONTEXT}");
    for (unit, expected_argument, alternate_forms) in [
        (
            "build-force-rm-true-build.service",
            BUILD_FORCE_RM_ARGUMENT,
            BUILD_FORCE_RM_TRUE_ALTERNATE_FORMS,
        ),
        (
            "build-force-rm-false-build.service",
            BUILD_FORCE_RM_FALSE_ARGUMENT,
            BUILD_FORCE_RM_FALSE_ALTERNATE_FORMS,
        ),
    ] {
        let generated_unit = generated_unit(version, &generated, unit, output)?;
        let podman_build = generated_unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
            .ok_or_else(|| {
                format!(
                    "Podman {version} generator output for {unit} is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
                )
            })?;
        let argument_positions: Vec<_> = podman_build
            .match_indices(expected_argument)
            .map(|(position, _)| position)
            .collect();
        let all_force_rm_count = podman_build.matches(BUILD_FORCE_RM_ARGUMENT).count();
        let alternate_forms: Vec<_> = alternate_forms
            .iter()
            .copied()
            .filter(|form| podman_build.contains(form))
            .collect();
        let context_position = podman_build.rfind(&terminal_context);
        let argument_precedes_context = context_position.is_some_and(|context_position| {
            argument_positions
                .first()
                .is_some_and(|position| *position < context_position)
        });
        if argument_positions.len() != 1
            || all_force_rm_count != 1
            || !alternate_forms.is_empty()
            || !podman_build.ends_with(&terminal_context)
            || !argument_precedes_context
        {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{expected_argument}` before final positional `{BUILD_FORCE_RM_CONTEXT}`, with no bare/equals/quoted/alternate/duplicate or post-context form; found expected={argument_positions:?}, all-force-rm={all_force_rm_count}, alternates={alternate_forms:?}, terminal={}, precedes-context={argument_precedes_context}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                podman_build.ends_with(&terminal_context),
            ));
        }
    }
    eprintln!(
        "Podman {version} Build ForceRM: true emits one bare --force-rm and false one --force-rm=false before the final positional context"
    );
    Ok(())
}

fn verify_build_group_add_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build GroupAdd generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build GroupAdd generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build GroupAdd generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-group-add-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-group-add-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let first_positions: Vec<_> = podman_build
        .match_indices(BUILD_GROUP_ADD_FIRST_PAIR)
        .map(|(position, _)| position)
        .collect();
    let second_positions: Vec<_> = podman_build
        .match_indices(BUILD_GROUP_ADD_SECOND_PAIR)
        .map(|(position, _)| position)
        .collect();
    let flag_count = podman_build.matches(BUILD_GROUP_ADD_FLAG).count();
    let alternate_forms: Vec<_> = BUILD_GROUP_ADD_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    let terminal_context = format!(" {BUILD_GROUP_ADD_CONTEXT}");
    let context_position = podman_build.rfind(&terminal_context);
    let pairs_are_ordered_before_context = matches!(
        (first_positions.first(), second_positions.first(), context_position),
        (Some(first), Some(second), Some(context)) if first < second && second < &context
    );
    if first_positions.len() != 1
        || second_positions.len() != 1
        || flag_count != 2
        || !alternate_forms.is_empty()
        || !podman_build.ends_with(&terminal_context)
        || !pairs_are_ordered_before_context
    {
        return Err(format!(
            "Podman {version} generator output for build-group-add-build.service must contain exactly one ordered separate `{BUILD_GROUP_ADD_FIRST_PAIR}` then `{BUILD_GROUP_ADD_SECOND_PAIR}` pair before final positional `{BUILD_GROUP_ADD_CONTEXT}`, with no equals, quoted, merged, duplicate, reordered, or post-context form; found first={first_positions:?}, second={second_positions:?}, flags={flag_count}, alternates={alternate_forms:?}, terminal={}, ordered-before-context={pairs_are_ordered_before_context}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.ends_with(&terminal_context),
        ));
    }
    eprintln!(
        "Podman {version} Build GroupAdd: ordered separate --group-add 1234 then --group-add 5678 pairs precede the final positional context"
    );
    Ok(())
}

fn verify_build_dns_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build DNS generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build DNS generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build DNS generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-dns-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-dns-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let first_positions: Vec<_> = podman_build
        .match_indices(BUILD_DNS_FIRST_PAIR)
        .map(|(position, _)| position)
        .collect();
    let second_positions: Vec<_> = podman_build
        .match_indices(BUILD_DNS_SECOND_PAIR)
        .map(|(position, _)| position)
        .collect();
    let flag_count = podman_build.matches(BUILD_DNS_FLAG).count();
    let alternate_forms: Vec<_> = BUILD_DNS_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    let terminal_context = format!(" {BUILD_DNS_CONTEXT}");
    let context_position = podman_build.rfind(&terminal_context);
    let pairs_are_ordered_before_context = matches!(
        (first_positions.first(), second_positions.first(), context_position),
        (Some(first), Some(second), Some(context)) if first < second && second < &context
    );
    if first_positions.len() != 1
        || second_positions.len() != 1
        || flag_count != 2
        || !alternate_forms.is_empty()
        || !podman_build.ends_with(&terminal_context)
        || !pairs_are_ordered_before_context
    {
        return Err(format!(
            "Podman {version} generator output for build-dns-build.service must contain exactly one ordered separate `{BUILD_DNS_FIRST_PAIR}` then `{BUILD_DNS_SECOND_PAIR}` pair before final positional `{BUILD_DNS_CONTEXT}`, with no equals, quoted, empty, merged, duplicate, reordered, or post-context form; found first={first_positions:?}, second={second_positions:?}, flags={flag_count}, alternates={alternate_forms:?}, terminal={}, ordered-before-context={pairs_are_ordered_before_context}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.ends_with(&terminal_context),
        ));
    }
    eprintln!(
        "Podman {version} Build DNS: ordered separate --dns 9.9.9.9 then --dns 2001:4860:4860::8888 pairs precede the final positional context"
    );
    Ok(())
}

fn verify_build_dns_option_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build DNSOption generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build DNSOption output is missing `{fragment}`"
            ));
        }
    }
    let unit = generated_unit(version, &generated, "build-dns-option-build.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| format!("Podman {version} Build DNSOption command is missing"))?;
    let first: Vec<_> = command
        .match_indices(BUILD_DNS_OPTION_FIRST)
        .map(|(index, _)| index)
        .collect();
    let second: Vec<_> = command
        .match_indices(BUILD_DNS_OPTION_SECOND)
        .map(|(index, _)| index)
        .collect();
    let terminal = " .";
    let context = command.rfind(terminal);
    let forbidden = [
        "--dns-option rotate",
        "--dns-option=",
        "--dns-option \"",
        "--dns-option '",
        "--dns-option ndots:1,use-vc",
        "--dns-option=ndots:1",
        "--dns-option=use-vc",
        "--dns-option ndots:1 use-vc",
    ];
    let ordered = matches!((first.first(), second.first(), context), (Some(a), Some(b), Some(c)) if a < b && b < &c);
    if first.len() != 1
        || second.len() != 1
        || command.matches(BUILD_DNS_OPTION_FLAG).count() != 2
        || forbidden.iter().any(|form| command.contains(form))
        || !command.ends_with(terminal)
        || !ordered
    {
        return Err(format!(
            "Podman {version} Build DNSOption must retain only ordered separate ndots:1 then use-vc pairs before final context, rejecting reset, empty, equals, quoted, merged, duplicate, reordered, and post-context forms\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build DNSOption: reset leaves ordered --dns-option ndots:1 then --dns-option use-vc before the final context"
    );
    Ok(())
}

fn verify_build_dns_search_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build DNSSearch generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build DNSSearch output is missing `{fragment}`"
            ));
        }
    }
    let unit = generated_unit(version, &generated, "build-dns-search-build.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| format!("Podman {version} Build DNSSearch command is missing"))?;
    let first: Vec<_> = command
        .match_indices(BUILD_DNS_SEARCH_FIRST)
        .map(|(index, _)| index)
        .collect();
    let second: Vec<_> = command
        .match_indices(BUILD_DNS_SEARCH_SECOND)
        .map(|(index, _)| index)
        .collect();
    let terminal = " .";
    let context = command.rfind(terminal);
    let forbidden = [
        "--dns-search old.example",
        "--dns-search=",
        "--dns-search \"\"",
        "--dns-search ''",
        "--dns-search=corp.example",
        "--dns-search=.",
        "--dns-search \"corp.example\"",
        "--dns-search 'corp.example'",
        "--dns-search \".\"",
        "--dns-search '.'",
        "--dns-search corp.example,.",
        "--dns-search=corp.example,.",
        "--dns-search corp.example .",
    ];
    let ordered = matches!((first.first(), second.first(), context), (Some(a), Some(b), Some(c)) if a < b && b < &c);
    if first.len() != 1
        || second.len() != 1
        || command.matches(BUILD_DNS_SEARCH_FLAG).count() != 2
        || forbidden.iter().any(|form| command.contains(form))
        || !command.ends_with(terminal)
        || !ordered
    {
        return Err(format!(
            "Podman {version} Build DNSSearch must retain only ordered separate corp.example then literal-dot pairs before final context, rejecting reset, empty, equals, quoted, merged, duplicate, reordered, and post-context forms\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build DNSSearch: reset leaves ordered --dns-search corp.example then --dns-search . before the final context"
    );
    Ok(())
}

fn verify_build_auth_file_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build AuthFile generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build AuthFile output is missing `{fragment}`"
            ));
        }
    }
    let terminal = " .";
    let cases = [
        (
            "build-auth-file-single-build.service",
            Some(BUILD_AUTH_FILE_SINGLE),
            &[
                "/run/quadlet-lens/last-auth.json",
                "/run/quadlet-lens/earlier-auth.json",
                "/run/quadlet-lens/earlier-empty-auth.json",
            ][..],
        ),
        (
            "build-auth-file-last-build.service",
            Some(BUILD_AUTH_FILE_LAST),
            &[
                "/run/quadlet-lens/single-auth.json",
                "/run/quadlet-lens/earlier-auth.json",
                "/run/quadlet-lens/earlier-empty-auth.json",
            ][..],
        ),
        (
            "build-auth-file-empty-build.service",
            None,
            &[
                "/run/quadlet-lens/single-auth.json",
                "/run/quadlet-lens/earlier-auth.json",
                "/run/quadlet-lens/last-auth.json",
                "/run/quadlet-lens/earlier-empty-auth.json",
            ][..],
        ),
    ];
    for (unit, expected_argument, rejected_paths) in cases {
        let generated_unit = generated_unit(version, &generated, unit, output)?;
        let command = generated_unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
            .ok_or_else(|| format!("Podman {version} Build AuthFile command is missing for {unit}"))?;
        let expected_count = expected_argument.map_or(0, |argument| command.matches(argument).count());
        let flag_count = command.matches(BUILD_AUTH_FILE_FLAG).count();
        let forbidden = [
            "--authfile=",
            "--authfile \"",
            "--authfile '",
            "\"--authfile",
            "'--authfile",
        ];
        let argument_precedes_context = expected_argument.is_none_or(|argument| {
            matches!(
                (command.find(argument), command.rfind(terminal)),
                (Some(argument_position), Some(context_position)) if argument_position < context_position
            )
        });
        if expected_count != usize::from(expected_argument.is_some())
            || flag_count != usize::from(expected_argument.is_some())
            || rejected_paths.iter().any(|path| command.contains(path))
            || forbidden.iter().any(|form| command.contains(form))
            || !command.ends_with(terminal)
            || !argument_precedes_context
        {
            return Err(format!(
                "Podman {version} Build AuthFile command for {unit} must retain only its expected separate pre-context --authfile pair (or no flag after final empty), rejecting equals, quoted, duplicate, alternate, and post-context forms\n{command}"
            ));
        }
    }
    eprintln!(
        "Podman {version} Build AuthFile: single value emits one separate pair, repeated values keep only the effective last path, and final empty emits no flag"
    );
    Ok(())
}

fn verify_build_ignore_file_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 7, 0) {
        let flag_count = generated.matches(BUILD_IGNORE_FILE_FLAG).count();
        let rejected_or_excluded = !output.status.success()
            || !generated.contains("---build-ignore-file-single-build.service---")
            || diagnostics.contains("IgnoreFile");
        if rejected_or_excluded {
            if flag_count != 0 {
                return Err(format!(
                    "Podman {version} rejects or excludes IgnoreFile but emitted --ignorefile; found flags={flag_count}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
                ));
            }
            eprintln!("Podman {version} Build IgnoreFile: rejected or excluded with no --ignorefile argument");
            return Ok(());
        }
    }

    ensure_success(version, "Build IgnoreFile generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build IgnoreFile output is missing `{fragment}`"
            ));
        }
    }
    let terminal = " .";
    let cases = [
        (
            "build-ignore-file-single-build.service",
            Some(BUILD_IGNORE_FILE_SINGLE),
            &[
                "/run/quadlet-lens/last.ignore",
                "/run/quadlet-lens/earlier.ignore",
                "/run/quadlet-lens/earlier-empty.ignore",
            ][..],
        ),
        (
            "build-ignore-file-last-build.service",
            Some(BUILD_IGNORE_FILE_LAST),
            &[
                "/run/quadlet-lens/single.ignore",
                "/run/quadlet-lens/earlier.ignore",
                "/run/quadlet-lens/earlier-empty.ignore",
            ][..],
        ),
        (
            "build-ignore-file-empty-build.service",
            None,
            &[
                "/run/quadlet-lens/single.ignore",
                "/run/quadlet-lens/earlier.ignore",
                "/run/quadlet-lens/last.ignore",
                "/run/quadlet-lens/earlier-empty.ignore",
            ][..],
        ),
    ];
    for (unit, expected_argument, rejected_paths) in cases {
        let generated_unit = generated_unit(version, &generated, unit, output)?;
        let command = generated_unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
            .ok_or_else(|| format!("Podman {version} Build IgnoreFile command is missing for {unit}"))?;
        let expected_count = expected_argument.map_or(0, |argument| command.matches(argument).count());
        let flag_count = command.matches(BUILD_IGNORE_FILE_FLAG).count();
        let forbidden = [
            "--ignorefile=",
            "--ignorefile \"",
            "--ignorefile '",
            "\"--ignorefile",
            "'--ignorefile",
        ];
        let argument_precedes_context = expected_argument.is_none_or(|argument| {
            matches!(
                (command.find(argument), command.rfind(terminal)),
                (Some(argument_position), Some(context_position)) if argument_position < context_position
            )
        });
        if expected_count != usize::from(expected_argument.is_some())
            || flag_count != usize::from(expected_argument.is_some())
            || rejected_paths.iter().any(|path| command.contains(path))
            || forbidden.iter().any(|form| command.contains(form))
            || !command.ends_with(terminal)
            || !argument_precedes_context
        {
            return Err(format!(
                "Podman {version} Build IgnoreFile command for {unit} must retain only its expected separate pre-context --ignorefile pair (or no flag after final empty), rejecting equals, quoted, duplicate, alternate, and post-context forms\n{command}"
            ));
        }
    }
    eprintln!(
        "Podman {version} Build IgnoreFile: single value emits one separate pair, repeated values keep only the effective last path, and final empty emits no flag"
    );
    Ok(())
}

fn verify_build_annotation_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build Annotation generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build Annotation output is missing `{fragment}`"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-annotation-build.service", output)?;
    let command = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| format!("Podman {version} Build Annotation command is missing"))?;
    let encoded_space = if parsed < PodmanVersion::new(5, 5, 0) {
        " "
    } else {
        r"\x20"
    };
    let mut expected_arguments = vec!["--annotation =".to_owned()];
    if parsed >= PodmanVersion::new(5, 6, 0) {
        expected_arguments.extend(["--annotation bare".to_owned(), "--annotation malformed".to_owned()]);
    }
    expected_arguments.extend([
        "--annotation org.example.alpha=final".to_owned(),
        format!(r#"--annotation "org.example.escape=literal{encoded_space}text""#),
        format!(r#"--annotation "org.example.quoted=Authored{encoded_space}Value""#),
        "--annotation org.example.zeta=first".to_owned(),
    ]);
    if parsed >= PodmanVersion::new(5, 6, 0) {
        expected_arguments.push("--annotation value".to_owned());
    }
    let positions: Vec<_> = expected_arguments
        .iter()
        .map(|argument| {
            let matches: Vec<_> = command.match_indices(argument).map(|(position, _)| position).collect();
            (argument, matches)
        })
        .collect();
    if positions.iter().any(|(_, matches)| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0].1[0] < pair[1].1[0])
        || command.matches("--annotation").count() != expected_arguments.len()
        || BUILD_ANNOTATION_PRE_RESET_OR_REPLACED
            .iter()
            .any(|value| command.contains(value))
        || (parsed < PodmanVersion::new(5, 6, 0)
            && ["--annotation bare", "--annotation malformed", "--annotation value"]
                .iter()
                .any(|argument| command.contains(argument)))
        || !command.ends_with(" .")
    {
        return Err(format!(
            "Podman {version} Build Annotation must emit only the target-reset, tokenized/unquoted/C-unescaped, last-key-collapsed, key-sorted separate --annotation arguments before the final context, with the recorded bare/malformed-token boundary; found {positions:?}\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build Annotation: reset, tokenization/unquoting/C-unescaping, duplicate-key collapse, and sorting preserve the recorded bare/malformed-token boundary"
    );
    Ok(())
}

fn verify_build_environment_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build Environment generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Build Environment is missing fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "build-environment-build.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or("missing build command")?;
    let encoded = if parsed < PodmanVersion::new(5, 5, 0) {
        " "
    } else {
        r"\x20"
    };
    let mut arguments = vec![
        "--env =".to_owned(),
        format!(r#"--env "ESCAPED=literal{encoded}text""#),
        "--env NAME=final".to_owned(),
        format!(r#"--env "QUOTED=Authored{encoded}Value""#),
    ];
    if parsed >= PodmanVersion::new(5, 6, 0) {
        arguments.push("--env bare".to_owned());
    }
    arguments.push("--env embedded=a=b".to_owned());
    if parsed >= PodmanVersion::new(5, 6, 0) {
        arguments.extend(["--env malformed".to_owned(), "--env value".to_owned()]);
    }
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || command.matches("--env").count() != arguments.len()
        || command.contains("PRE=one")
        || command.contains("NAME=first")
        || !command.ends_with(" .")
    {
        return Err(format!(
            "Podman {version} Build Environment must retain only its effective target map\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build Environment: reset, tokenization, final-name selection, and separate --env output"
    );
    Ok(())
}

fn verify_build_containers_conf_module_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build ContainersConfModule generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Build ContainersConfModule is missing fixture fragment"
        ));
    }
    let unit = generated_unit(
        version,
        &generated,
        "build-containers-conf-module-build.service",
        output,
    )?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing build command")?;
    let arguments = ["--module=post-one", "--module=post-two"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let build = command.find(" build ").ok_or("missing build subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || positions[0][0] >= positions[1][0]
        || positions.iter().any(|matches| matches[0] >= build)
        || command.matches("--module=").count() != arguments.len()
        || command.contains("pre-one")
        || command.contains("pre-two")
        || !command.ends_with(" .")
    {
        return Err(format!(
            "Podman {version} Build ContainersConfModule must retain only ordered post-reset --module arguments before build and context\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build ContainersConfModule: logical reset and ordered separate --module output before build"
    );
    Ok(())
}

fn verify_build_global_args_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build GlobalArgs generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Build GlobalArgs is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "build-global-args-build.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing build command")?;
    let arguments = ["--log-level=debug", "--events-backend=none", "--events-backend=file"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let podman = command.find("/usr/bin/podman ").ok_or("missing podman command")?;
    let build = command.find(" build ").ok_or("missing build subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || positions
            .iter()
            .any(|matches| matches[0] <= podman || matches[0] >= build)
        || command.contains("--log-level=info")
        || command.contains("GlobalArgs")
        || !command.ends_with(" .")
    {
        return Err(format!(
            "Podman {version} Build GlobalArgs must retain only ordered post-reset target tokens between podman and build\\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Build GlobalArgs: reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered pre-build tokens"
    );
    Ok(())
}

fn verify_build_service_name_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build ServiceName generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Build ServiceName is missing fixture fragment"
        ));
    }
    let template = if parsed < PodmanVersion::new(5, 7, 0) {
        "---service-name-template@-build.service---"
    } else {
        "---service-name-template-build@.service---"
    };
    let unmatched = if parsed < PodmanVersion::new(5, 8, 2) {
        "---unmatched.service---"
    } else {
        "---\"unmatched.service---"
    };
    let headers = [
        "---service-name-default-build.service---",
        "---chosen-override.service---",
        template,
        unmatched,
    ];
    if headers.iter().any(|header| generated.matches(header).count() != 1)
        || generated.contains("ignored-first.service")
        || generated.contains("---service-name-template@-build.service---") && parsed >= PodmanVersion::new(5, 7, 0)
        || generated.contains("---service-name-template-build@.service---") && parsed < PodmanVersion::new(5, 7, 0)
        || generated.contains("---unmatched.service---") && parsed >= PodmanVersion::new(5, 8, 2)
        || generated.contains("---\"unmatched.service---") && parsed < PodmanVersion::new(5, 8, 2)
    {
        return Err(format!(
            "Podman {version} Build ServiceName must select the last physical value, append .service, retain the recorded ordinary/template default boundary, and retain the recorded unmatched-quote lookup boundary\n{generated}"
        ));
    }
    eprintln!(
        "Podman {version} Build ServiceName: duplicate-last selection, .service addition, ordinary/template defaults, and unmatched-quote boundary"
    );
    Ok(())
}

fn verify_build_volume_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Build Volume generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Build Volume is missing fixture fragment"));
    }
    let reset = generated_unit(version, &generated, "build-volume-reset-build.service", output)?;
    let relative = generated_unit(version, &generated, "build-volume-relative-build.service", output)?;
    let native = generated_unit(version, &generated, "build-volume-native-build.service", output)?;
    let reset_command = reset
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or("missing reset build command")?;
    let relative_command = relative
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or("missing relative build command")?;
    let native_command = native
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or("missing native build command")?;
    let continuation_space = if parsed < PodmanVersion::new(5, 5, 0) {
        " "
    } else {
        r"\x20"
    };
    let reset_arguments = [
        "-v post-one:/post-one",
        "-v post-two:/post-two",
        &format!(r#"-v "continued-one:/continued-one{continuation_space}continued-two:/continued-two""#),
    ];
    let positions: Vec<_> = reset_arguments
        .iter()
        .map(|argument| reset_command.find(argument))
        .collect();
    if positions.iter().any(Option::is_none)
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || reset_command.contains("pre-one")
        || reset_command.contains("pre-two")
        || !reset_command.ends_with(" .")
        || !relative.contains("RequiresMountsFor=/fixtures")
        || !relative_command.contains("-v /fixtures:/workspace")
        || !relative_command.ends_with(" .")
        || !native.contains("Requires=cache-volume.service")
        || !native.contains("After=cache-volume.service")
        || !native_command.contains("-v quadlet-lens-cache:/var/cache:Z")
        || !native_command.ends_with(" .")
    {
        return Err(format!(
            "Podman {version} Build Volume must retain reset/continuation -v order, resolve relative ., and substitute/depend on an exact .volume source\nreset:\n{reset_command}\nrelative:\n{relative_command}\nnative:\n{native_command}"
        ));
    }
    eprintln!(
        "Podman {version} Build Volume: reset/continuation -v list, relative source resolution, and .volume substitution/dependency"
    );
    Ok(())
}

fn verify_volume_containers_conf_module_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume ContainersConfModule generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Volume ContainersConfModule is missing fixture fragment"
        ));
    }
    let unit = generated_unit(
        version,
        &generated,
        "volume-containers-conf-module-volume.service",
        output,
    )?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing volume create command")?;
    let continuation_space = if parsed < PodmanVersion::new(5, 5, 0) {
        " "
    } else {
        r"\x20"
    };
    let arguments = [
        "--module=post-one".to_owned(),
        "--module=post-two".to_owned(),
        format!(r#""--module=continued-one{continuation_space}continued-two""#),
    ];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let volume_create = command
        .find(" volume create ")
        .ok_or("missing volume create subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || positions.iter().any(|matches| matches[0] >= volume_create)
        || command.matches("--module=").count() != arguments.len()
        || command.contains("pre-one")
        || command.contains("pre-two")
        || !command.ends_with(" volume create --ignore quadlet-lens-volume-module")
    {
        return Err(format!(
            "Podman {version} Volume ContainersConfModule must retain only ordered post-reset --module arguments before volume create\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Volume ContainersConfModule: logical reset, continuation presentation, and ordered --module output before volume create"
    );
    Ok(())
}

fn verify_volume_global_args_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume GlobalArgs generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Volume GlobalArgs is missing fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "volume-global-args-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing volume create command")?;
    let arguments = ["--log-level=debug", "--events-backend=none", "--events-backend=file"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let podman = command.find("/usr/bin/podman ").ok_or("missing podman command")?;
    let volume_create = command
        .find(" volume create ")
        .ok_or("missing volume create subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || positions
            .iter()
            .any(|matches| matches[0] <= podman || matches[0] >= volume_create)
        || command.contains("--log-level=info")
        || command.contains("--events-backend=journald")
        || command.contains("GlobalArgs")
        || !command.ends_with(" volume create --ignore quadlet-lens-volume-global-args")
    {
        return Err(format!(
            "Podman {version} Volume GlobalArgs must retain only ordered post-reset target tokens between podman and volume create\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Volume GlobalArgs: reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered pre-volume-create tokens"
    );
    Ok(())
}

fn verify_volume_podman_args_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume PodmanArgs generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Volume PodmanArgs is missing fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "volume-podman-args-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or("missing volume create command")?;
    let arguments = ["--label=post-one", "--label=quoted", "--label=escaped"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let volume_name = command
        .find(" quadlet-lens-volume-podman-args")
        .ok_or("missing volume name")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || positions.iter().any(|matches| matches[0] >= volume_name)
        || command.matches("--label=").count() != arguments.len()
        || command.contains("pre-one")
        || command.contains("pre-two")
        || command.contains("PodmanArgs")
        || !command.ends_with(" quadlet-lens-volume-podman-args")
    {
        return Err(format!(
            "Podman {version} Volume PodmanArgs must retain only ordered post-reset target tokens at the end of volume create before the volume name\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Volume PodmanArgs: reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered terminal volume-create tokens"
    );
    Ok(())
}

fn verify_volume_user_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume User generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Volume User is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "volume-user-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or("missing volume create command")?;
    let uid = command.find("o=uid=123").ok_or("missing o=uid=123 option")?;
    let volume_name = command.find(" quadlet-lens-volume-user").ok_or("missing volume name")?;
    if command.matches("o=uid=123").count() != 1
        || uid >= volume_name
        || !command.ends_with(" quadlet-lens-volume-user")
    {
        return Err(format!(
            "Podman {version} Volume User must emit exactly one o=uid=123 before the volume name\n{command}"
        ));
    }
    eprintln!("Podman {version} Volume User: unambiguous numeric UID emits o=uid=123 before the volume name");
    Ok(())
}

fn verify_volume_group_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume Group generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Volume Group is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "volume-group-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or("missing volume create command")?;
    let gid = command.find("o=gid=456").ok_or("missing o=gid=456 option")?;
    let volume_name = command
        .find(" quadlet-lens-volume-group")
        .ok_or("missing volume name")?;
    if command.matches("o=gid=456").count() != 1
        || gid >= volume_name
        || !command.ends_with(" quadlet-lens-volume-group")
    {
        return Err(format!(
            "Podman {version} Volume Group must emit exactly one o=gid=456 before the volume name\n{command}"
        ));
    }
    eprintln!("Podman {version} Volume Group: unambiguous numeric GID emits o=gid=456 before the volume name");
    Ok(())
}

fn verify_volume_uid_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    if parsed < PodmanVersion::new(6, 0, 0) {
        let diagnostics = String::from_utf8_lossy(&output.stderr);
        if output.status.success()
            || generated.contains("--uid")
            || generated.contains("1234")
            || !diagnostics.contains("unsupported key 'UID'")
        {
            return Err(format!(
                "Podman {version} must reject unsupported Volume UID\n{diagnostics}"
            ));
        }
        return Ok(());
    }
    ensure_success(version, "Volume UID generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Volume UID is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "volume-uid-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or("missing volume create command")?;
    let uid = command.find("--uid 1234").ok_or("missing --uid 1234")?;
    let volume_name = command.find(" quadlet-lens-volume-uid").ok_or("missing volume name")?;
    if command.matches("--uid 1234").count() != 1
        || uid >= volume_name
        || !command.ends_with(" quadlet-lens-volume-uid")
    {
        return Err(format!(
            "Podman {version} Volume UID must emit exactly one --uid 1234 before volume name\n{command}"
        ));
    }
    Ok(())
}

fn verify_volume_gid_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    if parsed < PodmanVersion::new(6, 0, 0) {
        let diagnostics = String::from_utf8_lossy(&output.stderr);
        if output.status.success()
            || generated.contains("--gid")
            || generated.contains("5678")
            || !diagnostics.contains("unsupported key 'GID'")
        {
            return Err(format!(
                "Podman {version} must reject unsupported Volume GID\n{diagnostics}"
            ));
        }
        return Ok(());
    }
    ensure_success(version, "Volume GID generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Volume GID is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "volume-gid-volume.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman volume create "))
        .ok_or("missing volume create command")?;
    let gid = command.find("--gid 5678").ok_or("missing --gid 5678")?;
    let volume_name = command.find(" quadlet-lens-volume-gid").ok_or("missing volume name")?;
    if command.matches("--gid 5678").count() != 1
        || gid >= volume_name
        || !command.ends_with(" quadlet-lens-volume-gid")
    {
        return Err(format!(
            "Podman {version} Volume GID must emit exactly one --gid 5678 before volume name\n{command}"
        ));
    }
    Ok(())
}

fn verify_volume_service_name_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Volume ServiceName generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Volume ServiceName is missing fixture fragment"
        ));
    }
    let template = if parsed < PodmanVersion::new(5, 7, 0) {
        "---service-name-template@-volume.service---"
    } else {
        "---service-name-template-volume@.service---"
    };
    let unmatched = if parsed < PodmanVersion::new(5, 8, 2) {
        "---unmatched.service---"
    } else {
        "---\"unmatched.service---"
    };
    let headers = [
        "---service-name-default-volume.service---",
        "---chosen-override.service---",
        template,
        unmatched,
    ];
    if headers.iter().any(|header| generated.matches(header).count() != 1)
        || generated.contains("ignored-first.service")
        || generated.contains("---service-name-template@-volume.service---") && parsed >= PodmanVersion::new(5, 7, 0)
        || generated.contains("---service-name-template-volume@.service---") && parsed < PodmanVersion::new(5, 7, 0)
        || generated.contains("---unmatched.service---") && parsed >= PodmanVersion::new(5, 8, 2)
        || generated.contains("---\"unmatched.service---") && parsed < PodmanVersion::new(5, 8, 2)
    {
        return Err(format!(
            "Podman {version} Volume ServiceName must select the last physical value, append .service, retain the recorded ordinary/template default boundary, and retain the recorded unmatched-quote lookup boundary\n{generated}"
        ));
    }
    eprintln!(
        "Podman {version} Volume ServiceName: duplicate-last selection, .service addition, ordinary/template defaults, and unmatched-quote boundary"
    );
    Ok(())
}

fn verify_volume_image_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Volume Image is missing fixture fragment"));
    }
    if output.status.success() {
        return Err(format!(
            "Podman {version} Volume Image fixture must report the missing Image error"
        ));
    }
    let literal = generated_unit(version, &generated, "image-literal-volume.service", output)?;
    let ignored = generated_unit(version, &generated, "image-ignored-volume.service", output)?;
    for unit in [literal, ignored] {
        if !unit.contains("ExecStart=/usr/bin/podman volume create ") {
            return Err(format!(
                "Podman {version} Volume Image fixture lacks volume create command\n{unit}"
            ));
        }
    }
    if !literal.contains("--driver image") || !literal.contains("image=example.invalid/quadlet-lens-volume:literal") {
        return Err(format!(
            "Podman {version} Volume Image literal driver form missing\n{literal}"
        ));
    }
    if ignored.contains("image=example.invalid/quadlet-lens-volume:ignored") {
        return Err(format!(
            "Podman {version} non-image Volume driver must ignore Image\n{ignored}"
        ));
    }
    for source in ["volume-source.image", "volume-source.build"] {
        if !generated.contains(source) || !generated.contains("Requires=") || !generated.contains("After=") {
            return Err(format!(
                "Podman {version} Volume Image must retain generated dependency for {source}\n{generated}"
            ));
        }
    }
    eprintln!(
        "Podman {version} Volume Image: literal, missing, ignored-driver, and exact image/build-reference observations"
    );
    Ok(())
}

fn verify_image_core_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image core is missing fixture fragment"));
    }
    if output.status.success() {
        return Err(format!(
            "Podman {version} Image core fixture must report missing and empty Image errors"
        ));
    }
    let unit = generated_unit(version, &generated, "image-core-image.service", output)?;
    let command = "ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-image:final";
    if unit.matches(command).count() != 1 {
        return Err(format!(
            "Podman {version} Image core must generate one pull command from the final duplicate Image value\n{unit}"
        ));
    }
    eprintln!("Podman {version} Image core: literal pull, missing/empty errors, and duplicate-last target behavior");
    Ok(())
}

fn verify_image_image_tag_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} ImageTag is missing fixture fragment"));
    }
    ensure_success(version, "ImageTag generator", output)?;

    for (name, source, resource_name) in [
        (
            "image-tag-normal",
            "example.invalid/quadlet-lens-normal:source",
            "example.invalid/quadlet-lens-normal:final",
        ),
        (
            "image-tag-archive",
            "docker-archive:/tmp/quadlet-lens-image.tar",
            "example.invalid/quadlet-lens-archive:final",
        ),
        (
            "image-tag-default",
            "example.invalid/quadlet-lens-default:source",
            "example.invalid/quadlet-lens-default:source",
        ),
    ] {
        let image_unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull {source}");
        if image_unit.matches(&pull).count() != 1 {
            return Err(format!(
                "Podman {version} ImageTag must retain one Image source command for {name}\n{image_unit}"
            ));
        }
        let container_unit = generated_unit(version, &generated, &format!("{name}.service"), output)?;
        for dependency in [
            format!("Requires={name}-image.service"),
            format!("After={name}-image.service"),
        ] {
            if !container_unit.contains(&dependency) {
                return Err(format!(
                    "Podman {version} ImageTag must retain generated dependency `{dependency}`\n{container_unit}"
                ));
            }
        }
        if !container_unit.contains(&format!(" -d {resource_name}")) {
            return Err(format!(
                "Podman {version} ImageTag must substitute `{resource_name}` for {name}\n{container_unit}"
            ));
        }
    }

    let quoted = generated_unit(version, &generated, "image-tag-quoted.service", output)?;
    let unmatched = generated_unit(version, &generated, "image-tag-unmatched.service", output)?;
    let quoted_resource = if parsed <= PodmanVersion::new(5, 4, 2) {
        " -d \"example.invalid/quadlet lens:final\""
    } else {
        " -d \"example.invalid/quadlet\\x20lens:final\""
    };
    let unmatched_resource = if parsed < PodmanVersion::new(5, 8, 2) {
        " -d example.invalid/quadlet-lens-unmatched:final"
    } else {
        " -d \"example.invalid/quadlet-lens-unmatched:final\\\"\""
    };
    if !quoted.contains(quoted_resource) || !unmatched.contains(unmatched_resource) {
        return Err(format!(
            "Podman {version} ImageTag quote presentation did not match its release boundary\nquoted:\n{quoted}\nunmatched:\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} ImageTag: source retention, duplicate-last/default resource names, dependencies, and quote presentation"
    );
    Ok(())
}

fn verify_image_service_name_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image ServiceName generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Image ServiceName is missing fixture fragment"
        ));
    }
    let template = if parsed < PodmanVersion::new(5, 7, 0) {
        "---service-name-template@-image.service---"
    } else {
        "---service-name-template-image@.service---"
    };
    let unmatched = if parsed < PodmanVersion::new(5, 8, 2) {
        "---unmatched.service---"
    } else {
        "---\"unmatched.service---"
    };
    let headers = [
        "---service-name-default-image.service---",
        "---chosen-override.service---",
        template,
        unmatched,
    ];
    if headers.iter().any(|header| generated.matches(header).count() != 1)
        || generated.contains("ignored-first.service")
        || generated.contains("---service-name-template@-image.service---") && parsed >= PodmanVersion::new(5, 7, 0)
        || generated.contains("---service-name-template-image@.service---") && parsed < PodmanVersion::new(5, 7, 0)
        || generated.contains("---unmatched.service---") && parsed >= PodmanVersion::new(5, 8, 2)
        || generated.contains("---\"unmatched.service---") && parsed < PodmanVersion::new(5, 8, 2)
    {
        return Err(format!(
            "Podman {version} Image ServiceName must select the last physical value, append .service, retain the recorded ordinary/template default boundary, and retain the recorded unmatched-quote lookup boundary\n{generated}"
        ));
    }
    eprintln!(
        "Podman {version} Image ServiceName: duplicate-last selection, .service addition, ordinary/template defaults, and unmatched-quote boundary"
    );
    Ok(())
}

fn verify_image_all_tags_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image AllTags generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image AllTags is missing fixture fragment"));
    }
    for (name, all_tags) in [
        ("all-tags-absent", None),
        ("all-tags-true", Some("--all-tags")),
        ("all-tags-false", Some("--all-tags=false")),
        ("all-tags-duplicate", Some("--all-tags=false")),
        ("all-tags-blank", Some("--all-tags=false")),
    ] {
        let unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-{name}:latest");
        let expected_pull = all_tags.map_or(pull, |all_tags| {
            format!("ExecStart=/usr/bin/podman image pull {all_tags} example.invalid/quadlet-lens-{name}:latest")
        });
        if unit.matches(&expected_pull).count() != 1 || unit.contains(" --all-tags") != all_tags.is_some() {
            return Err(format!(
                "Podman {version} Image AllTags must retain the target {name} command-text observation\\n{unit}"
            ));
        }
    }
    let unmatched = generated_unit(version, &generated, "all-tags-unmatched-image.service", output)?;
    let unmatched_all_tags = if parsed < PodmanVersion::new(5, 8, 2) {
        "--all-tags"
    } else {
        "--all-tags=false"
    };
    let unmatched_pull = format!(
        "ExecStart=/usr/bin/podman image pull {unmatched_all_tags} example.invalid/quadlet-lens-all-tags-unmatched:latest"
    );
    if unmatched.matches(&unmatched_pull).count() != 1 {
        return Err(format!(
            "Podman {version} Image AllTags unmatched-quote command text did not match the recorded 5.8.2 boundary\\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} Image AllTags: true/false, duplicate-last, absent/blank, and unmatched-quote command text"
    );
    Ok(())
}

fn verify_image_arch_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image Arch generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image Arch is missing fixture fragment"));
    }
    for (name, arch) in [
        ("image-arch-normal", Some("arm64")),
        ("image-arch-duplicate", Some("amd64")),
        ("image-arch-blank", None),
    ] {
        let unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-{name}:latest");
        let expected_pull = arch.map_or(pull, |arch| {
            format!("ExecStart=/usr/bin/podman image pull --arch {arch} example.invalid/quadlet-lens-{name}:latest")
        });
        if unit.matches(&expected_pull).count() != 1 || unit.contains(" --arch") != arch.is_some() {
            return Err(format!(
                "Podman {version} Image Arch must retain the target {name} command-text observation\\n{unit}"
            ));
        }
    }
    let unmatched = generated_unit(version, &generated, "image-arch-unmatched-image.service", output)?;
    let unmatched_arch = if parsed < PodmanVersion::new(5, 8, 2) {
        "amd64"
    } else {
        "\"amd64\\\"\""
    };
    let unmatched_pull = format!(
        "ExecStart=/usr/bin/podman image pull --arch {unmatched_arch} example.invalid/quadlet-lens-image-arch-unmatched:latest"
    );
    if unmatched.matches(&unmatched_pull).count() != 1 {
        return Err(format!(
            "Podman {version} Image Arch unmatched-quote command text did not match the recorded 5.8.2 boundary\\n{unmatched}"
        ));
    }
    eprintln!("Podman {version} Image Arch: normal, duplicate-last, blank omission, and unmatched-quote command text");
    Ok(())
}

fn verify_image_auth_file_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image AuthFile generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image AuthFile is missing fixture fragment"));
    }
    for (name, auth_file) in [
        (
            "image-auth-file-normal",
            Some("/placeholder/quadlet-lens-image-auth-file-normal.json"),
        ),
        (
            "image-auth-file-duplicate",
            Some("/placeholder/quadlet-lens-image-auth-file-last.json"),
        ),
        ("image-auth-file-blank", None),
    ] {
        let unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-{name}:latest");
        let expected_pull = auth_file.map_or(pull, |auth_file| {
            format!(
                "ExecStart=/usr/bin/podman image pull --authfile {auth_file} example.invalid/quadlet-lens-{name}:latest"
            )
        });
        if unit.matches(&expected_pull).count() != 1 || unit.contains(" --authfile") != auth_file.is_some() {
            return Err(format!(
                "Podman {version} Image AuthFile must retain the target {name} command-text observation\\n{unit}"
            ));
        }
    }
    let unmatched = generated_unit(version, &generated, "image-auth-file-unmatched-image.service", output)?;
    let unmatched_auth_file = if parsed < PodmanVersion::new(5, 8, 2) {
        "/placeholder/quadlet-lens-image-auth-file-unmatched.json"
    } else {
        "\"/placeholder/quadlet-lens-image-auth-file-unmatched.json\\\"\""
    };
    let unmatched_pull = format!(
        "ExecStart=/usr/bin/podman image pull --authfile {unmatched_auth_file} example.invalid/quadlet-lens-image-auth-file-unmatched:latest"
    );
    if unmatched.matches(&unmatched_pull).count() != 1 {
        return Err(format!(
            "Podman {version} Image AuthFile unmatched-quote command text did not match the recorded 5.8.2 boundary\\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} Image AuthFile: normal, duplicate-last, blank omission, and unmatched-quote command text"
    );
    Ok(())
}

fn verify_image_creds_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image Creds generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image Creds is missing fixture fragment"));
    }
    let normal = generated_unit(version, &generated, "image-creds-normal-image.service", output)?;
    let duplicate = generated_unit(version, &generated, "image-creds-duplicate-image.service", output)?;
    let blank = generated_unit(version, &generated, "image-creds-blank-image.service", output)?;
    let unmatched = generated_unit(version, &generated, "image-creds-unmatched-image.service", output)?;
    let command = |unit: &str| {
        unit.lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman image pull "))
            .map(str::to_owned)
            .ok_or("missing image pull command")
    };
    let normal_command = command(normal)?;
    let duplicate_command = command(duplicate)?;
    let blank_command = command(blank)?;
    let unmatched_command = command(unmatched)?;
    let cred_flags = |command: &str| command.matches(" --creds ").count();
    if cred_flags(&normal_command) != 1
        || cred_flags(&duplicate_command) != 1
        || cred_flags(&blank_command) != 0
        || cred_flags(&unmatched_command) != 1
        || !normal_command.contains("quadlet-lens-placeholder-normal-user:quadlet-lens-placeholder-normal-password")
        || !duplicate_command.contains("quadlet-lens-placeholder-last-user:quadlet-lens-placeholder-last-password")
        || duplicate_command.contains("quadlet-lens-placeholder-first-user:quadlet-lens-placeholder-first-password")
        || blank_command.contains("quadlet-lens-placeholder")
    {
        return Err(format!(
            "Podman {version} Image Creds must retain normal, duplicate-last, final-blank omission, and unmatched-quote observations without imposing flag ordering or a version boundary\nnormal:\n{normal}\nduplicate:\n{duplicate}\nblank:\n{blank}\nunmatched:\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} Image Creds: normal, duplicate-last, final-blank omission, and unmatched-quote command text (no flag-order or quote-boundary claim)"
    );
    Ok(())
}

fn verify_image_decryption_key_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image DecryptionKey generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Image DecryptionKey is missing fixture fragment"
        ));
    }
    let normal = generated_unit(version, &generated, "image-decryption-key-normal-image.service", output)?;
    let duplicate = generated_unit(
        version,
        &generated,
        "image-decryption-key-duplicate-image.service",
        output,
    )?;
    let blank = generated_unit(version, &generated, "image-decryption-key-blank-image.service", output)?;
    let unmatched = generated_unit(
        version,
        &generated,
        "image-decryption-key-unmatched-image.service",
        output,
    )?;
    let command = |unit: &str| {
        unit.lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman image pull "))
            .map(str::to_owned)
            .ok_or("missing image pull command")
    };
    let normal_command = command(normal)?;
    let duplicate_command = command(duplicate)?;
    let blank_command = command(blank)?;
    let unmatched_command = command(unmatched)?;
    let decryption_key_flags = |command: &str| command.matches(" --decryption-key ").count();
    if decryption_key_flags(&normal_command) != 1
        || decryption_key_flags(&duplicate_command) != 1
        || decryption_key_flags(&blank_command) != 0
        || decryption_key_flags(&unmatched_command) != 1
        || !normal_command.contains("quadlet-lens-decryption-key-placeholder-normal")
        || !duplicate_command.contains("quadlet-lens-decryption-key-placeholder-last")
        || duplicate_command.contains("quadlet-lens-decryption-key-placeholder-first")
        || blank_command.contains("quadlet-lens-decryption-key-placeholder")
    {
        return Err(format!(
            "Podman {version} Image DecryptionKey must retain normal, duplicate-last, final-blank omission, and unmatched-quote observations without imposing flag ordering or a version boundary\\nnormal:\\n{normal}\\nduplicate:\\n{duplicate}\\nblank:\\n{blank}\\nunmatched:\\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} Image DecryptionKey: normal, duplicate-last, final-blank omission, and unmatched-quote command text (no flag-order or quote-boundary claim)"
    );
    Ok(())
}

fn verify_image_global_args_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image GlobalArgs generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image GlobalArgs is missing fixture fragment"));
    }
    let unit = generated_unit(version, &generated, "image-global-args-image.service", output)?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing image pull command")?;
    let arguments = ["--log-level=debug", "--events-backend=none", "--events-backend=file"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let podman = command.find("/usr/bin/podman ").ok_or("missing podman command")?;
    let image_pull = command.find(" image pull ").ok_or("missing image pull subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || !positions.windows(2).all(|pair| pair[0][0] < pair[1][0])
        || positions
            .iter()
            .any(|matches| matches[0] <= podman || matches[0] >= image_pull)
        || command.contains("--log-level=info")
        || command.contains("GlobalArgs")
    {
        return Err(format!(
            "Podman {version} Image GlobalArgs must retain only ordered post-reset target tokens between podman and image pull\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Image GlobalArgs: reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered pre-pull tokens"
    );
    Ok(())
}

fn verify_image_os_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image OS generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image OS is missing fixture fragment"));
    }
    for (name, os) in [
        ("image-os-normal", Some("windows")),
        ("image-os-duplicate", Some("linux")),
        ("image-os-blank", None),
    ] {
        let unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-{name}:latest");
        let expected_pull = os.map_or(pull, |os| {
            format!("ExecStart=/usr/bin/podman image pull --os {os} example.invalid/quadlet-lens-{name}:latest")
        });
        if unit.matches(&expected_pull).count() != 1 || unit.contains(" --os") != os.is_some() {
            return Err(format!(
                "Podman {version} Image OS must retain the target {name} command-text observation\n{unit}"
            ));
        }
    }
    let unmatched = generated_unit(version, &generated, "image-os-unmatched-image.service", output)?;
    let unmatched_os = if parsed < PodmanVersion::new(5, 8, 2) {
        "linux"
    } else {
        "\"linux\\\"\""
    };
    let unmatched_pull = format!(
        "ExecStart=/usr/bin/podman image pull --os {unmatched_os} example.invalid/quadlet-lens-image-os-unmatched:latest"
    );
    if unmatched.matches(&unmatched_pull).count() != 1 {
        return Err(format!(
            "Podman {version} Image OS unmatched-quote command text did not match the recorded endpoint difference\n{unmatched}"
        ));
    }
    eprintln!("Podman {version} Image OS: normal, duplicate-last, blank omission, and unmatched-quote command text");
    Ok(())
}

fn verify_image_cert_dir_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image CertDir generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!("Podman {version} Image CertDir is missing fixture fragment"));
    }
    for (name, cert_dir) in [
        (
            "image-cert-dir-normal",
            Some("/placeholder/quadlet-lens-image-certs-normal"),
        ),
        (
            "image-cert-dir-duplicate",
            Some("/placeholder/quadlet-lens-image-certs-last"),
        ),
        ("image-cert-dir-blank", None),
    ] {
        let unit = generated_unit(version, &generated, &format!("{name}-image.service"), output)?;
        let pull = format!("ExecStart=/usr/bin/podman image pull example.invalid/quadlet-lens-{name}:latest");
        let expected_pull = cert_dir.map_or(pull, |cert_dir| {
            format!(
                "ExecStart=/usr/bin/podman image pull --cert-dir {cert_dir} example.invalid/quadlet-lens-{name}:latest"
            )
        });
        if unit.matches(&expected_pull).count() != 1 || unit.contains(" --cert-dir") != cert_dir.is_some() {
            return Err(format!(
                "Podman {version} Image CertDir must retain the target {name} command-text observation\\n{unit}"
            ));
        }
    }
    let unmatched = generated_unit(version, &generated, "image-cert-dir-unmatched-image.service", output)?;
    let unmatched_cert_dir = if parsed < PodmanVersion::new(5, 8, 2) {
        "/placeholder/quadlet-lens-image-certs-unmatched"
    } else {
        "\"/placeholder/quadlet-lens-image-certs-unmatched\\\"\""
    };
    let unmatched_pull = format!(
        "ExecStart=/usr/bin/podman image pull --cert-dir {unmatched_cert_dir} example.invalid/quadlet-lens-image-cert-dir-unmatched:latest"
    );
    if unmatched.matches(&unmatched_pull).count() != 1 {
        return Err(format!(
            "Podman {version} Image CertDir unmatched-quote command text did not match the recorded 5.8.2 boundary\\n{unmatched}"
        ));
    }
    eprintln!(
        "Podman {version} Image CertDir: normal, duplicate-last, blank omission, and unmatched-quote command text"
    );
    Ok(())
}

fn verify_image_containers_conf_module_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| error.to_string())?;
    ensure_success(version, "Image ContainersConfModule generator", output)?;
    if expected.iter().any(|fragment| !generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} Image ContainersConfModule is missing fixture fragment"
        ));
    }
    let unit = generated_unit(
        version,
        &generated,
        "image-containers-conf-module-image.service",
        output,
    )?;
    let command = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman "))
        .ok_or("missing image pull command")?;
    let arguments = ["--module=post-one", "--module=post-two"];
    let positions: Vec<_> = arguments
        .iter()
        .map(|argument| {
            command
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let image_pull = command.find(" image pull ").ok_or("missing image pull subcommand")?;
    if positions.iter().any(|matches| matches.len() != 1)
        || positions[0][0] >= positions[1][0]
        || positions.iter().any(|matches| matches[0] >= image_pull)
        || command.matches("--module=").count() != arguments.len()
        || command.contains("pre-one")
        || command.contains("pre-two")
    {
        return Err(format!(
            "Podman {version} Image ContainersConfModule must retain only ordered post-reset --module arguments before image pull\n{command}"
        ));
    }
    eprintln!(
        "Podman {version} Image ContainersConfModule: logical reset and ordered separate --module output before image pull"
    );
    Ok(())
}

fn verify_build_arg_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} BuildArg generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if parsed < PodmanVersion::new(5, 7, 0) {
        let argument_count = generated.matches("--build-arg").count();
        let rejected_or_excluded = !output.status.success()
            || !generated.contains("---build-arg-build.service---")
            || diagnostics.contains("BuildArg");
        if rejected_or_excluded {
            if argument_count != 0 {
                return Err(format!(
                    "Podman {version} rejects or excludes BuildArg but emitted --build-arg; found build-arg-arguments={argument_count}, status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                    output.status
                ));
            }
            eprintln!("Podman {version} BuildArg: rejected or excluded with no --build-arg argument");
            return Ok(());
        }
    }

    ensure_success(version, "BuildArg generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} BuildArg generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-arg-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-arg-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let positions: Vec<_> = BUILD_ARG_ARGUMENTS
        .iter()
        .map(|argument| {
            podman_build
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let all_argument_count = podman_build.matches("--build-arg").count();
    if positions.iter().any(|matches| matches.len() != 1)
        || all_argument_count != BUILD_ARG_ARGUMENTS.len()
        || podman_build.contains("--build-arg=")
    {
        return Err(format!(
            "Podman {version} generator output for build-arg-build.service must contain exactly one separate `{}` and `{}`, with no duplicate or equals form; found positions={positions:?}, all-build-arg={all_argument_count}, equals-form={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            BUILD_ARG_ARGUMENTS[0],
            BUILD_ARG_ARGUMENTS[1],
            podman_build.contains("--build-arg=")
        ));
    }
    eprintln!("Podman {version} BuildArg: exact key=value and empty-value key= --build-arg arguments");
    Ok(())
}

fn verify_build_secret_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build Secret generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build Secret generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build Secret generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-secret-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-secret-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let positions: Vec<_> = BUILD_SECRET_ARGUMENTS
        .iter()
        .map(|argument| {
            podman_build
                .match_indices(argument)
                .map(|(position, _)| position)
                .collect::<Vec<_>>()
        })
        .collect();
    let all_argument_count = podman_build.matches("--secret").count();
    if positions.iter().any(|matches| matches.len() != 1)
        || positions[0][0] >= positions[1][0]
        || all_argument_count != BUILD_SECRET_ARGUMENTS.len()
        || podman_build.contains("--secret=")
    {
        return Err(format!(
            "Podman {version} generator output for build-secret-build.service must contain exactly two separate ordered Build Secret arguments, with no duplicate or equals form; found positions={positions:?}, all-secrets={all_argument_count}, equals-form={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.contains("--secret=")
        ));
    }
    eprintln!("Podman {version} Build Secret: two exact ordered --secret arguments");
    Ok(())
}

fn verify_build_platform_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build platform generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build platform generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build platform generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-platform-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-platform-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let arch_count = podman_build.matches(BUILD_ARCH_ARGUMENT).count();
    let variant_count = podman_build.matches(BUILD_VARIANT_ARGUMENT).count();
    let all_arch_count = podman_build.matches("--arch").count();
    let all_variant_count = podman_build.matches("--variant").count();
    if arch_count != 1
        || variant_count != 1
        || all_arch_count != 1
        || all_variant_count != 1
        || podman_build.contains("--arch=")
        || podman_build.contains("--variant=")
    {
        return Err(format!(
            "Podman {version} generator output for build-platform-build.service must contain exactly one separate `{BUILD_ARCH_ARGUMENT}` and exactly one separate `{BUILD_VARIANT_ARGUMENT}`, with no duplicate or equals form; found arch={arch_count}/{all_arch_count}, variant={variant_count}/{all_variant_count}, arch-equals={}, variant-equals={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.contains("--arch="),
            podman_build.contains("--variant=")
        ));
    }
    eprintln!("Podman {version} Build platform: exact --arch arm64 and --variant v8 arguments");
    Ok(())
}

fn verify_build_pull_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build Pull generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build Pull generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build Pull generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-pull-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-pull-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_build.matches(BUILD_PULL_ARGUMENT).count();
    let all_pull_count = podman_build.matches("--pull").count();
    if expected_count != 1 || all_pull_count != 1 || podman_build.contains("--pull always") {
        return Err(format!(
            "Podman {version} generator output for build-pull-build.service must contain exactly one `{BUILD_PULL_ARGUMENT}` and no separate or duplicate pull form; found expected={expected_count}, all-pull={all_pull_count}, separate-form={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
            podman_build.contains("--pull always")
        ));
    }
    eprintln!("Podman {version} Build Pull: exactly one --pull=always argument");
    Ok(())
}

fn verify_build_podman_args_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build PodmanArgs generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-podman-args-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{BUILD_PODMAN_ARGS_ARGUMENT} {BUILD_PODMAN_ARGS_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_build_context_count = podman_build.matches("--build-context").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_build_context_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-build.service must end with exactly one separate `{BUILD_PODMAN_ARGS_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_CONTEXT}`, with no equals, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-build-context={all_build_context_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs: one separate --build-context argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_no_cache_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --no-cache generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --no-cache generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --no-cache generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-podman-args-no-cache-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-no-cache-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{BUILD_PODMAN_ARGS_NO_CACHE_ARGUMENT} {BUILD_PODMAN_ARGS_NO_CACHE_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_no_cache_count = podman_build.matches(BUILD_PODMAN_ARGS_NO_CACHE_ARGUMENT).count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_NO_CACHE_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_no_cache_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-no-cache-build.service must end with exactly one separate `{BUILD_PODMAN_ARGS_NO_CACHE_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_NO_CACHE_CONTEXT}`, with no equals, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-no-cache={all_no_cache_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --no-cache: one separate --no-cache argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_isolation_chroot_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --isolation=chroot generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --isolation=chroot generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --isolation=chroot generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-isolation-chroot-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-isolation-chroot-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair =
        format!("{BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ARGUMENT} {BUILD_PODMAN_ARGS_ISOLATION_CHROOT_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_isolation_count = podman_build
        .matches(BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ARGUMENT)
        .count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_isolation_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-isolation-chroot-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_ISOLATION_CHROOT_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_ISOLATION_CHROOT_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-isolation={all_isolation_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --isolation=chroot: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_ssh_default_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --ssh=default generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --ssh=default generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --ssh=default generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-ssh-default-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-ssh-default-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{BUILD_PODMAN_ARGS_SSH_DEFAULT_ARGUMENT} {BUILD_PODMAN_ARGS_SSH_DEFAULT_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_ssh_count = podman_build.matches("--ssh").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_SSH_DEFAULT_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_ssh_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-ssh-default-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_SSH_DEFAULT_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_SSH_DEFAULT_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-ssh={all_ssh_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --ssh=default: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_shm_size_32m_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --shm-size=32m generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --shm-size=32m generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --shm-size=32m generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-shm-size-32m-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-shm-size-32m-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{BUILD_PODMAN_ARGS_SHM_SIZE_32M_ARGUMENT} {BUILD_PODMAN_ARGS_SHM_SIZE_32M_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_shm_size_count = podman_build.matches("--shm-size").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_SHM_SIZE_32M_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_shm_size_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-shm-size-32m-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_SHM_SIZE_32M_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_SHM_SIZE_32M_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-shm-size={all_shm_size_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --shm-size=32m: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_ulimit_nproc_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --ulimit=nproc=4096:8192 generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --ulimit=nproc=4096:8192 generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --ulimit=nproc=4096:8192 generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-ulimit-nproc-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-ulimit-nproc-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!("{BUILD_PODMAN_ARGS_ULIMIT_NPROC_ARGUMENT} {BUILD_PODMAN_ARGS_ULIMIT_NPROC_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_ulimit_count = podman_build.matches("--ulimit").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_ULIMIT_NPROC_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_ulimit_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-ulimit-nproc-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_ULIMIT_NPROC_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_ULIMIT_NPROC_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-ulimit={all_ulimit_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --ulimit=nproc=4096:8192: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_add_host_buildhost_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!(
            "{version} Build PodmanArgs --add-host=buildhost:192.0.2.10 generator emitted non-UTF-8 output: {error}"
        )
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(
        version,
        "Build PodmanArgs --add-host=buildhost:192.0.2.10 generator",
        output,
    )?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --add-host=buildhost:192.0.2.10 generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-add-host-buildhost-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-add-host-buildhost-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair =
        format!("{BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_ARGUMENT} {BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_add_host_count = podman_build.matches("--add-host").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_add_host_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-add-host-buildhost-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_ADD_HOST_BUILDHOST_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-add-host={all_add_host_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --add-host=buildhost:192.0.2.10: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_cap_add_cap_sys_admin_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs --cap-add=CAP_SYS_ADMIN generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs --cap-add=CAP_SYS_ADMIN generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs --cap-add=CAP_SYS_ADMIN generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-cap-add-cap-sys-admin-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-cap-add-cap-sys-admin-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair =
        format!("{BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_ARGUMENT} {BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_CONTEXT}");
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let all_cap_add_count = podman_build.matches("--cap-add").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1 || !expected_is_terminal || all_cap_add_count != 1 || !alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-cap-add-cap-sys-admin-build.service must end with exactly one equals-form `{BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_ARGUMENT}` immediately before final positional `{BUILD_PODMAN_ARGS_CAP_ADD_CAP_SYS_ADMIN_CONTEXT}`, with no separate, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, all-cap-add={all_cap_add_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs --cap-add=CAP_SYS_ADMIN: one equals-form argument immediately precedes the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_cache_locations_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone()).map_err(|error| {
        format!("{version} Build PodmanArgs cache-locations generator emitted non-UTF-8 output: {error}")
    })?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs cache-locations generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs cache-locations generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(
        version,
        &generated,
        "build-podman-args-cache-locations-build.service",
        output,
    )?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-cache-locations-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_chain = format!(
        "{BUILD_PODMAN_ARGS_CACHE_FROM_ARGUMENT} {BUILD_PODMAN_ARGS_CACHE_TO_ARGUMENT} {BUILD_PODMAN_ARGS_CACHE_LOCATIONS_CONTEXT}"
    );
    let expected_count = podman_build.matches(&expected_chain).count();
    let expected_is_terminal = podman_build.ends_with(&expected_chain);
    let cache_from_count = podman_build.matches("--cache-from").count();
    let cache_to_count = podman_build.matches("--cache-to").count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_CACHE_LOCATIONS_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1
        || !expected_is_terminal
        || cache_from_count != 1
        || cache_to_count != 1
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-cache-locations-build.service must end with exactly one ordered terminal chain `{BUILD_PODMAN_ARGS_CACHE_FROM_ARGUMENT} {BUILD_PODMAN_ARGS_CACHE_TO_ARGUMENT} {BUILD_PODMAN_ARGS_CACHE_LOCATIONS_CONTEXT}`, with no equals, quoted, missing, duplicate, or reordered form; found expected-chain={expected_count}, terminal={expected_is_terminal}, cache-from={cache_from_count}, cache-to={cache_to_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs cache locations: one ordered --cache-from/--cache-to terminal chain before the final positional context"
    );
    Ok(())
}

fn verify_build_podman_args_sbom_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} Build PodmanArgs SBOM generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    ensure_success(version, "Build PodmanArgs SBOM generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} Build PodmanArgs SBOM generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "build-podman-args-sbom-build.service", output)?;
    let podman_build = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman build "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for build-podman-args-sbom-build.service is missing its Podman build command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_pair = format!(
        "{BUILD_PODMAN_ARGS_SBOM_PRESET_ARGUMENT} {BUILD_PODMAN_ARGS_SBOM_OUTPUT_ARGUMENT} {BUILD_PODMAN_ARGS_SBOM_CONTEXT}"
    );
    let expected_count = podman_build.matches(&expected_pair).count();
    let expected_is_terminal = podman_build.ends_with(&expected_pair);
    let sbom_preset_count = podman_build
        .split_whitespace()
        .filter(|argument| *argument == BUILD_PODMAN_ARGS_SBOM_PRESET_ARGUMENT)
        .count();
    let sbom_output_count = podman_build
        .split_whitespace()
        .filter(|argument| *argument == BUILD_PODMAN_ARGS_SBOM_OUTPUT_ARGUMENT)
        .count();
    let alternate_forms: Vec<_> = BUILD_PODMAN_ARGS_SBOM_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_build.contains(form))
        .collect();
    if expected_count != 1
        || !expected_is_terminal
        || sbom_preset_count != 1
        || sbom_output_count != 1
        || !alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for build-podman-args-sbom-build.service must end with exactly one ordered terminal pair `{BUILD_PODMAN_ARGS_SBOM_PRESET_ARGUMENT} {BUILD_PODMAN_ARGS_SBOM_OUTPUT_ARGUMENT}` before final positional `{BUILD_PODMAN_ARGS_SBOM_CONTEXT}`, with no missing output, quoted, alternate, duplicate, or reordered form; found expected-pair={expected_count}, terminal={expected_is_terminal}, sbom={sbom_preset_count}, sbom-output={sbom_output_count}, alternate={alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!(
        "Podman {version} Build PodmanArgs SBOM: one ordered --sbom/--sbom-output terminal pair before the final positional context"
    );
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

fn verify_container_batch_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} container batch generator emitted non-UTF-8 output: {error}"))?;
    ensure_success(version, "container batch generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} container batch generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let unit = generated_unit(version, &generated, "batch.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} container batch output has no podman run command"))?;
    let arguments = exec_arguments(run);
    for pair in [
        ["--cgroups", "split"],
        ["--tz", "UTC"],
        ["--env-host=false", ""],
        ["--read-only-tmpfs", ""],
        ["--read-only", ""],
        ["--label", "io.containers.autoupdate=registry"],
        ["--mount", "type=tmpfs,destination=/scratch,tmpfs-size=65536"],
        ["--health-on-failure", "kill"],
    ] {
        if count_argument_pair(&arguments, pair[0], pair[1]) != 1 {
            return Err(format!(
                "Podman {version} container batch output must contain exact argument pair {pair:?} once"
            ));
        }
    }
    verify_container_batch_variants(version, &generated, output)?;
    eprintln!(
        "Podman {version} container batch: stable cgroup, timezone, environment, mapping, mount, read-only tmpfs, auto-update, and health-failure command construction"
    );
    Ok(())
}

fn exec_arguments(command: &str) -> Vec<&str> {
    // These fixtures use opaque CLI values without whitespace or C escapes, so token splitting
    // verifies exact argument pairs without claiming a general systemd ExecStart parser.
    command.split_whitespace().collect()
}

fn count_argument_pair(arguments: &[&str], flag: &str, value: &str) -> usize {
    if value.is_empty() {
        return arguments.iter().filter(|argument| **argument == flag).count();
    }
    arguments
        .windows(2)
        .filter(|pair| pair[0] == flag && pair[1] == value)
        .count()
}

fn verify_container_batch_variants(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let expected_modes = ["enabled", "disabled", "no-conmon", "split"];
    for mode in expected_modes {
        let unit = generated_unit(version, generated, &format!("cgroups-{mode}.service"), output)?;
        let run = unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
            .ok_or_else(|| format!("Podman {version} cgroups {mode} output has no run command"))?;
        if count_argument_pair(&exec_arguments(run), "--cgroups", mode) != 1 {
            return Err(format!("Podman {version} must emit one exact --cgroups {mode} pair"));
        }
    }
    for (unit_name, value) in [("batch.service", "registry"), ("auto-local.service", "local")] {
        let unit = generated_unit(version, generated, unit_name, output)?;
        let run = unit
            .lines()
            .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
            .ok_or_else(|| format!("Podman {version} {unit_name} output has no run command"))?;
        if count_argument_pair(
            &exec_arguments(run),
            "--label",
            &format!("io.containers.autoupdate={value}"),
        ) != 1
        {
            return Err(format!(
                "Podman {version} must emit AutoUpdate={value} as one exact label pair"
            ));
        }
    }
    let unit = generated_unit(version, generated, "mount-reset.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} mount reset output has no run command"))?;
    let arguments = exec_arguments(run);
    let final_one = ["--mount", "type=tmpfs,destination=/final-one"];
    let final_two = ["--mount", "type=tmpfs,destination=/final-two"];
    if count_argument_pair(&arguments, final_one[0], final_one[1]) != 1
        || count_argument_pair(&arguments, final_two[0], final_two[1]) != 1
        || arguments
            .windows(2)
            .position(|pair| pair == final_one)
            .unwrap_or(usize::MAX)
            >= arguments.windows(2).position(|pair| pair == final_two).unwrap_or(0)
        || arguments.iter().any(|argument| argument.contains("/pre-"))
    {
        return Err(format!(
            "Podman {version} must retain only ordered post-reset Mount arguments"
        ));
    }
    Ok(())
}

fn verify_container_retry_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    if parsed < PodmanVersion::new(5, 5, 0) {
        if output.status.success() || String::from_utf8_lossy(&output.stdout).contains("--retry") {
            return Err(format!(
                "Podman {version} must reject Container Retry and RetryDelay without emitting retry arguments"
            ));
        }
        return Ok(());
    }
    ensure_success(version, "container retry generator", output)?;
    let generated = String::from_utf8_lossy(&output.stdout);
    if !expected.iter().all(|fragment| generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} container retry output misses an expected fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "retry.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} retry output has no podman run command"))?;
    let arguments = exec_arguments(run);
    if count_argument_pair(&arguments, "--retry", "4") != 1
        || count_argument_pair(&arguments, "--retry-delay", "7s") != 1
        || arguments.iter().filter(|argument| **argument == "--retry").count() != 1
        || arguments
            .iter()
            .filter(|argument| **argument == "--retry-delay")
            .count()
            != 1
    {
        return Err(format!(
            "Podman {version} must emit exactly one separate --retry 4 and --retry-delay 7s argument"
        ));
    }
    Ok(())
}

fn verify_container_http_proxy_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    if parsed < PodmanVersion::new(5, 7, 0) {
        if output.status.success() || String::from_utf8_lossy(&output.stdout).contains("--http-proxy") {
            return Err(format!(
                "Podman {version} must reject Container HttpProxy without emitting proxy arguments"
            ));
        }
        return Ok(());
    }
    ensure_success(version, "container HttpProxy generator", output)?;
    let generated = String::from_utf8_lossy(&output.stdout);
    if !expected.iter().all(|fragment| generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} container HttpProxy output misses an expected fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "http-proxy.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} HttpProxy output has no podman run command"))?;
    let arguments = exec_arguments(run);
    if arguments
        .iter()
        .filter(|argument| **argument == "--http-proxy=false")
        .count()
        != 1
        || arguments
            .iter()
            .filter(|argument| argument.starts_with("--http-proxy"))
            .count()
            != 1
    {
        return Err(format!(
            "Podman {version} must emit exactly one --http-proxy=false argument"
        ));
    }
    Ok(())
}

fn verify_container_start_with_pod_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "container StartWithPod generator", output)?;
    let generated = String::from_utf8_lossy(&output.stdout);
    if !expected.iter().all(|fragment| generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} container StartWithPod output misses an expected relationship fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "batch.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} StartWithPod output has no podman run command"))?;
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let expected_argument = if parsed < PodmanVersion::new(5, 7, 0) {
        "--pod-id-file %t/batch-pod.pod-id"
    } else {
        "--pod systemd-batch"
    };
    let arguments = exec_arguments(run);
    let expected_count = if expected_argument.starts_with("--pod-id-file") {
        count_argument_pair(&arguments, "--pod-id-file", "%t/batch-pod.pod-id")
    } else {
        count_argument_pair(&arguments, "--pod", "systemd-batch")
    };
    if expected_count != 1 {
        return Err(format!(
            "Podman {version} StartWithPod output must contain exactly one `{expected_argument}` argument"
        ));
    }
    Ok(())
}

fn verify_container_direct_maps_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "container direct maps generator", output)?;
    let generated = String::from_utf8_lossy(&output.stdout);
    if !expected.iter().all(|fragment| generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} direct maps output misses an expected fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "maps.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} direct maps output has no run command"))?;
    let arguments = exec_arguments(run);
    for (flag, first, second, pre) in [
        ("--uidmap", "0:200000:65536", "1:300000:1", "0:100000:65536"),
        ("--gidmap", "0:200000:65536", "1:300000:1", "0:100000:65536"),
    ] {
        let first_position = arguments
            .windows(2)
            .position(|pair| pair[0] == flag && pair[1] == first);
        let second_position = arguments
            .windows(2)
            .position(|pair| pair[0] == flag && pair[1] == second);
        if first_position.is_none()
            || second_position.is_none()
            || first_position >= second_position
            || arguments.contains(&pre)
            || arguments.iter().filter(|argument| **argument == flag).count() != 2
        {
            return Err(format!(
                "Podman {version} must emit two ordered post-reset {flag} pairs without pre-reset values"
            ));
        }
    }
    Ok(())
}

fn verify_container_sub_maps_generator_output(
    version: &str,
    expected: &[String],
    output: &Output,
) -> Result<(), String> {
    ensure_success(version, "container subordinate maps generator", output)?;
    let generated = String::from_utf8_lossy(&output.stdout);
    if !expected.iter().all(|fragment| generated.contains(fragment)) {
        return Err(format!(
            "Podman {version} subordinate maps output misses an expected fixture fragment"
        ));
    }
    let unit = generated_unit(version, &generated, "sub-maps.service", output)?;
    let run = unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| format!("Podman {version} subordinate maps output has no run command"))?;
    let arguments = exec_arguments(run);
    if count_argument_pair(&arguments, "--subuidname", "keep-id") != 1
        || count_argument_pair(&arguments, "--subgidname", "keep-id") != 1
        || arguments
            .iter()
            .any(|argument| *argument == "--uidmap" || *argument == "--gidmap")
    {
        return Err(format!(
            "Podman {version} must emit independent subordinate mapping pairs without direct maps"
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
