use anyhow::{anyhow, Context as _, Result};
use async_recursion::async_recursion;
use camino::Utf8PathBuf;
use docker_api::Docker;
use futures::{
    future::{self, FutureExt},
    stream::{self, StreamExt, TryStreamExt},
};
use mdbook::{
    book::{Book, Chapter},
    errors::Result as MdbookResult,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use tokio::{runtime::Handle, sync::Semaphore};
use toml::Value;
use tracing::{debug, info, instrument};
use url::Url;

mod docker;
#[cfg(test)]
mod tests;

pub use docker::Instance;

const COMMAND_NAME: &'static str = "docker-run";
const LABEL: &'static str = "docker-run";
const DOCKER_HOST_DEFAULT: &'static str = "unix:///run/docker.sock";

/// Configuration for the plugin
#[derive(Deserialize, Default)]
pub struct Config {
    /// URL of Docker socket
    #[serde(default)]
    pub docker: Option<Url>,

    /// Default image to use
    #[serde(default)]
    pub image: Option<String>,

    /// Prefix for all paths
    #[serde(default)]
    pub prefix: Option<Utf8PathBuf>,

    /// How many commands to run in parallel
    #[serde(default = "num_cpus::get")]
    pub parallel: usize,
}

#[derive(Clone, Debug)]
pub struct DockerRunPreprocessor {
    handle: Handle,
}

#[derive(Debug)]
pub struct Context {
    /// Semaphore to limit concurrent tasks
    tasks: Semaphore,
    /// How many tasks to run in parallel
    parallel: usize,
    /// Handle to Docker
    docker: Docker,
    /// Prefix to use for paths
    prefix: Utf8PathBuf,
}

impl Context {
    /// Create new [`Context`] from a [`Config`].
    async fn new(config: &Config) -> Result<Self> {
        let docker = match &config.docker {
            None => Docker::new(
                &std::env::var("DOCKER_HOST")
                    .as_deref()
                    .unwrap_or(DOCKER_HOST_DEFAULT),
            ),
            Some(url) => Docker::new(url.as_str()),
        };
        let docker = docker.context("Connecting to Docker")?;
        docker.ping().await.context("Pinging Docker daemon")?;
        info!("Connected to Docker");
        Ok(Context {
            tasks: Semaphore::new(config.parallel),
            docker,
            prefix: config.prefix.clone().unwrap_or_else(|| ".".into()),
            parallel: config.parallel,
        })
    }

    pub fn label(&self) -> &str {
        LABEL
    }

    #[instrument(skip(self, book))]
    async fn map_book(&self, mut book: Book) -> Result<Book> {
        info!("Processing book");
        let sections = std::mem::take(&mut book.sections);
        let sections = stream::iter(sections.into_iter())
            .map(|item| self.map_book_item(item))
            .buffered(self.parallel)
            .try_collect()
            .await?;

        book.sections = sections;
        Ok(book)
    }

    #[instrument(skip(self, item))]
    async fn map_book_item(&self, item: BookItem) -> Result<BookItem> {
        info!("Processing book item");
        use BookItem::*;
        let item = match item {
            Chapter(chapter) => Chapter(self.map_chapter(chapter).await?),
            Separator => Separator,
            PartTitle(title) => PartTitle(title),
        };
        Ok(item)
    }

    #[instrument(skip(self, chapter), fields(name = chapter.name, path = ?chapter.path))]
    #[async_recursion(?Send)]
    async fn map_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
        info!("Processing chapter");

        chapter.content = self
            .map_markdown(std::mem::take(&mut chapter.content))
            .await?;

        // map sub items
        let sub_items = std::mem::take(&mut chapter.sub_items);
        let sub_items = stream::iter(sub_items.into_iter())
            .map(|item| self.map_book_item(item))
            .buffered(self.parallel)
            .try_collect()
            .await?;
        chapter.sub_items = sub_items;

        Ok(chapter)
    }

    #[instrument(skip(self, markdown))]
    async fn map_markdown(&self, markdown: String) -> Result<String> {
        info!("Processing markdown {markdown}");
        let parser = Parser::new_ext(&markdown, Options::all());

        // check if this event is a code start event with our label
        let is_code_start = |event: &Event<'_>| matches!(event, Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(label))) if &**label == self.label());

        // process code blocks in parallel
        let events: Vec<Event<'_>> = stream::unfold(parser.peekable(), |mut iter| async move {
            iter.next().map(|event: Event<'_>| {
                if is_code_start(&event) {
                    let code = iter.next();
                    let end = iter.next();
                    let future = async move {
                        if !matches!(end, Some(Event::End(_))) {
                            return Err(anyhow!("Missing end event, got {end:?}"));
                        }
                        match code {
                            Some(Event::Text(code)) => self.map_code(&code).await,
                            other => Err(anyhow!("Missing code block, got {other:?}")),
                        }
                    }
                    .boxed();
                    (future, iter)
                } else {
                    let mut events = vec![event];
                    while iter
                        .peek()
                        .map(|event| !is_code_start(event))
                        .unwrap_or(false)
                    {
                        events.push(iter.next().unwrap());
                    }
                    (future::ready(Ok(events)).boxed(), iter)
                }
            })
        })
        .buffered(self.parallel)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|events| events.into_iter())
        .collect();

        // turn result back into markdown
        let mut buf = String::with_capacity(markdown.len());
        let markdown = cmark(events.iter(), &mut buf).map(|_| buf)?;
        info!("output {markdown}");
        Ok(markdown)
    }

    #[instrument(skip(self, code))]
    async fn map_code(&self, code: &str) -> Result<Vec<Event<'static>>> {
        info!("Mapping code");
        let instance = serde_yaml::from_str(code)?;
        let output = self.run(&instance).await?;
        info!("Output {output}");
        let output = format!("<pre><code>{output}</code></pre>");
        let events = vec![Event::Start(Tag::Paragraph), Event::Html(output.into())];
        Ok(events)
    }
}

impl DockerRunPreprocessor {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    pub fn new_current() -> Self {
        Self::new(Handle::current())
    }
}

impl Preprocessor for DockerRunPreprocessor {
    fn name(&self) -> &str {
        COMMAND_NAME
    }

    #[instrument(name = "mdbook_docker_run", skip(self, ctx, book))]
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> MdbookResult<Book> {
        let config = ctx.config.get_preprocessor(self.name()).unwrap();
        let config: Config = Value::Table(config.clone()).try_into()?;
        let book = self
            .handle
            .block_on(async move { Context::new(&config).await?.map_book(book).await })?;
        Ok(book)
    }
}
