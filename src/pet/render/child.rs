use super::PetArt;

pub const CHILD: &[PetArt] = &[
    // 元気
    PetArt {
        creature_type: "元気",
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
