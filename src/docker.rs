use super::*;
use anyhow::{Result, Context as _};
use bollard::{
    container::{AttachContainerOptions, Config, CreateContainerOptions, StartContainerOptions},
    errors::Error,
    image::CreateImageOptions,
};
use futures::StreamExt;
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
        let image_info = self.docker.inspect_image(&image).await;
        let image_missing = matches!(image_info, Err(Error::DockerResponseServerError { status_code, ..}) if status_code == 404);
        if image_missing {
            info!("Pulling image {image}");
            let options = Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            });

            let mut image_pull = self.docker.create_image(options, None, None);
            while let Some(event) = image_pull.next().await {
                let event = event?;
                debug!("{event:?}");
            }
        }

        Ok(())
    }

    #[instrument(skip(self, instance))]
    pub async fn run(&self, instance: &Instance) -> Result<String> {
        // get semaphore lease to avoid running too many things at once
        let weight = instance.weight.unwrap_or(1).min(self.parallel as u32);
        let _lease = self.tasks.acquire_many(weight).await?;

        let image = instance.image.as_ref().ok_or(anyhow!("Missing image"))?;
        self.fetch_image(&image).await.context("Fetching docker image")?;

        // check if image exists, and if not pull it

        let container_options = CreateContainerOptions {
            name: "",
            platform: None,
        };

        let command = instance.script.join(" && ");
        let config = Config {
            image: Some(image.as_str()),
            cmd: Some(vec!["sh", "-c", command.as_str()]),
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(Some(container_options.clone()), config)
            .await?;

        let options = Some(AttachContainerOptions::<String> {
            stdin: Some(true),
            stdout: Some(true),
            stderr: Some(true),
            stream: Some(true),
            logs: Some(true),
            detach_keys: None,
        });

        let mut result = self.docker.attach_container(&container.id, options).await?;

        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        let mut output = Vec::new();
        while let Some(chunk) = result.output.next().await {
            debug!("{chunk:?}");
            output.extend(chunk?.into_bytes().into_iter());
        }

        let output = String::from_utf8_lossy(&output);
        let output = ansi_to_html::convert(&output)?;
        Ok(output)
    }
}
