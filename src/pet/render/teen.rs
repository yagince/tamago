use super::PetArt;

pub const TEEN: &[PetArt] = &[
    // クール
    PetArt {
        creature_type: "クール",
        color: "\x1b[94m",
        art: "\
\n      ▄▄▄\
\n    ▄█   █▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █   ─   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    },
    // やんちゃ
    PetArt {
        creature_type: "やんちゃ",
        color: "\x1b[91m",
        art: "\
\n    ▄█▄ ▄█▄\
\n   █ ◉   ◉ █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    },
    // 凛々しい
    PetArt {
        creature_type: "凛々しい",
        color: "\x1b[96m",
        art: "\
\n    ╱▀▀▀▀╲\
\n   █ ▪   ▪ █\
\n   █       █\
\n   █   △   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    },
    // 元気
    PetArt {
        creature_type: "元気",
        color: "\x1b[91m",
        art: "\
\n      ▄▄▄\
\n    ▄█   █▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █  ◡◡   █\
\n   ██     ██\
\n    █     █\
\n    █▀   █▀\n",
    },
];
