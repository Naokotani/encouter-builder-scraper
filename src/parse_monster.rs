use core::fmt;
use scraper::{Html, Selector};

pub struct Monster {
    pub url: String,
    pub name: String,
    pub level: i32,
    pub alignment: String,
    pub monster_type: String,
    pub size: String,
    pub traits: Vec<String>,
    pub is_caster: bool,
    pub is_ranged: bool,
    pub is_aquatic: bool,
}

impl Monster {
    pub fn new(document: &Html, monster_url: &str) -> Monster {
        println!("Gettting data for {}", monster_url);
        let title = Title::new(document, monster_url);
        let traits = Traits::new(document);

        Monster {
            url: title.url.trim().to_string(),
            name: title.name.trim().to_string(),
            level: title.level,
            alignment: traits.alignment.trim().to_string(),
            monster_type: traits.monster_type.trim().to_string(),
            size: traits.size.trim().to_string(),
            traits: traits.traits,
            is_caster: is_type(document, "Spells"),
            is_ranged: is_type(document, "Ranged"),
            is_aquatic: traits.aquatic,
        }
    }

    pub fn validate(&self) -> bool {
        // Database table layout
        //                  Table "public.monsters_new"
        //     Column    |          Type          | Collation | Nullable |
        // --------------+------------------------+-----------+----------+
        //  creature_id  | integer                |           | not null |
        //  url          | character varying(100) |           |          |
        //  name         | character varying(100) |           |          |
        //  level        | integer                |           |          |
        //  alignment    | character varying(15)  |           |          |
        //  monster_type | character varying(100) |           |          |
        //  size         | character varying(15)  |           |          |
        //  is_caster    | boolean                |           |          |
        //  is_ranged    | boolean                |           |          |
        //  aquatic      | boolean                |           |          |

        match &self.name {
            n if n.len() > 100 => return false,
            n if n.is_empty() => return false,
            _ => (),
        }

        match self.level {
            l if l > 25 => return false,
            _ => ()
        }

        match &self.alignment {
            a if a.len() > 2 => return false,
            _ => ()
        }

        match &self.monster_type {
            m if m.len() > 100 => return false,
            m if m.is_empty() => return false,
            _ => ()
        }

        match &self.size {
            s if s.len() > 20 => return false,
            _ => ()
        }

        for t in &self.traits {
            match t {
                t if t.len() > 50 => return false,
                t if t.is_empty() => return false,
                _ => ()
            }
        }
        true
    }
}

impl fmt::Display for Monster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\
name: {}
level: {}
type: {}
alignment: {}
size: {}
traits: {:?}
Caster? {}
Ranged? {}
Aquatic? {}\n",
            self.name,
            self.level,
            self.monster_type,
            self.alignment,
            self.size,
            self.traits,
            self.is_caster,
            self.is_ranged,
            self.is_aquatic,
        )
    }
}

struct Traits {
    alignment: String,
    monster_type: String,
    size: String,
    aquatic: bool,
    traits: Vec<String>,
}

impl Traits {
    pub fn new(document: &Html) -> Traits {
        let alignment_selector_short = Selector::parse(r#"span[class=alignment]"#).unwrap();
        let alignment_string_short = document
            .select(&alignment_selector_short)
            .map(|x| x.inner_html())
            .next();

        let alignment_selector_long = Selector::parse(r#"span[class=creature-alignment]"#).unwrap();
        let alignment_string_long = document
            .select(&alignment_selector_long)
            .map(|x| x.inner_html())
            .next();

        let type_selector_long = Selector::parse(r#"span[class=type]"#).unwrap();
        let type_string = document
            .select(&type_selector_long)
            .map(|x| x.inner_html())
            .next();

        let alignment: String = if let Some(s) = alignment_string_long {
            s
        } else if let Some(s) = alignment_string_short {
            s
        } else {
            String::from("NO ALIGN")
        };

        let traits_selector = Selector::parse(r#"span[class=trait]"#).unwrap();
        let traits: Vec<String> = document
            .select(&traits_selector)
            .map(|x| x.inner_html())
            .collect();

        let size_selector = Selector::parse(r#"span[class=size]"#).unwrap();
        let size_base = document
            .select(&size_selector)
            .map(|x| x.inner_html())
            .next();

        let anchor_selector = Selector::parse("a").unwrap();
        let mut clean_traits: Vec<String> = Vec::new();
        for t in traits {
            let anchor = Html::parse_fragment(&t)
                .select(&anchor_selector)
                .map(|x| x.inner_html())
                .next();
            if let Some(s) = anchor {
                clean_traits.push(s.trim().to_string());
            } else {
                clean_traits.push(t.trim().to_string());
            }
        }

        let mut traits = clean_traits;

        let size: String = if let Some(s) = size_base {
            s
        } else {
            let mut size_string = String::from("NO SIZE");
            let mut new_traits: Vec<String> = Vec::new();
            for t in traits {
                match t.as_str() {
                    "Tiny" => size_string = t,
                    "Small" => size_string = t,
                    "Medium" => size_string = t,
                    "Large" => size_string = t,
                    "Huge" => size_string = t,
                    "Gargantuan" => size_string = t,
                    _ => new_traits.push(t),
                };
            }
            traits = new_traits;
            size_string
        };

        let monster_type = if let Some(s) = type_string {
            s
        } else {
            let mut type_string = String::from("NO TYPE");
            let mut new_traits: Vec<String> = Vec::new();
            for t in traits {
                match t.as_str() {
                    "Aberration" => type_string = t,
                    "Animal" => type_string = t,
                    "Beast" => type_string = String::from("Animal"),
                    "Artificial intelligence " => type_string = t,
                    "Elemental" => type_string = t,
                    "Construct" => type_string = t,
                    "Dragon" => type_string = t,
                    "Fey" => type_string = t,
                    "Humanoid" => type_string = t,
                    "Magical beast " => type_string = t,
                    "Monstrous humanoid " => type_string = t,
                    "Ooze" => type_string = t,
                    "Outsider" => type_string = t,
                    "Celestial" => type_string = String::from("Outsider"),
                    "Fiend" => type_string = String::from("Outsider"),
                    "Monitor" => type_string = t,
                    "Plant" => type_string = t,
                    "Undead" => type_string = t,
                    "Vermin" => type_string = t,
                    _ => new_traits.push(t),
                };
            }
            traits = new_traits;
            type_string
        };

        let mut aquatic = false;
        let mut new_traits: Vec<String> = Vec::new();

        for t in traits {
            match t.as_str() {
                "Amphibious" => aquatic = true,
                "Aquatic" => aquatic = true,
                _ => new_traits.push(t),
            }
        }

        traits = new_traits;

        Traits {
            alignment,
            monster_type,
            size,
            aquatic,
            traits,
        }
    }
}

struct Title {
    url: String,
    name: String,
    level: i32,
}

impl Title {
    pub fn new(document: &Html, url: &str) -> Title {
        let name_selector = Selector::parse(r#"h4[class="monster"]"#).unwrap();
        let mut html = document.select(&name_selector).map(|x| x.inner_html());
        let title_string = html.next();
        if let Some(s) = title_string {
            parse_title(s, url)
        } else {
            let name_selector = Selector::parse("h4").unwrap();
            let html = document.select(&name_selector).map(|x| x.inner_html());

            let mut title = Title {
                url: String::from(url),
                name: String::from("Parse Failed"),
                level: 0,
            };

            for tag in html {
                if tag.contains("level") {
                    println!("{}", tag);
                    title = parse_title(tag, url);
                }
            }
            title
        }
    }
}

fn parse_title(title: String, url: &str) -> Title {
    let title_split = title.split_once('<');
    match title_split {
        Option::Some(s) => Title {
            url: String::from(url),
            name: s.0.to_string(),
            level: get_digit(s.1, s.0),
        },
        None => panic!("Failed to parse title string"),
    }
}

fn is_type(document: &Html, type_str: &str) -> bool {
    let selector = Selector::parse(r#"p>b"#).unwrap();
    let headers = document.select(&selector).map(|x| x.inner_html());
    for header in headers {
        if header.contains(type_str) {
            return true;
        }
    }
    false
}

fn get_digit(string: &str, name: &str) -> i32 {
    let chars = string.chars();
    let mut num_string = String::new();
    let mut previous_char = 'a';
    for char in chars {
        if char.is_ascii_digit() && previous_char == '-' {
            num_string.push_str("-1");
        } else if char.is_ascii_digit() {
            num_string.push_str(&char.to_string())
        }
        previous_char = char;
    }

    match num_string.parse::<i32>() {
        Ok(d) => d,
        Err(e) => panic!("Didn't find digit for {} {}", name, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monster_validation() {
        let long_string = String::from(
            "
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
aaaaaaaaaaaa
",
        );

        //This test should pass
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 19,
            alignment: String::from("CE"),
            monster_type: String::from("Undead"),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![monster.validate()];
        //Test long name over 100 characters
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: long_string.clone(),
            level: 19,
            alignment: String::from("CE"),
            monster_type: String::from("Undead"),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![!monster.validate()];

        //Test level over 26
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 26,
            alignment: String::from("CE"),
            monster_type: String::from("Undead"),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };
        assert![!monster.validate()];

        //Test long alignment over 2
        let alignment = String::from("123");
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 19,
            alignment,
            monster_type: String::from("Undead"),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![!monster.validate()];

        //Test monster type over 100
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 19,
            alignment: String::from("CE"),
            monster_type: long_string.clone(),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![!monster.validate()];

        //Test long size over 20
        let size = String::from("1234567890123456789012312");
        let traits = vec![
            String::from("Fast"),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 19,
            alignment: long_string.clone(),
            monster_type: String::from("Undead"),
            size,
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![!monster.validate()];

        //Test for long traits
        let traits = vec![
            long_string.clone(),
            String::from("Slow"),
            String::from("speedy"),
        ];
        let monster = Monster {
            url: String::from("www.foo.com"),
            name: String::from("ghost"),
            level: 19,
            alignment: String::from("CE"),
            monster_type: String::from("Undead"),
            size: String::from("Tiny"),
            traits,
            is_caster: false,
            is_ranged: false,
            is_aquatic: false,
        };

        assert![!monster.validate()];
    }
}
