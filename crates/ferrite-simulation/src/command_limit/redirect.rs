//! Strict ordinary-redirect cardinality and error routing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierResult<T> {
    Outputs(Vec<T>),
    SyntaxError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSource {
    OriginalCommandSource,
    CurrentSource { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectErrorKind {
    ForkLimit { limit: i32 },
    ModifierSyntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedirectError {
    pub kind: RedirectErrorKind,
    pub source: ErrorSource,
    pub tracer_receives_error: bool,
    pub user_facing_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectPlan<T> {
    pub automatic_cost: i32,
    pub outputs: Vec<T>,
    pub errors: Vec<RedirectError>,
    pub aborted: bool,
    pub executable_scheduled: bool,
}

pub fn evaluate_standard_redirect<T>(
    fork_limit: i32,
    already_forked: bool,
    results_in_source_order: impl IntoIterator<Item = ModifierResult<T>>,
) -> RedirectPlan<T> {
    let mut outputs = Vec::new();
    let mut errors = Vec::new();
    for (index, result) in results_in_source_order.into_iter().enumerate() {
        match result {
            ModifierResult::Outputs(new_outputs) => {
                if outputs.len().saturating_add(new_outputs.len()) >= fork_limit.max(0) as usize {
                    errors.push(RedirectError {
                        kind: RedirectErrorKind::ForkLimit { limit: fork_limit },
                        source: ErrorSource::OriginalCommandSource,
                        tracer_receives_error: true,
                        user_facing_failure: !already_forked,
                    });
                    return RedirectPlan {
                        automatic_cost: 1,
                        outputs: Vec::new(),
                        errors,
                        aborted: true,
                        executable_scheduled: false,
                    };
                }
                outputs.extend(new_outputs);
            }
            ModifierResult::SyntaxError => {
                errors.push(RedirectError {
                    kind: RedirectErrorKind::ModifierSyntax,
                    source: ErrorSource::CurrentSource { index },
                    tracer_receives_error: true,
                    user_facing_failure: !already_forked,
                });
                if !already_forked {
                    return RedirectPlan {
                        automatic_cost: 1,
                        outputs: Vec::new(),
                        errors,
                        aborted: true,
                        executable_scheduled: false,
                    };
                }
            }
        }
    }
    let executable_scheduled = !outputs.is_empty();
    RedirectPlan {
        automatic_cost: 1,
        outputs,
        errors,
        aborted: false,
        executable_scheduled,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomRedirectPlan {
    pub automatic_cost: i32,
    pub generic_fork_limit_checked: bool,
    pub returns_from_generic_stage: bool,
}

pub const CUSTOM_REDIRECT_PLAN: CustomRedirectPlan = CustomRedirectPlan {
    automatic_cost: 0,
    generic_fork_limit_checked: false,
    returns_from_generic_stage: true,
};
