use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
    Statusline,
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

const STATUSLINE_HOOK: &str = "\
# tamago - terminal pet\n\
tamago tick --claude-turn &>/dev/null &\n\
pet_status=$(tamago status 2>/dev/null)\n";

pub fn run(shell: &Shell) {
    match shell {
        Shell::Zsh => print!("{ZSH_HOOK}"),
        Shell::Bash => print!("{BASH_HOOK}"),
        Shell::Statusline => {
            print!("{STATUSLINE_HOOK}");
            eprintln!("~/.claude/statusline-command.sh に以下を追記してください:");
            eprintln!();
            eprintln!("  1. tamago hook statusline >> ~/.claude/statusline-command.sh");
            eprintln!("  2. printf の出力に $pet_status を追加");
            eprintln!("     例: printf '... %s' \"$pet_status\"");
        }
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

    #[test]
    fn statusline_hook_contains_status() {
        assert!(STATUSLINE_HOOK.contains("tamago status"));
        assert!(STATUSLINE_HOOK.contains("tamago tick --claude-turn"));
    }
}
