//! Lexical path forms retained before key-specific Quadlet resolution.

/// Lexical form of an authored Quadlet path value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PathForm {
    /// Literal path beginning at the filesystem root.
    AbsoluteLiteral,
    /// Literal path beginning with `./` or `../`; supported keys resolve it from the unit file.
    UnitRelativeLiteral,
    /// Other literal relative text, including `~`, which systemd does not treat as shell expansion.
    RelativeLiteral,
    /// Text containing at least one non-escaped systemd `%` specifier.
    SystemdSpecifier,
}

/// Classifies path spelling without expanding specifiers or applying key-specific rules.
///
/// A doubled `%%` is a literal percent and does not by itself make a value a
/// specifier expression. The function deliberately does not claim that every
/// Quadlet key accepts every returned form.
#[must_use]
pub fn classify_path(value: &str) -> PathForm {
    if contains_systemd_specifier(value) {
        return PathForm::SystemdSpecifier;
    }
    if value.starts_with('/') {
        PathForm::AbsoluteLiteral
    } else if value == "." || value == ".." || value.starts_with("./") || value.starts_with("../") {
        PathForm::UnitRelativeLiteral
    } else {
        PathForm::RelativeLiteral
    }
}

fn contains_systemd_specifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            match bytes.next() {
                Some(b'%') => {}
                Some(_) | None => return true,
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{PathForm, classify_path};

    #[test]
    fn distinguishes_literals_unit_relative_paths_and_specifiers() {
        assert_eq!(classify_path("/srv/data"), PathForm::AbsoluteLiteral);
        assert_eq!(classify_path("./data"), PathForm::UnitRelativeLiteral);
        assert_eq!(classify_path("../data"), PathForm::UnitRelativeLiteral);
        assert_eq!(classify_path("data"), PathForm::RelativeLiteral);
        assert_eq!(classify_path("~/data"), PathForm::RelativeLiteral);
        assert_eq!(classify_path("%h/data"), PathForm::SystemdSpecifier);
        assert_eq!(classify_path("/srv/%n/data"), PathForm::SystemdSpecifier);
        assert_eq!(classify_path("%%h/data"), PathForm::RelativeLiteral);
    }
}
