use std::sync::Arc;

use anyhow::Result;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
};

use crate::{
    bus::{EventReceiver, EventSender, create_bus},
    runtime::RuntimeState,
    services::{
        Service, audio::AudioSpectrumService, camera::CameraService, media::MediaService,
        microphone::MicrophoneService, notifications::NotificationService,
    },
    utils::{artwork_dir, cache_dir, icons_dir},
};

pub struct IslandCore {
    tx: EventSender,
    rx: EventReceiver,
    runtime: Arc<RuntimeState>,
    executor: tokio::runtime::Runtime,
}

impl IslandCore {
    pub fn new() -> Self {
        let (tx, rx) = create_bus();

        let _ = std::fs::create_dir_all(cache_dir());
        let _ = std::fs::create_dir_all(artwork_dir());
        let _ = std::fs::create_dir_all(icons_dir());

        Self {
            tx,
            rx,
            runtime: Arc::new(RuntimeState::new()),
            executor: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.rx.clone()
    }

    pub fn runtime(&self) -> Arc<RuntimeState> {
        self.runtime.clone()
    }

    pub fn sender(&self) -> EventSender {
        self.tx.clone()
    }

    pub fn start(&self) {
        let runtime = self.runtime.clone();
        let tx = self.tx.clone();

        let handle = &self.executor.handle();

        run_service::<MediaService>(handle, tx.clone(), runtime.clone());
        run_service::<NotificationService>(handle, tx.clone(), runtime.clone());
        run_service::<CameraService>(handle, tx.clone(), runtime.clone());
        run_service::<MicrophoneService>(handle, tx.clone(), runtime.clone());
        run_service::<AudioSpectrumService>(handle, tx.clone(), runtime.clone());
    }

    pub fn dismiss_notification(&self, id: u64) {
        self.runtime.notifications.lock().unwrap().retain(|n| n.id != id);

        let _ = self.tx.send(crate::CoreEvent::Arbitrary);
    }

    async fn current_session(&self) -> Result<GlobalSystemMediaTransportControlsSession> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;

        Ok(manager.GetCurrentSession()?)
    }

    pub async fn toggle_playback(&self) -> Result<()> {
        let session = self.current_session().await?;
        session.TryTogglePlayPauseAsync()?.await?;

        Ok(())
    }

    pub async fn next(&self) -> Result<()> {
        let session = self.current_session().await?;
        session.TrySkipNextAsync()?.await?;

        Ok(())
    }

    pub async fn previous(&self) -> Result<()> {
        let session = self.current_session().await?;
        session.TrySkipPreviousAsync()?.await?;

        Ok(())
    }

    pub async fn seek(&self, position_ms: u64) -> Result<()> {
        let session = self.current_session().await?;
        session.TryChangePlaybackPositionAsync((position_ms * 10_000) as i64)?.await?;

        Ok(())
    }
}

fn run_service<S: Service>(
    handle: &tokio::runtime::Handle,
    tx: EventSender,
    runtime: Arc<RuntimeState>,
) {
    handle.spawn(async move {
        S::new().run(tx, runtime).await;
    });
}
