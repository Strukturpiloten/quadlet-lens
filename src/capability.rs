//! Data-driven Quadlet capability catalogue and target-range evaluation.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    str::FromStr,
};

use serde::Deserialize;

const SCHEMA_VERSION: u32 = 1;

/// Numeric Podman version used by evidence and target profiles.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PodmanVersion(u64, u64, u64);

impl PodmanVersion {
    /// Creates a numeric version.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(major, minor, patch)
    }

    /// Returns the major number.
    #[must_use]
    pub const fn major(self) -> u64 {
        self.0
    }

    /// Returns the minor number.
    #[must_use]
    pub const fn minor(self) -> u64 {
        self.1
    }

    /// Returns the patch number.
    #[must_use]
    pub const fn patch(self) -> u64 {
        self.2
    }
}

impl FromStr for PodmanVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = value.split('.').collect();
        if !(2..=3).contains(&parts.len()) {
            return Err(VersionParseError(value.to_owned()));
        }
        let major = parse_part(parts[0], value)?;
        let minor = parse_part(parts[1], value)?;
        let patch = parts.get(2).map_or(Ok(0), |part| parse_part(part, value))?;
        Ok(Self(major, minor, patch))
    }
}

impl fmt::Display for PodmanVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// Rejected Podman version spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionParseError(String);

impl VersionParseError {
    /// Returns the rejected spelling.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid numeric Podman version `{}`", self.0)
    }
}

impl Error for VersionParseError {}

/// Numeric systemd release used only where a capability has direct systemd evidence.
///
/// This is deliberately not a general distribution or systemd capability catalogue. A release
/// number is optional target context supplied by the caller; `QuadletLens` never probes a host.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemdVersion(u64);

impl SystemdVersion {
    /// Creates a numeric systemd release.
    #[must_use]
    pub const fn new(release: u64) -> Self {
        Self(release)
    }

    /// Returns the numeric systemd release.
    #[must_use]
    pub const fn release(self) -> u64 {
        self.0
    }
}

impl FromStr for SystemdVersion {
    type Err = SystemdVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || (value.len() > 1 && value.starts_with('0'))
            || !value.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(SystemdVersionParseError(value.to_owned()));
        }
        value
            .parse()
            .map(Self)
            .map_err(|_| SystemdVersionParseError(value.to_owned()))
    }
}

impl fmt::Display for SystemdVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Rejected systemd release spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemdVersionParseError(String);

impl SystemdVersionParseError {
    /// Returns the rejected spelling.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SystemdVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid numeric systemd version `{}`", self.0)
    }
}

impl Error for SystemdVersionParseError {}

/// Inclusive finite systemd release range attached to systemd-specific evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemdVersionRange {
    minimum: SystemdVersion,
    maximum: SystemdVersion,
}

impl SystemdVersionRange {
    /// Creates a coherent inclusive systemd release range.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogueError::InvalidRange`] for an inverted range.
    pub fn new(
        minimum: SystemdVersion,
        maximum: SystemdVersion,
        field: impl Into<String>,
    ) -> Result<Self, CatalogueError> {
        if maximum < minimum {
            return Err(CatalogueError::InvalidRange(field.into()));
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive minimum release.
    #[must_use]
    pub const fn minimum(self) -> SystemdVersion {
        self.minimum
    }

    /// Returns the inclusive maximum release.
    #[must_use]
    pub const fn maximum(self) -> SystemdVersion {
        self.maximum
    }

    /// Returns whether this evidence range contains one release.
    #[must_use]
    pub fn contains(self, version: SystemdVersion) -> bool {
        self.minimum <= version && version <= self.maximum
    }
}

/// Inclusive finite version range in validated catalogue data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VersionRange {
    minimum: PodmanVersion,
    maximum: PodmanVersion,
}

impl VersionRange {
    /// Creates a coherent inclusive range.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogueError::InvalidRange`] for an inverted range.
    pub fn new(
        minimum: PodmanVersion,
        maximum: PodmanVersion,
        field: impl Into<String>,
    ) -> Result<Self, CatalogueError> {
        if maximum < minimum {
            return Err(CatalogueError::InvalidRange(field.into()));
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive minimum.
    #[must_use]
    pub const fn minimum(self) -> PodmanVersion {
        self.minimum
    }

    /// Returns the inclusive maximum.
    #[must_use]
    pub const fn maximum(self) -> PodmanVersion {
        self.maximum
    }

    /// Returns whether this range covers another complete range.
    #[must_use]
    pub fn covers(self, other: Self) -> bool {
        self.minimum <= other.minimum && self.maximum >= other.maximum
    }

    /// Returns whether the ranges overlap.
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.maximum >= other.minimum && other.maximum >= self.minimum
    }
}

/// `podmanMinimumVersion` and optional `podmanMaximumVersion` target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PodmanTarget {
    minimum: PodmanVersion,
    maximum: Option<PodmanVersion>,
    systemd_version: Option<SystemdVersion>,
}

impl PodmanTarget {
    /// Creates a coherent target range.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogueError::InvalidRange`] for an inverted range.
    pub fn new(minimum: PodmanVersion, maximum: Option<PodmanVersion>) -> Result<Self, CatalogueError> {
        if maximum.is_some_and(|maximum| maximum < minimum) {
            return Err(CatalogueError::InvalidRange("target".to_owned()));
        }
        Ok(Self {
            minimum,
            maximum,
            systemd_version: None,
        })
    }

    /// Returns `podmanMinimumVersion`.
    #[must_use]
    pub const fn minimum(self) -> PodmanVersion {
        self.minimum
    }

    /// Returns `podmanMaximumVersion`.
    #[must_use]
    pub const fn maximum(self) -> Option<PodmanVersion> {
        self.maximum
    }

    /// Adds caller-supplied systemd target context without probing a host.
    #[must_use]
    pub const fn with_systemd_version(mut self, systemd_version: SystemdVersion) -> Self {
        self.systemd_version = Some(systemd_version);
        self
    }

    /// Returns the optional caller-supplied systemd target release.
    #[must_use]
    pub const fn systemd_version(self) -> Option<SystemdVersion> {
        self.systemd_version
    }
}

/// Support result for one capability over an entire target range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SupportClassification {
    /// Direct Quadlet support covers the range.
    Native,
    /// A documented fallback covers the range.
    Fallback,
    /// Support remains available but is deprecated in the range.
    Deprecated,
    /// The capability was removed throughout the range.
    Removed,
    /// No known representation covers the range.
    Unsupported,
    /// Evidence or catalogue coverage cannot establish behavior.
    Unknown,
    /// A known broken range overlaps the target.
    Broken,
}

/// Evidence strength for a catalogue claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationLevel {
    /// Primary documentation evidence without real-generator execution.
    Documentation,
    /// Exact-version generator execution evidence.
    Generator,
}

/// Shared evidence for one or more capabilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    id: String,
    level: VerificationLevel,
    url: String,
    versions: VersionRange,
    claim: String,
    test: String,
    gap: Option<String>,
}

impl EvidenceRecord {
    /// Returns the stable evidence identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the evidence strength.
    #[must_use]
    pub const fn level(&self) -> VerificationLevel {
        self.level
    }

    /// Returns the primary evidence URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the finite version range covered by this evidence record.
    #[must_use]
    pub const fn versions(&self) -> VersionRange {
        self.versions
    }

    /// Returns the narrowly stated claim.
    #[must_use]
    pub fn claim(&self) -> &str {
        &self.claim
    }

    /// Returns the automated test or review identifier.
    #[must_use]
    pub fn test(&self) -> &str {
        &self.test
    }

    /// Returns the explicit evidence gap.
    #[must_use]
    pub fn gap(&self) -> Option<&str> {
        self.gap.as_deref()
    }
}

/// Immutable, versioned systemd evidence for a systemd target requirement.
///
/// This is intentionally separate from [`EvidenceRecord`], whose version range describes Podman
/// behavior. It contains documentation provenance only and makes no host or distribution claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemdEvidenceRecord {
    id: String,
    versions: SystemdVersionRange,
    url: String,
    claim: String,
    test: String,
    gap: String,
}

impl SystemdEvidenceRecord {
    /// Returns the stable evidence identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the finite systemd release range established by this record.
    #[must_use]
    pub const fn versions(&self) -> SystemdVersionRange {
        self.versions
    }

    /// Returns the immutable or versioned primary source URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the narrowly stated systemd claim.
    #[must_use]
    pub fn claim(&self) -> &str {
        &self.claim
    }

    /// Returns the automated test or review identifier.
    #[must_use]
    pub fn test(&self) -> &str {
        &self.test
    }

    /// Returns the explicit evidence gap.
    #[must_use]
    pub fn gap(&self) -> &str {
        &self.gap
    }
}

/// Fallback representation for a bounded target range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackRecord {
    kind: String,
    versions: VersionRange,
    semantic_difference: String,
    evidence: Vec<String>,
}

impl FallbackRecord {
    /// Returns the semantic fallback kind.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the fallback range.
    #[must_use]
    pub const fn versions(&self) -> VersionRange {
        self.versions
    }

    /// Returns the documented semantic difference.
    #[must_use]
    pub fn semantic_difference(&self) -> &str {
        &self.semantic_difference
    }

    /// Returns supporting evidence identifiers.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

/// Known broken target range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownBugRecord {
    versions: VersionRange,
    summary: String,
    evidence: Vec<String>,
}

/// Evidence-backed range where a capability has no native representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedRecord {
    versions: VersionRange,
    summary: String,
    evidence: Vec<String>,
}

impl UnsupportedRecord {
    /// Returns the exact unsupported version range.
    #[must_use]
    pub const fn versions(&self) -> VersionRange {
        self.versions
    }

    /// Returns why the capability is classified as unsupported.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns supporting evidence identifiers.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

impl KnownBugRecord {
    /// Returns the broken range.
    #[must_use]
    pub const fn versions(&self) -> VersionRange {
        self.versions
    }

    /// Returns the problem summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns supporting evidence identifiers.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

/// One validated semantic capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRecord {
    id: String,
    description: String,
    unit_types: Vec<String>,
    sections: Vec<String>,
    required: bool,
    repeatable: bool,
    value_forms: Vec<String>,
    systemd_minimum: Option<SystemdVersion>,
    systemd_evidence: Vec<String>,
    native: Option<VersionRange>,
    deprecated_from: Option<PodmanVersion>,
    removed_from: Option<PodmanVersion>,
    fallbacks: Vec<FallbackRecord>,
    known_bugs: Vec<KnownBugRecord>,
    unsupported: Vec<UnsupportedRecord>,
    evidence: Vec<String>,
}

impl CapabilityRecord {
    /// Returns the stable namespaced identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the semantic description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns applicable unit types.
    #[must_use]
    pub fn unit_types(&self) -> &[String] {
        &self.unit_types
    }

    /// Returns applicable section names.
    #[must_use]
    pub fn sections(&self) -> &[String] {
        &self.sections
    }

    /// Returns whether the capability is required for its unit form.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    /// Returns whether authored entries may repeat.
    #[must_use]
    pub const fn is_repeatable(&self) -> bool {
        self.repeatable
    }

    /// Returns documented accepted value forms.
    #[must_use]
    pub fn value_forms(&self) -> &[String] {
        &self.value_forms
    }

    /// Returns an evidence-backed minimum systemd release, when this capability has one.
    #[must_use]
    pub const fn systemd_minimum(&self) -> Option<SystemdVersion> {
        self.systemd_minimum
    }

    /// Returns systemd-specific evidence identifiers supporting [`Self::systemd_minimum`].
    #[must_use]
    pub fn systemd_evidence(&self) -> &[String] {
        &self.systemd_evidence
    }

    /// Returns documentation-backed native coverage.
    #[must_use]
    pub const fn native_range(&self) -> Option<VersionRange> {
        self.native
    }

    /// Returns the optional deprecation boundary.
    #[must_use]
    pub const fn deprecated_from(&self) -> Option<PodmanVersion> {
        self.deprecated_from
    }

    /// Returns the optional removal boundary.
    #[must_use]
    pub const fn removed_from(&self) -> Option<PodmanVersion> {
        self.removed_from
    }

    /// Returns fallback records.
    #[must_use]
    pub fn fallbacks(&self) -> &[FallbackRecord] {
        &self.fallbacks
    }

    /// Returns known bug records.
    #[must_use]
    pub fn known_bugs(&self) -> &[KnownBugRecord] {
        &self.known_bugs
    }

    /// Returns evidence-backed ranges with no native representation.
    #[must_use]
    pub fn unsupported_ranges(&self) -> &[UnsupportedRecord] {
        &self.unsupported
    }

    /// Returns direct evidence identifiers.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

/// Evaluation of one capability over a target range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityEvaluation {
    capability: String,
    classification: SupportClassification,
    evaluated: VersionRange,
    assumes_later_versions: bool,
    selected_fallback: Option<String>,
    evidence: Vec<String>,
    systemd_evidence: Vec<String>,
    note: Option<String>,
}

impl CapabilityEvaluation {
    /// Returns the queried identifier.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Returns support for the complete evaluated range.
    #[must_use]
    pub const fn classification(&self) -> SupportClassification {
        self.classification
    }

    /// Returns the finite range actually evaluated.
    #[must_use]
    pub const fn evaluated_range(&self) -> VersionRange {
        self.evaluated
    }

    /// Returns whether later versions remain untested because no maximum was supplied.
    #[must_use]
    pub const fn assumes_later_versions(&self) -> bool {
        self.assumes_later_versions
    }

    /// Returns the selected fallback kind.
    #[must_use]
    pub fn selected_fallback(&self) -> Option<&str> {
        self.selected_fallback.as_deref()
    }

    /// Returns evidence identifiers used by the result.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    /// Returns systemd-specific evidence identifiers used by this evaluation.
    ///
    /// This is populated only for an in-coverage known capability with a declared systemd
    /// requirement, regardless of whether the caller supplied a missing, too-old, or sufficient
    /// systemd target. [`Self::evidence`] remains Podman evidence only.
    #[must_use]
    pub fn systemd_evidence(&self) -> &[String] {
        &self.systemd_evidence
    }

    /// Returns additional range or evidence context.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// Versioned, semantically validated capability catalogue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityCatalogue {
    schema: u32,
    id: String,
    coverage: VersionRange,
    evidence: Vec<EvidenceRecord>,
    systemd_evidence: Vec<SystemdEvidenceRecord>,
    capabilities: Vec<CapabilityRecord>,
}

impl CapabilityCatalogue {
    /// Parses and validates a versioned TOML catalogue.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogueError`] for syntax, schema, identifier, range,
    /// evidence-reference, or required-field failures.
    pub fn parse(source: &str) -> Result<Self, CatalogueError> {
        let raw: RawCatalogue = toml::from_str(source).map_err(|error| CatalogueError::Decode(error.to_string()))?;
        Self::from_raw(raw)
    }

    /// Loads the built-in rolling supported-range catalogue.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogueError`] if embedded data violates the schema.
    pub fn supported_range() -> Result<Self, CatalogueError> {
        Self::parse(include_str!("../catalogue/v1/podman-supported-range.toml"))
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// Returns the catalogue identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns finite evidence coverage.
    #[must_use]
    pub const fn coverage(&self) -> VersionRange {
        self.coverage
    }

    /// Returns evidence records.
    #[must_use]
    pub fn evidence(&self) -> &[EvidenceRecord] {
        &self.evidence
    }

    /// Returns systemd-specific evidence records in authored order.
    #[must_use]
    pub fn systemd_evidence(&self) -> &[SystemdEvidenceRecord] {
        &self.systemd_evidence
    }

    /// Returns capabilities in authored order.
    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityRecord] {
        &self.capabilities
    }

    /// Finds one capability.
    #[must_use]
    pub fn capability(&self, id: &str) -> Option<&CapabilityRecord> {
        self.capabilities.iter().find(|item| item.id == id)
    }

    /// Evaluates one capability over an explicit target range.
    #[must_use]
    pub fn evaluate(&self, id: &str, target: PodmanTarget) -> CapabilityEvaluation {
        let assumes_later = target.maximum().is_none();
        let evaluated = VersionRange {
            minimum: target.minimum(),
            maximum: target
                .maximum()
                .unwrap_or_else(|| self.coverage.maximum().max(target.minimum())),
        };
        if !self.coverage.covers(evaluated) {
            return make_evaluation(
                id,
                SupportClassification::Unknown,
                evaluated,
                assumes_later,
                None,
                Vec::new(),
                Some(format!(
                    "requested range is outside catalogue coverage {} through {}",
                    self.coverage.minimum(),
                    self.coverage.maximum()
                )),
            );
        }
        let Some(capability) = self.capability(id) else {
            return make_evaluation(
                id,
                SupportClassification::Unknown,
                evaluated,
                assumes_later,
                None,
                Vec::new(),
                Some("capability is absent from this catalogue".to_owned()),
            );
        };
        let evaluation = evaluate_record(capability, evaluated, assumes_later);
        evaluate_systemd_requirement(capability, target.systemd_version(), evaluation)
    }

    fn from_raw(raw: RawCatalogue) -> Result<Self, CatalogueError> {
        if raw.schema != SCHEMA_VERSION {
            return Err(CatalogueError::UnsupportedSchema(raw.schema));
        }
        validate_id(&raw.id, false)?;
        let coverage = parse_required_range("coverage", &raw.coverage)?;
        let evidence = parse_evidence(raw.evidence, coverage)?;
        let evidence_ids: BTreeSet<_> = evidence.iter().map(|item| item.id.as_str()).collect();
        let systemd_evidence = parse_systemd_evidence(raw.systemd_evidence, &evidence_ids)?;
        let systemd_evidence_ranges: BTreeMap<_, _> = systemd_evidence
            .iter()
            .map(|item| (item.id.as_str(), item.versions))
            .collect();
        let capabilities = parse_capabilities(raw.capability, coverage, &evidence_ids, &systemd_evidence_ranges)?;
        Ok(Self {
            schema: raw.schema,
            id: raw.id,
            coverage,
            evidence,
            systemd_evidence,
            capabilities,
        })
    }
}

/// Catalogue decoding or semantic validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CatalogueError {
    /// TOML decoding failed.
    Decode(String),
    /// Schema version is unsupported.
    UnsupportedSchema(u32),
    /// A stable identifier is malformed.
    InvalidIdentifier(String),
    /// A stable identifier occurs more than once.
    DuplicateIdentifier(String),
    /// A version string is invalid.
    InvalidVersion {
        /// Field path.
        field: String,
        /// Rejected spelling.
        value: String,
    },
    /// A systemd version string is invalid.
    InvalidSystemdVersion {
        /// Field path.
        field: String,
        /// Rejected spelling.
        value: String,
    },
    /// A range is inverted or outside catalogue coverage.
    InvalidRange(String),
    /// A referenced evidence record is absent.
    MissingEvidence {
        /// Referring record.
        owner: String,
        /// Missing identifier.
        evidence: String,
    },
    /// A required field is empty or incoherent.
    InvalidField(String),
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "cannot decode capability catalogue: {error}"),
            Self::UnsupportedSchema(schema) => {
                write!(
                    formatter,
                    "unsupported catalogue schema {schema}; expected {SCHEMA_VERSION}"
                )
            }
            Self::InvalidIdentifier(id) => write!(formatter, "invalid stable identifier `{id}`"),
            Self::DuplicateIdentifier(id) => write!(formatter, "duplicate stable identifier `{id}`"),
            Self::InvalidVersion { field, value } => {
                write!(formatter, "invalid Podman version `{value}` in `{field}`")
            }
            Self::InvalidSystemdVersion { field, value } => {
                write!(formatter, "invalid systemd version `{value}` in `{field}`")
            }
            Self::InvalidRange(field) => write!(formatter, "invalid version range in `{field}`"),
            Self::MissingEvidence { owner, evidence } => {
                write!(formatter, "`{owner}` refers to missing evidence `{evidence}`")
            }
            Self::InvalidField(field) => write!(formatter, "invalid required field `{field}`"),
        }
    }
}

impl Error for CatalogueError {}

fn evaluate_record(record: &CapabilityRecord, range: VersionRange, assumes_later: bool) -> CapabilityEvaluation {
    if let Some(bug) = record.known_bugs.iter().find(|bug| bug.versions.overlaps(range)) {
        return make_evaluation(
            &record.id,
            SupportClassification::Broken,
            range,
            assumes_later,
            None,
            bug.evidence.clone(),
            Some(bug.summary.clone()),
        );
    }
    if let Some(unsupported) = record.unsupported.iter().find(|item| item.versions.covers(range)) {
        return make_evaluation(
            &record.id,
            SupportClassification::Unsupported,
            range,
            assumes_later,
            None,
            unsupported.evidence.clone(),
            Some(unsupported.summary.clone()),
        );
    }
    if record.removed_from.is_some_and(|version| version <= range.minimum()) {
        return make_evaluation(
            &record.id,
            SupportClassification::Removed,
            range,
            assumes_later,
            None,
            record.evidence.clone(),
            assumption_note(assumes_later),
        );
    }
    if record.native.is_some_and(|native| native.covers(range)) {
        let deprecated = record.deprecated_from.is_some_and(|version| version <= range.maximum());
        return make_evaluation(
            &record.id,
            if deprecated {
                SupportClassification::Deprecated
            } else {
                SupportClassification::Native
            },
            range,
            assumes_later,
            None,
            record.evidence.clone(),
            assumption_note(assumes_later),
        );
    }
    if let Some(fallback) = record.fallbacks.iter().find(|item| item.versions.covers(range)) {
        return make_evaluation(
            &record.id,
            SupportClassification::Fallback,
            range,
            assumes_later,
            Some(fallback.kind.clone()),
            fallback.evidence.clone(),
            Some(fallback.semantic_difference.clone()),
        );
    }
    make_evaluation(
        &record.id,
        SupportClassification::Unknown,
        range,
        assumes_later,
        None,
        record.evidence.clone(),
        Some("capability evidence does not cover the complete requested range".to_owned()),
    )
}

fn evaluate_systemd_requirement(
    record: &CapabilityRecord,
    systemd_version: Option<SystemdVersion>,
    evaluation: CapabilityEvaluation,
) -> CapabilityEvaluation {
    let Some(minimum) = record.systemd_minimum else {
        return evaluation;
    };
    if !matches!(
        evaluation.classification,
        SupportClassification::Native | SupportClassification::Deprecated | SupportClassification::Fallback
    ) {
        return evaluation;
    }
    let mut evaluation = evaluation;
    evaluation.systemd_evidence.clone_from(&record.systemd_evidence);
    match systemd_version {
        None => {
            evaluation.classification = SupportClassification::Unknown;
            evaluation.selected_fallback = None;
            evaluation.note = Some(format!(
                "capability requires systemd {minimum} or newer, but the target has no systemd version"
            ));
            evaluation
        }
        Some(version) if version < minimum => {
            evaluation.classification = SupportClassification::Unsupported;
            evaluation.selected_fallback = None;
            evaluation.note = Some(format!(
                "capability requires systemd {minimum} or newer, but target systemd is {version}"
            ));
            evaluation
        }
        Some(_) => evaluation,
    }
}

fn make_evaluation(
    capability: &str,
    classification: SupportClassification,
    evaluated: VersionRange,
    assumes_later_versions: bool,
    selected_fallback: Option<String>,
    evidence: Vec<String>,
    note: Option<String>,
) -> CapabilityEvaluation {
    CapabilityEvaluation {
        capability: capability.to_owned(),
        classification,
        evaluated,
        assumes_later_versions,
        selected_fallback,
        evidence,
        systemd_evidence: Vec::new(),
        note,
    }
}

fn assumption_note(assumes: bool) -> Option<String> {
    assumes.then(|| "podmanMaximumVersion is omitted; later versions remain untested assumptions".to_owned())
}

fn parse_evidence(raw: Vec<RawEvidence>, coverage: VersionRange) -> Result<Vec<EvidenceRecord>, CatalogueError> {
    let mut ids = BTreeSet::new();
    let mut result = Vec::with_capacity(raw.len());
    for item in raw {
        validate_id(&item.id, false)?;
        if !ids.insert(item.id.clone()) {
            return Err(CatalogueError::DuplicateIdentifier(item.id));
        }
        if !item.url.starts_with("https://") || item.claim.is_empty() || item.test.is_empty() {
            return Err(CatalogueError::InvalidField(format!("evidence.{}", item.id)));
        }
        let versions = match (item.target, item.versions) {
            (Some(target), None) => {
                let target = parse_version(&format!("evidence.{}.target", item.id), &target)?;
                VersionRange::new(target, target, format!("evidence.{}.target", item.id))?
            }
            (None, Some(versions)) => parse_required_range(&format!("evidence.{}.versions", item.id), &versions)?,
            _ => {
                return Err(CatalogueError::InvalidField(format!(
                    "evidence.{}.target-or-versions",
                    item.id
                )));
            }
        };
        if !coverage.covers(versions) {
            return Err(CatalogueError::InvalidRange(format!("evidence.{}.versions", item.id)));
        }
        let level = match item.verification.as_str() {
            "documentation" => VerificationLevel::Documentation,
            "generator" => VerificationLevel::Generator,
            _ => {
                return Err(CatalogueError::InvalidField(format!(
                    "evidence.{}.verification",
                    item.id
                )));
            }
        };
        if level == VerificationLevel::Documentation && item.gap.as_deref().is_none_or(str::is_empty) {
            return Err(CatalogueError::InvalidField(format!("evidence.{}.gap", item.id)));
        }
        result.push(EvidenceRecord {
            id: item.id,
            level,
            url: item.url,
            versions,
            claim: item.claim,
            test: item.test,
            gap: item.gap,
        });
    }
    if result.is_empty() {
        return Err(CatalogueError::InvalidField("evidence".to_owned()));
    }
    Ok(result)
}

fn parse_systemd_evidence(
    raw: Vec<RawSystemdEvidence>,
    podman_evidence_ids: &BTreeSet<&str>,
) -> Result<Vec<SystemdEvidenceRecord>, CatalogueError> {
    let mut ids = BTreeSet::new();
    let mut result = Vec::with_capacity(raw.len());
    for item in raw {
        validate_id(&item.id, false)?;
        if !ids.insert(item.id.clone()) || podman_evidence_ids.contains(item.id.as_str()) {
            return Err(CatalogueError::DuplicateIdentifier(item.id));
        }
        if item.claim.is_empty() || item.test.is_empty() || item.gap.is_empty() || !is_versioned_systemd_url(&item.url)
        {
            return Err(CatalogueError::InvalidField(format!("systemd_evidence.{}", item.id)));
        }
        let versions = match (item.target, item.versions) {
            (Some(target), None) => {
                let target = parse_systemd_version(&format!("systemd_evidence.{}.target", item.id), &target)?;
                SystemdVersionRange::new(target, target, format!("systemd_evidence.{}.target", item.id))?
            }
            (None, Some(versions)) => {
                parse_systemd_range(&format!("systemd_evidence.{}.versions", item.id), &versions)?
            }
            _ => {
                return Err(CatalogueError::InvalidField(format!(
                    "systemd_evidence.{}.target-or-versions",
                    item.id
                )));
            }
        };
        if !url_mentions_systemd_version(&item.url, versions.minimum()) {
            return Err(CatalogueError::InvalidField(format!(
                "systemd_evidence.{}.url",
                item.id
            )));
        }
        result.push(SystemdEvidenceRecord {
            id: item.id,
            versions,
            url: item.url,
            claim: item.claim,
            test: item.test,
            gap: item.gap,
        });
    }
    Ok(result)
}

fn is_versioned_systemd_url(url: &str) -> bool {
    url.starts_with("https://") && !url.contains("/latest/")
}

fn url_mentions_systemd_version(url: &str, version: SystemdVersion) -> bool {
    let version = version.to_string();
    url.contains(&format!("/{version}/")) || url.contains(&format!("v{version}"))
}

fn parse_capabilities(
    raw: Vec<RawCapability>,
    coverage: VersionRange,
    evidence_ids: &BTreeSet<&str>,
    systemd_evidence_ranges: &BTreeMap<&str, SystemdVersionRange>,
) -> Result<Vec<CapabilityRecord>, CatalogueError> {
    let mut ids = BTreeSet::new();
    let mut result = Vec::with_capacity(raw.len());
    for item in raw {
        validate_id(&item.id, true)?;
        if !ids.insert(item.id.clone()) {
            return Err(CatalogueError::DuplicateIdentifier(item.id));
        }
        if item.description.is_empty() || item.unit_types.is_empty() || item.sections.is_empty() {
            return Err(CatalogueError::InvalidField(item.id));
        }
        validate_evidence(&item.id, &item.evidence, evidence_ids)?;
        let native = item
            .native
            .map(|raw| parse_covered_range(&format!("{}.native", item.id), &raw, coverage))
            .transpose()?;
        let (systemd_minimum, systemd_evidence) =
            parse_systemd_requirement(&item.id, item.systemd, systemd_evidence_ranges)?;
        let deprecated_from = optional_covered_version(&item.id, "deprecated_from", item.deprecated_from, coverage)?;
        let removed_from = optional_covered_version(&item.id, "removed_from", item.removed_from, coverage)?;
        if deprecated_from
            .zip(removed_from)
            .is_some_and(|(deprecated, removed)| removed < deprecated)
        {
            return Err(CatalogueError::InvalidRange(format!("{}.lifecycle", item.id)));
        }
        let fallbacks = parse_fallbacks(&item.id, item.fallback, coverage, evidence_ids)?;
        let known_bugs = parse_bugs(&item.id, item.known_bug, coverage, evidence_ids)?;
        let unsupported = parse_unsupported(&item.id, item.unsupported, coverage, evidence_ids)?;
        if native.is_some_and(|native| unsupported.iter().any(|item| item.versions.overlaps(native))) {
            return Err(CatalogueError::InvalidRange(format!("{}.unsupported", item.id)));
        }
        if native.is_none() && fallbacks.is_empty() && removed_from.is_none() {
            return Err(CatalogueError::InvalidField(format!("{}.support", item.id)));
        }
        result.push(CapabilityRecord {
            id: item.id,
            description: item.description,
            unit_types: item.unit_types,
            sections: item.sections,
            required: item.required,
            repeatable: item.repeatable,
            value_forms: item.value_forms,
            systemd_minimum,
            systemd_evidence,
            native,
            deprecated_from,
            removed_from,
            fallbacks,
            known_bugs,
            unsupported,
            evidence: item.evidence,
        });
    }
    if result.is_empty() {
        return Err(CatalogueError::InvalidField("capability".to_owned()));
    }
    Ok(result)
}

fn parse_fallbacks(
    owner: &str,
    raw: Vec<RawFallback>,
    coverage: VersionRange,
    evidence_ids: &BTreeSet<&str>,
) -> Result<Vec<FallbackRecord>, CatalogueError> {
    raw.into_iter()
        .enumerate()
        .map(|(index, item)| {
            let field = format!("{owner}.fallback[{index}]");
            if item.kind.is_empty() || item.semantic_difference.is_empty() {
                return Err(CatalogueError::InvalidField(field));
            }
            validate_evidence(&field, &item.evidence, evidence_ids)?;
            Ok(FallbackRecord {
                kind: item.kind,
                versions: parse_covered_range(&field, &item.versions, coverage)?,
                semantic_difference: item.semantic_difference,
                evidence: item.evidence,
            })
        })
        .collect()
}

fn parse_bugs(
    owner: &str,
    raw: Vec<RawKnownBug>,
    coverage: VersionRange,
    evidence_ids: &BTreeSet<&str>,
) -> Result<Vec<KnownBugRecord>, CatalogueError> {
    raw.into_iter()
        .enumerate()
        .map(|(index, item)| {
            let field = format!("{owner}.known_bug[{index}]");
            if item.summary.is_empty() {
                return Err(CatalogueError::InvalidField(field));
            }
            validate_evidence(&field, &item.evidence, evidence_ids)?;
            Ok(KnownBugRecord {
                versions: parse_covered_range(&field, &item.versions, coverage)?,
                summary: item.summary,
                evidence: item.evidence,
            })
        })
        .collect()
}

fn parse_unsupported(
    owner: &str,
    raw: Vec<RawUnsupported>,
    coverage: VersionRange,
    evidence_ids: &BTreeSet<&str>,
) -> Result<Vec<UnsupportedRecord>, CatalogueError> {
    raw.into_iter()
        .enumerate()
        .map(|(index, item)| {
            let field = format!("{owner}.unsupported[{index}]");
            if item.summary.is_empty() {
                return Err(CatalogueError::InvalidField(field));
            }
            validate_evidence(&field, &item.evidence, evidence_ids)?;
            Ok(UnsupportedRecord {
                versions: parse_covered_range(&field, &item.versions, coverage)?,
                summary: item.summary,
                evidence: item.evidence,
            })
        })
        .collect()
}

fn validate_evidence(owner: &str, ids: &[String], existing: &BTreeSet<&str>) -> Result<(), CatalogueError> {
    if ids.is_empty() {
        return Err(CatalogueError::InvalidField(format!("{owner}.evidence")));
    }
    for id in ids {
        if !existing.contains(id.as_str()) {
            return Err(CatalogueError::MissingEvidence {
                owner: owner.to_owned(),
                evidence: id.clone(),
            });
        }
    }
    Ok(())
}

fn parse_required_range(field: &str, raw: &RawRequiredRange) -> Result<VersionRange, CatalogueError> {
    let minimum = parse_version(&format!("{field}.minimum"), &raw.minimum)?;
    let maximum = parse_version(&format!("{field}.maximum"), &raw.maximum)?;
    VersionRange::new(minimum, maximum, field)
}

fn parse_covered_range(field: &str, raw: &RawRange, coverage: VersionRange) -> Result<VersionRange, CatalogueError> {
    let minimum = parse_version(&format!("{field}.minimum"), &raw.minimum)?;
    let maximum = raw.maximum.as_deref().map_or(Ok(coverage.maximum()), |value| {
        parse_version(&format!("{field}.maximum"), value)
    })?;
    let range = VersionRange::new(minimum, maximum, field)?;
    if !coverage.covers(range) {
        return Err(CatalogueError::InvalidRange(field.to_owned()));
    }
    Ok(range)
}

fn optional_covered_version(
    owner: &str,
    name: &str,
    value: Option<String>,
    coverage: VersionRange,
) -> Result<Option<PodmanVersion>, CatalogueError> {
    let parsed = value
        .map(|value| parse_version(&format!("{owner}.{name}"), &value))
        .transpose()?;
    if parsed.is_some_and(|version| version < coverage.minimum() || version > coverage.maximum()) {
        return Err(CatalogueError::InvalidRange(format!("{owner}.{name}")));
    }
    Ok(parsed)
}

fn parse_version(field: &str, value: &str) -> Result<PodmanVersion, CatalogueError> {
    value
        .parse()
        .map_err(|_error: VersionParseError| CatalogueError::InvalidVersion {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_systemd_version(field: &str, value: &str) -> Result<SystemdVersion, CatalogueError> {
    value.parse().map_err(|_| CatalogueError::InvalidSystemdVersion {
        field: field.to_owned(),
        value: value.to_owned(),
    })
}

fn parse_systemd_range(field: &str, raw: &RawSystemdRange) -> Result<SystemdVersionRange, CatalogueError> {
    let minimum = parse_systemd_version(&format!("{field}.minimum"), &raw.minimum)?;
    let maximum = parse_systemd_version(&format!("{field}.maximum"), &raw.maximum)?;
    SystemdVersionRange::new(minimum, maximum, field)
}

fn parse_systemd_requirement(
    owner: &str,
    raw: Option<RawSystemdRequirement>,
    evidence_ranges: &BTreeMap<&str, SystemdVersionRange>,
) -> Result<(Option<SystemdVersion>, Vec<String>), CatalogueError> {
    let Some(raw) = raw else {
        return Ok((None, Vec::new()));
    };
    let minimum = parse_systemd_version(&format!("{owner}.systemd.minimum"), &raw.minimum)?;
    if raw.evidence.is_empty() {
        return Err(CatalogueError::InvalidField(format!("{owner}.systemd.evidence")));
    }
    let mut references = BTreeSet::new();
    for evidence in &raw.evidence {
        if !references.insert(evidence.as_str()) {
            return Err(CatalogueError::DuplicateIdentifier(evidence.clone()));
        }
        if !evidence_ranges.contains_key(evidence.as_str()) {
            return Err(CatalogueError::MissingEvidence {
                owner: format!("{owner}.systemd"),
                evidence: evidence.clone(),
            });
        }
    }
    if !raw
        .evidence
        .iter()
        .any(|id| evidence_ranges[id.as_str()].contains(minimum))
    {
        return Err(CatalogueError::InvalidField(format!("{owner}.systemd.minimum")));
    }
    Ok((Some(minimum), raw.evidence))
}

fn parse_part(part: &str, full: &str) -> Result<u64, VersionParseError> {
    if part.is_empty() || (part.len() > 1 && part.starts_with('0')) {
        return Err(VersionParseError(full.to_owned()));
    }
    part.parse().map_err(|_error| VersionParseError(full.to_owned()))
}

fn validate_id(value: &str, namespaced: bool) -> Result<(), CatalogueError> {
    let valid = !value.is_empty()
        && (!namespaced || value.contains('.'))
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-'));
    if !valid {
        return Err(CatalogueError::InvalidIdentifier(value.to_owned()));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalogue {
    schema: u32,
    id: String,
    coverage: RawRequiredRange,
    #[serde(default)]
    evidence: Vec<RawEvidence>,
    #[serde(default)]
    systemd_evidence: Vec<RawSystemdEvidence>,
    #[serde(default)]
    capability: Vec<RawCapability>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequiredRange {
    minimum: String,
    maximum: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRange {
    minimum: String,
    maximum: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvidence {
    id: String,
    verification: String,
    url: String,
    target: Option<String>,
    versions: Option<RawRequiredRange>,
    claim: String,
    test: String,
    gap: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSystemdEvidence {
    id: String,
    url: String,
    target: Option<String>,
    versions: Option<RawSystemdRange>,
    claim: String,
    test: String,
    gap: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSystemdRange {
    minimum: String,
    maximum: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCapability {
    id: String,
    description: String,
    unit_types: Vec<String>,
    sections: Vec<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    repeatable: bool,
    #[serde(default)]
    value_forms: Vec<String>,
    systemd: Option<RawSystemdRequirement>,
    native: Option<RawRange>,
    deprecated_from: Option<String>,
    removed_from: Option<String>,
    #[serde(default)]
    fallback: Vec<RawFallback>,
    #[serde(default)]
    known_bug: Vec<RawKnownBug>,
    #[serde(default)]
    unsupported: Vec<RawUnsupported>,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSystemdRequirement {
    minimum: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFallback {
    kind: String,
    versions: RawRange,
    semantic_difference: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawKnownBug {
    versions: RawRange,
    summary: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawUnsupported {
    versions: RawRange,
    summary: String,
    #[serde(default)]
    evidence: Vec<String>,
}
