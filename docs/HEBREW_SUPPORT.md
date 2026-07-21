# Hebrew meeting support

Meetily supports Hebrew meeting content across live transcription, audio import,
retranscription, AI summaries, editing, and copy/export workflows. The application
interface itself remains in English; Hebrew and mixed Hebrew/English meeting content
is rendered bidirectionally.

## Recommended setup

1. Open **Settings → Transcription**.
2. Choose **Local Whisper**. The bundled Parakeet TDT 0.6B v3 model does not support
   Hebrew.
3. Download a multilingual model. `large-v3-turbo` is the recommended accuracy/speed
   balance; `large-v3-turbo-q5_0` uses less memory.
4. On the recording screen, open **Language** and choose **Hebrew (`he`)**. Selecting
   the language explicitly is more reliable than automatic detection for short audio
   chunks.
5. In a meeting's summary panel, leave the language on **Auto** to follow the dominant
   transcript language, or choose **Hebrew** explicitly.

Do not use an English-only Whisper model (`*.en`). Meetily's downloadable model catalog
contains multilingual models.

## Feature behavior

| Feature | Hebrew behavior |
| --- | --- |
| Live recording | Local multilingual Whisper with the `he` language hint |
| Import audio | Choose a downloaded Whisper model; Hebrew+Parakeet is blocked |
| Retranscription | Choose Hebrew and a downloaded Whisper model |
| AI summary | Auto detects dominant Hebrew, or Hebrew can be selected explicitly |
| Summary templates | Template structure is preserved when translated to Hebrew |
| Editing and viewing | Each paragraph automatically uses RTL or LTR based on its content |
| Mixed technical text | English product names, code, URLs, and numbers remain readable |
| Copy/export | Unicode Hebrew text and Markdown are preserved |

Summary quality still depends on the selected LLM. For local Ollama or Built-in AI,
choose a model with strong Hebrew instruction-following and translation capability.

## Verification checklist

- Record at least two minutes of Hebrew conversation with two speakers.
- Include names, dates, numbers, an English product name, and a URL.
- Confirm live transcript paragraphs align RTL while timestamps remain stable.
- Import the same audio and compare it with the live transcript.
- Retranscribe with `large-v3-turbo` and Hebrew selected.
- Generate both Auto and explicitly Hebrew summaries.
- Edit Hebrew headings and list items, then save and reopen the meeting.
- Copy the transcript and summary into a Unicode-aware editor and verify ordering.

Automated Rust tests cover Hebrew language detection, Auto summary routing, and
Hebrew-specific summary prompt rules. Visual bidirectional behavior should also be
checked manually on macOS, Windows, and Linux because text shaping is platform-owned.
