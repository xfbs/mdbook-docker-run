use anyhow::Result;
use async_recursion::async_recursion;
use bollard::{Docker, API_DEFAULT_VERSION};
use camino::Utf8PathBuf;
use futures::stream::{self, StreamExt, TryStreamExt};
use mdbook::{
    book::{Book, Chapter},
    errors::Result as MdbookResult,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use serde::Deserialize;
use tokio::{runtime::Handle, sync::Semaphore};
use toml::Value;
use url::Url;

/// Configuration for the plugin
#[derive(Deserialize)]
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
            None => Docker::connect_with_local_defaults()?,
            Some(url) => Docker::connect_with_http(url.as_str(), 60, API_DEFAULT_VERSION)?,
        };
        docker.ping().await?;
        Ok(Context {
            tasks: Semaphore::new(config.parallel),
            docker,
            prefix: config.prefix.clone().unwrap_or_else(|| ".".into()),
            parallel: config.parallel,
        })
    }

    #[tracing::instrument(skip(self, book))]
    async fn map_book(&self, mut book: Book) -> Result<Book> {
        let sections = std::mem::take(&mut book.sections);
        let sections = stream::iter(sections.into_iter())
            .map(|item| self.map_book_item(item))
            .buffered(self.parallel)
            .try_collect()
            .await?;

        book.sections = sections;
        Ok(book)
    }

    #[tracing::instrument(skip(self, item))]
    async fn map_book_item(&self, item: BookItem) -> Result<BookItem> {
        use BookItem::*;
        let item = match item {
            Chapter(chapter) => Chapter(self.map_chapter(chapter).await?),
            Separator => Separator,
            PartTitle(title) => PartTitle(title),
        };
        Ok(item)
    }

    #[tracing::instrument(skip(self, content))]
    #[async_recursion]
    async fn map_markdown(&self, content: String) -> Result<String> {
        let lease = self.tasks.acquire().await?;
        Ok(content)
    }

    #[tracing::instrument(skip(self, chapter))]
    #[async_recursion]
    async fn map_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
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
        "files"
    }

    #[tracing::instrument(name = "mdbook_docker_run", skip(self, ctx, book))]
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> MdbookResult<Book> {
        let config = ctx.config.get_preprocessor(self.name()).unwrap();
        let config: Config = Value::Table(config.clone()).try_into()?;
        let book = self
            .handle
            .block_on(async move { Context::new(&config).await?.map_book(book).await })?;
        Ok(book)
    }
}
