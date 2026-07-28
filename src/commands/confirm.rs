use std::io::{self, IsTerminal, Write};

/// Require an explicit confirmation for a state-changing operation.
///
/// Automation has no safe interactive fallback: it must pass --yes. JSON
/// output deliberately follows the same rule so it cannot accidentally turn
/// into an execution bypass.
pub(crate) fn require_confirmation(summary: &str, yes: bool) -> anyhow::Result<()> {
    if yes {
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        anyhow::bail!(
            "This operation changes server state; rerun with --yes in non-interactive mode."
        );
    }

    println!("{summary}");
    print!("Continue? [y/N] ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        anyhow::bail!("Operation cancelled; no request was sent.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yes_bypasses_prompt() {
        require_confirmation("test", true).expect("--yes should proceed");
    }

    #[test]
    fn non_interactive_mode_requires_yes_before_any_request() {
        if !io::stdin().is_terminal() {
            let error = require_confirmation("test", false)
                .expect_err("non-interactive state-changing commands must require --yes");
            assert!(error.to_string().contains("--yes"));
        }
    }
}
