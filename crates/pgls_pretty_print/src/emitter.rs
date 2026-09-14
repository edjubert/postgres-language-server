use crate::FormatConfig;
pub use crate::codegen::group_kind::GroupKind;
pub use crate::codegen::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    /// Must break (semicolon, etc.)
    Hard,
    /// Break if group doesn't fit
    Soft,
    /// Break if group doesn't fit, but collapse to space if it does
    SoftOrSpace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutEvent {
    Token(TokenKind),
    Space,
    Line(LineType),
    GroupStart { kind: GroupKind },
    GroupEnd,
    IndentStart,
    IndentEnd,
}

/// Collects layout events for the renderer.
///
/// The emitter holds the configuration because some options decide which tokens exist at all,
/// such as where a comma sits in a list, and not merely how a token is rendered.
#[derive(Debug)]
pub struct EventEmitter {
    pub events: Vec<LayoutEvent>,
    config: FormatConfig,
}

impl EventEmitter {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            events: Vec::new(),
            config,
        }
    }

    // Later option PRs inspect this while deciding which layout events to emit.
    #[allow(dead_code)]
    pub fn config(&self) -> &FormatConfig {
        &self.config
    }

    pub fn token(&mut self, token: TokenKind) {
        self.events.push(LayoutEvent::Token(token));
    }

    pub fn space(&mut self) {
        self.events.push(LayoutEvent::Space);
    }

    pub fn line(&mut self, line_type: LineType) {
        self.events.push(LayoutEvent::Line(line_type));
    }

    pub fn group_start(&mut self, kind: GroupKind) {
        self.events.push(LayoutEvent::GroupStart { kind });
    }

    pub fn group_end(&mut self) {
        self.events.push(LayoutEvent::GroupEnd);
    }

    pub fn indent_start(&mut self) {
        self.events.push(LayoutEvent::IndentStart);
    }

    pub fn indent_end(&mut self) {
        self.events.push(LayoutEvent::IndentEnd);
    }
}
