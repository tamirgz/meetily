# Hebrew meeting support

Meetily supports Hebrew meeting content across live transcription, audio import,
retranscription, AI summaries, editing, and copy/export workflows. The application
interface itself remains in English; Hebrew and mixed Hebrew/English meeting content
is rendered bidirectionally.

## Recommended setup

1. Open **Settings → Transcription**.
2. Choose **Local Whisper**. The bundled Parakeet TDT 0.6B v3 model does not support
   Hebrew.
3. Download **Hebrew Large V3 (Ivrit.AI)** for the highest Hebrew accuracy.
   **Hebrew Large V3 Turbo (Ivrit.AI)** is faster and uses less memory. Standard
   multilingual `large-v3` and `large-v3-turbo` remain useful when a meeting
   regularly switches between several languages.
4. Add participant names, companies, acronyms, and technical terms to
   **Meeting vocabulary**. Meetily adds these terms to the Hebrew Whisper prompt
   to improve spelling consistency.
5. On the recording screen, open **Language** and choose **Hebrew (`he`)**. Selecting
   the language explicitly is more reliable than automatic detection for short audio
   chunks.
6. In a meeting's summary panel, leave the language on **Auto** to follow the dominant
   transcript language, or choose **Hebrew** explicitly.

Do not use an English-only Whisper model (`*.en`). Meetily's downloadable model catalog
contains multilingual models.

## Feature behavior

| Feature | Hebrew behavior |
| --- | --- |
| Live recording | Local multilingual Whisper with the `he` language hint |
| macOS microphone cleanup | DeepFilterNet3 full-band neural enhancement before VAD |
| macOS speaker labels | Offline Core ML diarization after saving; labels can be renamed |
| Import audio | Choose a downloaded Whisper model; Hebrew+Parakeet is blocked |
| Retranscription | Choose Hebrew and a downloaded Whisper model |
| AI summary | Auto detects dominant Hebrew, or Hebrew can be selected explicitly |
| Summary templates | Template structure is preserved when translated to Hebrew |
| Editing and viewing | Each paragraph automatically uses RTL or LTR based on its content |
| Mixed technical text | English product names, code, URLs, and numbers remain readable |
| Copy/export | Unicode Hebrew text and Markdown are preserved |

## macOS voice pipeline

The Apple Silicon recording pipeline is intentionally ordered to protect speech
detail:

1. Capture microphone and digital system audio separately at their native rates.
2. Convert microphone audio to 48 kHz with a persistent band-limited sinc
   resampler when required.
3. Remove sub-80 Hz rumble, run DeepFilterNet3 full-band neural enhancement on a
   dedicated worker thread, and apply conservative EBU R128 loudness control.
   Digital system audio is not denoised because it is already a clean source.
4. Mix in 50 ms windows with headroom and continuous soft limiting.
5. Convert the transcription stream to 16 kHz with a stateful band-limited
   resampler, then segment speech with Silero VAD.
6. Decode Hebrew with multilingual Whisper beam search, the explicit `he`
   language hint, recent-segment context, and the meeting vocabulary.
7. After saving, run FluidAudio/Core ML diarization over the complete recording
   and match speaker turns to timestamped transcript segments.

The diarizer emits anonymous labels (`Speaker 1`, `Speaker 2`, and so on). In a
saved meeting, click a label to replace it with the participant's real name.
Speaker labels are included in copied transcripts and in the transcript supplied
to the summary model.

DeepFilterNet3's weights are bundled in the macOS binary. FluidAudio's
diarization models are downloaded and compiled on first use and cached locally;
this initial run can take longer than later meetings.

Summary quality still depends on the selected LLM. For local Ollama or Built-in AI,
choose a model with strong Hebrew instruction-following and translation capability.
Qwen 3.5 is the recommended local family for Hebrew and English summaries. Prefer a
larger model when enough memory is available and the meeting contains many decisions,
owners, or follow-up items.

## Verification checklist

- Record at least two minutes of Hebrew conversation with two speakers.
- Include names, dates, numbers, an English product name, and a URL.
- Confirm live transcript paragraphs align RTL while timestamps remain stable.
- Confirm background fan/keyboard noise is reduced without clipping quiet Hebrew
  consonants.
- Wait for speaker labeling, rename `Speaker 1`, and verify the name persists on
  every segment from that speaker.
- Import the same audio and compare it with the live transcript.
- Retranscribe with `hebrew-large-v3` and Hebrew selected.
- Generate both Auto and explicitly Hebrew summaries.
- Edit Hebrew headings and list items, then save and reopen the meeting.
- Copy the transcript and summary into a Unicode-aware editor and verify ordering.

Automated Rust tests cover Hebrew language detection, Auto summary routing, and
Hebrew-specific summary prompt rules. Visual bidirectional behavior should also be
checked manually on macOS, Windows, and Linux because text shaping is platform-owned.
