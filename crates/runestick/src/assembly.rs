//! Helpers for building assembly.

use crate::collections::HashMap;
use crate::{Hash, Inst, Span, UnitError};
use std::fmt;

/// A label that can be jumped to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label {
    name: &'static str,
    ident: usize,
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}_{}", self.name, self.ident)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AssemblyInst {
    Jump { label: Label },
    JumpIf { label: Label },
    JumpIfNot { label: Label },
    JumpIfBranch { branch: i64, label: Label },
    PopAndJumpIf { count: usize, label: Label },
    PopAndJumpIfNot { count: usize, label: Label },
    Raw { raw: Inst },
}

/// Helper structure to build instructions and maintain certain invariants.
#[derive(Debug, Clone, Default)]
pub struct Assembly {
    /// Label to offset.
    pub(crate) labels: HashMap<Label, usize>,
    /// Registered label by offset.
    pub(crate) labels_rev: HashMap<usize, Label>,
    /// Instructions with spans.
    pub(crate) instructions: Vec<(AssemblyInst, Span)>,
    /// Comments associated with instructions.
    pub(crate) comments: HashMap<usize, Vec<String>>,
    /// The number of labels.
    pub(crate) label_count: usize,
    /// The collection of functions required by this assembly.
    pub(crate) required_functions: HashMap<Hash, Vec<Span>>,
}

impl Assembly {
    /// Construct a new assembly.
    pub(crate) fn new(label_count: usize) -> Self {
        Self {
            labels: Default::default(),
            labels_rev: Default::default(),
            instructions: Default::default(),
            comments: Default::default(),
            label_count,
            required_functions: Default::default(),
        }
    }

    /// Construct and return a new label.
    pub fn new_label(&mut self, name: &'static str) -> Label {
        let label = Label {
            name,
            ident: self.label_count,
        };

        self.label_count += 1;
        label
    }

    /// Apply the label at the current instruction offset.
    pub fn label(&mut self, label: Label) -> Result<Label, UnitError> {
        let offset = self.instructions.len();

        if self.labels.insert(label, offset).is_some() {
            return Err(UnitError::DuplicateLabel { label });
        }

        self.labels_rev.insert(offset, label);
        Ok(label)
    }

    /// Add a jump to the given label.
    pub fn jump(&mut self, label: Label, span: Span) {
        self.instructions.push((AssemblyInst::Jump { label }, span));
    }

    /// Add a conditional jump to the given label.
    pub fn jump_if(&mut self, label: Label, span: Span) {
        self.instructions
            .push((AssemblyInst::JumpIf { label }, span));
    }

    /// Add a conditional jump to the given label.
    pub fn jump_if_not(&mut self, label: Label, span: Span) {
        self.instructions
            .push((AssemblyInst::JumpIfNot { label }, span));
    }

    /// Add a conditional jump-if-branch instruction.
    pub fn jump_if_branch(&mut self, branch: i64, label: Label, span: Span) {
        self.instructions
            .push((AssemblyInst::JumpIfBranch { branch, label }, span));
    }

    /// Add a pop-and-jump-if instruction to a label.
    pub fn pop_and_jump_if(&mut self, count: usize, label: Label, span: Span) {
        self.instructions
            .push((AssemblyInst::PopAndJumpIf { count, label }, span));
    }

    /// Add a pop-and-jump-if-not instruction to a label.
    pub fn pop_and_jump_if_not(&mut self, count: usize, label: Label, span: Span) {
        self.instructions
            .push((AssemblyInst::PopAndJumpIfNot { count, label }, span));
    }

    /// Push a raw instruction.
    pub fn push(&mut self, raw: Inst, span: Span) {
        if let Inst::Call { hash, .. } = raw {
            self.required_functions.entry(hash).or_default().push(span);
        }

        self.instructions.push((AssemblyInst::Raw { raw }, span));
    }

    /// Push a raw instruction.
    pub fn push_with_comment<C>(&mut self, raw: Inst, span: Span, comment: C)
    where
        C: AsRef<str>,
    {
        let pos = self.instructions.len();

        self.comments
            .entry(pos)
            .or_default()
            .push(comment.as_ref().to_owned());

        self.push(raw, span);
    }
}
