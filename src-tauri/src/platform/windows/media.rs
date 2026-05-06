use crate::types::MediaInfo;
use base64::Engine;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

pub async fn get_current_media() -> Result<String, String> {
    let mut info = MediaInfo {
        title: "Now listening...".to_string(),
        artist: String::new(),
        album: String::new(),
        app_id: "Generic".to_string(),
        thumbnail_base64: String::new(),
        is_playing: false,
        position_ms: 0,
        duration_ms: 0,
    };

    if let Ok(op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
        if let Ok(manager) = op.get() {
            if let Ok(session) = manager.GetCurrentSession() {
                if let Ok(playback) = session.GetPlaybackInfo() {
                    if let Ok(status) = playback.PlaybackStatus() {
                        info.is_playing = status
                            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
                    }
                }

                if let Ok(timeline) = session.GetTimelineProperties() {
                    if let (Ok(end), Ok(start)) = (timeline.EndTime(), timeline.StartTime()) {
                        let dur = end.Duration - start.Duration;
                        if dur > 0 {
                            info.duration_ms = (dur / 10_000) as u64;
                        }
                    }

                    if let Ok(pos) = timeline.Position() {
                        let mut pos_100ns = pos.Duration.max(0);
                        if info.is_playing {
                            if let Ok(last_updated) = timeline.LastUpdatedTime() {
                                let unix_now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default();
                                let now_100ns = (unix_now.as_nanos() / 100) as i64
                                    + 116_444_736_000_000_000i64;
                                let elapsed = now_100ns - last_updated.UniversalTime;
                                if elapsed > 0 {
                                    pos_100ns += elapsed;
                                }
                            }
                        }
                        info.position_ms = (pos_100ns / 10_000) as u64;
                        if info.duration_ms > 0 && info.position_ms > info.duration_ms {
                            info.position_ms = info.duration_ms;
                        }
                    }
                }

                if let Ok(app_id) = session.SourceAppUserModelId() {
                    info.app_id = app_id.to_string();
                }

                if let Ok(op2) = session.TryGetMediaPropertiesAsync() {
                    if let Ok(props) = op2.get() {
                        info.title = props.Title().map(|s| s.to_string()).unwrap_or_default();
                        info.artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
                        info.album = props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();

                        if let Ok(thumb) = props.Thumbnail() {
                            if let Ok(stream_op) = thumb.OpenReadAsync() {
                                if let Ok(stream) = stream_op.get() {
                                    let size = stream.Size().unwrap_or(0);
                                    if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                                        if let Ok(op3) = reader.LoadAsync(size as u32) {
                                            if op3.get().is_ok() {
                                                let mut buf = vec![0u8; size as usize];
                                                if reader.ReadBytes(&mut buf).is_ok() {
                                                    info.thumbnail_base64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    serde_json::to_string(&info).map_err(|e| e.to_string())
}
