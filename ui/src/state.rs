use lumen_core::{MediaState, NotificationState};

use crate::geometry::IslandBounds;

#[derive(Debug, Clone)]
pub struct IslandState {
    pub content: ContentState,

    pub mic: bool,
    pub camera: bool,

    pub expanded: bool
}

impl IslandState {
    pub fn new() -> Self {
        Self {
            content: ContentState::Idle,
            mic: false,
            camera: false,
            expanded: false
        }
    }

    pub fn bounds(&self) -> IslandBounds {
        let bounds = match (&self.content, self.expanded) {
            (ContentState::Idle, _) => IslandBounds {
                y: -48,
                width: 180,
                height: 48,
                radius: 24,
            },

            (ContentState::Media(_), false) => IslandBounds {
                y: 8,
                width: 240,
                height: 48,
                radius: 24,
            },
            (ContentState::Media(_), true) => IslandBounds {
                y: 8,
                width: 400,
                height: 200,
                radius: 24,
            },

            (ContentState::Notification(_), false) => IslandBounds {
                y: 8,
                width: 320,
                height: 80,
                radius: 24,
            },
            (ContentState::Notification(_), true) => IslandBounds {
                y: 8,
                width: 400,
                height: 180,
                radius: 24,
            },
        };

        bounds
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ContentState {
    Idle,
    Media(MediaState),
    Notification(NotificationState),
}