//! The News page: the Discord banner, the news posts in full, and the list of
//! release notes beside them.

mod markdown;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use serde::Deserialize;

use crate::pages::GUTTER;
use crate::shell::{Page, Shell};
use crate::theme::{Palette, ScrollArea};
use markdown::{Block, SITE, Span, safe_url};

pub const FEED_URL: &str = "https://f1.swofty.net/news.json";
/// Whether the page downloads [`FEED_URL`]. Off: that is Swofty Launcher's feed, and F1 Multiplayer
/// shows only the posts built into it.
const ONLINE: bool = false;
const EMBEDDED: &str = include_str!(concat!(env!("OUT_DIR"), "/news.json"));
const CACHE_DIR: &str = "cache/news";
const FEED_FILE: &str = "feed.json";
const TIMEOUT: Duration = Duration::from_secs(10);
const STALE: Duration = Duration::from_secs(30 * 60);
const IMAGES: usize = 8;
const IMAGE_LIMIT: u64 = 8 * 1024 * 1024;
const FEED_LIMIT: u64 = 4 * 1024 * 1024;
const STACK_BELOW: f32 = 680.;
const TEXT: f32 = 15.;
const LINE: f32 = 24.;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Post {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub body: String,
}

impl Post {
    fn release(&self) -> bool {
        self.kind == "release"
    }

    fn version(&self) -> &str {
        self.title.strip_prefix("F1 Multiplayer ").unwrap_or(&self.title)
    }
}

#[derive(Deserialize)]
struct Feed {
    posts: Vec<Post>,
}

pub fn parse(text: &str) -> Option<Vec<Post>> {
    serde_json::from_str::<Feed>(text).ok().map(|f| f.posts)
}

fn version(id: &str) -> Vec<u64> {
    let slug = id.rsplit('/').next().unwrap_or(id);
    slug.split('.').map(|n| n.parse().unwrap_or(0)).collect()
}

/// Every post of `base` and `newer` once, `newer`'s copy where both have it
/// (keeping `base`'s body when `newer`'s has none), newest first.
pub fn merge(mut base: Vec<Post>, newer: Vec<Post>) -> Vec<Post> {
    let mut posts = Vec::with_capacity(base.len() + newer.len());
    for mut post in newer {
        if let Some(at) = base.iter().position(|b| b.id == post.id) {
            let old = base.swap_remove(at);
            if post.body.is_empty() {
                post.body = old.body;
            }
        }
        posts.push(post);
    }
    posts.extend(base);
    posts.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then(a.release().cmp(&b.release()))
            .then_with(|| version(&b.id).cmp(&version(&a.id)))
    });
    posts
}

pub fn embedded() -> Vec<Post> {
    merge(parse(EMBEDDED).unwrap_or_default(), Vec::new())
}

pub fn format_date(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let mut parts = date.splitn(3, '-').map(|n| n.parse::<usize>().ok());
    match (
        parts.next().flatten(),
        parts.next().flatten(),
        parts.next().flatten(),
    ) {
        (Some(y), Some(m), Some(d)) if (1..=12).contains(&m) => format!("{d} {} {y}", MONTHS[m - 1]),
        _ => date.to_owned(),
    }
}

pub(crate) fn image_format(bytes: &[u8]) -> Option<ImageFormat> {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => Some(ImageFormat::Png),
        [0xff, 0xd8, ..] => Some(ImageFormat::Jpeg),
        [b'G', b'I', b'F', ..] => Some(ImageFormat::Gif),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some(ImageFormat::Webp),
        _ => None,
    }
}

pub(crate) fn image(dir: &Path, url: &str) -> Option<Arc<Image>> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let path = dir.join(format!("{:016x}", hasher.finish()));
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => {
            let bytes = swofty_core::update::get(url, Some(TIMEOUT))
                .ok()?
                .with_config()
                .limit(IMAGE_LIMIT)
                .read_to_vec()
                .ok()?;
            std::fs::create_dir_all(dir).ok();
            std::fs::write(&path, &bytes).ok();
            bytes
        }
    };
    Some(Arc::new(Image::from_bytes(image_format(&bytes)?, bytes)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Trouble {
    Offline,
    Failed,
}

fn download() -> Result<String, (Trouble, String)> {
    if !ONLINE {
        return Ok(EMBEDDED.to_owned());
    }
    let mut response = ureq::get(FEED_URL)
        .header(
            "User-Agent",
            concat!("swofty-launcher/", env!("CARGO_PKG_VERSION")),
        )
        .config()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| (Trouble::Offline, format!("{FEED_URL}: {e}")))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err((Trouble::Failed, format!("{FEED_URL}: HTTP {status}")));
    }
    response
        .body_mut()
        .with_config()
        .limit(FEED_LIMIT)
        .read_to_string()
        .map_err(|e| (Trouble::Failed, format!("{FEED_URL}: {e}")))
}

struct Fetched {
    posts: Option<Vec<Post>>,
    images: Vec<(String, Arc<Image>)>,
    error: Option<(Trouble, String)>,
}

fn fetch(root: &Path, shown: Vec<Post>) -> Fetched {
    let dir = root.join(CACHE_DIR);
    let remote = download().and_then(|text| {
        parse(&text)
            .map(|p| (text, p))
            .ok_or_else(|| (Trouble::Failed, format!("{FEED_URL}: not a news feed")))
    });
    let (posts, error) = match remote {
        Ok((text, posts)) => {
            std::fs::create_dir_all(&dir).ok();
            if let Err(e) = std::fs::write(dir.join(FEED_FILE), text) {
                eprintln!("swofty-launcher: could not save the news: {e}");
            }
            (Some(merge(embedded(), posts)), None)
        }
        Err(e) => (None, Some(e)),
    };
    let images = posts
        .as_ref()
        .unwrap_or(&shown)
        .iter()
        .filter(|p| !p.release())
        .take(IMAGES)
        .filter_map(|p| p.image.clone())
        .filter(|url| url.starts_with("https://"))
        .filter_map(|url| image(&dir, &url).map(|i| (url, i)))
        .collect();
    Fetched { posts, images, error }
}

pub struct News {
    pub posts: Vec<Post>,
    images: HashMap<String, Arc<Image>>,
    blocks: Vec<Vec<Block>>,
    revision: u64,
    checked: Option<Instant>,
    live: bool,
    saved: bool,
    fetching: bool,
    trouble: Option<Trouble>,
}

#[derive(Clone, Copy)]
enum NewsRow {
    Header(usize, bool),
    Block(usize, usize),
    Link(usize),
    Separator,
}

#[derive(Clone, Copy)]
enum StackedRow {
    Banner,
    ArticlesTop,
    Article(usize),
    ArticlesBottom,
    ReleasesHeader,
    Release(usize),
    ReleasesBottom,
}

#[derive(Clone, Copy)]
enum NewsPane {
    Articles,
    Releases,
    Stacked,
}

pub(crate) struct NewsScroll {
    articles: ListState,
    releases: ListState,
    stacked: ListState,
    rows: Vec<NewsRow>,
    stacked_rows: Vec<StackedRow>,
    release_indices: Vec<usize>,
    revision: Option<u64>,
    short: bool,
    stacked_layout: bool,
    motion: [crate::scroll::SmoothScroll; 3],
}

impl Default for NewsScroll {
    fn default() -> Self {
        Self {
            articles: ListState::new(0, ListAlignment::Top, px(200.)),
            releases: ListState::new(0, ListAlignment::Top, px(100.)),
            stacked: ListState::new(0, ListAlignment::Top, px(200.)),
            rows: Vec::new(),
            stacked_rows: Vec::new(),
            release_indices: Vec::new(),
            revision: None,
            short: false,
            stacked_layout: false,
            motion: Default::default(),
        }
    }
}

impl NewsScroll {
    pub(crate) fn cancel(&self) {
        for motion in &self.motion {
            motion.cancel();
        }
    }

    fn list(&self, pane: NewsPane) -> &ListState {
        match pane {
            NewsPane::Articles => &self.articles,
            NewsPane::Releases => &self.releases,
            NewsPane::Stacked => &self.stacked,
        }
    }

    fn sync(&mut self, news: &News, short: bool, stacked: bool) {
        if self.stacked_layout != stacked {
            self.stacked_layout = stacked;
            self.cancel();
        }
        if self.short != short {
            self.short = short;
            if self.stacked.item_count() > 0 {
                self.stacked.remeasure_items(0..1);
            }
        }
        if self.revision == Some(news.revision) {
            return;
        }
        self.cancel();
        let first = self.revision.is_none();
        self.revision = Some(news.revision);
        self.rows.clear();
        self.release_indices.clear();
        for (i, post) in news.posts.iter().enumerate() {
            if post.release() {
                self.release_indices.push(i);
                continue;
            }
            let first = self.rows.is_empty();
            if !first {
                self.rows.push(NewsRow::Separator);
            }
            self.rows.push(NewsRow::Header(i, first));
            self.rows
                .extend((0..news.blocks[i].len()).map(|b| NewsRow::Block(i, b)));
            if post.url.as_deref().and_then(safe_url).is_some() {
                self.rows.push(NewsRow::Link(i));
            }
        }
        self.stacked_rows.clear();
        self.stacked_rows
            .extend([StackedRow::Banner, StackedRow::ArticlesTop]);
        self.stacked_rows
            .extend((0..self.rows.len()).map(StackedRow::Article));
        self.stacked_rows
            .extend([StackedRow::ArticlesBottom, StackedRow::ReleasesHeader]);
        self.stacked_rows
            .extend((0..self.release_indices.len()).map(StackedRow::Release));
        self.stacked_rows.push(StackedRow::ReleasesBottom);
        // Preserve the current item and its offset while remeasuring refreshed content.
        for (state, count) in [
            (&self.articles, self.rows.len()),
            (&self.releases, self.release_indices.len()),
            (&self.stacked, self.stacked_rows.len()),
        ] {
            if first {
                state.reset_with_uniform_height(count, px(64.));
                continue;
            }
            let old = state.item_count();
            if count > old {
                state.splice(old..old, count - old);
            } else if count < old {
                state.splice(count..old, 0);
            }
            state.remeasure_items(0..count);
        }
    }
}

impl News {
    pub fn load(root: &Path) -> Self {
        let cached = std::fs::read_to_string(root.join(CACHE_DIR).join(FEED_FILE))
            .ok()
            .and_then(|t| parse(&t))
            .unwrap_or_default();
        let saved = !cached.is_empty();
        let posts = merge(embedded(), cached);
        let blocks = posts.iter().map(parsed_body).collect();
        Self {
            saved,
            posts,
            blocks,
            revision: 0,
            images: HashMap::new(),
            checked: None,
            live: false,
            fetching: false,
            trouble: None,
        }
    }

    fn stale(&self) -> bool {
        !self.fetching && self.checked.is_none_or(|t| t.elapsed() >= STALE)
    }

    fn subtitle(&self) -> String {
        if self.fetching {
            return "Checking for news…".into();
        }
        if !ONLINE {
            return "What's new in F1 Multiplayer".into();
        }
        if self.live {
            return "From f1.swofty.net".into();
        }
        let shown = if self.saved {
            "showing the news saved last time"
        } else {
            "showing the news that came with this version"
        };
        match self.trouble {
            Some(Trouble::Failed) => format!("Couldn't load the latest news · {shown}"),
            _ => format!("Offline · {shown}"),
        }
    }
}

fn news_tag(p: Palette) -> Div {
    div()
        .px(px(7.))
        .h(px(20.))
        .flex()
        .items_center()
        .rounded(px(4.))
        .border_1()
        .border_color(p.accent)
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(p.accent)
        .whitespace_nowrap()
        .child("News")
}

fn rich(id: String, spans: &[Span], p: Palette, mono: &SharedString) -> AnyElement {
    let mut text = String::new();
    let mut highlights = Vec::new();
    let mut fonts = Vec::new();
    let mut ranges = Vec::new();
    let mut urls = Vec::new();
    for span in spans {
        let start = text.len();
        text.push_str(&span.text);
        let range = start..text.len();
        let mut style = HighlightStyle::default();
        if span.strong {
            style.font_weight = Some(FontWeight::BOLD);
            style.color = Some(p.text);
        }
        if span.emphasis {
            style.font_style = Some(FontStyle::Italic);
        }
        if span.code {
            style.color = Some(p.text);
            style.background_color = Some(p.elevated);
            fonts.push((range.clone(), mono.clone()));
        }
        if let Some(url) = &span.link {
            style.color = Some(p.accent_hover);
            style.underline = Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(p.accent_hover.opacity(0.45)),
                wavy: false,
            });
            ranges.push(range.clone());
            urls.push(url.clone());
        }
        if style != HighlightStyle::default() {
            highlights.push((range, style));
        }
    }
    let styled = StyledText::new(text)
        .with_highlights(highlights)
        .with_font_family_overrides(fonts);
    if ranges.is_empty() {
        return styled.into_any_element();
    }
    InteractiveText::new(SharedString::from(id), styled)
        .on_click(ranges, move |ix, _, cx| {
            if let Some(url) = urls.get(ix) {
                cx.open_url(url);
            }
        })
        .into_any_element()
}

fn parsed_body(post: &Post) -> Vec<Block> {
    if post.body.trim().is_empty() {
        vec![Block::Paragraph(markdown::inline(&post.summary))]
    } else {
        markdown::parse(&post.body)
    }
}

pub(crate) fn render_markdown(id: &str, text: &str, p: Palette, mono: &SharedString) -> Vec<AnyElement> {
    blocks_view(id, &markdown::parse(text), p, mono)
}

fn blocks_view(prefix: &str, blocks: &[Block], p: Palette, mono: &SharedString) -> Vec<AnyElement> {
    blocks
        .iter()
        .enumerate()
        .map(|(i, block)| block_view(prefix, i, block, i.checked_sub(1).map(|j| &blocks[j]), p, mono))
        .collect()
}

fn block_view(
    prefix: &str,
    i: usize,
    block: &Block,
    previous: Option<&Block>,
    p: Palette,
    mono: &SharedString,
) -> AnyElement {
    let text = p.text.opacity(0.84);
    let id = format!("{prefix}-{i}");
    let top = match (previous, block) {
        (None, _) => 0.,
        (_, Block::Heading(..)) => 26.,
        (Some(Block::Heading(..)), _) => 8.,
        (Some(Block::Item { .. }), Block::Item { .. }) => 8.,
        (_, Block::Rule) | (Some(Block::Rule), _) => 22.,
        _ => 12.,
    };
    let el = match block {
        Block::Heading(level, spans) => div()
            .text_size(px(match level {
                1 => 21.,
                2 => 18.,
                _ => 15.5,
            }))
            .line_height(px(if *level <= 2 { 26. } else { 22. }))
            .font_weight(FontWeight::BOLD)
            .text_color(p.text)
            .child(rich(id, spans, p, mono))
            .into_any_element(),
        Block::Paragraph(spans) => div()
            .text_color(text)
            .child(rich(id, spans, p, mono))
            .into_any_element(),
        Block::Item { depth, number, spans } => h_flex()
            .items_start()
            .gap(px(12.))
            .pl(px(2. + 22. * *depth as f32))
            .child(match number {
                Some(n) => div()
                    .min_w(px(18.))
                    .flex_shrink_0()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.accent_hover)
                    .child(format!("{n}.")),
                None => div()
                    .mt(px((LINE - 6.) / 2.))
                    .size(px(6.))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(if *depth == 0 { p.accent } else { p.text_placeholder }),
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(text)
                    .child(rich(id, spans, p, mono)),
            )
            .into_any_element(),
        Block::Quote(spans) => div()
            .pl(px(14.))
            .border_l_2()
            .border_color(p.accent)
            .text_color(p.text_muted)
            .child(rich(id, spans, p, mono))
            .into_any_element(),
        Block::Code(code) => div()
            .px(px(14.))
            .py(px(10.))
            .rounded(px(6.))
            .bg(p.chrome)
            .border_1()
            .border_color(p.border_variant)
            .font_family(mono.clone())
            .text_size(px(13.))
            .line_height(px(20.))
            .text_color(p.text)
            .child(code.clone())
            .into_any_element(),
        Block::Rule => div().h(px(1.)).bg(p.border_variant).into_any_element(),
    };
    div().pt(px(top)).child(el).into_any_element()
}

fn changelog_header(count: usize, p: Palette) -> Div {
    h_flex()
        .flex_shrink_0()
        .px(px(16.))
        .h(px(46.))
        .gap(px(8.))
        .items_center()
        .border_b_1()
        .border_color(p.border)
        .child(
            div()
                .text_size(px(14.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Changelogs"),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(12.))
                .text_color(p.text_placeholder)
                .child(format!("{count} releases")),
        )
}

fn changelog_row(post: &Post, first: bool, p: Palette) -> AnyElement {
    let url = post.url.as_deref().and_then(safe_url);
    let current = post.version() == env!("CARGO_PKG_VERSION");
    v_flex()
        .id(SharedString::from(format!("news-{}", post.id)))
        .flex_shrink_0()
        .px(px(16.))
        .py(px(11.))
        .gap(px(4.))
        .when(!first, |d| d.border_t_1().border_color(p.border_variant))
        .when_some(url, |d, url| {
            d.cursor_pointer()
                .hover(move |s| s.bg(p.hover))
                .on_click(move |_, _, cx| cx.open_url(&url))
        })
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(
                    div()
                        .flex_shrink_0()
                        .whitespace_nowrap()
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(post.version().to_owned()),
                )
                .when(current, |d| {
                    d.child(
                        div()
                            .flex_shrink_0()
                            .px(px(6.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .rounded(px(4.))
                            .bg(p.accent.opacity(0.18))
                            .text_size(px(11.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.accent_hover)
                            .child("Installed"),
                    )
                })
                .child(div().flex_1())
                .child(
                    div()
                        .flex_shrink_0()
                        .whitespace_nowrap()
                        .text_size(px(12.))
                        .text_color(p.text_placeholder)
                        .child(format_date(&post.date)),
                ),
        )
        .when(!post.summary.is_empty(), |d| {
            d.child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(p.text_muted)
                    .line_clamp(2)
                    .text_ellipsis()
                    .child(post.summary.clone()),
            )
        })
        .into_any_element()
}

fn empty(text: &'static str, p: Palette) -> Div {
    div()
        .py(px(4.))
        .text_size(px(13.))
        .text_color(p.text_placeholder)
        .child(text)
}

impl Shell {
    pub(crate) fn fetch_news(&mut self, cx: &mut Context<Self>) {
        if self.news.fetching {
            return;
        }
        self.news.fetching = true;
        let root = self.store.root.clone();
        let shown = self.news.posts.clone();
        cx.spawn(async move |this, cx| {
            let fetched = cx
                .background_executor()
                .spawn(async move { fetch(&root, shown) })
                .await;
            this.update(cx, |s, cx| {
                let news = &mut s.news;
                news.fetching = false;
                news.checked = Some(Instant::now());
                news.trouble = fetched.error.as_ref().map(|(t, _)| *t);
                if let Some((_, e)) = &fetched.error {
                    eprintln!("swofty-launcher: could not check for news: {e}");
                }
                let mut changed = !fetched.images.is_empty();
                if let Some(posts) = fetched.posts {
                    if news.posts != posts {
                        changed = true;
                        news.blocks = posts.iter().map(parsed_body).collect();
                        news.posts = posts;
                    }
                    news.live = true;
                }
                news.images.extend(fetched.images);
                if changed {
                    news.revision = news.revision.wrapping_add(1);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn news_page(&mut self, p: Palette, short: bool, cx: &mut Context<Self>) -> AnyElement {
        if self.news.stale() {
            self.fetch_news(cx);
        }
        let refresh = Button::new("news-refresh")
            .ghost()
            .small()
            .icon(IconName::RefreshCw)
            .label("Refresh")
            .disabled(self.news.fetching)
            .on_click(cx.listener(|this, _, _, cx| this.fetch_news(cx)));
        let stacked = self.content_width < STACK_BELOW;
        let side = (self.content_width * 0.32).clamp(260., 380.);
        self.news_scroll.sync(&self.news, short, stacked);
        if stacked {
            return v_flex()
                .size_full()
                .child(self.header(Page::News.label(), self.news.subtitle(), refresh, p))
                .child(self.stacked_news(short, cx))
                .into_any_element();
        }
        let posts = self.news_posts(p, cx);
        let changelogs = self.changelogs(p, cx);
        let columns = h_flex()
            .flex_1()
            .min_h_0()
            .items_stretch()
            .gap(px(16.))
            .child(posts)
            .child(v_flex().w(px(side)).flex_shrink_0().min_h_0().child(changelogs));
        let content = v_flex()
            .flex_1()
            .min_h_0()
            .px(px(GUTTER))
            .pt(px(4.))
            .pb(px(20.))
            .gap(px(16.))
            .child(crate::hero::banner(short))
            .child(columns);
        v_flex()
            .size_full()
            .child(self.header(Page::News.label(), self.news.subtitle(), refresh, p))
            .map(|d| {
                if stacked {
                    d.child(content.scroll_y("news"))
                } else {
                    d.child(content)
                }
            })
            .into_any_element()
    }

    fn news_scrollbar(&self, pane: NewsPane, cx: &Context<Self>) -> Div {
        self.news_scroll.motion[pane as usize].overlay(self.news_scroll.list(pane), cx)
    }

    fn news_posts(&self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let frame = v_flex()
            .relative()
            .rounded(px(10.))
            .border_1()
            .border_color(p.border)
            .bg(p.surface)
            .overflow_hidden();
        if self.news_scroll.rows.is_empty() {
            return frame
                .flex_1()
                .child(empty("No news yet.", p).p(px(32.)))
                .into_any_element();
        }
        let scroll = &self.news_scroll.articles;
        frame
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                list(
                    scroll.clone(),
                    cx.processor(|this, i, _, cx| {
                        let p = crate::theme::palette(cx);
                        let mono = cx.theme().mono_font_family.clone();
                        div()
                            .w_full()
                            .px(px(32.))
                            // Include edge spacing in row measurements so virtualization
                            // keeps partially visible rows until they leave the viewport.
                            .when(i == 0, |d| d.pt(px(28.)))
                            .when(i + 1 == this.news_scroll.rows.len(), |d| d.pb(px(36.)))
                            .child(this.news_row(i, p, &mono))
                            .into_any_element()
                    }),
                )
                .flex_1()
                .min_h_0(),
            )
            .child(self.news_scrollbar(NewsPane::Articles, cx))
            .into_any_element()
    }

    fn news_row(&self, i: usize, p: Palette, mono: &SharedString) -> AnyElement {
        let Some(row) = self.news_scroll.rows.get(i) else {
            return div().into_any_element();
        };
        let content = match *row {
            NewsRow::Header(pi, first) => self.article_header(&self.news.posts[pi], first, p),
            NewsRow::Block(pi, bi) => {
                let blocks = &self.news.blocks[pi];
                div()
                    .text_size(px(TEXT))
                    .line_height(px(LINE))
                    .when(bi == 0, |d| d.pt(px(14.)))
                    .child(block_view(
                        &format!("news-{}", self.news.posts[pi].id),
                        bi,
                        &blocks[bi],
                        bi.checked_sub(1).map(|j| &blocks[j]),
                        p,
                        mono,
                    ))
                    .into_any_element()
            }
            NewsRow::Link(pi) => self
                .article_link(&self.news.posts[pi])
                .unwrap_or_else(|| div().into_any_element()),
            NewsRow::Separator => {
                return div()
                    .py(px(34.))
                    .child(div().h(px(1.)).bg(p.border))
                    .into_any_element();
            }
        };
        div().w_full().max_w(px(780.)).child(content).into_any_element()
    }

    fn article_header(&self, post: &Post, first: bool, p: Palette) -> AnyElement {
        let image = post.image.as_ref().and_then(|u| self.news.images.get(u)).cloned();
        v_flex()
            .flex_shrink_0()
            .w_full()
            .max_w(px(780.))
            .children(image.map(|i| {
                div()
                    .w_full()
                    .h(px(220.))
                    .mb(px(22.))
                    .rounded(px(8.))
                    .overflow_hidden()
                    .child(img(i).size_full().object_fit(ObjectFit::Cover))
            }))
            .child(
                h_flex().gap(px(10.)).items_center().child(news_tag(p)).child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_placeholder)
                        .child(format_date(&post.date)),
                ),
            )
            .child(
                div()
                    .pt(px(10.))
                    .text_size(px(if first { 26. } else { 22. }))
                    .line_height(px(if first { 32. } else { 28. }))
                    .font_weight(FontWeight::BOLD)
                    .child(post.title.clone()),
            )
            .into_any_element()
    }

    fn article_link(&self, post: &Post) -> Option<AnyElement> {
        let url = post.url.as_deref().and_then(safe_url)?;
        let on_site = url.starts_with(SITE);
        Some(
            h_flex()
                .pt(px(24.))
                .child(
                    Button::new(SharedString::from(format!("news-open-{}", post.id)))
                        .outline()
                        .small()
                        .icon(IconName::ExternalLink)
                        .label(if on_site {
                            "Read on f1.swofty.net"
                        } else {
                            "Open in browser"
                        })
                        .on_click(move |_, _, cx| cx.open_url(&url)),
                )
                .into_any_element(),
        )
    }

    fn changelogs(&self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let scroll = &self.news_scroll.releases;
        let count = self.news_scroll.release_indices.len();
        v_flex()
            .rounded(px(10.))
            .border_1()
            .border_color(p.border)
            .bg(p.surface)
            .overflow_hidden()
            .flex_1()
            .min_h_0()
            .child(changelog_header(count, p))
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .when(count == 0, |d| {
                        d.child(empty("No release notes yet.", p).px(px(16.)).py(px(12.)))
                    })
                    .child(
                        list(
                            scroll.clone(),
                            cx.processor(|this, i, _, cx| {
                                let p = crate::theme::palette(cx);
                                let row = this
                                    .news_scroll
                                    .release_indices
                                    .get(i)
                                    .map(|&pi| changelog_row(&this.news.posts[pi], i == 0, p));
                                div().w_full().children(row).into_any_element()
                            }),
                        )
                        .flex_1()
                        .min_h_0(),
                    )
                    .child(self.news_scrollbar(NewsPane::Releases, cx)),
            )
            .into_any_element()
    }

    fn stacked_news(&self, short: bool, cx: &mut Context<Self>) -> AnyElement {
        let scroll = &self.news_scroll.stacked;
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(
                list(
                    scroll.clone(),
                    cx.processor(move |this, i, _, cx| {
                        let p = crate::theme::palette(cx);
                        let mono = cx.theme().mono_font_family.clone();
                        let news = this.news_scroll.rows.len();
                        let releases = this.news_scroll.release_indices.len();
                        let panel = || div().bg(p.surface).border_x_1().border_color(p.border);
                        let row = match this.news_scroll.stacked_rows[i] {
                            StackedRow::Banner => div()
                                .pb(px(16.))
                                .child(crate::hero::banner(short))
                                .into_any_element(),
                            StackedRow::ArticlesTop => panel()
                                .rounded_t(px(10.))
                                .border_t_1()
                                .pt(px(28.))
                                .when(news == 0, |d| d.child(empty("No news yet.", p).px(px(32.))))
                                .into_any_element(),
                            StackedRow::Article(row) => panel()
                                .px(px(32.))
                                .child(this.news_row(row, p, &mono))
                                .into_any_element(),
                            StackedRow::ArticlesBottom => panel()
                                .rounded_b(px(10.))
                                .border_b_1()
                                .h(px(36.))
                                .mb(px(16.))
                                .into_any_element(),
                            StackedRow::ReleasesHeader => panel()
                                .rounded_t(px(10.))
                                .border_t_1()
                                .child(changelog_header(releases, p))
                                .when(releases == 0, |d| {
                                    d.child(empty("No release notes yet.", p).px(px(16.)).py(px(12.)))
                                })
                                .into_any_element(),
                            StackedRow::Release(ri) => {
                                let pi = this.news_scroll.release_indices[ri];
                                panel()
                                    .child(changelog_row(&this.news.posts[pi], ri == 0, p))
                                    .into_any_element()
                            }
                            StackedRow::ReleasesBottom => panel()
                                .rounded_b(px(10.))
                                .border_b_1()
                                .h(px(1.))
                                .into_any_element(),
                        };
                        div()
                            .w_full()
                            .px(px(GUTTER))
                            .when(i == 0, |d| d.pt(px(4.)))
                            .when(i + 1 == this.news_scroll.stacked_rows.len(), |d| d.pb(px(20.)))
                            .child(row)
                            .into_any_element()
                    }),
                )
                .flex_1()
                .min_h_0(),
            )
            .child(self.news_scrollbar(NewsPane::Stacked, cx))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{News, Post, Trouble, embedded, format_date, image_format, merge, parse};

    fn post(id: &str, date: &str) -> Post {
        Post {
            id: id.into(),
            kind: id.split('/').next().unwrap_or_default().into(),
            title: id.into(),
            date: date.into(),
            summary: String::new(),
            image: None,
            url: None,
            body: String::new(),
        }
    }

    #[test]
    fn embeds_the_posts_in_full_and_the_release_notes_newest_first() {
        let posts = embedded();
        let news = posts.iter().find(|p| p.id == "news/welcome").unwrap();
        assert_eq!(news.kind, "news");
        assert!(news.body.starts_with("F1 Multiplayer is built for two friends"));
        assert!(news.body.contains("## What works today"));
        assert!(posts.windows(2).all(|w| w[0].date >= w[1].date));
        assert!(
            posts.iter().all(|p| !p.title.contains("Swofty")),
            "only F1 Multiplayer's own posts"
        );
        let notes = posts.iter().find(|p| p.id == "release/0.1.0").unwrap();
        assert_eq!(notes.title, "F1 Multiplayer 0.1.0");
        assert_eq!(notes.version(), "0.1.0");
        assert_eq!(notes.url, None);
        assert!(!notes.summary.is_empty() && !notes.date.is_empty() && notes.body.is_empty());
    }

    #[test]
    fn merges_by_id_and_sorts_by_date_then_version() {
        let mut base = vec![
            post("release/1.2.9", "2026-10-21"),
            post("release/1.2.10", "2026-10-21"),
            post("news/hello", "2026-10-21"),
        ];
        base[2].body = "kept".into();
        let mut newer = vec![
            post("release/1.2.9", "2026-10-21"),
            post("news/hello", "2026-10-21"),
        ];
        newer[0].summary = "fresh".into();
        let merged = merge(base, newer);
        let ids: Vec<&str> = merged.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["news/hello", "release/1.2.10", "release/1.2.9"]);
        assert_eq!(merged[2].summary, "fresh");
        assert_eq!(merged[0].body, "kept");
    }

    #[test]
    fn reads_new_and_old_feeds() {
        let posts = parse(r#"{"posts":[{"id":"news/a","title":"A","extra":1}],"version":2}"#).unwrap();
        assert_eq!(posts[0].title, "A");
        assert!(posts[0].body.is_empty());
        let posts =
            parse(r###"{"posts":[{"id":"news/b","kind":"news","title":"B","body":"## Hi\n\n- one"}]}"###)
                .unwrap();
        assert_eq!(posts[0].body, "## Hi\n\n- one");
        assert!(parse("<html>").is_none());
    }

    #[test]
    fn shows_its_own_posts_without_going_online() {
        let news = News::load(std::path::Path::new("/nonexistent"));
        assert_eq!(news.subtitle(), "What's new in F1 Multiplayer");
    }

    #[test]
    #[ignore = "the feed download is off (ONLINE)"]
    fn says_why_the_latest_news_is_missing() {
        let mut news = News::load(std::path::Path::new("/nonexistent"));
        assert_eq!(
            news.subtitle(),
            "Offline · showing the news that came with this version"
        );
        news.trouble = Some(Trouble::Failed);
        assert_eq!(
            news.subtitle(),
            "Couldn't load the latest news · showing the news that came with this version"
        );
        news.saved = true;
        news.trouble = Some(Trouble::Offline);
        assert_eq!(news.subtitle(), "Offline · showing the news saved last time");
        news.live = true;
        assert_eq!(news.subtitle(), "From f1.swofty.net");
    }

    #[test]
    fn formats_dates() {
        assert_eq!(format_date("2026-10-22"), "22 October 2026");
        assert_eq!(format_date("soon"), "soon");
    }

    #[test]
    fn knows_image_formats() {
        assert!(image_format(b"\x89PNG\r\n").is_some());
        assert!(image_format(b"<svg").is_none());
    }
}
