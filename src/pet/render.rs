use super::{Archetype, Stage};

pub fn ascii_art(stage: &Stage, archetype: &Option<Archetype>) -> &'static str {
    match stage {
        Stage::Egg => EGG,
        Stage::Baby => BABY,
        Stage::Child => CHILD,
        Stage::Teen => TEEN,
        Stage::Adult => match archetype {
            Some(Archetype::Versionist) => ADULT_VERSIONIST,
            Some(Archetype::AiMage) => ADULT_AIMAGE,
            Some(Archetype::CloudDweller) => ADULT_CLOUD,
            Some(Archetype::AncientMage) => ADULT_ANCIENT,
            Some(Archetype::Generalist) | None => ADULT_GENERALIST,
        },
    }
}

const EGG: &str = r"
    ___
   /   \
  |     |
  |     |
   \___/
";

const BABY: &str = r"
    ^__^
   (o  o)
   / ** \
    ||||
";

const CHILD: &str = r"
   \( ^o^ )/
    |    |
   / \  / \
  ~       ~
";

const TEEN: &str = r"
    /\_/\
   ( o.o )
   > ^ <
  /|    |\
 (_|    |_)
";

const ADULT_VERSIONIST: &str = r"
    ___
   /o o\
  ( === )
 /||   ||\
(_||   ||_)
   \_^_/
  🐙 Git Master
";

const ADULT_AIMAGE: &str = r"
    /\
   /  \
  | ** |
  |/~~\|
  /    \
 / \  / \
  🧙 AI Mage
";

const ADULT_CLOUD: &str = r"
    .---.
   (     )
  (       )
 (    *    )
  (       )
   '---'
  ☁️ Cloud Dweller
";

const ADULT_ANCIENT: &str = r"
   .----.
  / .--. \
 | | ** | |
 | | /\ | |
 | '----' |
  \______/
  📜 Ancient Mage
";

const ADULT_GENERALIST: &str = r"
   /\_/\
  ( o.o )
  (> * <)
  /|   |\
 / |   | \
  🦊 Generalist
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_stage_has_art() {
        assert!(!ascii_art(&Stage::Egg, &None).is_empty());
        assert!(!ascii_art(&Stage::Baby, &None).is_empty());
        assert!(!ascii_art(&Stage::Child, &None).is_empty());
        assert!(!ascii_art(&Stage::Teen, &None).is_empty());
        assert!(!ascii_art(&Stage::Adult, &None).is_empty());
    }

    #[test]
    fn adult_archetypes_have_different_art() {
        let versionist = ascii_art(&Stage::Adult, &Some(Archetype::Versionist));
        let aimage = ascii_art(&Stage::Adult, &Some(Archetype::AiMage));
        let generalist = ascii_art(&Stage::Adult, &None);
        assert_ne!(versionist, aimage);
        assert_ne!(aimage, generalist);
    }
}
