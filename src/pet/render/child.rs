use super::PetArt;

pub const CHILD: &[PetArt] = &[
    // 元気
    PetArt {
        creature_type: "元気",
        color: "\x1b[91m",
        art: "\
\n     ▄███▄\
\n   █ ●   ● █\
\n   █ ░   ░ █\
\n   █  ▽▽   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    },
    // おすまし
    PetArt {
        creature_type: "おすまし",
        color: "\x1b[94m",
        art: "\
\n     ▄███▄\
\n   █ ◉   ◉ █\
\n   █       █\
\n   █   ─   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    },
    // にっこり
    PetArt {
        creature_type: "にっこり",
        color: "\x1b[93m",
        art: "\
\n     ▄███▄\
\n   █ ^   ^ █\
\n   █ ░   ░ █\
\n   █   ω   █\
\n   █       █\
\n    █     █\
\n     █▄ █▄\n",
    },
    // わくわく
    PetArt {
        creature_type: "わくわく",
        color: "\x1b[95m",
        art: "\
\n     ▄███▄\
\n   █ ★   ★ █\
\n   █ ░   ░ █\
\n   █   ▽   █\
\n   █  ♪    █\
\n    █     █\
\n     █▄ █▄\n",
    },
];
