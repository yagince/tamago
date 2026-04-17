use super::{PetArt, PetColor};

pub const CHILD: &[PetArt] = &[
    // 子鳥
    PetArt {
        creature_type: "子鳥",
        color: PetColor::Yellow,
        art: "\
\n     ████████\
\n  ▄▄▀▄▄▄▄▄▄▄▄▀▄▄\
\n  ██▀▀▀▀▀▀▀▀▀▀██\
\n▄▄█            █▄▄\
\n██  ▀    ▀▀     ██\
\n██     █▀       ██\
\n██              ██\
\n▀▀▄▄          ▄▄▀▀\
\n    █▄▄▄▄▄▄▄▄█\
\n     ██    ██\
\n    ▄██  ▄▄██\n",
    },
    // 子うさぎ
    PetArt {
        creature_type: "子うさぎ",
        color: PetColor::White,
        art: "\
\n        ▄▀▀█   ▄▄▀▀▄\
\n      ▄▀  ▄█ ▄▀  ▄▀\
\n    ▄▀  ▄▀▀▀▀█ ▄█\
\n  ▄▀        ▀█▄\
\n ▄█ ▄▄   ▄    ▀▄\
\n▄██            █▄\
\n███            ███\
\n ▀▀            ██▀\
\n   █▄         ▄▀\
\n    ▀█▄    ▄▄█▀\
\n      ██▀▀▀██\
\n    ███    ██\n",
    },
    // 子猫
    PetArt {
        creature_type: "子猫",
        color: PetColor::Yellow,
        art: "\
\n    █        █\
\n  ██ ██    ██ ██\
\n  ██ ▀██████▀ ██\
\n▄▄▀▀          ▀▀▄▄\
\n██  ▄      ▄    ██\
\n██  ▀  ▄▄  ▀▀   ██\
\n▀▀▄▄   ▀▀     ▄▄▀▀\
\n  ██▄        ▄██\
\n  ▀▀██████████▀▀\
\n     ██     ▀█\
\n    ███    ███\n",
    },
    // 子狐
    PetArt {
        creature_type: "子狐",
        color: PetColor::Red,
        art: "\
\n    ▄▄▀▀▀▀▀▀▄▄\
\n  ▄▀          ▀▄\
\n █              █\
\n█        ▀    ▄  █\
\n█             █  █\
\n▀▄ ▄          ▀ ▄▀\
\n  ▀█         █\
\n     ▀▄▄▄▄▄▄▀▀\
\n     ▄█    █▄\n",
    },
    // 子犬
    PetArt {
        creature_type: "子犬",
        color: PetColor::Yellow,
        art: "\
\n    ███████\
\n ▄█▀       ██\
\n██▀ ▄   ▄  ▀██\
\n██  ▀▄▄ ▀    █\
\n▀▀▄▄ ▀▀    ▄▄▀▄▄ ▄\
\n  ▀▀▄▄▄▄▄▄▄▀  ▀█▄▀\
\n    █▀▀▀▀▀     █▀\
\n    ██▄     ▄██\
\n     ██████████\
\n     ▀▀ █  ▀▀▀▀\n",
    },
    // 子スライム
    PetArt {
        creature_type: "子スライム",
        color: PetColor::Green,
        art: "\
\n    ▄█▀▀▀▀▀▀█▄\
\n  ▄▀          ▀▄\
\n █              █\
\n█   ▄▄     ▄▄    █\
\n█   ▀▀     ▀▀    █\
\n█      ▀▀▀       █\
\n ▀▄            ▄▀\
\n   ▀█ ▄▄▄▄▄▄ █▀\
\n    █▄█    █▄█\n",
    },
];
