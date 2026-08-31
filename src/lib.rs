//! Source-aware parsing, modeling, generation, compatibility evaluation, and rendering of Podman
//! Quadlet files.
//!
//! `QuadletLens` keeps physical source syntax separate from typed native meaning and target-version
//! evidence. Parsing and generation are in-memory operations: the crate never invokes Podman,
//! operates systemd, discovers neighboring files, or mutates a host.
//!
//! # Parse a document
//!
//! ```
//! use quadlet_lens::{
//!     model::{QuadletDocument, QuadletUnitType},
//!     source::SourceId,
//! };
//!
//! let source = "[Container]\nImage=example.invalid/web:1\n";
//! let parsed = QuadletDocument::parse(
//!     QuadletUnitType::Container,
//!     SourceId::new(1),
//!     source,
//! )
//! .expect("valid Quadlet input");
//!
//! assert!(parsed.is_valid());
//! ```
//!
//! # Modules
//!
//! - [`syntax`] retains ordered physical syntax and renders preserved or canonical text.
//! - [`model`] exposes typed native documents and multi-file relationships.
//! - [`render`] constructs and validates new documents.
//! - [`capability`] evaluates evidence over explicit Podman and systemd targets.
//! - [`diagnostic`], [`source`], and [`path`] provide reporting and conservative source metadata.

pub mod capability;
pub mod diagnostic;
pub mod model;
pub mod path;
pub mod render;
pub mod source;
pub mod syntax;
