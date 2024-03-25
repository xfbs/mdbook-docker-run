use crate::Context;
use anyhow::{anyhow, Context as _, Result};
use docker_api::{
    errors::Error,
    opts::{ContainerCreateOpts, PullOpts},
};
use futures::StreamExt;
use serde::Deserialize;
use tracing::{debug, info, instrument};

/// Instance of a run declaration
// TODO: network
// TODO: platform
// TODO: mode (read-only, copy-on-write, write): https://stackoverflow.com/questions/29550736/can-i-mount-docker-host-directory-as-copy-on-write-overlay
#[derive(Deserialize, Default)]
pub struct Instance {
    /// Image to use
    #[serde(default)]
    pub image: Option<String>,

    /// Setup commands to run (output will not be captured).
    #[serde(default)]
    pub setup: Vec<String>,

    /// Script to run (output will be captured).
    pub script: Vec<String>,

    /// Weight of this task. When set to zero, will disable concurrency control.
    #[serde(default)]
    pub weight: Option<u32>,
}

impl Context {
    #[instrument(skip(self))]
    async fn fetch_image(&self, image: &str) -> Result<()> {
        let images = self.docker.images();

        let info = images.get(image).inspect().await;
        println!("{info:?}");
        match info {
            Ok(_) => {
                debug!("Image is present, skipping pulling");
                return Ok(());
            }
            Err(Error::Fault { code, ref message })
                if code == 404 && message.starts_with("No such image") => {}
            Err(other) => return Err(other.into()),
        }

        info!("Pulling image {image}");
        let mut stream = images.pull(&PullOpts::builder().image(image).build());

        while let Some(event) = stream.next().await {
            let event = event?;
            debug!("{event:?}");
        }

        Ok(())
    }

    #[instrument(skip(self, instance))]
    pub async fn run(&self, instance: &Instance) -> Result<String> {
        // get semaphore lease to avoid running too many things at once
        let weight = instance.weight.unwrap_or(1).min(self.parallel as u32);
        let _lease = self.tasks.acquire_many(weight).await?;

        // check if image exists, and if not pull it
        let image = instance.image.as_ref().ok_or(anyhow!("Missing image"))?;
        self.fetch_image(&image)
            .await
            .context("Fetching docker image")?;

        let containers = self.docker.containers();
        let command = instance.script.join(" && ");
        let container = containers
            .create(
                &ContainerCreateOpts::builder()
                    .attach_stdout(true)
                    .attach_stderr(true)
                    .image(image.as_str())
                    .command(vec!["sh", "-c", command.as_str()])
                    .build(),
            )
            .await?;

        let mut stream = container.attach().await?;
        container.start().await?;

        let mut output = Vec::new();
        while let Some(chunk) = stream.next().await {
            debug!("{chunk:?}");
            output.extend(chunk?.as_slice().into_iter());
        }

        let output = String::from_utf8_lossy(&output);
        let output = ansi_to_html::convert(&output)?;
        Ok(output)
    }
}
