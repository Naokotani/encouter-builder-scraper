mod parse_monster;
use clap::Parser;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    max: Option<usize>,
    #[arg(short, long)]
    url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let list_url = "https://pf2.d20pfsrd.com/monster/";
    let list_document = get_body(list_url).await?;
    let tr_selector = scraper::Selector::parse("tr>td>a").unwrap();
    let input = list_document.select(&tr_selector);

    let cli = Cli::parse();

    if let Some(s) = cli.url {
        let monster_document = get_body(&s).await?;
        let monster = parse_monster::Monster::new(&monster_document, &s);
        if monster.validate() {
            send_db(monster).await?;
        } else {
            println!("Monster did not validate:\n{}", monster);
        }
    } else {
        if let Some(s) = cli.max {
            println!("Fetching {} results", s);
        }

        for (i, node) in input.enumerate() {
            let href = node.attr("href").unwrap();
            if href.contains("/monster/")
                && !href.contains("-dragon/")
                && !href.contains("naiad-queen")
            {
                let monster_document = get_body(href).await?;
                let monster = parse_monster::Monster::new(&monster_document, href);
                if monster.validate() {
                    send_db(monster).await?;
                } else {
                    println!("Monster did not validate:\n{}", monster);
                }
            }

            if let Some(s) = cli.max {
                if i == s {
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn get_body(url: &str) -> Result<scraper::Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    let document = scraper::Html::parse_document(&resp);
    Ok(document)
}

async fn send_db(monster: parse_monster::Monster) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("Env var didn't load"))
        .await?;

    let id = sqlx::query!(
        r#"
INSERT INTO monsters_new (
"url",
"name",
"level",
"monster_type",
"alignment",
"size",
"aquatic",
"is_caster",
"is_ranged")
VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING creature_id;"#,
        monster.url,
        monster.name,
        monster.level,
        monster.monster_type,
        monster.alignment,
        monster.size,
        monster.is_aquatic,
        monster.is_caster,
        monster.is_ranged,
    )
    .fetch_one(&pool)
    .await?;

    println!("{}", id.creature_id);

    for t in monster.traits {
        sqlx::query!(
            r#"
INSERT INTO traits_new ("creature_id", "trait") VALUES ($1, $2);
"#,
            id.creature_id,
            t
        )
        .execute(&pool)
        .await?;
    }
    Ok(())
}

