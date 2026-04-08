use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
}

const ZSH_HOOK: &str = r#"# tamago - terminal pet
_tamago_preexec() { command tamago tick --cmd "$1" &!; }
autoload -Uz add-zsh-hook
add-zsh-hook preexec _tamago_preexec
"#;

const BASH_HOOK: &str = r#"# tamago - terminal pet
_tamago_preexec() { command tamago tick --cmd "$1" & disown; }
trap '_tamago_preexec "$BASH_COMMAND"' DEBUG
"#;

pub fn run(shell: &Shell) {
    match shell {
        Shell::Zsh => print!("{ZSH_HOOK}"),
        Shell::Bash => print!("{BASH_HOOK}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_hook_contains_preexec() {
        assert!(ZSH_HOOK.contains("_tamago_preexec"));
        assert!(ZSH_HOOK.contains("add-zsh-hook preexec"));
        assert!(ZSH_HOOK.contains("tamago tick --cmd"));
        assert!(ZSH_HOOK.contains("&!"));
    }

    #[test]
    fn bash_hook_contains_debug_trap() {
        assert!(BASH_HOOK.contains("_tamago_preexec"));
        assert!(BASH_HOOK.contains("trap"));
        assert!(BASH_HOOK.contains("DEBUG"));
        assert!(BASH_HOOK.contains("tamago tick --cmd"));
    }
}
