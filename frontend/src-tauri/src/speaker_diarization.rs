//! On-device speaker diarization for saved macOS meetings.
//!
//! Transcription and diarization are deliberately separate jobs. Whisper
//! remains responsible for Hebrew ASR, while FluidAudio's Core ML pipeline
//! assigns anonymous speaker labels from the complete recording. Processing
//! the complete file gives the clustering model more context and keeps model
//! compilation/downloads away from the real-time capture path.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::state::AppState;

const DEFAULT_CLUSTERING_THRESHOLD: f64 = 0.60;

static DIARIZATION_IN_PROGRESS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct DiarizationGuard {
    meeting_id: String,
}

impl DiarizationGuard {
    fn acquire(meeting_id: &str) -> Result<Self, String> {
        let mut meetings = DIARIZATION_IN_PROGRESS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !meetings.insert(meeting_id.to_string()) {
            return Err("Speaker identification is already running for this meeting".to_string());
        }
        Ok(Self {
            meeting_id: meeting_id.to_string(),
        })
    }
}

impl Drop for DiarizationGuard {
    fn drop(&mut self) {
        DIARIZATION_IN_PROGRESS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.meeting_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerSegment {
    pub speaker: String,
    pub start_time: f64,
    pub end_time: f64,
    pub quality_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiarizationResult {
    pub meeting_id: String,
    pub speakers: usize,
    pub segments: usize,
    pub transcripts_labeled: usize,
}

#[derive(Debug, sqlx::FromRow)]
struct TimedTranscript {
    id: String,
    audio_start_time: Option<f64>,
    audio_end_time: Option<f64>,
}

fn recording_path(folder: &Path) -> Result<PathBuf, String> {
    ["audio.mp4", "audio.wav", "audio.m4a"]
        .into_iter()
        .map(|name| folder.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            format!(
                "No saved meeting audio was found in {}",
                folder.to_string_lossy()
            )
        })
}

#[cfg(target_os = "macos")]
fn run_diarization(audio_path: PathBuf) -> Result<Vec<SpeakerSegment>, String> {
    use fluidaudio_rs::FluidAudio;

    let engine = FluidAudio::new()
        .map_err(|error| format!("Could not initialize speaker diarization: {error}"))?;
    engine
        .init_diarization(DEFAULT_CLUSTERING_THRESHOLD)
        .map_err(|error| format!("Could not load speaker diarization models: {error}"))?;

    let raw_segments = engine
        .diarize_file(&audio_path)
        .map_err(|error| format!("Speaker diarization failed: {error}"))?;

    // FluidAudio's model IDs are implementation details. Convert them to
    // deterministic, reader-friendly labels in first-appearance order.
    let mut speaker_labels: HashMap<String, String> = HashMap::new();
    let mut next_speaker = 1usize;
    let mut segments = Vec::with_capacity(raw_segments.len());

    for segment in raw_segments {
        let label = speaker_labels
            .entry(segment.speaker_id)
            .or_insert_with(|| {
                let label = format!("Speaker {next_speaker}");
                next_speaker += 1;
                label
            })
            .clone();

        segments.push(SpeakerSegment {
            speaker: label,
            start_time: segment.start_time as f64,
            end_time: segment.end_time as f64,
            quality_score: segment.quality_score,
        });
    }

    Ok(segments)
}

#[cfg(not(target_os = "macos"))]
fn run_diarization(_audio_path: PathBuf) -> Result<Vec<SpeakerSegment>, String> {
    Err("On-device speaker diarization is currently available on macOS only".to_string())
}

fn best_speaker_for_transcript<'a>(
    transcript: &TimedTranscript,
    segments: &'a [SpeakerSegment],
) -> Option<&'a str> {
    let start = transcript.audio_start_time?;
    let end = transcript.audio_end_time.unwrap_or(start);

    segments
        .iter()
        .filter_map(|segment| {
            let overlap = end.min(segment.end_time) - start.max(segment.start_time);
            (overlap > 0.0).then_some((segment.speaker.as_str(), overlap))
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(speaker, _)| speaker)
        .or_else(|| {
            // VAD/ASR boundaries and diarization boundaries are produced by
            // different models. For a short gap, use the nearest speaker
            // segment rather than leaving a whole utterance unlabeled.
            let midpoint = (start + end) / 2.0;
            segments
                .iter()
                .map(|segment| {
                    let segment_midpoint = (segment.start_time + segment.end_time) / 2.0;
                    (
                        segment.speaker.as_str(),
                        (midpoint - segment_midpoint).abs(),
                    )
                })
                .filter(|(_, distance)| *distance <= 1.5)
                .min_by(|left, right| left.1.total_cmp(&right.1))
                .map(|(speaker, _)| speaker)
        })
}

async fn apply_speaker_labels(
    pool: &SqlitePool,
    meeting_id: &str,
    segments: &[SpeakerSegment],
) -> Result<usize, String> {
    let transcripts = sqlx::query_as::<_, TimedTranscript>(
        "SELECT id, audio_start_time, audio_end_time
         FROM transcripts
         WHERE meeting_id = ?
         ORDER BY audio_start_time ASC",
    )
    .bind(meeting_id)
    .fetch_all(pool)
    .await
    .map_err(|error| format!("Could not load transcript timings: {error}"))?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| format!("Could not start speaker-label update: {error}"))?;
    let mut labeled = 0usize;

    for transcript in transcripts {
        if let Some(speaker) = best_speaker_for_transcript(&transcript, segments) {
            sqlx::query("UPDATE transcripts SET speaker = ? WHERE id = ?")
                .bind(speaker)
                .bind(&transcript.id)
                .execute(&mut *transaction)
                .await
                .map_err(|error| format!("Could not save speaker label: {error}"))?;
            labeled += 1;
        }
    }

    transaction
        .commit()
        .await
        .map_err(|error| format!("Could not commit speaker labels: {error}"))?;
    Ok(labeled)
}

/// Run offline diarization for one saved meeting and persist anonymous labels.
///
/// The first call downloads and compiles the Core ML models. The blocking
/// FluidAudio bridge runs outside Tokio's async worker pool.
#[tauri::command]
pub async fn api_diarize_meeting<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    meeting_id: String,
) -> Result<DiarizationResult, String> {
    let folder_path: Option<String> =
        sqlx::query_scalar("SELECT folder_path FROM meetings WHERE id = ?")
            .bind(&meeting_id)
            .fetch_optional(state.db_manager.pool())
            .await
            .map_err(|error| format!("Could not find meeting recording: {error}"))?
            .flatten();

    let folder_path =
        folder_path.ok_or_else(|| "This meeting does not have a recording folder".to_string())?;
    let audio_path = recording_path(Path::new(&folder_path))?;
    let _diarization_guard = DiarizationGuard::acquire(&meeting_id)?;

    let segments = tauri::async_runtime::spawn_blocking(move || run_diarization(audio_path))
        .await
        .map_err(|error| format!("Speaker diarization worker failed: {error}"))??;

    if segments.is_empty() {
        return Err("No speakers were detected in this recording".to_string());
    }

    let transcripts_labeled =
        apply_speaker_labels(state.db_manager.pool(), &meeting_id, &segments).await?;
    let speakers = segments
        .iter()
        .map(|segment| segment.speaker.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    let result = DiarizationResult {
        meeting_id,
        speakers,
        segments: segments.len(),
        transcripts_labeled,
    };
    let _ = app.emit("speaker-diarization-complete", &result);
    Ok(result)
}

/// Rename an anonymous label throughout a meeting.
#[tauri::command]
pub async fn api_rename_speaker<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, AppState>,
    meeting_id: String,
    current_name: String,
    new_name: String,
) -> Result<usize, String> {
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err("Speaker name cannot be empty".to_string());
    }
    if new_name.chars().count() > 80 {
        return Err("Speaker name must be 80 characters or fewer".to_string());
    }

    let result = sqlx::query(
        "UPDATE transcripts
         SET speaker = ?
         WHERE meeting_id = ? AND speaker = ?",
    )
    .bind(new_name)
    .bind(&meeting_id)
    .bind(current_name.trim())
    .execute(state.db_manager.pool())
    .await
    .map_err(|error| format!("Could not rename speaker: {error}"))?;

    Ok(result.rows_affected() as usize)
}

#[cfg(test)]
mod tests {
    use super::{best_speaker_for_transcript, SpeakerSegment, TimedTranscript};

    fn transcript(start: f64, end: f64) -> TimedTranscript {
        TimedTranscript {
            id: "transcript".to_string(),
            audio_start_time: Some(start),
            audio_end_time: Some(end),
        }
    }

    fn segment(speaker: &str, start: f64, end: f64) -> SpeakerSegment {
        SpeakerSegment {
            speaker: speaker.to_string(),
            start_time: start,
            end_time: end,
            quality_score: 1.0,
        }
    }

    #[test]
    fn assigns_the_speaker_with_the_largest_timestamp_overlap() {
        let segments = vec![
            segment("Speaker 1", 0.0, 3.0),
            segment("Speaker 2", 3.0, 8.0),
        ];

        assert_eq!(
            best_speaker_for_transcript(&transcript(2.5, 6.0), &segments),
            Some("Speaker 2")
        );
    }

    #[test]
    fn bridges_small_boundary_gaps_but_not_large_ones() {
        let segments = vec![segment("Speaker 1", 0.0, 2.0)];

        assert_eq!(
            best_speaker_for_transcript(&transcript(2.1, 2.3), &segments),
            Some("Speaker 1")
        );
        assert_eq!(
            best_speaker_for_transcript(&transcript(5.0, 5.5), &segments),
            None
        );
    }
}
