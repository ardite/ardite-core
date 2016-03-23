use inflector;

pub enum Case {
  Same,
  Camel,
  Kebab,
  Snake
}

impl Case {
  pub fn from_str(string: &str) -> Option<Self> {
    match string {
      "same" => Some(Case::Same),
      "camel" => Some(Case::Camel),
      "kebab" => Some(Case::Kebab),
      "snake" => Some(Case::Snake),
      _ => None
    }
  }

  pub fn to_case(&self, string: String) -> String {
    match *self {
      Case::Same => string,
      Case::Camel => inflector::cases::camelcase::to_camel_case(string),
      Case::Kebab => inflector::cases::kebabcase::to_kebab_case(string),
      Case::Snake => inflector::cases::snakecase::to_snake_case(string)
    }
  }
}
