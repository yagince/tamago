use crate::pet::Category;

pub struct Score {
    pub exp: u64,
    pub category: Category,
}

pub fn score(cmd: &str) -> Score {
    let mut parts = cmd.split_whitespace();
    let base = match parts.next() {
        Some(s) => s,
        None => {
            return Score {
                exp: 1,
                category: Category::Basic,
            };
        }
    };

    // コマンド名からパスを除去（/usr/bin/git → git）
    let name = base.rsplit('/').next().unwrap_or(base);
    let sub = parts.next().unwrap_or("");

    match name {
        "git" => {
            let bonus = match sub {
                "commit" => 10,
                "push" => 8,
                "status" => 3,
                _ => 0,
            };
            Score {
                exp: 10 + bonus,
                category: Category::Git,
            }
        }
        "claude" => Score {
            exp: 15,
            category: Category::Ai,
        },
        "cargo" | "rustc" | "go" | "gcc" | "g++" | "javac" | "python" | "ruby" | "node" => Score {
            exp: 8,
            category: Category::Dev,
        },
        "docker" | "kubectl" | "gcloud" | "terraform" | "aws" => Score {
            exp: 8,
            category: Category::Infra,
        },
        "emacs" | "emacsclient" | "vim" | "nvim" | "vi" => Score {
            exp: 5,
            category: Category::Editor,
        },
        "make" | "npm" | "yarn" | "pnpm" | "bundle" | "pip" => Score {
            exp: 5,
            category: Category::Dev,
        },
        "ssh" | "scp" | "rsync" => Score {
            exp: 4,
            category: Category::Infra,
        },
        "cd" | "ls" | "cat" | "pwd" | "echo" | "true" | "false" => Score {
            exp: 1,
            category: Category::Basic,
        },
        _ => Score {
            exp: 2,
            category: Category::Other,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_commit_scores_20() {
        let s = score("git commit -m fix");
        assert_eq!(s.exp, 20);
        assert_eq!(s.category, Category::Git);
    }

    #[test]
    fn git_push_scores_18() {
        let s = score("git push origin main");
        assert_eq!(s.exp, 18);
        assert_eq!(s.category, Category::Git);
    }

    #[test]
    fn git_status_scores_13() {
        let s = score("git status");
        assert_eq!(s.exp, 13);
        assert_eq!(s.category, Category::Git);
    }

    #[test]
    fn git_base_scores_10() {
        let s = score("git log");
        assert_eq!(s.exp, 10);
        assert_eq!(s.category, Category::Git);
    }

    #[test]
    fn claude_scores_15_ai() {
        let s = score("claude");
        assert_eq!(s.exp, 15);
        assert_eq!(s.category, Category::Ai);
    }

    #[test]
    fn cargo_scores_8_dev() {
        let s = score("cargo build");
        assert_eq!(s.exp, 8);
        assert_eq!(s.category, Category::Dev);
    }

    #[test]
    fn docker_scores_8_infra() {
        let s = score("docker run nginx");
        assert_eq!(s.exp, 8);
        assert_eq!(s.category, Category::Infra);
    }

    #[test]
    fn vim_scores_5_editor() {
        let s = score("vim src/main.rs");
        assert_eq!(s.exp, 5);
        assert_eq!(s.category, Category::Editor);
    }

    #[test]
    fn npm_scores_5_dev() {
        let s = score("npm install");
        assert_eq!(s.exp, 5);
        assert_eq!(s.category, Category::Dev);
    }

    #[test]
    fn ssh_scores_4_infra() {
        let s = score("ssh user@host");
        assert_eq!(s.exp, 4);
        assert_eq!(s.category, Category::Infra);
    }

    #[test]
    fn ls_scores_1_basic() {
        let s = score("ls -la");
        assert_eq!(s.exp, 1);
        assert_eq!(s.category, Category::Basic);
    }

    #[test]
    fn unknown_scores_2_other() {
        let s = score("htop");
        assert_eq!(s.exp, 2);
        assert_eq!(s.category, Category::Other);
    }

    #[test]
    fn empty_scores_1_basic() {
        let s = score("");
        assert_eq!(s.exp, 1);
        assert_eq!(s.category, Category::Basic);
    }

    #[test]
    fn full_path_command() {
        let s = score("/usr/bin/git commit -m test");
        assert_eq!(s.exp, 20);
        assert_eq!(s.category, Category::Git);
    }
}
