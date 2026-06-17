//! Dual-consumer error output: an RFC 9457 Problem Details envelope.
//!
//! A command-line tool now answers to two audiences: the human who reads the
//! terminal, and the LLM agent that parses the bytes and decides whether to
//! retry, escalate, or abandon. The human is served by the [`Error`] type's
//! `Display` (unchanged). The agent is served by [`ProblemDetails`] — a
//! serializable [RFC 9457] *Problem Details* envelope carrying the five
//! standard members plus the three agent extensions (`retry_after`,
//! `suggested_fix`, `code_actions`) and an [`Applicability`] marker on every
//! suggested fix and code action.
//!
//! Map any [`Error`] to an envelope with [`Error::to_problem`], or render an
//! error for a chosen [`OutputFormat`] with [`Error::render`]. The binary
//! selects the format with [`OutputFormat::select`] (an explicit `--format`
//! flag, falling back to stderr TTY detection).
//!
//! [RFC 9457]: https://www.rfc-editor.org/rfc/rfc9457

use serde::{Deserialize, Serialize};

use crate::Error;

/// Base URI under which this crate's problem-type documentation is published.
///
/// Every [`Error`](enum@crate::Error) `type` URI is derived as
/// `{ERROR_TYPE_BASE_URI}/{slug}/{version}` (e.g.
/// `https://zircote.com/rust-template/errors/invalid-input/v1`). Because this is
/// a **template**, this is the single knob an adopter changes: point it at your
/// own documentation host and every type URI follows. The default value is the
/// template's own docs site, where each `/{slug}/{version}` path resolves to a
/// live problem-type reference page.
///
/// The occurrence `instance` URN namespace is derived separately from the crate
/// name (`CARGO_PKG_NAME`), so renaming the crate in `Cargo.toml` needs no edit
/// here.
pub const ERROR_TYPE_BASE_URI: &str = "https://zircote.com/rust-template/errors";

/// How confidently an agent may apply a [`SuggestedFix`] or [`CodeAction`].
///
/// Modeled on the rustc diagnostic `Applicability` enum. Without this marker an
/// agent may apply a plausible-looking but wrong edit, so every suggested fix
/// and code action carries one. `Unspecified` is the safe default and must be
/// treated as `MaybeIncorrect` (escalate to a human) by consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Applicability {
    /// The agent may apply the edit and retry without human confirmation.
    MachineApplicable,
    /// The agent must escalate to a human before applying.
    MaybeIncorrect,
    /// The fix contains slots the agent must fill; lower confidence.
    HasPlaceholders,
    /// Applicability is unknown; consumers treat this as [`Self::MaybeIncorrect`].
    #[default]
    Unspecified,
}

/// A recovery suggestion tagged with an [`Applicability`] marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SuggestedFix {
    /// Free-text description of the recovery action.
    pub description: String,
    /// How confidently the fix may be applied.
    pub applicability: Applicability,
}

impl SuggestedFix {
    /// Creates a suggested fix from a description and an applicability marker.
    ///
    /// # Arguments
    ///
    /// * `description` - What the consumer should do to recover.
    /// * `applicability` - How confidently the fix may be applied.
    ///
    /// # Returns
    ///
    /// A new [`SuggestedFix`].
    #[must_use]
    pub fn new(description: impl Into<String>, applicability: Applicability) -> Self {
        Self {
            description: description.into(),
            applicability,
        }
    }
}

/// A structured edit an agent can apply directly, modeled on the LSP
/// `CodeAction` interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CodeAction {
    /// Short, human-readable title for the action.
    pub title: String,
    /// The kind of action (e.g. `"quickfix"`), following LSP conventions.
    pub kind: String,
    /// How confidently the action may be applied.
    pub applicability: Applicability,
}

impl CodeAction {
    /// Creates a code action from a title, kind, and applicability marker.
    ///
    /// # Arguments
    ///
    /// * `title` - Short summary of the action.
    /// * `kind` - LSP-style action kind, e.g. `"quickfix"`.
    /// * `applicability` - How confidently the action may be applied.
    ///
    /// # Returns
    ///
    /// A new [`CodeAction`].
    #[must_use]
    pub fn new(
        title: impl Into<String>,
        kind: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        Self {
            title: title.into(),
            kind: kind.into(),
            applicability,
        }
    }
}

/// An [RFC 9457] *Problem Details* envelope for machine consumers.
///
/// Serializes under the `application/problem+json` media type. It carries the
/// five standard members (`type`, `title`, `status`, `detail`, `instance`), the
/// three agent extensions (`retry_after`, `suggested_fix`, `code_actions`), and
/// the optional `exit_code` extension. `retry_after` serializes even when
/// `None` (as JSON `null`) so an agent never has to guess whether a class is
/// transient.
///
/// Build one with [`ProblemDetails::new`] and the `with_*` methods, or map an
/// existing [`Error`](enum@crate::Error) with [`Error::to_problem`].
///
/// [RFC 9457]: https://www.rfc-editor.org/rfc/rfc9457
///
/// # Examples
///
/// ```rust
/// use rust_template::{Applicability, ProblemDetails, SuggestedFix};
///
/// let problem = ProblemDetails::new(
///     "https://zircote.com/rust-template/errors/invalid-input/v1",
///     "Invalid input",
///     400,
///     "divisor cannot be zero",
///     "urn:rust_template:invalid-input",
/// )
/// .with_exit_code(2)
/// .with_suggested_fix(SuggestedFix::new(
///     "Provide a non-zero divisor.",
///     Applicability::MaybeIncorrect,
/// ));
///
/// assert_eq!(problem.status, 400);
/// assert_eq!(problem.retry_after, None);
/// assert!(problem.to_json().contains("\"type\""));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProblemDetails {
    /// A URI reference identifying the problem type. Stable and versioned.
    #[serde(rename = "type")]
    pub problem_type: String,
    /// Short, human-readable summary of the problem type. Stable per `type`.
    pub title: String,
    /// Numeric status mapping to a status class (see also `exit_code`).
    pub status: u16,
    /// Human-readable explanation specific to this occurrence.
    pub detail: String,
    /// URI reference identifying this specific occurrence.
    pub instance: String,
    /// When the operation may safely be retried (delta-seconds). Explicitly
    /// `null` for non-transient errors so agents do not have to guess.
    pub retry_after: Option<u64>,
    /// A recovery suggestion, tagged with an applicability marker.
    pub suggested_fix: Option<SuggestedFix>,
    /// Structured edits the agent can apply directly.
    pub code_actions: Vec<CodeAction>,
    /// The process exit code emitted alongside the error, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<u8>,
}

impl ProblemDetails {
    /// Creates an envelope from the five RFC 9457 standard members.
    ///
    /// `retry_after`, `suggested_fix`, `code_actions`, and `exit_code` start
    /// empty; add them with the `with_*` methods.
    ///
    /// # Arguments
    ///
    /// * `problem_type` - Stable, versioned problem-type URI.
    /// * `title` - Short summary, stable per `problem_type`.
    /// * `status` - Numeric status class.
    /// * `detail` - This-occurrence explanation.
    /// * `instance` - URI identifying this occurrence.
    ///
    /// # Returns
    ///
    /// A new [`ProblemDetails`] with no extensions set.
    #[must_use]
    pub fn new(
        problem_type: impl Into<String>,
        title: impl Into<String>,
        status: u16,
        detail: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            problem_type: problem_type.into(),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: instance.into(),
            retry_after: None,
            suggested_fix: None,
            code_actions: Vec::new(),
            exit_code: None,
        }
    }

    /// Sets `retry_after` to `seconds`, marking the error as transient.
    #[must_use]
    pub const fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after = Some(seconds);
        self
    }

    /// Attaches a [`SuggestedFix`].
    #[must_use]
    pub fn with_suggested_fix(mut self, fix: SuggestedFix) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Appends a [`CodeAction`] to `code_actions`.
    #[must_use]
    pub fn with_code_action(mut self, action: CodeAction) -> Self {
        self.code_actions.push(action);
        self
    }

    /// Sets the `exit_code` extension.
    #[must_use]
    pub const fn with_exit_code(mut self, code: u8) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Serializes the envelope as a compact `application/problem+json` string.
    ///
    /// # Returns
    ///
    /// The compact JSON representation. Returns `"{}"` only if serialization
    /// fails, which cannot happen for this all-owned, self-describing struct.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
    }

    /// Serializes the envelope as pretty-printed `application/problem+json`.
    ///
    /// # Returns
    ///
    /// The indented JSON representation, suitable for human inspection.
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| String::from("{}"))
    }
}

/// The rendering format for an [`Error`](enum@crate::Error) reported to a consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputFormat {
    /// The human-readable `Display` line ("as lush as it ever was").
    Pretty,
    /// The RFC 9457 `application/problem+json` envelope.
    Json,
}

impl OutputFormat {
    /// Selects the output format for a consumer.
    ///
    /// JSON when `--format=json` is given explicitly, or when no format is
    /// given and the error stream is not a terminal. Pretty when
    /// `--format=pretty` is given, or when no format is given and the error
    /// stream is a terminal. An unrecognized explicit value falls back to the
    /// TTY heuristic.
    ///
    /// # Arguments
    ///
    /// * `explicit` - The value of an explicit `--format` flag, if any.
    /// * `is_terminal` - Whether the error stream is a TTY.
    ///
    /// # Returns
    ///
    /// The selected [`OutputFormat`].
    #[must_use]
    pub fn select(explicit: Option<&str>, is_terminal: bool) -> Self {
        match explicit {
            Some("json") => Self::Json,
            Some("pretty") => Self::Pretty,
            _ if is_terminal => Self::Pretty,
            _ => Self::Json,
        }
    }
}

/// Per-variant problem-type metadata: the identity, version, status, and exit
/// code that compose an [`Error`](enum@crate::Error)'s envelope. Kept in one place so
/// extending the enum means adding a single arm in [`Error::meta`] (plus a
/// recovery arm in [`Error::to_problem`]), not editing several parallel matches.
struct ProblemMeta {
    /// Stable, URL-safe slug for the problem type.
    slug: &'static str,
    /// Version segment of the type URI (e.g. `"v1"`). Per-type, so one type can
    /// advance independently of the others.
    version: &'static str,
    /// Short, stable title for the problem type.
    title: &'static str,
    /// Numeric status class.
    status: u16,
    /// Process exit code emitted alongside the error.
    exit_code: u8,
}

impl Error {
    /// The stable, version-embedded problem-type URI for this error.
    ///
    /// Derived as `{base}/{slug}/{version}`, where `base` is the configurable
    /// [`ERROR_TYPE_BASE_URI`], `slug` is [`Error::type_slug`], and `version` is
    /// the per-type version. The version is the stability commitment: the
    /// meaning of a given URI never changes; a breaking change to a problem type
    /// ships a new version (e.g. `/v2`) rather than redefining the existing one.
    ///
    /// # Returns
    ///
    /// The fully-qualified type URI for this variant.
    #[must_use]
    pub fn type_uri(&self) -> String {
        let meta = self.meta();
        format!("{ERROR_TYPE_BASE_URI}/{}/{}", meta.slug, meta.version)
    }

    /// Stable, URL-safe slug identifying this error's problem type.
    ///
    /// Used both as the path segment in [`Error::type_uri`] and as the
    /// occurrence id in the `instance` URN. Stable across releases.
    ///
    /// # Returns
    ///
    /// The `'static` slug for this variant.
    #[must_use]
    pub const fn type_slug(&self) -> &'static str {
        self.meta().slug
    }

    /// All per-variant problem-type metadata in one place.
    const fn meta(&self) -> ProblemMeta {
        match self {
            Self::InvalidInput(_) => ProblemMeta {
                slug: "invalid-input",
                version: "v1",
                title: "Invalid input",
                status: 400,
                exit_code: 2,
            },
            Self::OperationFailed { .. } => ProblemMeta {
                slug: "operation-failed",
                version: "v1",
                title: "Operation failed",
                status: 500,
                exit_code: 1,
            },
        }
    }

    /// Maps this error to an RFC 9457 [`ProblemDetails`] envelope.
    ///
    /// The `detail` member is this error's `Display` string, so the human
    /// rendering and the machine `detail` never drift. Both current variants
    /// are non-transient, so `retry_after` is left `None` (serialized as JSON
    /// `null`). Every envelope carries a `suggested_fix` and a `code_action`,
    /// each with an [`Applicability`] marker.
    ///
    /// # Returns
    ///
    /// A fully-populated [`ProblemDetails`] for this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_template::{Error, divide};
    ///
    /// let err = divide(10, 0).unwrap_err();
    /// let problem = err.to_problem();
    ///
    /// assert_eq!(problem.problem_type, "https://zircote.com/rust-template/errors/invalid-input/v1");
    /// assert_eq!(problem.detail, "invalid input: divisor cannot be zero");
    /// assert_eq!(problem.retry_after, None);
    /// assert!(problem.suggested_fix.is_some());
    /// ```
    #[must_use]
    pub fn to_problem(&self) -> ProblemDetails {
        let (fix, action) = match self {
            Self::InvalidInput(_) => (
                SuggestedFix::new(
                    "Correct the input so it satisfies the documented constraints, then retry.",
                    Applicability::MaybeIncorrect,
                ),
                CodeAction::new(
                    "Replace the offending input with a valid value",
                    "quickfix",
                    Applicability::MaybeIncorrect,
                ),
            ),
            Self::OperationFailed { .. } => (
                SuggestedFix::new(
                    "Inspect the operation's operands; the operation could not complete.",
                    Applicability::Unspecified,
                ),
                CodeAction::new(
                    "Adjust the operands so the operation can complete",
                    "quickfix",
                    Applicability::Unspecified,
                ),
            ),
        };

        let meta = self.meta();
        ProblemDetails::new(
            self.type_uri(),
            meta.title,
            meta.status,
            self.to_string(),
            format!("urn:{}:{}", env!("CARGO_PKG_NAME"), meta.slug),
        )
        .with_exit_code(meta.exit_code)
        .with_suggested_fix(fix)
        .with_code_action(action)
    }

    /// Renders this error for the given [`OutputFormat`].
    ///
    /// Pretty rendering is byte-identical to the binary's historical
    /// `Error: {e}` line. JSON rendering is the compact RFC 9457 envelope.
    ///
    /// # Arguments
    ///
    /// * `format` - The format to render.
    ///
    /// # Returns
    ///
    /// The rendered error string (without a trailing newline).
    #[must_use]
    pub fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Pretty => format!("Error: {self}"),
            OutputFormat::Json => self.to_problem().to_json(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{divide, process};

    #[test]
    fn applicability_serializes_snake_case() {
        let json = serde_json::to_string(&Applicability::MachineApplicable).unwrap();
        assert_eq!(json, "\"machine_applicable\"");
        assert_eq!(Applicability::default(), Applicability::Unspecified);
    }

    #[test]
    fn builder_sets_every_extension() {
        let problem = ProblemDetails::new("t", "T", 429, "d", "urn:x")
            .with_retry_after(180)
            .with_suggested_fix(SuggestedFix::new("wait", Applicability::MachineApplicable))
            .with_code_action(CodeAction::new(
                "retry",
                "quickfix",
                Applicability::MachineApplicable,
            ))
            .with_exit_code(2);

        assert_eq!(problem.retry_after, Some(180));
        assert_eq!(problem.exit_code, Some(2));
        assert_eq!(problem.code_actions.len(), 1);
        assert_eq!(
            problem.suggested_fix.unwrap().applicability,
            Applicability::MachineApplicable
        );
    }

    #[test]
    fn invalid_input_maps_to_versioned_envelope() {
        let problem = divide(1, 0).unwrap_err().to_problem();

        assert_eq!(
            problem.problem_type,
            "https://zircote.com/rust-template/errors/invalid-input/v1"
        );
        assert!(problem.problem_type.ends_with("/v1"));
        assert_eq!(problem.title, "Invalid input");
        assert_eq!(problem.status, 400);
        assert_eq!(problem.detail, "invalid input: divisor cannot be zero");
        assert_eq!(problem.instance, "urn:rust_template:invalid-input");
        assert_eq!(problem.retry_after, None);
        assert_eq!(problem.exit_code, Some(2));
        assert_eq!(
            problem.suggested_fix.unwrap().applicability,
            Applicability::MaybeIncorrect
        );
        assert_eq!(
            problem.code_actions[0].applicability,
            Applicability::MaybeIncorrect
        );
    }

    #[test]
    fn operation_failed_maps_to_distinct_versioned_envelope() {
        let problem = process("-5").unwrap_err().to_problem();

        assert_eq!(
            problem.problem_type,
            "https://zircote.com/rust-template/errors/operation-failed/v1"
        );
        assert!(problem.problem_type.ends_with("/v1"));
        assert_eq!(problem.title, "Operation failed");
        assert_eq!(problem.status, 500);
        assert_eq!(problem.instance, "urn:rust_template:operation-failed");
        assert_eq!(problem.retry_after, None);
        assert_eq!(problem.exit_code, Some(1));
        assert_eq!(
            problem.code_actions[0].applicability,
            Applicability::Unspecified
        );
    }

    #[test]
    fn every_variant_has_a_distinct_type_uri() {
        let invalid = divide(1, 0).unwrap_err().type_uri();
        let failed = process("-1").unwrap_err().type_uri();
        assert_ne!(invalid, failed);
        assert!(invalid.ends_with("/v1"));
        assert!(failed.ends_with("/v1"));
    }

    #[test]
    fn type_uri_derives_from_the_configurable_base() {
        let err = divide(1, 0).unwrap_err();
        assert!(err.type_uri().starts_with(ERROR_TYPE_BASE_URI));
        assert_eq!(
            err.type_uri(),
            format!("{ERROR_TYPE_BASE_URI}/{}/v1", err.type_slug())
        );
    }

    #[test]
    fn json_envelope_carries_all_required_members() {
        let json = divide(1, 0).unwrap_err().to_problem().to_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Five RFC 9457 standard members.
        for member in ["type", "title", "status", "detail", "instance"] {
            assert!(value.get(member).is_some(), "missing {member}");
        }
        // Three agent extensions; retry_after present even when null.
        assert!(value.get("retry_after").is_some());
        assert!(value["retry_after"].is_null());
        assert!(value.get("suggested_fix").is_some());
        assert!(value.get("code_actions").is_some());
        // Applicability marker present on the fix and the action.
        assert!(value["suggested_fix"]["applicability"].is_string());
        assert!(value["code_actions"][0]["applicability"].is_string());
        // Optional exit_code extension is emitted when set (here, Some(2)).
        assert_eq!(value["exit_code"], 2);
    }

    #[test]
    fn pretty_render_is_byte_identical_to_display_line() {
        let err = divide(1, 0).unwrap_err();
        assert_eq!(err.render(OutputFormat::Pretty), format!("Error: {err}"));
    }

    #[test]
    fn json_render_matches_envelope_json() {
        let err = divide(1, 0).unwrap_err();
        assert_eq!(err.render(OutputFormat::Json), err.to_problem().to_json());
    }

    #[test]
    fn format_selection_honors_flag_then_tty() {
        assert_eq!(OutputFormat::select(Some("json"), true), OutputFormat::Json);
        assert_eq!(
            OutputFormat::select(Some("pretty"), false),
            OutputFormat::Pretty
        );
        assert_eq!(OutputFormat::select(None, true), OutputFormat::Pretty);
        assert_eq!(OutputFormat::select(None, false), OutputFormat::Json);
        // Unrecognized explicit value falls back to the TTY heuristic.
        assert_eq!(
            OutputFormat::select(Some("xml"), true),
            OutputFormat::Pretty
        );
    }

    #[test]
    fn envelope_round_trips_through_json() {
        let problem = divide(1, 0).unwrap_err().to_problem();
        let json = problem.to_json();
        let back: ProblemDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(problem, back);
    }

    #[test]
    fn pretty_json_is_indented() {
        let pretty = divide(1, 0).unwrap_err().to_problem().to_json_pretty();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  \"type\""));
    }
}
