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
    assert_eq!(catalogue.coverage().maximum(), version(6, 1, 0));

    let actual: BTreeSet<_> = catalogue
        .capabilities()
        .iter()
        .map(quadlet_lens::capability::CapabilityRecord::id)
        .collect();
    let expected: BTreeSet<_> = EXPECTED_CAPABILITIES.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(actual, expected);
    assert_eq!(catalogue.capabilities().len(), 258);
    assert_eq!(catalogue.evidence().len(), 660);
    assert_eq!(catalogue.systemd_evidence().len(), 1);

    let documentation: Vec<_> = catalogue
        .evidence()
        .iter()
        .filter(|evidence| evidence.level() == VerificationLevel::Documentation)
        .collect();
    assert!(!documentation.is_empty());
    assert_eq!(documentation.len(), 552);
    assert!(documentation.iter().all(|evidence| evidence.gap().is_some()));
    let generator = catalogue
        .evidence()
        .iter()
        .find(|evidence| evidence.id() == "podman-5-4-through-current-first-conversion-generators")
        .ok_or_else(|| "supported range must have generator evidence".to_owned())?;
    assert_eq!(generator.versions().minimum(), version(5, 4, 0));
    assert_eq!(generator.versions().maximum(), version(6, 1, 0));
    assert_eq!(generator.gap(), None);
    Ok(())
}

#[test]
fn supported_range_records_container_environment_reset() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.environment")
        .ok_or_else(|| "Container Environment capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        [
            "systemd-environment-assignment-list",
            "literal-ascii-name-single-line-unicode-value",
            "non-empty-grouped-literal-ascii-name-single-line-unicode-values",
            "empty-reset-directive",
        ]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container",
            "podman-5-4-through-current-first-conversion-generators",
            "podman-5-4-container-environment-reset-source",
            "podman-6-0-2-container-environment-reset-source",
            "podman-5-4-through-current-container-environment-reset-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Container Environment must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.environment", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_image_volume() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.image-volume")
        .ok_or_else(|| "Container ImageVolume capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-volume-policy"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-6-1-container-image-volume",
            "podman-6-1-container-image-volume-source",
            "podman-6-1-container-image-volume-generator",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Container ImageVolume must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(6, 1, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(6, 0, 2), SupportClassification::Unsupported),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.image-volume", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_network_and_image_completion_boundaries() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, minimum) in [
        ("quadlet.network.containers-conf-module", true, version(5, 4, 0)),
        ("quadlet.network.disable-dns", false, version(5, 4, 0)),
        ("quadlet.network.dns", true, version(5, 4, 0)),
        ("quadlet.network.global-args", true, version(5, 4, 0)),
        ("quadlet.network.interface-name", false, version(5, 6, 0)),
        ("quadlet.network.delete-on-stop", false, version(5, 5, 0)),
        ("quadlet.network.podman-args", true, version(5, 4, 0)),
        ("quadlet.network.service-name", false, version(5, 4, 0)),
        ("quadlet.image.podman-args", true, version(5, 4, 0)),
        ("quadlet.image.policy", false, version(5, 6, 0)),
        ("quadlet.image.retry", false, version(5, 5, 0)),
        ("quadlet.image.retry-delay", false, version(5, 5, 0)),
        ("quadlet.image.tls-verify", false, version(5, 4, 0)),
        ("quadlet.image.variant", false, version(5, 4, 0)),
    ] {
        let record = catalogue.capability(id).ok_or_else(|| format!("missing {id}"))?;
        assert_eq!(record.is_repeatable(), repeatable, "{id}");
        assert_eq!(
            record
                .native_range()
                .map(quadlet_lens::capability::VersionRange::minimum),
            Some(minimum),
            "{id}"
        );
    }
    for (id, target) in [
        ("quadlet.network.delete-on-stop", version(5, 4, 2)),
        ("quadlet.network.interface-name", version(5, 5, 2)),
        ("quadlet.image.retry", version(5, 4, 2)),
        ("quadlet.image.retry-delay", version(5, 4, 2)),
        ("quadlet.image.policy", version(5, 5, 2)),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    id,
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            SupportClassification::Unsupported,
            "{id} at {target}"
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_kube_keys() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable) in [
        ("quadlet.kube.auto-update", true),
        ("quadlet.kube.config-map", true),
        ("quadlet.kube.containers-conf-module", true),
        ("quadlet.kube.exit-code-propagation", false),
        ("quadlet.kube.global-args", true),
        ("quadlet.kube.down-force", false),
        ("quadlet.kube.log-driver", false),
        ("quadlet.kube.network", true),
        ("quadlet.kube.podman-args", true),
        ("quadlet.kube.publish-port", true),
        ("quadlet.kube.service-name", false),
        ("quadlet.kube.set-working-directory", false),
        ("quadlet.kube.userns", false),
        ("quadlet.kube.yaml", true),
        ("quadlet.kube.log-opt", true),
        ("quadlet.kube.remap-gid", true),
        ("quadlet.kube.remap-uid", true),
        ("quadlet.kube.remap-uid-size", false),
        ("quadlet.kube.remap-users", false),
    ] {
        let record = catalogue.capability(id).ok_or_else(|| format!("missing {id}"))?;
        assert_eq!(record.unit_types(), ["kube"], "{id}");
        assert_eq!(record.sections(), ["Kube"], "{id}");
        assert_eq!(record.is_repeatable(), repeatable, "{id}");
        let native = record
            .native_range()
            .ok_or_else(|| format!("missing native range for {id}"))?;
        assert_eq!(native.minimum(), version(5, 4, 0), "{id}");
        assert_eq!(native.maximum(), version(6, 1, 0), "{id}");
    }
    Ok(())
}

#[test]
fn supported_range_records_artifact_and_default_dependencies() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable) in [
        ("quadlet.artifact.artifact", false),
        ("quadlet.artifact.auth-file", false),
        ("quadlet.artifact.cert-dir", false),
        ("quadlet.artifact.creds", false),
        ("quadlet.artifact.decryption-key", false),
        ("quadlet.artifact.quiet", false),
        ("quadlet.artifact.retry", false),
        ("quadlet.artifact.retry-delay", false),
        ("quadlet.artifact.service-name", false),
        ("quadlet.artifact.tls-verify", false),
        ("quadlet.artifact.containers-conf-module", true),
        ("quadlet.artifact.global-args", true),
        ("quadlet.artifact.podman-args", true),
        ("quadlet.artifact.default-dependencies", false),
    ] {
        let record = catalogue.capability(id).ok_or_else(|| format!("missing {id}"))?;
        assert_eq!(record.unit_types(), ["artifact"], "{id}");
        if id == "quadlet.artifact.default-dependencies" {
            assert_eq!(record.sections(), ["Quadlet"], "{id}");
        } else {
            assert_eq!(record.sections(), ["Artifact"], "{id}");
        }
        assert_eq!(record.is_repeatable(), repeatable, "{id}");
        let native = record
            .native_range()
            .ok_or_else(|| format!("missing native range for {id}"))?;
        assert_eq!(native.minimum(), version(5, 7, 0), "{id}");
        assert_eq!(native.maximum(), version(6, 1, 0), "{id}");
        for target in [version(5, 4, 0), version(5, 6, 2)] {
            assert_eq!(
                catalogue
                    .evaluate(
                        id,
                        PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                    )
                    .classification(),
                SupportClassification::Unsupported,
                "{id} at {target}"
            );
        }
        for target in [version(5, 7, 0), version(6, 1, 0)] {
            assert_eq!(
                catalogue
                    .evaluate(
                        id,
                        PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                    )
                    .classification(),
                SupportClassification::Native,
                "{id} at {target}"
            );
        }
    }
    let defaults = catalogue
        .capability("quadlet.default-dependencies")
        .ok_or("missing shared DefaultDependencies capability")?;
    assert_eq!(
        defaults.unit_types(),
        ["container", "pod", "network", "volume", "build", "image", "kube"]
    );
    assert_eq!(defaults.sections(), ["Quadlet"]);
    assert!(!defaults.is_repeatable());
    let native = defaults
        .native_range()
        .ok_or("DefaultDependencies native range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    Ok(())
}

#[test]
fn supported_range_records_container_batch_keys() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, minimum) in [
        ("quadlet.container.auto-update", false, Some(version(5, 4, 0))),
        ("quadlet.container.cgroups-mode", false, Some(version(5, 4, 0))),
        ("quadlet.container.environment-host", false, Some(version(5, 4, 0))),
        ("quadlet.container.gid-map", true, Some(version(5, 4, 0))),
        ("quadlet.container.http-proxy", false, Some(version(5, 7, 0))),
        ("quadlet.container.mount", true, Some(version(5, 4, 0))),
        ("quadlet.container.read-only-tmpfs", false, Some(version(5, 4, 0))),
        ("quadlet.container.retry", false, Some(version(5, 5, 0))),
        ("quadlet.container.retry-delay", false, Some(version(5, 5, 0))),
        ("quadlet.container.start-with-pod", false, Some(version(5, 4, 0))),
        ("quadlet.container.subgid-map", false, Some(version(5, 4, 0))),
        ("quadlet.container.subuid-map", false, Some(version(5, 4, 0))),
        ("quadlet.container.timezone", false, Some(version(5, 4, 0))),
        ("quadlet.container.uid-map", true, Some(version(5, 4, 0))),
        ("quadlet.container.health-on-failure", false, Some(version(5, 4, 0))),
        ("quadlet.container.containers-conf-module", true, Some(version(5, 4, 0))),
        ("quadlet.container.global-args", true, Some(version(5, 4, 0))),
        (
            "quadlet.container.health-log-destination",
            false,
            Some(version(5, 4, 0)),
        ),
        ("quadlet.container.health-max-log-count", false, Some(version(5, 4, 0))),
        ("quadlet.container.health-max-log-size", false, Some(version(5, 4, 0))),
        ("quadlet.container.health-startup-cmd", false, Some(version(5, 4, 0))),
        (
            "quadlet.container.health-startup-interval",
            false,
            Some(version(5, 4, 0)),
        ),
        (
            "quadlet.container.health-startup-retries",
            false,
            Some(version(5, 4, 0)),
        ),
        (
            "quadlet.container.health-startup-success",
            false,
            Some(version(5, 4, 0)),
        ),
        (
            "quadlet.container.health-startup-timeout",
            false,
            Some(version(5, 4, 0)),
        ),
        ("quadlet.container.service-name", false, Some(version(5, 4, 0))),
    ] {
        let capability = catalogue.capability(id).ok_or_else(|| format!("missing {id}"))?;
        assert_eq!(capability.unit_types(), ["container"]);
        assert_eq!(capability.sections(), ["Container"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        let target = PodmanTarget::new(version(6, 1, 0), Some(version(6, 1, 0))).map_err(|error| error.to_string())?;
        if let Some(minimum) = minimum {
            let native = capability
                .native_range()
                .ok_or_else(|| format!("{id} lacks native range"))?;
            assert_eq!(native.minimum(), minimum);
            assert_eq!(native.maximum(), version(6, 1, 0));
            assert_eq!(
                catalogue.evaluate(id, target).classification(),
                SupportClassification::Native
            );
            assert!(!capability.evidence().is_empty());
        } else {
            assert!(capability.native_range().is_none());
            assert_eq!(
                catalogue.evaluate(id, target).classification(),
                SupportClassification::Unknown
            );
        }
    }
    for id in [
        "quadlet.container.http-proxy",
        "quadlet.container.retry",
        "quadlet.container.retry-delay",
    ] {
        let target = PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 0))).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate(id, target).classification(),
            SupportClassification::Unsupported
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_arg() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.build-arg")
        .ok_or_else(|| "BuildArg capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-arg"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-7-build-arg",
            "podman-5-8-build-arg",
            "podman-5-7-build-arg-source",
            "podman-5-8-build-arg-source",
            "podman-current-build-arg",
            "podman-5-4-through-5-6-build-arg-rejection",
            "podman-5-7-through-6-0-2-build-arg-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "BuildArg must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 7, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(capability.unsupported_ranges().len(), 1);
    assert_eq!(
        capability.unsupported_ranges()[0].versions().minimum(),
        version(5, 4, 0)
    );
    assert_eq!(
        capability.unsupported_ranges()[0].versions().maximum(),
        version(5, 6, 2)
    );
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 6, 2), SupportClassification::Unsupported),
        (version(5, 7, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.build-arg", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_secret() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.secret")
        .ok_or_else(|| "Build Secret capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-secret"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-secret",
            "podman-6-0-2-build-secret",
            "podman-5-4-build-secret-source",
            "podman-6-0-2-build-secret-source",
            "podman-5-4-through-current-build-secret-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build Secret must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.secret", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args")
        .ok_or_else(|| "Build PodmanArgs capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-podman-args"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args",
            "podman-6-0-2-build-podman-args",
            "podman-5-4-build-podman-args-source",
            "podman-6-0-2-build-podman-args-source",
            "podman-5-4-through-current-build-podman-args-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.podman-args", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_no_cache() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.no-cache")
        .ok_or_else(|| "Build PodmanArgs --no-cache capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--no-cache"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-no-cache",
            "podman-5-4-build-podman-args-no-cache-cli",
            "podman-6-0-2-build-podman-args-no-cache",
            "podman-6-0-2-build-podman-args-no-cache-cli",
            "podman-5-4-build-podman-args-no-cache-source",
            "podman-6-0-2-build-podman-args-no-cache-source",
            "podman-5-4-through-current-build-podman-args-no-cache-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --no-cache must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.no-cache", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_isolation_chroot() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.isolation-chroot")
        .ok_or_else(|| "Build PodmanArgs --isolation=chroot capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--isolation=chroot"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-isolation-chroot",
            "podman-5-4-build-podman-args-isolation-chroot-cli",
            "podman-6-0-2-build-podman-args-isolation-chroot",
            "podman-6-0-2-build-podman-args-isolation-chroot-cli",
            "podman-5-4-build-podman-args-isolation-chroot-source",
            "podman-6-0-2-build-podman-args-isolation-chroot-source",
            "podman-5-4-through-current-build-podman-args-isolation-chroot-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --isolation=chroot must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.isolation-chroot", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_ssh_default() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.ssh-default")
        .ok_or_else(|| "Build PodmanArgs --ssh=default capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--ssh=default"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-ssh-default",
            "podman-5-4-build-podman-args-ssh-default-cli",
            "podman-6-0-2-build-podman-args-ssh-default",
            "podman-6-0-2-build-podman-args-ssh-default-cli",
            "podman-5-4-build-podman-args-ssh-default-source",
            "podman-6-0-2-build-podman-args-ssh-default-source",
            "podman-5-4-through-current-build-podman-args-ssh-default-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --ssh=default must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.ssh-default", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_shm_size_32m() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.shm-size-32m")
        .ok_or_else(|| "Build PodmanArgs --shm-size=32m capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--shm-size=32m"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-shm-size-32m",
            "podman-5-4-build-podman-args-shm-size-32m-cli",
            "podman-6-0-2-build-podman-args-shm-size-32m",
            "podman-6-0-2-build-podman-args-shm-size-32m-cli",
            "podman-5-4-build-podman-args-shm-size-32m-source",
            "podman-6-0-2-build-podman-args-shm-size-32m-source",
            "podman-5-4-through-current-build-podman-args-shm-size-32m-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --shm-size=32m must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.shm-size-32m", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_ulimit_nproc() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.ulimit-nproc")
        .ok_or_else(|| "Build PodmanArgs --ulimit=nproc=4096:8192 capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--ulimit=nproc=4096:8192"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-ulimit-nproc",
            "podman-5-4-build-podman-args-ulimit-nproc-cli",
            "podman-6-0-2-build-podman-args-ulimit-nproc",
            "podman-6-0-2-build-podman-args-ulimit-nproc-cli",
            "podman-5-4-build-podman-args-ulimit-nproc-source",
            "podman-6-0-2-build-podman-args-ulimit-nproc-source",
            "podman-5-4-through-current-build-podman-args-ulimit-nproc-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --ulimit=nproc=4096:8192 must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.ulimit-nproc", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_add_host_buildhost() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.add-host-buildhost")
        .ok_or_else(|| "Build PodmanArgs --add-host=buildhost:192.0.2.10 capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["exact-command-text--add-host=buildhost:192.0.2.10"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-add-host-buildhost",
            "podman-5-4-build-podman-args-add-host-buildhost-cli",
            "podman-6-0-2-build-podman-args-add-host-buildhost",
            "podman-6-0-2-build-podman-args-add-host-buildhost-cli",
            "podman-5-4-build-podman-args-add-host-buildhost-source",
            "podman-6-0-2-build-podman-args-add-host-buildhost-source",
            "podman-5-4-through-current-build-podman-args-add-host-buildhost-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --add-host=buildhost:192.0.2.10 must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.add-host-buildhost", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_cap_add_cap_sys_admin() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.cap-add-cap-sys-admin")
        .ok_or_else(|| "Build PodmanArgs --cap-add=CAP_SYS_ADMIN capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-command-text--cap-add=CAP_SYS_ADMIN"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-cap-add-cap-sys-admin",
            "podman-5-4-build-podman-args-cap-add-cap-sys-admin-cli",
            "podman-6-0-2-build-podman-args-cap-add-cap-sys-admin",
            "podman-6-0-2-build-podman-args-cap-add-cap-sys-admin-cli",
            "podman-5-4-build-podman-args-cap-add-cap-sys-admin-source",
            "podman-6-0-2-build-podman-args-cap-add-cap-sys-admin-source",
            "podman-5-4-through-current-build-podman-args-cap-add-cap-sys-admin-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs --cap-add=CAP_SYS_ADMIN must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.cap-add-cap-sys-admin", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_cache_locations() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.cache-locations")
        .ok_or_else(|| "Build PodmanArgs cache-locations capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["exact-command-text-cache-from", "exact-command-text-cache-to"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-cache-locations",
            "podman-5-4-build-podman-args-cache-from-cli",
            "podman-5-4-build-podman-args-cache-to-cli",
            "podman-6-0-2-build-podman-args-cache-locations",
            "podman-6-0-2-build-podman-args-cache-from-cli",
            "podman-6-0-2-build-podman-args-cache-to-cli",
            "podman-5-4-build-podman-args-cache-locations-source",
            "podman-6-0-2-build-podman-args-cache-locations-source",
            "podman-5-4-through-current-build-podman-args-cache-locations-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs cache locations must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.cache-locations", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_podman_args_sbom() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.podman-args.sbom")
        .ok_or_else(|| "Build PodmanArgs SBOM capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        [
            "exact-command-text--sbom=syft",
            "exact-command-text--sbom-output=/tmp/quadlet-lens-sbom.json",
        ]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-podman-args-sbom",
            "podman-5-4-build-podman-args-sbom-preset-cli",
            "podman-5-4-build-podman-args-sbom-output-cli",
            "podman-6-0-2-build-podman-args-sbom",
            "podman-6-0-2-build-podman-args-sbom-preset-cli",
            "podman-6-0-2-build-podman-args-sbom-output-cli",
            "podman-5-4-build-podman-args-sbom-source",
            "podman-6-0-2-build-podman-args-sbom-source",
            "podman-5-4-through-current-build-podman-args-sbom-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build PodmanArgs SBOM must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.build.podman-args.sbom", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_platform() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, value_forms, evidence) in [
        (
            "quadlet.build.arch",
            &["opaque-one-line-architecture"][..],
            [
                "podman-5-4-build-arch",
                "podman-6-0-2-build-arch",
                "podman-5-4-build-arch-singleton-source",
                "podman-6-0-2-build-arch-singleton-source",
                "podman-5-4-through-current-build-arch-generators",
            ],
        ),
        (
            "quadlet.build.variant",
            &["opaque-one-line-architecture-variant"][..],
            [
                "podman-5-4-build-variant",
                "podman-6-0-2-build-variant",
                "podman-5-4-build-variant-singleton-source",
                "podman-6-0-2-build-variant-singleton-source",
                "podman-5-4-through-current-build-variant-generators",
            ],
        ),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), ["build"]);
        assert_eq!(capability.sections(), ["Build"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), value_forms);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_build_pull() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.pull")
        .ok_or_else(|| "Build Pull capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-pull-policy"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-pull",
            "podman-6-0-2-build-pull",
            "podman-5-4-build-pull-singleton-source",
            "podman-6-0-2-build-pull-singleton-source",
            "podman-5-4-through-current-build-pull-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build Pull must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.pull", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_retry() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, value_forms, evidence) in [
        (
            "quadlet.build.retry",
            &[("opaque-one-line-retry-attempts-separate-flag-value")][..],
            [
                "podman-5-5-build-retry",
                "podman-6-0-2-build-retry",
                "podman-5-5-build-retry-source",
                "podman-6-0-2-build-retry-source",
                "podman-5-4-build-retry-generator-rejection",
                "podman-5-5-through-current-build-retry-generators",
            ],
        ),
        (
            "quadlet.build.retry-delay",
            &[("opaque-one-line-retry-delay-separate-flag-value")][..],
            [
                "podman-5-5-build-retry-delay",
                "podman-6-0-2-build-retry-delay",
                "podman-5-5-build-retry-delay-source",
                "podman-6-0-2-build-retry-delay-source",
                "podman-5-4-build-retry-generator-rejection",
                "podman-5-5-through-current-build-retry-generators",
            ],
        ),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), ["build"]);
        assert_eq!(capability.sections(), ["Build"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), value_forms);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 5, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        assert_eq!(capability.unsupported_ranges().len(), 1);
        assert_eq!(
            capability.unsupported_ranges()[0].versions().minimum(),
            version(5, 4, 0)
        );
        assert_eq!(
            capability.unsupported_ranges()[0].versions().maximum(),
            version(5, 4, 2)
        );
        for (target, expected) in [
            (version(5, 3, 3), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Unsupported),
            (version(5, 4, 2), SupportClassification::Unsupported),
            (version(5, 5, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_build_tls_verify() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.tls-verify")
        .ok_or_else(|| "Build TLSVerify capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-tls-verify"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-tls-verify",
            "podman-6-0-2-build-tls-verify",
            "podman-5-4-build-tls-verify-source",
            "podman-6-0-2-build-tls-verify-source",
            "podman-5-4-through-current-build-tls-verify-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build TLSVerify must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.tls-verify", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_force_rm() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.force-rm")
        .ok_or_else(|| "Build ForceRM capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-force-rm"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-force-rm",
            "podman-6-0-2-build-force-rm",
            "podman-5-4-build-force-rm-source",
            "podman-6-0-2-build-force-rm-source",
            "podman-5-4-through-current-build-force-rm-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build ForceRM must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.force-rm", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_group_add() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.group-add")
        .ok_or_else(|| "Build GroupAdd capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-group-add"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-group-add",
            "podman-6-0-2-build-group-add",
            "podman-5-4-build-group-add-source",
            "podman-6-0-2-build-group-add-source",
            "podman-5-4-through-current-build-group-add-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build GroupAdd must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.group-add", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_dns() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.dns")
        .ok_or_else(|| "Build DNS capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-dns"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-dns",
            "podman-6-0-2-build-dns",
            "podman-5-4-build-dns-source",
            "podman-6-0-2-build-dns-source",
            "podman-5-4-through-current-build-dns-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "Build DNS must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.build.dns", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_dns_option() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.dns-option")
        .ok_or("Build DNSOption missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-dns-option"]);
    assert_eq!(
        capability
            .native_range()
            .ok_or("Build DNSOption range missing")?
            .minimum(),
        version(5, 4, 0)
    );
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.dns-option",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_dns_search() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.dns-search")
        .ok_or("Build DNSSearch missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-dns-search"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-dns-search",
            "podman-6-0-2-build-dns-search",
            "podman-5-4-build-dns-search-source",
            "podman-6-0-2-build-dns-search-source",
            "podman-5-4-through-current-build-dns-search-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build DNSSearch range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.dns-search",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_auth_file() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.auth-file")
        .ok_or("Build AuthFile missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-auth-file"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-auth-file",
            "podman-6-0-2-build-auth-file",
            "podman-5-4-build-auth-file-source",
            "podman-6-0-2-build-auth-file-source",
            "podman-5-4-through-current-build-auth-file-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build AuthFile range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.auth-file",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_ignore_file() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.ignore-file")
        .ok_or("Build IgnoreFile missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-ignore-file"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-7-build-ignore-file",
            "podman-5-8-build-ignore-file",
            "podman-current-build-ignore-file",
            "podman-5-7-build-ignore-file-source",
            "podman-5-8-build-ignore-file-source",
            "podman-5-4-through-5-6-build-ignore-file-rejection",
            "podman-5-7-through-6-0-2-build-ignore-file-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build IgnoreFile range missing")?;
    assert_eq!(native.minimum(), version(5, 7, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(capability.unsupported_ranges().len(), 1);
    assert_eq!(
        capability.unsupported_ranges()[0].versions().minimum(),
        version(5, 4, 0)
    );
    assert_eq!(
        capability.unsupported_ranges()[0].versions().maximum(),
        version(5, 6, 2)
    );
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 6, 2), SupportClassification::Unsupported),
        (version(5, 7, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.ignore-file",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_annotation() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.annotation")
        .ok_or("Build Annotation missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-annotation"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-annotation",
            "podman-6-0-2-build-annotation",
            "podman-5-4-build-annotation-source",
            "podman-6-0-2-build-annotation-source",
            "podman-5-4-build-annotation-reset-source",
            "podman-6-0-2-build-annotation-reset-source",
            "podman-5-4-through-current-build-annotation-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build Annotation range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.annotation",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_environment() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.environment")
        .ok_or("Build Environment missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-environment"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-environment",
            "podman-6-0-2-build-environment",
            "podman-5-4-build-environment-source",
            "podman-6-0-2-build-environment-source",
            "podman-5-4-build-environment-parser-source",
            "podman-5-6-build-environment-parser-source",
            "podman-5-4-through-current-build-environment-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build Environment range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.environment",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_containers_conf_module() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.containers-conf-module")
        .ok_or("Build ContainersConfModule missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["opaque-one-line-build-containers-conf-module"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-containers-conf-module",
            "podman-6-0-2-build-containers-conf-module",
            "podman-5-4-containers-conf-module-base-command-source",
            "podman-6-0-2-containers-conf-module-base-command-source",
            "podman-5-4-containers-conf-module-parser-source",
            "podman-6-0-2-containers-conf-module-parser-source",
            "podman-5-4-through-current-build-containers-conf-module-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or("Build ContainersConfModule range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.containers-conf-module",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_global_args() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.global-args")
        .ok_or("Build GlobalArgs missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-global-args"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-global-args",
            "podman-6-0-2-build-global-args",
            "podman-5-4-through-current-build-global-args-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build GlobalArgs range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.global-args",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_service_name() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.service-name")
        .ok_or("Build ServiceName missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-build-service-name"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-through-5-8-5-build-service-name-docs-gap",
            "podman-6-0-0-build-service-name",
            "podman-6-0-2-build-service-name",
            "podman-5-4-through-current-build-service-name-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build ServiceName range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.service-name",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_volume() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.build.volume")
        .ok_or("Build Volume missing")?;
    assert_eq!(capability.unit_types(), ["build"]);
    assert_eq!(capability.sections(), ["Build"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["opaque-one-line-build-volume", "volume-unit-reference"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-build-volume",
            "podman-6-0-2-build-volume",
            "podman-5-4-build-volume-source",
            "podman-6-0-2-build-volume-source",
            "podman-5-4-through-current-build-volume-generators",
        ]
    );
    let native = capability.native_range().ok_or("Build Volume range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.build.volume",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_containers_conf_module() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.containers-conf-module")
        .ok_or("Volume ContainersConfModule missing")?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["opaque-one-line-volume-containers-conf-module"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-containers-conf-module",
            "podman-6-0-2-volume-containers-conf-module",
            "podman-5-4-volume-containers-conf-module-source",
            "podman-6-0-2-volume-containers-conf-module-source",
            "podman-5-4-through-current-volume-containers-conf-module-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or("Volume ContainersConfModule range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.containers-conf-module",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_global_args() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.global-args")
        .ok_or("Volume GlobalArgs missing")?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-global-args"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-global-args",
            "podman-6-0-2-volume-global-args",
            "podman-5-4-volume-global-args-source",
            "podman-6-0-2-volume-global-args-source",
            "podman-5-4-through-current-volume-global-args-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume GlobalArgs range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.global-args",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_podman_args() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.podman-args")
        .ok_or("Volume PodmanArgs missing")?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-podman-args"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-podman-args",
            "podman-6-0-2-volume-podman-args",
            "podman-5-4-volume-podman-args-source",
            "podman-6-0-2-volume-podman-args-source",
            "podman-5-4-through-current-volume-podman-args-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume PodmanArgs range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.podman-args",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_user() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.user")
        .ok_or("Volume User missing")?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-user"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-user",
            "podman-6-0-2-volume-user",
            "podman-5-4-volume-user-source",
            "podman-6-0-2-volume-user-source",
            "podman-5-4-through-current-volume-user-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume User range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.user",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_group() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.group")
        .ok_or("Volume Group missing")?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-group"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-group",
            "podman-6-0-2-volume-group",
            "podman-5-4-volume-group-source",
            "podman-6-0-2-volume-group-source",
            "podman-5-4-through-current-volume-group-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume Group range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.group",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_uid() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue.capability("quadlet.volume.uid").ok_or("Volume UID missing")?;
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-uid"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-8-5-volume-uid-absence",
            "podman-6-0-0-volume-uid",
            "podman-6-0-2-volume-uid",
            "podman-6-0-0-volume-uid-source",
            "podman-6-0-2-volume-uid-source",
            "podman-6-0-volume-uid-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume UID range missing")?;
    assert_eq!(native.minimum(), version(6, 0, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 8, 5), SupportClassification::Unsupported),
        (version(6, 0, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.uid",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_gid() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue.capability("quadlet.volume.gid").ok_or("Volume GID missing")?;
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-gid"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-8-5-volume-gid-absence",
            "podman-6-0-0-volume-gid",
            "podman-6-0-2-volume-gid",
            "podman-6-0-0-volume-gid-source",
            "podman-6-0-2-volume-gid-source",
            "podman-6-0-volume-gid-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume GID range missing")?;
    assert_eq!(native.minimum(), version(6, 0, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 8, 5), SupportClassification::Unsupported),
        (version(6, 0, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.gid",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_service_name() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.service-name")
        .ok_or("Volume ServiceName missing")?;
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-service-name"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-through-5-8-5-volume-service-name-docs-gap",
            "podman-6-0-0-volume-service-name",
            "podman-6-0-2-volume-service-name",
            "podman-5-4-volume-service-name-source",
            "podman-6-0-2-volume-service-name-source",
            "podman-5-4-through-current-volume-service-name-generators",
        ]
    );
    let native = capability.native_range().ok_or("Volume ServiceName range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.service-name",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_image() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.image")
        .ok_or("Volume Image missing")?;
    assert!(!capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["opaque-one-line-volume-image", "exact-image-or-build-unit-reference"]
    );
    let native = capability.native_range().ok_or("Volume Image range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.volume.image",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_core() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.image")
        .ok_or("Image core missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-source"]);
    let native = capability.native_range().ok_or("Image core range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.image",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_image_tag() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.image-tag")
        .ok_or("ImageTag missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-tag"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-image-tag",
            "podman-6-0-2-image-image-tag",
            "podman-5-4-image-image-tag-source",
            "podman-6-0-2-image-image-tag-source",
            "podman-5-4-through-current-image-image-tag-generators",
        ]
    );
    let native = capability.native_range().ok_or("ImageTag range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.image-tag",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_service_name() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.service-name")
        .ok_or("Image ServiceName missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-service-name"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-through-6-0-2-image-service-name-docs-gap",
            "podman-5-4-image-service-name-source",
            "podman-6-0-2-image-service-name-source",
            "podman-5-4-through-current-image-service-name-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image ServiceName range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.service-name",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_all_tags() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.all-tags")
        .ok_or("Image AllTags missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-all-tags"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-all-tags",
            "podman-6-0-2-image-all-tags",
            "podman-5-4-image-all-tags-source",
            "podman-6-0-2-image-all-tags-source",
            "podman-5-4-through-current-image-all-tags-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image AllTags range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.all-tags",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_arch() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue.capability("quadlet.image.arch").ok_or("Image Arch missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-architecture"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-arch",
            "podman-6-0-2-image-arch",
            "podman-5-4-image-arch-source",
            "podman-6-0-2-image-arch-source",
            "podman-5-4-through-current-image-arch-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image Arch range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.arch",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_auth_file() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.auth-file")
        .ok_or("Image AuthFile missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-auth-file"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-auth-file",
            "podman-6-0-2-image-auth-file",
            "podman-5-4-image-auth-file-source",
            "podman-6-0-2-image-auth-file-source",
            "podman-5-4-through-current-image-auth-file-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image AuthFile range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.auth-file",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_cert_dir() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.cert-dir")
        .ok_or("Image CertDir missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-cert-dir"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-cert-dir",
            "podman-6-0-2-image-cert-dir",
            "podman-5-4-image-cert-dir-source",
            "podman-6-0-2-image-cert-dir-source",
            "podman-5-4-through-current-image-cert-dir-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image CertDir range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.cert-dir",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_containers_conf_module() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.containers-conf-module")
        .ok_or("Image ContainersConfModule missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        ["opaque-one-line-image-containers-conf-module"]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-containers-conf-module",
            "podman-6-0-2-image-containers-conf-module",
            "podman-5-4-image-containers-conf-module-base-command-source",
            "podman-6-0-2-image-containers-conf-module-base-command-source",
            "podman-5-4-image-containers-conf-module-parser-source",
            "podman-6-0-2-image-containers-conf-module-parser-source",
            "podman-5-4-through-current-image-containers-conf-module-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or("Image ContainersConfModule range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.containers-conf-module",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_creds() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.creds")
        .ok_or("Image Creds missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-creds"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-creds",
            "podman-6-0-2-image-creds",
            "podman-5-4-image-creds-source",
            "podman-6-0-2-image-creds-source",
            "podman-5-4-through-current-image-creds-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image Creds range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.creds",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_decryption_key() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.decryption-key")
        .ok_or("Image DecryptionKey missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-decryption-key"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-decryption-key",
            "podman-6-0-2-image-decryption-key",
            "podman-5-4-image-decryption-key-source",
            "podman-6-0-2-image-decryption-key-source",
            "podman-5-4-through-current-image-decryption-key-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image DecryptionKey range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.decryption-key",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_global_args() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.image.global-args")
        .ok_or("Image GlobalArgs missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-global-args"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-global-args",
            "podman-6-0-2-image-global-args",
            "podman-5-4-image-global-args-source",
            "podman-6-0-2-image-global-args-source",
            "podman-5-4-through-current-image-global-args-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image GlobalArgs range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.global-args",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_image_os() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue.capability("quadlet.image.os").ok_or("Image OS missing")?;
    assert_eq!(capability.unit_types(), ["image"]);
    assert_eq!(capability.sections(), ["Image"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-image-os"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-image-os",
            "podman-6-0-2-image-os",
            "podman-5-4-image-os-source",
            "podman-6-0-2-image-os-source",
            "podman-5-4-through-current-image-os-generators",
        ]
    );
    let native = capability.native_range().ok_or("Image OS range missing")?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        assert_eq!(
            catalogue
                .evaluate(
                    "quadlet.image.os",
                    PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?
                )
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_build_core() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, value_forms) in [
        ("quadlet.unit-type.build", false, &[][..]),
        ("quadlet.build.image-tag", true, &["image-reference"][..]),
        (
            "quadlet.build.network",
            true,
            &["opaque-one-line-build-network", "network-unit-reference"][..],
        ),
        ("quadlet.build.label", true, &["opaque-one-line-build-label"][..]),
        ("quadlet.build.set-working-directory", false, &["build-context"][..]),
        ("quadlet.build.file", true, &["opaque-one-line-file"][..]),
        ("quadlet.build.target", false, &["opaque-one-line-target"][..]),
        ("quadlet.build.secret", true, &["opaque-one-line-build-secret"][..]),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), ["build"]);
        assert_eq!(capability.sections(), ["Build"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(capability.value_forms(), value_forms);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_container_podman_args_interactive() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.podman-args.interactive")
        .ok_or_else(|| "container PodmanArgs interactive capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-podman-run-interactive-flag"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-podman-args-interactive",
            "podman-6-0-2-container-podman-args-interactive",
            "podman-5-4-container-podman-args-interactive-command-source",
            "podman-6-0-2-container-podman-args-interactive-command-source",
            "podman-5-4-through-current-container-podman-args-interactive-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container PodmanArgs interactive must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.podman-args.interactive", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_podman_args_tty() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.podman-args.tty")
        .ok_or_else(|| "container PodmanArgs TTY capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["exact-podman-run-tty-flag"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-podman-args-tty",
            "podman-6-0-2-container-podman-args-tty",
            "podman-5-4-container-podman-args-tty-command-source",
            "podman-6-0-2-container-podman-args-tty-command-source",
            "podman-5-4-through-current-container-podman-args-tty-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container PodmanArgs TTY must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.podman-args.tty", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_container_podman_args_privileged() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.podman-args.privileged")
        .ok_or_else(|| "container PodmanArgs privileged capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(
        capability.value_forms(),
        [
            "exact-podman-run-privileged-true-flag",
            "exact-podman-run-privileged-false-flag",
        ]
    );
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-podman-args-privileged",
            "podman-6-0-2-container-podman-args-privileged",
            "podman-5-4-container-podman-args-privileged-command-source",
            "podman-6-0-2-container-podman-args-privileged-command-source",
            "podman-5-4-container-podman-args-privileged-cli",
            "podman-6-0-2-container-podman-args-privileged-cli",
            "podman-5-4-through-current-container-podman-args-privileged-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container PodmanArgs privileged must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.podman-args.privileged", target)
                .classification(),
            expected
        );
    }
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

    let current = PodmanTarget::new(version(6, 1, 0), Some(version(6, 1, 0))).map_err(|error| error.to_string())?;
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
fn supported_range_records_systemd_unit_relationships_and_rewrite_boundary() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for capability in [
        "systemd.unit.requires",
        "systemd.unit.wants",
        "systemd.unit.after",
        "systemd.unit.requisite",
        "systemd.unit.binds-to",
        "systemd.unit.part-of",
        "systemd.unit.upholds",
        "systemd.unit.conflicts",
        "systemd.unit.before",
    ] {
        let record = catalogue
            .capability(capability)
            .ok_or_else(|| format!("{capability} capability must exist"))?;
        assert_eq!(record.sections(), ["Unit"]);
        assert!(record.is_repeatable());
        assert_eq!(record.value_forms(), ["systemd-unit-list"]);
        let native = record
            .native_range()
            .ok_or_else(|| format!("{capability} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
    }

    let upholds = catalogue
        .capability("systemd.unit.upholds")
        .ok_or_else(|| "Upholds capability must exist".to_owned())?;
    assert_eq!(upholds.systemd_evidence(), ["systemd-249-upholds"]);
    let systemd_evidence = catalogue
        .systemd_evidence()
        .iter()
        .find(|evidence| evidence.id() == "systemd-249-upholds")
        .ok_or_else(|| "Upholds systemd evidence must exist".to_owned())?;
    assert_eq!(systemd_evidence.versions().minimum().release(), 249);
    assert!(systemd_evidence.url().contains("/249/"));

    let rewrite = catalogue
        .capability("systemd.unit.quadlet-reference-rewrite")
        .ok_or_else(|| "Quadlet relationship rewrite capability must exist".to_owned())?;
    assert_eq!(rewrite.sections(), ["Unit"]);
    assert!(rewrite.is_repeatable());
    assert_eq!(rewrite.value_forms(), ["native-quadlet-unit-list-rewrite"]);
    let native = rewrite
        .native_range()
        .ok_or_else(|| "rewrite capability must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 5, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(rewrite.unsupported_ranges().len(), 1);

    for (target, expected) in [
        (version(5, 4, 2), SupportClassification::Unsupported),
        (version(5, 5, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("systemd.unit.quadlet-reference-rewrite", target)
                .classification(),
            expected
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
        assert_eq!(native.maximum(), version(6, 1, 0));

        for target in [
            PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 0))),
            PodmanTarget::new(version(6, 1, 0), Some(version(6, 1, 0))),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
        assert_eq!(native.maximum(), version(6, 1, 0));

        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_pod_exit_policy() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.pod.exit-policy")
        .ok_or_else(|| "pod exit-policy capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["pod"]);
    assert_eq!(capability.sections(), ["Pod"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["continue", "stop"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-6-pod-exit-policy",
            "podman-6-0-2-pod-exit-policy",
            "podman-5-6-pod-exit-policy-cli",
            "podman-6-0-2-pod-exit-policy-cli",
            "podman-5-6-pod-exit-policy-source",
            "podman-6-0-2-pod-exit-policy-source",
            "podman-5-4-through-5-5-pod-exit-policy-generator-rejection",
            "podman-5-6-through-current-pod-exit-policy-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "pod exit-policy must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 6, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(capability.unsupported_ranges().len(), 1);
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 5, 2), SupportClassification::Unsupported),
        (version(5, 6, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.pod.exit-policy", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_pod_stop_timeout() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.pod.stop-timeout")
        .ok_or_else(|| "pod stop-timeout capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["pod"]);
    assert_eq!(capability.sections(), ["Pod"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-pod-stop-timeout"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-7-pod-stop-timeout",
            "podman-6-0-2-pod-stop-timeout",
            "podman-5-6-pod-stop-timeout-source-rejection",
            "podman-5-7-pod-stop-timeout-source",
            "podman-6-0-2-pod-stop-timeout-source",
            "podman-5-7-pod-stop-timeout-cli",
            "podman-5-4-through-5-6-pod-stop-timeout-generator-rejection",
            "podman-5-7-through-current-pod-stop-timeout-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "pod stop-timeout must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 7, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(capability.unsupported_ranges().len(), 1);
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 6, 2), SupportClassification::Unsupported),
        (version(5, 7, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.pod.stop-timeout", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_pod_service_name() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.pod.service-name")
        .ok_or_else(|| "pod service-name capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["pod"]);
    assert_eq!(capability.sections(), ["Pod"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-pod-service-name"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-pod-service-name",
            "podman-6-0-2-pod-service-name",
            "podman-5-4-pod-service-name-source",
            "podman-6-0-2-pod-service-name-source",
            "podman-5-4-through-current-pod-service-name-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "pod service-name must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert!(capability.unsupported_ranges().is_empty());
    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.pod.service-name", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_pod_completion_keys() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, native_minimum, unsupported_maximum) in [
        ("quadlet.pod.containers-conf-module", true, version(5, 4, 0), None),
        ("quadlet.pod.dns", true, version(5, 4, 0), None),
        ("quadlet.pod.dns-option", true, version(5, 4, 0), None),
        ("quadlet.pod.dns-search", true, version(5, 4, 0), None),
        ("quadlet.pod.gid-map", true, version(5, 4, 0), None),
        ("quadlet.pod.global-args", true, version(5, 4, 0), None),
        ("quadlet.pod.hostname", false, version(5, 5, 0), Some(version(5, 4, 2))),
        ("quadlet.pod.ip", false, version(5, 4, 0), None),
        ("quadlet.pod.ip6", false, version(5, 4, 0), None),
        ("quadlet.pod.label", true, version(5, 6, 0), Some(version(5, 5, 2))),
        ("quadlet.pod.network-alias", true, version(5, 4, 0), None),
        ("quadlet.pod.podman-args", true, version(5, 4, 0), None),
        ("quadlet.pod.subgid-map", false, version(5, 4, 0), None),
        ("quadlet.pod.subuid-map", false, version(5, 4, 0), None),
        ("quadlet.pod.uid-map", true, version(5, 4, 0), None),
    ] {
        let capability = catalogue.capability(id).ok_or_else(|| format!("missing {id}"))?;
        assert_eq!(capability.unit_types(), ["pod"]);
        assert_eq!(capability.sections(), ["Pod"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(
            capability
                .native_range()
                .map(quadlet_lens::capability::VersionRange::minimum),
            Some(native_minimum)
        );
        assert_eq!(
            capability
                .native_range()
                .map(quadlet_lens::capability::VersionRange::maximum),
            Some(version(6, 1, 0))
        );
        let expected_evidence: &[&str] = if id == "quadlet.pod.hostname" || id == "quadlet.pod.label" {
            &[
                "podman-6-0-2-pod-completion-keys",
                "podman-5-4-through-current-pod-completion-generators",
            ]
        } else {
            &[
                "podman-5-4-pod-completion-keys",
                "podman-6-0-2-pod-completion-keys",
                "podman-5-4-through-current-pod-completion-generators",
            ]
        };
        assert_eq!(capability.evidence(), expected_evidence);
        match unsupported_maximum {
            Some(maximum) => {
                assert_eq!(capability.unsupported_ranges().len(), 1);
                assert_eq!(
                    capability.unsupported_ranges()[0].versions().minimum(),
                    version(5, 4, 0)
                );
                assert_eq!(capability.unsupported_ranges()[0].versions().maximum(), maximum);
                let target = PodmanTarget::new(maximum, Some(maximum)).map_err(|error| error.to_string())?;
                assert_eq!(
                    catalogue.evaluate(id, target).classification(),
                    SupportClassification::Unsupported
                );
            }
            None => assert!(capability.unsupported_ranges().is_empty()),
        }
        let native_target =
            PodmanTarget::new(native_minimum, Some(native_minimum)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate(id, native_target).classification(),
            SupportClassification::Native
        );
        let target = PodmanTarget::new(version(6, 1, 0), Some(version(6, 1, 0))).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate(id, target).classification(),
            SupportClassification::Native
        );
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
fn supported_range_records_container_logging() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, value_form, evidence) in [
        (
            "quadlet.container.log-driver",
            false,
            "opaque-one-line-log-driver",
            [
                "podman-5-4-container-log-driver",
                "podman-6-0-2-container-log-driver",
                "podman-5-4-container-logging-command-source",
                "podman-6-0-2-container-logging-command-source",
                "podman-5-4-container-log-driver-lookup-source",
                "podman-6-0-2-container-log-driver-lookup-source",
                "podman-5-4-through-current-container-logging-generators",
            ],
        ),
        (
            "quadlet.container.log-opt",
            true,
            "opaque-one-line-log-option",
            [
                "podman-5-4-container-log-opt",
                "podman-6-0-2-container-log-opt",
                "podman-5-4-container-logging-command-source",
                "podman-6-0-2-container-logging-command-source",
                "podman-5-4-container-log-opt-reset-source",
                "podman-6-0-2-container-log-opt-reset-source",
                "podman-5-4-through-current-container-logging-generators",
            ],
        ),
    ] {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["container"]);
        assert_eq!(capability.sections(), ["Container"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));

        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_container_network_identity() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, repeatable, value_form, evidence) in [
        (
            "quadlet.container.ip",
            false,
            "opaque-one-line-ipv4-address",
            [
                "podman-5-4-container-ip",
                "podman-6-0-2-container-ip",
                "podman-5-4-through-current-container-network-identity-generators",
            ],
        ),
        (
            "quadlet.container.ip6",
            false,
            "opaque-one-line-ipv6-address",
            [
                "podman-5-4-container-ip6",
                "podman-6-0-2-container-ip6",
                "podman-5-4-through-current-container-network-identity-generators",
            ],
        ),
        (
            "quadlet.container.network-alias",
            true,
            "opaque-one-line-network-alias",
            [
                "podman-5-4-container-network-alias",
                "podman-6-0-2-container-network-alias",
                "podman-5-4-through-current-container-network-identity-generators",
            ],
        ),
    ] {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["container"]);
        assert_eq!(capability.sections(), ["Container"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));

        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_network_driver_and_options() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let cases: [(&str, bool, &str, &[&str]); 2] = [
        (
            "quadlet.network.driver",
            false,
            "opaque-one-line-network-driver",
            &[
                "podman-5-4-network-driver",
                "podman-6-0-2-network-driver",
                "podman-5-4-network-driver-command-source",
                "podman-6-0-2-network-driver-command-source",
                "podman-5-4-network-driver-lookup-source",
                "podman-6-0-2-network-driver-lookup-source",
                "podman-5-4-through-current-network-driver-options-generators",
            ],
        ),
        (
            "quadlet.network.options",
            true,
            "opaque-one-line-network-option",
            &[
                "podman-5-4-network-options",
                "podman-6-0-2-network-options",
                "podman-5-4-network-options-command-source",
                "podman-6-0-2-network-options-command-source",
                "podman-5-4-network-options-reset-source",
                "podman-5-4-network-options-collapse-bare-source",
                "podman-5-4-network-options-sort-source",
                "podman-6-0-2-network-options-reset-source",
                "podman-6-0-2-network-options-collapse-bare-source",
                "podman-6-0-2-network-options-sort-bare-source",
                "podman-5-4-through-current-network-driver-options-generators",
            ],
        ),
    ];
    for (id, repeatable, value_form, evidence) in cases {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["network"]);
        assert_eq!(capability.sections(), ["Network"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));

        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_driver_options_device_type_and_copy() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let cases: [(&str, &str, &[&str]); 5] = [
        (
            "quadlet.volume.driver",
            "opaque-one-line-volume-driver",
            &[
                "podman-5-4-volume-driver",
                "podman-6-0-2-volume-driver",
                "podman-5-4-volume-options-command-source",
                "podman-6-0-2-volume-options-command-source",
                "podman-5-4-volume-options-lookup-source",
                "podman-6-0-2-volume-options-lookup-source",
                "podman-5-4-through-current-volume-driver-options-generators",
            ],
        ),
        (
            "quadlet.volume.options",
            "opaque-one-line-volume-mount-options",
            &[
                "podman-5-4-volume-options-command-source",
                "podman-6-0-2-volume-options-command-source",
                "podman-5-4-volume-options-lookup-source",
                "podman-6-0-2-volume-options-lookup-source",
                "podman-5-8-2-volume-options-unmatched-quote-source",
                "podman-5-4-through-current-volume-driver-options-generators",
                "podman-5-4-through-5-8-volume-options-device-rejection",
                "podman-6-0-through-current-volume-options-without-device-generators",
                "podman-5-4-through-current-volume-options-unmatched-quote-generators",
            ],
        ),
        (
            "quadlet.volume.device",
            "opaque-one-line-volume-device",
            &[
                "podman-5-4-volume-device",
                "podman-6-0-2-volume-device",
                "podman-5-4-volume-options-command-source",
                "podman-6-0-2-volume-options-command-source",
                "podman-5-4-volume-options-lookup-source",
                "podman-6-0-2-volume-options-lookup-source",
                "podman-5-8-2-volume-options-unmatched-quote-source",
                "podman-5-4-through-current-volume-device-type-generators",
                "podman-5-4-through-current-volume-type-bind-requires-mounts-generators",
            ],
        ),
        (
            "quadlet.volume.type",
            "opaque-one-line-volume-type",
            &[
                "podman-5-4-volume-type",
                "podman-6-0-2-volume-type",
                "podman-5-4-volume-options-command-source",
                "podman-6-0-2-volume-options-command-source",
                "podman-5-4-volume-options-lookup-source",
                "podman-6-0-2-volume-options-lookup-source",
                "podman-5-8-2-volume-options-unmatched-quote-source",
                "podman-5-4-through-current-volume-device-type-generators",
                "podman-5-4-through-current-volume-type-without-device-rejection",
                "podman-5-4-through-current-volume-type-bind-requires-mounts-generators",
            ],
        ),
        (
            "quadlet.volume.copy",
            "opaque-one-line-volume-copy",
            &[
                "podman-5-4-volume-copy",
                "podman-6-0-2-volume-copy",
                "podman-5-4-volume-copy-command-source",
                "podman-6-0-2-volume-copy-command-source",
                "podman-5-4-volume-copy-parser-source",
                "podman-6-0-2-volume-copy-parser-source",
                "podman-5-8-2-volume-copy-unmatched-quote-parser-source",
                "podman-5-4-through-current-volume-copy-generators",
            ],
        ),
    ];
    for (id, value_form, evidence) in cases {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["volume"]);
        assert_eq!(capability.sections(), ["Volume"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_network_labels() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.network.label")
        .ok_or_else(|| "network label capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["network"]);
    assert_eq!(capability.sections(), ["Network"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-network-label"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-network-label",
            "podman-6-0-2-network-label",
            "podman-5-4-network-label-command-source",
            "podman-6-0-2-network-label-command-source",
            "podman-5-4-network-label-reset-source",
            "podman-6-0-2-network-label-reset-source",
            "podman-5-4-network-label-tokenization-source",
            "podman-6-0-2-network-label-tokenization-source",
            "podman-5-4-network-label-sort-source",
            "podman-6-0-2-network-label-sort-source",
            "podman-5-4-through-current-network-label-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "network label must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.network.label", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_volume_labels() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.volume.label")
        .ok_or_else(|| "volume label capability must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["volume"]);
    assert_eq!(capability.sections(), ["Volume"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-volume-label"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-volume-label",
            "podman-6-0-2-volume-label",
            "podman-5-4-volume-label-command-source",
            "podman-6-0-2-volume-label-command-source",
            "podman-5-4-volume-label-reset-source",
            "podman-6-0-2-volume-label-reset-source",
            "podman-5-4-volume-label-parser-source",
            "podman-6-0-2-volume-label-parser-source",
            "podman-5-4-volume-label-helper-source",
            "podman-6-0-2-volume-label-helper-source",
            "podman-5-4-through-current-volume-label-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "volume label must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.volume.label", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_network_ipam_columns() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let cases: [(&str, bool, &str); 4] = [
        ("quadlet.network.ipam-driver", false, "opaque-one-line-ipam-driver"),
        ("quadlet.network.subnet", true, "opaque-one-line-network-subnet"),
        ("quadlet.network.gateway", true, "opaque-one-line-network-gateway"),
        ("quadlet.network.ip-range", true, "opaque-one-line-network-ip-range"),
    ];
    for (id, repeatable, value_form) in cases {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["network"]);
        assert_eq!(capability.sections(), ["Network"]);
        assert_eq!(capability.is_repeatable(), repeatable);
        assert_eq!(capability.value_forms(), [value_form]);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_network_internal_and_ipv6() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let cases: [(&str, &str, &[&str]); 2] = [
        (
            "quadlet.network.internal",
            "opaque-one-line-network-internal",
            &[
                "podman-5-4-network-internal",
                "podman-6-0-2-network-internal",
                "podman-5-4-network-boolean-command-source",
                "podman-6-0-2-network-boolean-command-source",
                "podman-5-4-network-boolean-lookup-source",
                "podman-6-0-2-network-boolean-lookup-source",
                "podman-5-4-through-current-network-boolean-generators",
            ],
        ),
        (
            "quadlet.network.ipv6",
            "opaque-one-line-network-ipv6",
            &[
                "podman-5-4-network-ipv6",
                "podman-6-0-2-network-ipv6",
                "podman-5-4-network-boolean-command-source",
                "podman-6-0-2-network-boolean-command-source",
                "podman-5-4-network-boolean-lookup-source",
                "podman-6-0-2-network-boolean-lookup-source",
                "podman-5-4-through-current-network-boolean-generators",
            ],
        ),
    ];
    for (id, value_form, evidence) in cases {
        let capability = catalogue.capability(id).ok_or_else(|| format!("{id} must exist"))?;
        assert_eq!(capability.unit_types(), ["network"]);
        assert_eq!(capability.sections(), ["Network"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 4, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        for (target, expected) in [
            (version(5, 3, 0), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_dns() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.dns")
        .ok_or_else(|| "container dns must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-resolver-value"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-dns",
            "podman-6-0-2-container-dns",
            "podman-5-4-container-dns-command-source",
            "podman-6-0-2-container-dns-command-source",
            "podman-5-4-container-dns-lookup-all-source",
            "podman-6-0-2-container-dns-lookup-all-source",
            "podman-5-4-container-dns-reset-source",
            "podman-6-0-2-container-dns-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container dns must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.dns", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_dns_option() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.dns-option")
        .ok_or_else(|| "container dns-option must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-resolver-option"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-dns-option",
            "podman-6-0-2-container-dns-option",
            "podman-5-4-container-dns-option-command-source",
            "podman-6-0-2-container-dns-option-command-source",
            "podman-5-4-container-dns-option-lookup-all-source",
            "podman-6-0-2-container-dns-option-lookup-all-source",
            "podman-5-4-container-dns-option-reset-source",
            "podman-6-0-2-container-dns-option-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container dns-option must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.dns-option", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_dns_search() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.dns-search")
        .ok_or_else(|| "container dns-search must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-dns-search-value"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-dns-search",
            "podman-6-0-2-container-dns-search",
            "podman-5-4-container-dns-search-command-source",
            "podman-6-0-2-container-dns-search-command-source",
            "podman-5-4-container-dns-search-lookup-all-source",
            "podman-6-0-2-container-dns-search-lookup-all-source",
            "podman-5-4-container-dns-search-reset-source",
            "podman-6-0-2-container-dns-search-reset-source",
            "podman-5-4-through-current-first-conversion-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container dns-search must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.dns-search", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_expose_host_port() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.expose-host-port")
        .ok_or_else(|| "container expose-host-port must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-expose-host-port-value"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-expose-host-port",
            "podman-6-0-2-container-expose-host-port",
            "podman-5-4-container-expose-cli",
            "podman-6-0-2-container-expose-cli",
            "podman-5-4-container-expose-host-port-command-source",
            "podman-6-0-2-container-expose-host-port-command-source",
            "podman-5-4-container-expose-host-port-regex-source",
            "podman-6-0-2-container-expose-host-port-regex-source",
            "podman-5-4-container-expose-host-port-reset-source",
            "podman-6-0-2-container-expose-host-port-reset-source",
            "podman-5-4-through-current-container-expose-host-port-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container expose-host-port must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.expose-host-port", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_annotation() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.annotation")
        .ok_or_else(|| "container annotation must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-annotation-value"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-annotation",
            "podman-6-0-2-container-annotation",
            "podman-5-4-container-annotation-command-source",
            "podman-6-0-2-container-annotation-command-source",
            "podman-5-4-container-annotation-reset-source",
            "podman-6-0-2-container-annotation-reset-source",
            "podman-5-4-through-current-container-annotation-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container annotation must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.annotation", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_apparmor() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.apparmor")
        .ok_or_else(|| "container apparmor must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-apparmor-profile"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-apparmor",
            "podman-6-0-2-container-apparmor",
            "podman-6-0-2-container-apparmor-command-source",
            "podman-5-4-through-5-7-container-apparmor-rejection",
            "podman-5-8-through-current-container-apparmor-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container apparmor must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 8, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));
    assert_eq!(capability.unsupported_ranges().len(), 1);
    assert_eq!(
        capability.unsupported_ranges()[0].versions().minimum(),
        version(5, 4, 0)
    );
    assert_eq!(
        capability.unsupported_ranges()[0].versions().maximum(),
        version(5, 7, 1)
    );

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unsupported),
        (version(5, 7, 1), SupportClassification::Unsupported),
        (version(5, 8, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.apparmor", target)
                .classification(),
            expected
        );
    }
    let crossing = PodmanTarget::new(version(5, 7, 1), Some(version(5, 8, 0))).map_err(|error| error.to_string())?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.apparmor", crossing)
            .classification(),
        SupportClassification::Unknown
    );
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_no_new_privileges() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.no-new-privileges")
        .ok_or_else(|| "container no-new-privileges must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["literal-true-or-false"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-no-new-privileges",
            "podman-6-0-2-container-no-new-privileges",
            "podman-6-0-2-container-no-new-privileges-command-source",
            "podman-5-4-through-current-container-no-new-privileges-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container no-new-privileges must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.no-new-privileges", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_seccomp_profile() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.seccomp-profile")
        .ok_or_else(|| "container seccomp-profile must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-seccomp-profile"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-seccomp-profile",
            "podman-6-0-2-container-seccomp-profile",
            "podman-6-0-2-container-seccomp-profile-command-source",
            "podman-6-0-2-container-seccomp-profile-lookup-source",
            "podman-5-4-through-current-container-seccomp-profile-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container seccomp-profile must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.seccomp-profile", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_security_label_disable() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.security-label-disable")
        .ok_or_else(|| "container security-label-disable must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["literal-true-or-false"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-security-label-disable",
            "podman-6-0-2-container-security-label-disable",
            "podman-6-0-2-container-security-label-disable-command-source",
            "podman-5-4-through-current-container-security-label-disable-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container security-label-disable must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.security-label-disable", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_security_label_file_type() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.security-label-file-type")
        .ok_or_else(|| "container security-label-file-type must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-security-label-file-type"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-security-label-file-type",
            "podman-6-0-2-container-security-label-file-type",
            "podman-6-0-2-container-security-label-file-type-command-source",
            "podman-5-4-through-current-container-security-label-file-type-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container security-label-file-type must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.security-label-file-type", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_security_label_level() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.security-label-level")
        .ok_or_else(|| "container security-label-level must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-security-label-level"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-security-label-level",
            "podman-6-0-2-container-security-label-level",
            "podman-6-0-2-container-security-label-level-command-source",
            "podman-5-4-through-current-container-security-label-level-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container security-label-level must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.security-label-level", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_security_label_nested() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.security-label-nested")
        .ok_or_else(|| "container security-label-nested must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["literal-true-or-false"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-security-label-nested",
            "podman-6-0-2-container-security-label-nested",
            "podman-6-0-2-container-security-label-nested-command-source",
            "podman-5-4-through-current-container-security-label-nested-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container security-label-nested must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.security-label-nested", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_singleton_container_security_label_type() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.security-label-type")
        .ok_or_else(|| "container security-label-type must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-security-label-type"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-security-label-type",
            "podman-6-0-2-container-security-label-type",
            "podman-6-0-2-container-security-label-type-command-source",
            "podman-5-4-through-current-container-security-label-type-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container security-label-type must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.security-label-type", target)
                .classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_mask() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.mask")
        .ok_or_else(|| "container mask must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-mask-path-list"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-mask",
            "podman-6-0-2-container-mask",
            "podman-5-4-container-mask-command-source",
            "podman-6-0-2-container-mask-command-source",
            "podman-5-4-through-current-container-mask-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container mask must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.mask", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn supported_range_records_repeatable_container_unmask() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = catalogue
        .capability("quadlet.container.unmask")
        .ok_or_else(|| "container unmask must exist".to_owned())?;
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line-unmask-path-list-or-all"]);
    assert_eq!(
        capability.evidence(),
        [
            "podman-5-4-container-unmask",
            "podman-6-0-2-container-unmask",
            "podman-5-4-container-unmask-command-source",
            "podman-6-0-2-container-unmask-command-source",
            "podman-5-4-through-current-container-unmask-generators",
        ]
    );
    let native = capability
        .native_range()
        .ok_or_else(|| "container unmask must have native coverage".to_owned())?;
    assert_eq!(native.minimum(), version(5, 4, 0));
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 3), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
    ] {
        let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.unmask", target).classification(),
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Unknown),
        (version(5, 4, 2), SupportClassification::Unknown),
        (version(5, 5, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
fn supported_range_records_container_reload_keys() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (id, value_form, evidence) in [
        (
            "quadlet.container.reload-cmd",
            "opaque-one-line-reload-command",
            [
                "podman-5-5-container-reload-cmd",
                "podman-6-0-2-container-reload-cmd",
                "podman-5-5-container-reload-cmd-source",
                "podman-6-0-2-container-reload-cmd-source",
                "podman-5-4-container-reload-generator-rejection",
                "podman-5-5-through-current-container-reload-generators",
                "podman-5-5-through-current-container-reload-mutual-exclusion-generators",
            ],
        ),
        (
            "quadlet.container.reload-signal",
            "opaque-one-line-reload-signal",
            [
                "podman-5-5-container-reload-signal",
                "podman-6-0-2-container-reload-signal",
                "podman-5-5-container-reload-signal-source",
                "podman-6-0-2-container-reload-signal-source",
                "podman-5-4-container-reload-generator-rejection",
                "podman-5-5-through-current-container-reload-generators",
                "podman-5-5-through-current-container-reload-mutual-exclusion-generators",
            ],
        ),
    ] {
        let capability = catalogue
            .capability(id)
            .ok_or_else(|| format!("{id} capability must exist"))?;
        assert_eq!(capability.unit_types(), ["container"]);
        assert_eq!(capability.sections(), ["Container"]);
        assert!(!capability.is_repeatable());
        assert_eq!(capability.value_forms(), [value_form]);
        assert_eq!(capability.evidence(), evidence);
        let native = capability
            .native_range()
            .ok_or_else(|| format!("{id} must have native coverage"))?;
        assert_eq!(native.minimum(), version(5, 5, 0));
        assert_eq!(native.maximum(), version(6, 1, 0));
        assert_eq!(capability.unsupported_ranges().len(), 1);
        assert_eq!(
            capability.unsupported_ranges()[0].versions().minimum(),
            version(5, 4, 0)
        );
        assert_eq!(
            capability.unsupported_ranges()[0].versions().maximum(),
            version(5, 4, 2)
        );
        for (target, expected) in [
            (version(5, 3, 3), SupportClassification::Unknown),
            (version(5, 4, 0), SupportClassification::Unsupported),
            (version(5, 4, 2), SupportClassification::Unsupported),
            (version(5, 5, 0), SupportClassification::Native),
            (version(6, 1, 0), SupportClassification::Native),
            (version(6, 1, 1), SupportClassification::Unknown),
        ] {
            let target = PodmanTarget::new(target, Some(target)).map_err(|error| error.to_string())?;
            assert_eq!(catalogue.evaluate(id, target).classification(), expected);
        }
    }
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
    assert_eq!(native.maximum(), version(6, 1, 0));

    for (target, expected) in [
        (version(5, 3, 0), SupportClassification::Unknown),
        (version(5, 4, 0), SupportClassification::Native),
        (version(6, 1, 0), SupportClassification::Native),
        (version(6, 1, 1), SupportClassification::Unknown),
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
            "6.1.0..=6.1.0 native",
            PodmanTarget::new(version(6, 1, 0), Some(version(6, 1, 0))).map_err(|error| error.to_string())?,
        ),
        ("5.4.0..=6.1.0 native", target(5, 4, Some((6, 1)))?),
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
    assert_eq!(future_open_ended.evaluated_range().maximum(), version(6, 1, 0));

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

    let unsupported_record = catalogue
        .capability("quadlet.example.unsupported")
        .ok_or_else(|| "synthetic unsupported capability must exist".to_owned())?
        .unsupported_ranges()
        .first()
        .ok_or_else(|| "synthetic unsupported range must exist".to_owned())?;
    assert_eq!(unsupported_record.summary(), "native support starts in the final patch");
    assert_eq!(unsupported_record.evidence(), ["generator-5-4"]);
    let unsupported = catalogue.evaluate(
        "quadlet.example.unsupported",
        PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 1))).map_err(|error| error.to_string())?,
    );
    assert_eq!(unsupported.classification(), SupportClassification::Unsupported);
    let crossing = catalogue.evaluate(
        "quadlet.example.unsupported",
        PodmanTarget::new(version(5, 4, 1), Some(version(5, 4, 2))).map_err(|error| error.to_string())?,
    );
    assert_eq!(crossing.classification(), SupportClassification::Unknown);

    let overlapping = SYNTHETIC_CATALOGUE.replace(
        "native = { minimum = \"5.4.2\", maximum = \"5.4.2\" }",
        "native = { minimum = \"5.4.1\", maximum = \"5.4.2\" }",
    );
    assert!(CapabilityCatalogue::parse(&overlapping).is_err());
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

[[capability]]
id = "quadlet.example.unsupported"
description = "Synthetic evidence-backed unsupported range."
unit_types = ["container"]
sections = ["Container"]
native = { minimum = "5.4.2", maximum = "5.4.2" }
evidence = ["generator-5-4"]

[[capability.unsupported]]
versions = { minimum = "5.4.0", maximum = "5.4.1" }
summary = "native support starts in the final patch"
evidence = ["generator-5-4"]
"#;
