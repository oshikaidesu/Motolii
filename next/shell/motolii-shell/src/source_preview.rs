//! Browser の `PreviewMedia` を受ける Source Preview の所有者。
//!
//! Browser は素材カードの選択と handoff だけを持ち、Stage は Composition の
//! 合成結果だけを持つ。この module はその間にある素材単体の一時状態、実フレーム
//! の読み出し、イン/アウトと再生操作をまとめる。Document へは何も書かない。

/* motolii-component
id = "shell.source_preview"
kind = "semantic"
weight = "render_export"
maps = []
entry = ["SourcePreview", "open_source_preview"]
meaning = ["update", "read_preview_frame"]
evaluation = ["update", "yuv420p_to_rgba"]
render = ["view"]
observable = ["source_preview_renders_decoded_frame"]
*/

use std::path::PathBuf;

use iced::widget::{button, column, container, image as image_widget, row, text};
use iced::{Element, Length, Task};
use motolii_media::{MediaInfo, PreviewFrame};
use motolii_store::{AssetId, AssetStatus};

use crate::tokens::{Colors, Dimensions};
use crate::{browser_pane, Shell};

#[derive(Debug, Clone)]
pub struct LoadedFrame {
    pub name: String,
    pub path: PathBuf,
    pub info: MediaInfo,
    pub frame: PreviewFrame,
}

#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub asset_id: AssetId,
    pub name: String,
    pub path: PathBuf,
    pub info: MediaInfo,
    pub frame_index: i64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Previous,
    Next,
    TogglePlayback,
    SetIn,
    SetOut,
    Close,
    Tick,
    FrameLoaded {
        asset_id: AssetId,
        result: Result<LoadedFrame, String>,
    },
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Close,
    Load(LoadRequest),
    Error(String),
}

#[derive(Default)]
pub struct State {
    asset_id: Option<AssetId>,
    name: Option<String>,
    path: Option<PathBuf>,
    info: Option<MediaInfo>,
    frame_index: i64,
    in_frame: i64,
    out_frame: Option<i64>,
    image: Option<iced::widget::image::Handle>,
    loading: bool,
    playing: bool,
    error: Option<String>,
}

impl State {
    pub fn begin(&mut self, asset_id: AssetId, name: String, path: PathBuf) {
        self.asset_id = Some(asset_id);
        self.name = Some(name);
        self.path = Some(path);
        self.info = None;
        self.frame_index = 0;
        self.in_frame = 0;
        self.out_frame = None;
        self.image = None;
        self.loading = true;
        self.playing = false;
        self.error = None;
    }

    pub fn is_open(&self) -> bool {
        self.asset_id.is_some()
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Previous => self.load_relative(-1),
            Message::Next => self.load_relative(1),
            Message::Tick => {
                if self.playing {
                    self.load_relative(1)
                } else {
                    Action::None
                }
            }
            Message::TogglePlayback => {
                self.playing = !self.playing;
                Action::None
            }
            Message::SetIn => {
                self.in_frame = self.frame_index;
                if self.out_frame.is_some_and(|out| out < self.in_frame) {
                    self.out_frame = None;
                }
                Action::None
            }
            Message::SetOut => {
                self.out_frame = Some(self.frame_index.max(self.in_frame));
                Action::None
            }
            Message::Close => {
                *self = Self::default();
                Action::Close
            }
            Message::FrameLoaded { asset_id, result } => {
                if self.asset_id != Some(asset_id) {
                    return Action::None;
                }
                self.loading = false;
                match result {
                    Ok(loaded) => {
                        self.name = Some(loaded.name);
                        self.path = Some(loaded.path);
                        self.info = Some(loaded.info);
                        self.frame_index = loaded.frame.frame_index;
                        self.image = Some(iced::widget::image::Handle::from_rgba(
                            loaded.frame.width,
                            loaded.frame.height,
                            loaded.frame.rgba,
                        ));
                        self.error = None;
                        Action::None
                    }
                    Err(error) => {
                        self.playing = false;
                        self.error = Some(error.clone());
                        Action::Error(error)
                    }
                }
            }
        }
    }

    fn load_relative(&mut self, delta: i64) -> Action {
        if self.loading {
            return Action::None;
        }
        let Some(info) = self.info.clone() else {
            return Action::None;
        };
        let Some(path) = self.path.clone() else {
            return Action::None;
        };
        let Some(asset_id) = self.asset_id else {
            return Action::None;
        };
        let next = self.frame_index.saturating_add(delta).max(0);
        if let Some(frame_count) = info.nb_frames {
            if frame_count <= 0 || next >= frame_count {
                self.playing = false;
                return Action::None;
            }
        }
        if let Some(out) = self.out_frame {
            if next > out {
                self.playing = false;
                return Action::None;
            }
        }
        if next == self.frame_index {
            return Action::None;
        }
        self.frame_index = next;
        self.loading = true;
        Action::Load(LoadRequest {
            asset_id,
            name: self.name.clone().unwrap_or_else(|| "素材".to_owned()),
            path,
            info,
            frame_index: next,
        })
    }

    pub fn view(&self, dims: Dimensions, colors: Colors) -> Element<'_, Message> {
        let title = self.name.as_deref().unwrap_or("Source Preview");
        let picture: Element<'_, Message> = match &self.image {
            Some(handle) => container(image_widget(handle.clone()).width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            None if self.loading => container(text("素材を読み込み中…"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            None => container(text(self.error.as_deref().unwrap_or("フレームがありません")))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        };
        let end = self
            .out_frame
            .map_or_else(|| "—".to_owned(), |frame| frame.to_string());
        let controls = row![
            button(text(if self.playing { "Pause" } else { "Play" }))
                .on_press(Message::TogglePlayback),
            button(text("Frame −")).on_press(Message::Previous),
            text(format!("Frame {} / In {} / Out {}", self.frame_index, self.in_frame, end)),
            button(text("Frame +")).on_press(Message::Next),
            button(text("Set In")).on_press(Message::SetIn),
            button(text("Set Out")).on_press(Message::SetOut),
            button(text("Close")).on_press(Message::Close),
        ]
        .spacing(dims.theme().space.s)
        .align_y(iced::alignment::Vertical::Center);

        container(column![text(title).size(dims.theme().text.body), picture, controls]
            .spacing(dims.theme().space.m)
            .padding(dims.theme().space.l))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(colors.surface_panel)),
            ..Default::default()
        })
        .into()
    }
}

impl Shell {
    pub(crate) fn open_source_preview(
        &mut self,
        request: browser_pane::PreviewMedia,
    ) -> Task<crate::Message> {
        let asset = match self.doc.view().asset(request.asset_id()) {
            Ok(Some(asset)) => asset,
            Ok(None) => {
                self.status = Some("下見する素材が台帳に見つかりません".to_owned());
                return Task::none();
            }
            Err(error) => {
                self.status = Some(format!("素材を読めません: {error}"));
                return Task::none();
            }
        };
        let project_root = self
            .current_path
            .as_deref()
            .and_then(std::path::Path::parent);
        let path = match asset.resolve_status(project_root) {
            AssetStatus::Present { resolved_path } => PathBuf::from(resolved_path),
            AssetStatus::Missing => {
                self.status = Some(format!("素材 \"{}\" が見つかりません", asset.name));
                return Task::none();
            }
            AssetStatus::Unreadable { reason } => {
                self.status = Some(format!("素材 \"{}\" を読めません: {reason}", asset.name));
                return Task::none();
            }
            AssetStatus::Unchecked => {
                let fallback = asset
                    .path_absolute
                    .clone()
                    .map(PathBuf::from)
                    .or_else(|| {
                        asset.path_project_relative.clone().and_then(|relative| {
                            project_root.map(|root| root.join(relative))
                        })
                    });
                let Some(path) = fallback else {
                    self.status = Some(format!("素材 \"{}\" はファイル実体を持ちません", asset.name));
                    return Task::none();
                };
                path
            }
        };
        let asset_id = request.asset_id();
        let name = asset.name;
        self.source_preview.begin(asset_id, name.clone(), path.clone());
        Self::source_preview_probe(asset_id, name, path)
    }

    fn source_preview_probe(
        asset_id: AssetId,
        name: String,
        path: PathBuf,
    ) -> Task<crate::Message> {
        Task::perform(
            async move {
                let info = motolii_media::probe(&path).map_err(|error| error.to_string())?;
                let frame = motolii_media::read_preview_frame(&path, &info, 0)
                    .map_err(|error| error.to_string())?;
                Ok(LoadedFrame { name, path, info, frame })
            },
            move |result| {
                crate::Message::SourcePreview(Message::FrameLoaded { asset_id, result })
            },
        )
    }

    fn source_preview_load(
        asset_id: AssetId,
        name: String,
        path: PathBuf,
        info: MediaInfo,
        frame_index: i64,
    ) -> Task<crate::Message> {
        Task::perform(
            async move {
                let frame = motolii_media::read_preview_frame(&path, &info, frame_index)
                    .map_err(|error| error.to_string())?;
                Ok(LoadedFrame { name, path, info, frame })
            },
            move |result| {
                crate::Message::SourcePreview(Message::FrameLoaded { asset_id, result })
            },
        )
    }

    pub(crate) fn dispatch_source_preview(
        &mut self,
        message: crate::Message,
    ) -> Result<Task<crate::Message>, crate::Message> {
        let crate::Message::SourcePreview(message) = message else {
            return Err(message);
        };
        let action = self.source_preview.update(message);
        let task = match action {
            Action::None | Action::Close => Task::none(),
            Action::Error(error) => {
                self.status = Some(format!("素材プレビュー: {error}"));
                Task::none()
            }
            Action::Load(request) => Self::source_preview_load(
                request.asset_id,
                request.name,
                request.path,
                request.info,
                request.frame_index,
            ),
        };
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_and_out_are_front_state_only() {
        let mut state = State::default();
        state.frame_index = 12;
        assert!(matches!(state.update(Message::SetIn), Action::None));
        state.frame_index = 24;
        assert!(matches!(state.update(Message::SetOut), Action::None));
        assert_eq!(state.in_frame, 12);
        assert_eq!(state.out_frame, Some(24));
    }

    #[test]
    fn playback_does_not_cross_out_frame() {
        let mut state = State::default();
        state.asset_id = Some(AssetId::from_raw(1));
        state.path = Some(PathBuf::from("clip.mp4"));
        state.info = Some(MediaInfo {
            width: 2,
            height: 2,
            fps: motolii_core::Fps::try_new(30, 1).unwrap(),
            duration: None,
            nb_frames: Some(3),
            color_space: motolii_core::ColorSpace::Rec709Limited,
            rotation: 0,
        });
        state.out_frame = Some(1);
        state.frame_index = 1;
        state.playing = true;
        assert!(matches!(state.update(Message::Tick), Action::None));
        assert!(!state.playing);
    }

    #[test]
    fn source_preview_renders_decoded_frame() {
        let mut state = State::default();
        state.begin(
            AssetId::from_raw(7),
            "clip".to_owned(),
            PathBuf::from("clip.mp4"),
        );
        let frame = PreviewFrame {
            width: 2,
            height: 2,
            frame_index: 0,
            rgba: vec![
                0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
            ],
        };
        let result = state.update(Message::FrameLoaded {
            asset_id: AssetId::from_raw(7),
            result: Ok(LoadedFrame {
                name: "clip".to_owned(),
                path: PathBuf::from("clip.mp4"),
                info: MediaInfo {
                    width: 2,
                    height: 2,
                    fps: motolii_core::Fps::try_new(30, 1).unwrap(),
                    duration: None,
                    nb_frames: Some(1),
                    color_space: motolii_core::ColorSpace::Rec709Limited,
                    rotation: 0,
                },
                frame,
            }),
        });
        assert!(matches!(result, Action::None));
        assert!(state.image.is_some());
    }
}
