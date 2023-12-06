use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::Debug;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_text_size::Ranged;

use crate::{
    checkers::ast::Checker, docstrings::Docstring, rules::pydocstyle::settings::Convention,
};

/// ## What it does
/// Checks that docstring contains documentation on what is returned.
///
/// ## Why is this bad?
/// Docstrings are a good way to document the code,
/// and including information on the return value from a function helps to
/// understand what the function does.
///
/// ## Example
/// ```python
/// def integer_sum(a: int, b: int):  # [missing-return-doc]
///     """Returns sum of two integers
///     :param a: first integer
///     :param b: second integer
///     """
///     return a + b
/// ```
///
/// Use instead:
/// ```python
/// def integer_sum(a: int, b: int) -> int:
///     """Returns sum of two integers
///     :param a: first integer
///     :param b: second integer
///     :return: sum of parameters a and b
///     """
///     return a + b
/// ```
///
/// ## Options
/// - `pylint.convention`
#[violation]
pub struct MissingReturnDoc;

impl Violation for MissingReturnDoc {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Docstring missing documentation on what is returned")
    }
}

/// PLW9011
pub(crate) fn missing_return_doc(checker: &mut Checker, docstring: &Docstring) {
    static REST: Lazy<Regex> = Lazy::new(|| Regex::new(":return:").unwrap());
    static NUMPY: Lazy<Regex> = Lazy::new(|| Regex::new(r"Returns\n\s*-------\n").unwrap());
    static GOOGLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"Returns:\n").unwrap());

    if let Some(convention) = checker.settings.pylint.convention {
        let has_return_documentation = match convention {
            Convention::Google => &GOOGLE,
            Convention::Numpy => &NUMPY,
            Convention::Pep257 => &REST,
        }
        .is_match(docstring.contents);

        let is_public_method_with_return =
            docstring
                .definition
                .as_function_def()
                .map_or(false, |function| {
                    !function.name.starts_with('_')
                        && function
                            .body
                            .iter()
                            .filter_map(|statement| statement.as_return_stmt())
                            .any(|return_statement| {
                                return_statement
                                    .value
                                    .as_deref()
                                    .is_some_and(|value| !value.is_none_literal_expr())
                            })
                });

        if is_public_method_with_return && !has_return_documentation {
            checker
                .diagnostics
                .push(Diagnostic::new(MissingReturnDoc, docstring.range()));
        }
    }
}