//! Collections of my favorite news source.
use crate::ticker::Ticker;
use crate::utils::{fetch_text, OptionResult};
use crate::{care, include_page};
use anyhow::Result;
use axum::http::header::{HeaderName, CACHE_CONTROL, CONTENT_ENCODING, EXPIRES, REFRESH};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::io::Read as _;
use std::sync::Mutex; // with small data, Mutex seems faster than RwLock
use std::time::{Duration, SystemTime};

fn generate(mut i: &str, o: &mut String, mut limit: usize) -> Result<()> {
    while let Some(mut p) = i.split_once("<item>") {
        limit -= 1;

        *o += "\n<details>\n";

        // title
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        *o += "<summary>";
        *o += p.0;
        *o += "</summary>\n";

        // content
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        *o += "<section>";
        let break_marks = [
            "br>", "p>", "p ", "/p>", "div>", "div ", "/div>", "li>", "li ", "/li>",
        ];
        while let Some(v) = p.0.split_once('<') {
            p.0 = v.1.split_once('>').e()?.1;
            let c = v.0.trim();
            if !c.is_empty() {
                *o += c;
            }
            if !o.ends_with("<br>") {
                for mark in break_marks {
                    if v.1.starts_with(mark) {
                        *o += "<br>";
                        break;
                    }
                }
            }
        }
        *o += "</section>\n";

        // link
        i = p.1.split_once("<link>").e()?.1;
        p = i.split_once("</link>").e()?;
        *o += "<a href=\"";
        *o += p.0;
        *o += "\">[ Original Link ]</a>\n";

        *o += "</details>\n";
        i = p.1;

        if limit == 0 {
            break;
        }
    }
    *o += "\n<br>\n";
    Ok(())
}

type Res = ([(HeaderName, String); 2], Html<Vec<u8>>);

const PAGE: [&str; 2] = include_page!("page.html");

static CACHE: Lazy<Mutex<Res>> = Lazy::new(|| {
    let body = format!("{}<h2>Magazine is generating ...</h2>{}", PAGE[0], PAGE[1]);
    Mutex::new((
        [(CACHE_CONTROL, "no-store".into()), (REFRESH, "2".into())],
        Html(body.into_bytes()),
    ))
});

async fn refresh() -> Result<()> {
    let expires = httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(3600));
    // https://rsshub.app | https://rsshub.uneasy.win | https://rss.itggg.cn
    let r = tokio::join!(
        fetch_text("https://rsshub.uneasy.win/leetcode/dailyquestion/solution/en"),
        fetch_text("https://rsshub.uneasy.win/bbc"),
        fetch_text("https://rsshub.uneasy.win/zhihu/daily"),
        fetch_text("https://rsshub.uneasy.win/oschina/news/industry"),
        fetch_text("https://rsshub.uneasy.win/1point3acres/post/hot3"),
        fetch_text("https://rsshub.uneasy.win/rustcc/jobs"),
    );
    let o = tokio::task::spawn_blocking(move || {
        let mut o = PAGE[0].to_string();
        r.0.map(|v| generate(&v, &mut o, 5)).ok();
        r.1.map(|v| generate(&v, &mut o, 9)).ok();
        r.2.map(|v| generate(&v, &mut o, 9)).ok();
        r.3.map(|v| generate(&v, &mut o, 9)).ok();
        r.4.map(|v| generate(&v, &mut o, 9)).ok();
        r.5.map(|v| generate(&v, &mut o, 5)).ok();
        o += PAGE[1];
        let mut buf = Vec::new();
        let mut enc = flate2::read::GzEncoder::new(o.as_bytes(), Default::default());
        enc.read_to_end(&mut buf).unwrap();
        buf
    })
    .await?;
    *CACHE.lock().unwrap() = (
        [(EXPIRES, expires), (CONTENT_ENCODING, "gzip".into())],
        Html(o),
    );
    Ok(())
}

pub fn service() -> Router {
    tokio::spawn(async {
        care!(refresh().await).ok();
    });
    Router::new().route(
        "/magazine",
        MethodRouter::new().get(|| async { CACHE.lock().unwrap().clone() }),
    )
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(-1, 4, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }

    care!(refresh().await).ok();
}
