//! Catalogue schema, rolling supported range, and version-boundary behavior.

use std::collections::BTreeSet;

use quadlet_lens::capability::{
    CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification, VerificationLevel,
};

const EXPECTED_CAPABILITIES: &str =
    include_str!("../fixtures/capabilities/podman-supported-range/expected-capabilities.txt");
const EXPECTED_BOUNDARIES: &str = include_str!("../fixtures/version-boundaries/podman-5-4-floor/expected.txt");

#[test]
fn supported_range_has_the_reviewed_first_conversion_surface() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    assert_eq!(catalogue.schema(), 1);
    assert_eq!(catalogue.id(), "podman-supported-range");
    assert_eq!(catalogue.coverage().minimum(), version(5, 4, 0));
    assert_eq!(catalogue.coverage().maximum(), version(6, 0, 2));

    let actual: BTreeSet<_> = catalogue
        .capabilities()
        .iter()
        .map(quadlet_lens::capability::CapabilityRecord::id)
        .collect();
    let expected: BTreeSet<_> = EXPECTED_CAPABILITIES.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(actual, expected);
    assert_eq!(catalogue.capabilities().len(), 59);
    assert_eq!(catalogue.evidence().len(), 69);

    let documentation: Vec<_> = catalogue
        .evidence()
        .iter()
        .filter(|evidence| evidence.level() == VerificationLevel::Documentation)
        .collect();
    assert!(!documentation.is_empty());
    assert_eq!(documentation.len(), 66);
    assert!(documentation.iter().all(|evidence| evidence.gap().is_some()));
    let generator = catalogue
        .evidence()
        .iter()
        .find(|evidence| evidence.id() == "podman-5-4-through-current-first-conversion-generators")
        .ok_or_else(|| "supported range must have generator evidence".to_owned())?;
    assert_eq!(generator.versions().minimum(), version(5, 4, 0));
    assert_eq!(generator.versions().maximum(), version(6, 0, 2));
    assert_eq!(generator.gap(), None);
    Ok(())
}

#[test]
fn supported_range_records_paths_references_and_repetition() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let volume = catalogue
        .capability("quadlet.container.volume")
        .ok_or_else(|| "container volume capability must exist".to_owned())?;
    assert!(volume.is_repeatable());
    assert!(volume.value_forms().iter().any(|form| form == "unit-relative-path"));
    assert!(volume.value_forms().iter().any(|form| form == "volume-unit-reference"));

    let secret = catalogue
        .capability("quadlet.container.secret")
        .ok_or_else(|| "container secret capability must exist".to_owned())?;
    assert!(secret.is_repeatable());
    assert!(
        secret
            .value_forms()
            .iter()
            .any(|form| form == "podman-secret-mount-options")
    );

    let label = catalogue
        .capability("quadlet.container.label")
        .ok_or_else(|| "container label capability must exist".to_owned())?;
    assert!(label.is_repeatable());
    assert!(label.value_forms().iter().any(|form| form == "oci-label-assignment"));
    assert!(
        secret
            .value_forms()
            .iter()
            .any(|form| form == "podman-secret-environment-options")
    );

    let image = catalogue
        .capability("quadlet.container.image")
        .ok_or_else(|| "container image capability must exist".to_owned())?;
    let rootfs = catalogue
        .capability("quadlet.container.rootfs")
        .ok_or_else(|| "container rootfs capability must exist".to_owned())?;
    assert!(!image.is_required());
    assert!(!rootfs.is_required());
    assert!(rootfs.value_forms().iter().any(|form| form == "podman-rootfs"));

    let entrypoint = catalogue
        .capability("quadlet.container.entrypoint")
        .ok_or_else(|| "container entrypoint capability must exist".to_owned())?;
    assert!(!entrypoint.is_repeatable());
    assert!(entrypoint.value_forms().iter().any(|form| form == "executable"));
    assert!(entrypoint.value_forms().iter().any(|form| form == "json-command-array"));

    let run_init = catalogue
        .capability("quadlet.container.run-init")
        .ok_or_else(|| "container run-init capability must exist".to_owned())?;
    assert!(!run_init.is_repeatable());
    assert_eq!(run_init.value_forms(), ["literal-true-or-false"]);

    let restart = catalogue
        .capability("systemd.service.restart")
        .ok_or_else(|| "generic systemd restart capability must exist".to_owned())?;
    assert_eq!(restart.sections(), ["Service"]);

    for capability in ["systemd.unit.requires", "systemd.unit.wants", "systemd.unit.after"] {
        let record = catalogue
            .capability(capability)
            .ok_or_else(|| format!("{capability} capability must exist"))?;
        assert_eq!(record.sections(), ["Unit"]);
        assert!(record.is_repeatable());
    }

    let current = PodmanTarget::new(version(6, 0, 2), Some(version(6, 0, 2))).map_err(|error| error.to_string())?;
    for capability in [
        "quadlet.unit-type.pod",
        "quadlet.container.add-host",
        "quadlet.container.container-name",
        "quadlet.container.environment-file",
        "quadlet.container.entrypoint",
        "quadlet.container.user",
        "quadlet.container.group",
        "quadlet.container.userns",
        "quadlet.container.group-add",
        "quadlet.container.working-dir",
        "quadlet.container.read-only",
        "quadlet.container.rootfs",
        "quadlet.container.run-init",
        "quadlet.container.pids-limit",
        "quadlet.container.secret",
        "quadlet.container.label",
        "quadlet.container.pod",
        "quadlet.container.health-command",
        "quadlet.container.health-interval",
        "quadlet.container.health-retries",
        "quadlet.container.health-start-period",
        "quadlet.container.health-timeout",
        "quadlet.container.notify-healthy",
        "systemd.unit.requires",
        "systemd.unit.wants",
        "systemd.unit.after",
        "quadlet.pod.add-host",
        "quadlet.pod.name",
        "quadlet.pod.publish-port",
        "quadlet.pod.network",
        "quadlet.pod.volume",
        "quadlet.pod.userns",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, current).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_stop_lifecycle() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, value_form) in [
        ("quadlet.container.stop-signal", "signal-token-or-number"),
        ("quadlet.container.stop-timeout", "non-negative-integer-seconds"),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), ["container"]);
        assert_eq!(capability.sections(), ["Container"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), [value_form]);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 0, 2));

        for target in [
            PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 0))),
            PodmanTarget::new(version(6, 0, 2), Some(version(6, 0, 2))),
        ] {
            assert_eq!(
                catalogue
                    .evaluate(id, target.map_err(|error| error.to_string())?)
                    .classification(),
                SupportClassification::Native
            );
        }
        assert_eq!(
            catalogue
                .evaluate(
                    id,
                    PodmanTarget::new(version(5, 3, 0), Some(version(5, 3, 0))).map_err(|error| error.to_string())?,
                )
                .classification(),
            SupportClassification::Unknown
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_pull() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.pull")
        .ok_or_else(|| "container pull capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["always", "missing", "never", "newer"]);
    let native = capability
        .native_range()
        .ok_or_else(|| "container pull must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.pull", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_pids_limit() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.pids-limit")
        .ok_or_else(|| "container pids-limit capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["minus-one-unlimited", "positive-integer"]);
    let native = capability
        .native_range()
        .ok_or_else(|| "container pids-limit must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.pids-limit", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_hostname() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.hostname")
        .ok_or_else(|| "container hostname capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["podman-hostname"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-hostname",
            "podman-5-4-run-hostname-uts",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container hostname must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.hostname", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_and_pod_shm_size() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, unit_type, section, evidence) in [
        (
            "quadlet.container.shm-size",
            "container",
            "Container",
            &[
                "podman-5-4-quadlet-shm-size",
                "podman-5-4-run-shm-size",
                "podman-5-4-through-current-first-conversion-generators",
            ][..],
        ),
        (
            "quadlet.pod.shm-size",
            "pod",
            "Pod",
            &[
                "podman-5-4-quadlet-shm-size",
                "podman-5-4-pod-shm-size",
                "podman-5-4-pod-shared-ipc",
                "podman-5-4-through-current-first-conversion-generators",
            ][..],
        ),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), [unit_type]);
        assert_eq!(capability.sections(), [section]);
        assert!(!capability.is_repeatable());
        assert_eq!(
            capability.value_forms(),
            ["non-negative-ascii-decimal-bytes", "non-negative-ascii-decimal-b-k-m-g"]
        );
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 0, 2));

        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 0, 2), SupportClassification::Native),
            (version(6, 0, 3), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_drop_capability() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.drop-capability")
        .ok_or_else(|| "container drop-capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["space-separated-capability-list", "lowercase-all"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-drop-capability",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container drop-capability must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.drop-capability", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_add_capability() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.add-capability")
        .ok_or_else(|| "container add-capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["space-separated-capability-list", "source-and-generator-observed-all"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-add-capability",
            "podman-5-4-container-capability-command-source",
            "podman-6-0-2-container-capability-command-source",
            "podman-5-4-container-capability-reset-source",
            "podman-6-0-2-container-capability-reset-source",
            "podman-5-4-capability-merger-source",
            "podman-6-0-2-capability-merger-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container add-capability must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.add-capability", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_tmpfs() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.tmpfs")
        .ok_or_else(|| "container tmpfs must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["container-dir-optional-options"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-tmpfs",
            "podman-5-4-run-tmpfs",
            "podman-5-4-container-tmpfs-command-source",
            "podman-6-0-2-container-tmpfs-command-source",
            "podman-5-4-container-tmpfs-reset-source",
            "podman-6-0-2-container-tmpfs-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container tmpfs must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.tmpfs", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_sysctl() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.sysctl")
        .ok_or_else(|| "container sysctl must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["space-separated-name-equals-value-list"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-sysctl",
            "podman-6-0-2-container-sysctl",
            "podman-5-4-run-sysctl-namespaces",
            "podman-6-0-2-run-sysctl-namespaces",
            "podman-5-4-container-sysctl-command-source",
            "podman-6-0-2-container-sysctl-command-source",
            "podman-5-4-container-sysctl-tokenization-source",
            "podman-6-0-2-container-sysctl-tokenization-source",
            "podman-5-4-container-sysctl-reset-source",
            "podman-6-0-2-container-sysctl-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container sysctl must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.sysctl", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_ulimit() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.ulimit")
        .ok_or_else(|| "container ulimit must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["type-equals-soft-optional-hard"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-ulimit",
            "podman-6-0-2-container-ulimit",
            "podman-5-4-run-ulimit",
            "podman-6-0-2-run-ulimit",
            "podman-5-4-container-ulimit-command-source",
            "podman-6-0-2-container-ulimit-command-source",
            "podman-5-4-container-ulimit-lookup-all-source",
            "podman-6-0-2-container-ulimit-lookup-all-source",
            "podman-5-4-container-ulimit-reset-source",
            "podman-6-0-2-container-ulimit-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container ulimit must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.ulimit", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_add_device() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.add-device")
        .ok_or_else(|| "container add-device must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        [
            "host-device-optional-container-device-permissions",
            "conditional-leading-minus"
        ]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-add-device",
            "podman-6-0-2-container-add-device",
            "podman-5-4-run-device-caveats",
            "podman-6-0-2-run-device-caveats",
            "podman-5-4-container-add-device-command-source",
            "podman-6-0-2-container-add-device-command-source",
            "podman-5-4-container-add-device-tokenization-source",
            "podman-6-0-2-container-add-device-tokenization-source",
            "podman-5-4-container-add-device-reset-source",
            "podman-6-0-2-container-add-device-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container add-device must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.add-device", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_memory_from_5_5() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.memory")
        .ok_or_else(|| "container memory capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["positive-ascii-decimal-bytes", "positive-ascii-decimal-b-k-m-g"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-5-container-memory",
            "podman-6-0-2-container-memory",
            "podman-5-5-memory-introduction",
            "podman-5-5-container-memory-command-source",
            "podman-6-0-2-container-memory-command-source",
            "podman-5-5-container-memory-last-value-source",
            "podman-6-0-2-container-memory-last-value-source",
            "podman-5-4-memory-generator-rejection",
            "podman-5-5-through-current-memory-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container memory must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 5, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unknown),
        (version(5, 4, 2), SupportClassification::Unknown),
        (version(5, 5, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.memory", target).classification(),
            expected
        );
    }

    let crossing = PodmanTarget::new(version(5, 4, 2), Some(version(5, 5, 0))).map_err(|error| error.to_string())?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.memory", crossing)
            .classification(),
        SupportClassification::Unknown
    );
    Ok(())
}

#[test]
fn run_init_capability_is_bounded_and_describes_evidenced_boolean_text() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.run-init")
        .ok_or_else(|| "container run-init capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["literal-true-or-false"]);
    let native = capability
        .native_range()
        .ok_or_else(|| "container run-init must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 0, 2));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 0, 2), SupportClassification::Native),
        (version(6, 0, 3), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.run-init", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn podman_5_4_floor_is_fail_closed_outside_evidence_coverage() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = "quadlet.container.image";
    let cases = [
        ("5.3.0..=5.3.0 unknown", target(5, 3, Some((5, 3)))?),
        ("5.4.0..=5.4.0 native", target(5, 4, Some((5, 4)))?),
        ("5.4.0..=5.8.0 native", target(5, 4, Some((5, 8)))?),
        ("5.4.0..=6.0.0 native", target(5, 4, Some((6, 0)))?),
        (
            "6.0.2..=6.0.2 native",
            PodmanTarget::new(version(6, 0, 2), Some(version(6, 0, 2))).map_err(|error| error.to_string())?,
        ),
        ("5.4.0..=6.1.0 unknown", target(5, 4, Some((6, 1)))?),
    ];
    let mut observed = Vec::new();
    for (label, target) in cases {
        let evaluation = catalogue.evaluate(capability, target);
        let expected = label.rsplit_once(' ').map(|(_, value)| value).unwrap_or_default();
        assert_eq!(classification_name(evaluation.classification()), expected);
        observed.push(label);
    }
    assert_eq!(observed.join("\n") + "\n", EXPECTED_BOUNDARIES);

    let open_ended = catalogue.evaluate(capability, target(5, 4, None)?);
    assert_eq!(open_ended.classification(), SupportClassification::Native);
    assert!(open_ended.assumes_later_versions());
    assert!(open_ended.note().is_some());

    let future_open_ended = catalogue.evaluate(capability, target(5, 5, None)?);
    assert_eq!(future_open_ended.classification(), SupportClassification::Native);
    assert_eq!(future_open_ended.evaluated_range().minimum(), version(5, 5, 0));
    assert_eq!(future_open_ended.evaluated_range().maximum(), version(6, 0, 2));

    let unverified_form = catalogue.evaluate("quadlet.container.image-unit-reference", target(6, 0, Some((6, 0)))?);
    assert_eq!(unverified_form.classification(), SupportClassification::Unknown);
    Ok(())
}

#[test]
fn schema_evaluates_fallbacks_and_known_bugs_before_native_support() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::parse(SYNTHETIC_CATALOGUE).map_err(|error| error.to_string())?;
    let fallback = catalogue.evaluate("quadlet.example.fallback", target(5, 4, Some((5, 4)))?);
    assert_eq!(fallback.classification(), SupportClassification::Fallback);
    assert_eq!(fallback.selected_fallback(), Some("podman-argument"));

    let broken = catalogue.evaluate(
        "quadlet.example.broken",
        PodmanTarget::new(version(5, 4, 1), Some(version(5, 4, 1))).map_err(|error| error.to_string())?,
    );
    assert_eq!(broken.classification(), SupportClassification::Broken);
    assert_eq!(broken.note(), Some("documented patch regression"));

    let deprecated = catalogue.evaluate(
        "quadlet.example.lifecycle",
        PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 0))).map_err(|error| error.to_string())?,
    );
    assert_eq!(deprecated.classification(), SupportClassification::Deprecated);

    let removed = catalogue.evaluate(
        "quadlet.example.lifecycle",
        PodmanTarget::new(version(5, 4, 1), Some(version(5, 4, 1))).map_err(|error| error.to_string())?,
    );
    assert_eq!(removed.classification(), SupportClassification::Removed);
    Ok(())
}

fn target(major: u64, minor: u64, maximum: Option<(u64, u64)>) -> Result<PodmanTarget, String> {
    PodmanTarget::new(
        version(major, minor, 0),
        maximum.map(|(major, minor)| version(major, minor, 0)),
    )
    .map_err(|error| error.to_string())
}

const fn version(major: u64, minor: u64, patch: u64) -> PodmanVersion {
    PodmanVersion::new(major, minor, patch)
}

const fn classification_name(classification: SupportClassification) -> &'static str {
    match classification {
        SupportClassification::Native => "native",
        SupportClassification::Fallback => "fallback",
        SupportClassification::Deprecated => "deprecated",
        SupportClassification::Removed => "removed",
        SupportClassification::Unsupported => "unsupported",
        SupportClassification::Unknown => "unknown",
        SupportClassification::Broken => "broken",
        _ => "future",
    }
}

const SYNTHETIC_CATALOGUE: &str = r#"
schema = 1
id = "synthetic-5-4"

[coverage]
minimum = "5.4.0"
maximum = "5.4.2"

[[evidence]]
id = "generator-5-4"
verification = "generator"
url = "https://example.invalid/generator-evidence"
target = "5.4.1"
claim = "Synthetic exact-version behavior."
test = "capabilities::schema_evaluates_fallbacks_and_known_bugs_before_native_support"

[[capability]]
id = "quadlet.example.fallback"
description = "Synthetic fallback coverage."
unit_types = ["container"]
sections = ["Container"]
evidence = ["generator-5-4"]

[[capability.fallback]]
kind = "podman-argument"
versions = { minimum = "5.4.0", maximum = "5.4.2" }
semantic_difference = "The native key spelling is unavailable."
evidence = ["generator-5-4"]

[[capability]]
id = "quadlet.example.broken"
description = "Synthetic known patch regression."
unit_types = ["container"]
sections = ["Container"]
native = { minimum = "5.4.0", maximum = "5.4.2" }
evidence = ["generator-5-4"]

[[capability.known_bug]]
versions = { minimum = "5.4.1", maximum = "5.4.1" }
summary = "documented patch regression"
evidence = ["generator-5-4"]

[[capability]]
id = "quadlet.example.lifecycle"
description = "Synthetic deprecation and removal boundaries."
unit_types = ["container"]
sections = ["Container"]
native = { minimum = "5.4.0", maximum = "5.4.0" }
deprecated_from = "5.4.0"
removed_from = "5.4.1"
evidence = ["generator-5-4"]
"#;
