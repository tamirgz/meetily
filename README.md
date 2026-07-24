<div align="center" style="border-bottom: none">
    <h1>
        <img src="docs/Meetily-6.png" style="border-radius: 10px;" />
        <br>
        Privacy-First AI Meeting Assistant
    </h1>
    <a href="https://trendshift.io/repositories/21958" target="_blank"><img src="https://trendshift.io/api/badge/repositories/21958" alt="Zackriya-Solutions%2Fmeetily | Trendshift" style="width: 250px; height: 55px;" width="250" height="55"/></a>
    <br>
    <br>
    <a href="https://github.com/tamirgz/meetily/releases"><img src="https://img.shields.io/badge/Hebrew_builds-Releases-brightgreen" alt="Hebrew-enabled releases"></a>
    <a href="https://github.com/Zackriya-Solutions/meeting-minutes/releases"><img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/zackriya-solutions/meeting-minutes?style=flat">
</a>
 <a href="https://github.com/Zackriya-Solutions/meeting-minutes/releases"> <img alt="GitHub Downloads (all assets, all releases)" src="https://img.shields.io/github/downloads/zackriya-solutions/meeting-minutes/total?style=plastic"> </a>
    <a href="https://github.com/Zackriya-Solutions/meeting-minutes/releases"><img src="https://img.shields.io/badge/License-MIT-blue" alt="License"></a>
    <a href="https://github.com/tamirgz/meetily/releases"><img src="https://img.shields.io/badge/Supported_OS-macOS,_Windows,_Linux-white" alt="Supported OS"></a>
    <a href="https://github.com/Zackriya-Solutions/meeting-minutes/releases"><img alt="GitHub Tag" src="https://img.shields.io/github/v/tag/zackriya-solutions/meeting-minutes?include_prereleases&color=yellow">
</a>
    <br>
    <h3>
    <br>
    Open Source • Privacy-First • Enterprise-Ready
    </h3>
    <p align="center">
    Get latest <a href="https://www.zackriya.com/meetily-subscribe/"><b>Product updates</b></a> <br><br>
    <a href="https://meetily.ai"><b>Website</b></a> •
    <a href="https://www.linkedin.com/company/106363062/"><b>LinkedIn</b></a> •
    <a href="https://discord.gg/crRymMQBFH"><b>Meetily Discord</b></a> •
    <a href="https://discord.com/invite/vCFJvN4BwJ"><b>Privacy-First AI</b></a> •
    <a href="https://www.reddit.com/r/meetily/"><b>Reddit</b></a>
</p>
    <p align="center">

A privacy-first AI meeting assistant that captures, transcribes, and summarizes meetings entirely on your infrastructure. Built by expert AI engineers passionate about data sovereignty and open source solutions. Perfect for enterprises that need advanced meeting intelligence without compromising on privacy, compliance, or control.

</p>

<p align="center">
    <img src="docs/meetily_demo.gif" width="650" alt="Meetily Demo" />
    <br>
    <a href="https://youtu.be/6FnhSC_eSz8">View full Demo Video</a>
</p>

</div>

---

> **Meetily PRO Upgrade Offer** - Meetily PRO is available for users who need enhanced accuracy, advanced exports, custom summary workflows, and team-ready features. Use coupon code **LAUNCH20** for **20% off** until the next Meetily Community Edition release. Speaker diarization is also planned for PRO in mid-June. [Explore Meetily PRO →](https://meetily.ai/pro/)

---

<details>
<summary>Table of Contents</summary>

- [Introduction](#introduction)
- [Why Meetily?](#why-meetily)
- [Features](#features)
- [Installation](#installation)
- [Hebrew setup](#hebrew-setup)
- [Building release installers](#building-release-installers)
- [Key Features in Action](#key-features-in-action)
- [System Architecture](#system-architecture)
- [For Developers](#for-developers)
- [Meetily PRO](#meetily-pro)
- [Contributing](#contributing)
- [License](#license)

</details>

## Introduction

Meetily is a privacy-first AI meeting assistant that runs entirely on your local machine. It captures your meetings, transcribes them in real-time, and generates summaries, all without sending any data to the cloud. This makes it the perfect solution for professionals and enterprises who need to maintain complete control over their sensitive information.

## Why Meetily?

While there are many meeting transcription tools available, this solution stands out by offering:

- **Privacy First:** All processing happens locally on your device.
- **Cost-Effective:** Uses open-source AI models instead of expensive APIs.
- **Flexible:** Works offline and supports multiple meeting platforms.
- **Customizable:** Self-host and modify for your specific needs.

<details>
<summary>The Privacy Problem</summary>

Meeting AI tools create significant privacy and compliance risks across all sectors:

- **$4.4M average cost per data breach** (IBM 2024)
- **€5.88 billion in GDPR fines** issued by 2025
- **400+ unlawful recording cases** filed in California this year

Whether you're a defense consultant, enterprise executive, legal professional, or healthcare provider, your sensitive discussions shouldn't live on servers you don't control. Cloud meeting tools promise convenience but deliver privacy nightmares with unclear data storage practices and potential unauthorized access.

**Meetily solves this:** Complete data sovereignty on your infrastructure, zero vendor lock-in, and full control over your sensitive conversations.

</details>

## Features

- **Local First:** All processing is done on your machine. No data ever leaves your computer.
- **Real-time Transcription:** Get a live transcript of your meeting as it happens.
- **AI-Powered Summaries:** Generate summaries of your meetings using powerful language models.
- **Multi-Platform:** Works on macOS, Windows, and Linux.
- **Open Source:** Meetily is open source and free to use.
- **Flexible AI Provider Support:** Choose from Ollama (local), Claude, Groq, OpenRouter, or use your own OpenAI-compatible endpoint.
- **Hebrew Meeting Content:** Transcribe, summarize, edit, and export Hebrew and mixed Hebrew/English meetings with bidirectional text support. See [Hebrew meeting support](docs/HEBREW_SUPPORT.md).
- **Enhanced macOS Audio:** Apple Silicon builds apply full-band
  [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) speech enhancement
  to microphone audio before VAD and transcription.
- **macOS Speaker Labels:** Saved meetings are diarized locally with
  [FluidAudio](https://github.com/FluidInference/FluidAudio). Anonymous labels
  such as `Speaker 1` can be renamed to participant names and are included when
  copying transcripts or generating summaries.

## Installation

This fork publishes Hebrew-enabled installers from
[`tamirgz/meetily`](https://github.com/tamirgz/meetily). The release workflow
builds every platform from the same source revision on native macOS, Windows,
and Linux runners.

### 🪟 **Windows**

1. Download the latest `x64-setup.exe` or `.msi` from the
   [fork releases](https://github.com/tamirgz/meetily/releases/latest).
2. Run the installer. If Windows SmartScreen appears for an unsigned community
   build, choose **More info → Run anyway** only after verifying the SHA-256
   checksum published with the release.
3. Allow microphone access when Windows asks.

### 🍎 **macOS**

1. Apple Silicon users can download the newest `.dmg` from the
   [fork releases](https://github.com/tamirgz/meetily/releases/latest).
2. Open the downloaded `.dmg` file
3. Drag **Meetily** to your Applications folder
4. Open **Meetily** from the Applications folder. For an unsigned local build,
   right-click the app and choose **Open** the first time.
5. Allow microphone, system-audio, and screen-recording permissions when macOS
   asks. Restart Meetily after changing those permissions.

### 🐧 **Linux**

Download the latest `.AppImage` or `.deb` from the
[fork releases](https://github.com/tamirgz/meetily/releases/latest).

For an AppImage:

```bash
chmod +x meetily_*.AppImage
./meetily_*.AppImage
```

For Ubuntu/Debian:

```bash
sudo apt install ./meetily_*.deb
```

You can also build from source:

- [Building on Linux](docs/building_in_linux.md)
- [General Build Instructions](docs/BUILDING.md)

## Hebrew support: post-install configuration

Hebrew support is included in the application, but the large transcription and
summary models are downloaded after installation. The model downloaded by the
first-run onboarding may be Parakeet; **Parakeet cannot transcribe Hebrew**.
Complete the following setup before recording a Hebrew meeting.

### 1. Allow audio capture

On macOS, open **System Settings → Privacy & Security** and allow Meetily under:

- **Microphone**
- **Screen & System Audio Recording** (called **Screen Recording** on some
  macOS versions)

Quit and reopen Meetily after changing either permission. Then open
**Settings → Recordings → Default Audio Devices** and select the microphone and
system-audio devices that will be used for meetings.

### 2. Download and select a Hebrew transcription model

Open **Settings → Transcription** and set **Transcript Model** to
**Local Whisper (High Accuracy)**. Download one of these models:

| Model shown in Meetily | Download size | Recommended use |
| --- | ---: | --- |
| **Hebrew Large V3 (Ivrit.AI)** | 2.95 GB | Best available Hebrew accuracy; recommended for important meetings |
| **Hebrew Large V3 Turbo (Ivrit.AI)** | 1.55 GB | Faster transcription and lower resource use, with a small accuracy trade-off |

Wait until the model is marked **Available**, then click its card so it is the
selected model. For the best result, use **Hebrew Large V3 (Ivrit.AI)**.

In **Meeting vocabulary**, add the spelling of participant names, company and
product names, acronyms, and English technical terms used in the meeting. For
example:

```text
Tamir, Microsoft Sentinel, CrowdStrike, XDR, false positive, regional director
```

The vocabulary is a transcription hint, not a translation dictionary, and is
stored only on the local computer.

### 3. Select Hebrew for every recording or import

Return to the recording screen and use its **Language** selector to choose
**Hebrew** before pressing Record. The language selector is on the recorder; it
is not a separate page under Settings.

Also choose **Hebrew** in the **Import Audio** or **Enhance / Retranscribe**
dialog when processing an existing recording. Explicit Hebrew selection is
more reliable than **Auto** for short chunks and Hebrew/English meetings. If
Meetily reports that Hebrew requires Whisper, return to
**Settings → Transcription** and verify that the downloaded Ivrit.AI model is
selected.

### 4. Configure a Hebrew-capable summary model

Open **Settings → Summary → Summary Model Configuration**:

1. Select **Built-in AI (Offline, No API needed)** as the provider.
2. Download one of the Qwen models below.
3. Wait until it is marked **Ready**, click the model card so it is marked
   **Selected**, and click **Save**.

| Model shown in Meetily | Download size | Recommended use |
| --- | ---: | --- |
| **Qwen 3.5 4B (High Quality)** | 2.61 GB | Recommended local model for the most complete Hebrew or English summaries |
| **Qwen 3.5 2B (Balanced)** | 1.22 GB | Use on lower-memory computers or when faster generation is more important |

The Qwen 4B model is the recommended local summarizer when the computer has
enough available memory. A cloud provider such as OpenAI, Claude, Groq, or
OpenRouter can also summarize Hebrew, but requires its own API key and sends
the transcript to that provider.

Under **Settings → Summary → Summary Language**, choose **＋ Add language**, add
**Hebrew**, and click the Hebrew language chip to pin it as the default. You can
override this per meeting from the language control in the summary toolbar.
Choose **Hebrew** for Hebrew output, or **Auto** to follow the dominant
transcript language.

### 5. macOS noise reduction and speaker labels

No extra noise-reduction model needs to be configured. The macOS build bundles
DeepFilterNet3 weights and applies microphone enhancement automatically.

Speaker diarization is also local and is available after the recording is
saved. Open a saved meeting and click **Speakers** to identify speakers.
FluidAudio downloads and compiles its Core ML diarization models the first time
this feature runs, so keep an internet connection available for the first run
and allow extra processing time. Later runs reuse the local cache. Click a
speaker label such as `Speaker 1` to rename that speaker throughout the
meeting. This is speaker separation, not biometric identity recognition.

### 6. Disk space and model locations

Allow at least **7 GB of free disk space** for the recommended Hebrew Large V3
and Qwen 3.5 4B models plus download and runtime overhead. A smaller setup using
Hebrew Large V3 Turbo and Qwen 3.5 2B needs approximately 3 GB plus overhead.

Meetily manages the files automatically; do not manually rename or move them.
The main model directory is:

- macOS: `~/Library/Application Support/com.meetily.ai/models`
- Windows: `%APPDATA%\com.meetily.ai\models`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/com.meetily.ai/models`

Whisper models are stored in that directory and built-in summary models are in
its `summary` subdirectory.

### 7. Verify the setup

Record a 30–60 second test containing:

- normal Hebrew speech from two people;
- participant and company names from **Meeting vocabulary**;
- a number, a date, and several English technical terms.

After stopping, verify that the transcript is Hebrew, the names are spelled
correctly, **Speakers** assigns editable labels, and **Regenerate Summary**
produces Hebrew key points, decisions, next steps, and action items. If the
transcript is inaccurate, first confirm **Local Whisper**, **Hebrew Large V3
(Ivrit.AI)**, and the recorder's **Hebrew** language selection.

For mixed Hebrew/English guidance, accuracy checks, import/retranscription, and
additional troubleshooting, see
[Hebrew meeting support](docs/HEBREW_SUPPORT.md).

## Building release installers

Desktop installers should be built on their native operating system. In
particular, Windows packages require Microsoft's MSVC and WebView2 installer
toolchains and should not be treated as valid when cross-compiled from a Linux
container.

The [`Build fork installers`](.github/workflows/build-fork-installers.yml)
workflow builds:

- Apple Silicon macOS: `.dmg`
- Windows x64: `.msi` and NSIS `-setup.exe`
- Linux x64: `.deb` and `.AppImage`

Run the workflow manually from GitHub Actions. Pushing a tag such as `v0.4.0-hebrew.1`
also creates a draft release and attaches all installers plus SHA-256 manifests.

Linux can additionally be built reproducibly with Docker on macOS, Linux, or
Windows:

```bash
./scripts/build-linux-docker.sh
```

The Docker build writes its output to `release-artifacts/linux/`. See
[`release-artifacts/README.md`](release-artifacts/README.md) for the artifact
layout and platform limitations.

## Key Features in Action

### 🎯 Local Transcription

Transcribe meetings entirely on your device using **Whisper** or **Parakeet** models. No cloud required.

<p align="center">
    <img src="docs/home.png" width="650" style="border-radius: 10px;" alt="Meetily Demo" />
</p>

### 📥 Import & Enhance `Beta`

Import existing audio files to generate transcripts, or enhance to re-transcribe any recorded meeting with a different model or language, all processed locally.

> Contributed by [Jeremi Joslin](https://github.com/jeremi), improved by [Vishnu P S](https://github.com/p-s-vishnu) and [Mohammed Safvan](https://github.com/mohammedsafvan)

<p align="center">
    <img src="docs/meetily-export.gif" width="650" style="border-radius: 10px;" alt="Import and Enhance" />
</p>

### 🤖 AI-Powered Summaries

Generate meeting summaries with your choice of AI provider. **Ollama** (local) is recommended, with support for Claude, Groq, OpenRouter, and OpenAI.

<p align="center">
    <img src="docs/summary.png" width="650" style="border-radius: 10px;" alt="Summary generation" />
</p>

<p align="center">
    <img src="docs/editor1.png" width="650" style="border-radius: 10px;" alt="Editor Summary generation" />
</p>

### 🔒 Privacy-First Design

All data stays on your machine. Transcription models, recordings, and transcripts are stored locally.

<p align="center">
    <img src="docs/settings.png" width="650" style="border-radius: 10px;" alt="Local Transcription and storage" />
</p>

### 🌐 Custom OpenAI Endpoint Support

Use your own OpenAI-compatible endpoint for AI summaries. Perfect for organizations with custom AI infrastructure or preferred providers.

<p align="center">
    <img src="docs/custom.png" width="650" style="border-radius: 10px;" alt="Custom OpenAI Endpoint Configuration" />
</p>

### 🎙️ Professional Audio Mixing

Capture microphone and system audio simultaneously with intelligent ducking and clipping prevention.

<p align="center">
    <img src="docs/audio.png" width="650" style="border-radius: 10px;" alt="Device selection" />
</p>

### ⚡ GPU Acceleration

Built-in support for hardware acceleration across platforms:

- **macOS**: Apple Silicon (Metal) + CoreML
- **Windows/Linux**: NVIDIA (CUDA), AMD/Intel (Vulkan)

Automatically enabled at build time - no configuration needed.

## System Architecture

Meetily is a single, self-contained application built with [Tauri](https://tauri.app/). It uses a Rust-based backend to handle all the core logic, and a Next.js frontend for the user interface.

For more details, see the [Architecture documentation](docs/architecture.md).

## For Developers

If you want to contribute to Meetily or build it from source, you'll need to have Rust and Node.js installed. For detailed build instructions, please see the [Building from Source guide](docs/BUILDING.md).

Clone this Hebrew-enabled fork with:

```bash
git clone https://github.com/tamirgz/meetily.git
cd meetily
```

## Meetily Pro

<p align="center">
    <img src="docs/pv2.1.png" width="650" style="border-radius: 10px;" alt="Upcoming version" />
</p>

**Meetily PRO** is a professional-grade solution with enhanced accuracy and advanced features for serious users and teams. Built on a different codebase with superior transcription models and enterprise-ready capabilities.

### Community Thank-You Offer

Meetily Community Edition will remain free and open source. PRO exists for users and teams who want a more advanced meeting workflow, including higher transcription accuracy, custom summary templates, advanced exports, auto-meeting detection, and self-hosted deployment options.

For the community that helped Meetily grow, we are making the upgrade easier: use coupon code **LAUNCH20** for **20% off Meetily PRO** until the next Meetily Community Edition release.

This fork includes local speaker diarization in the macOS community build.

### Key Advantages Over Community Edition:

- **Enhanced Accuracy**: Superior transcription models for professional-grade accuracy
- **Custom Summary Templates**: Tailor summaries to your specific workflow and needs
- **Advanced Export Options**: PDF, DOCX, and Markdown exports with formatting
- **Auto-detect and Join Meetings**: Automatic meeting detection and joining
- **Speaker Identification**: Distinguish between speakers automatically
- **Chat with Meetings**: AI-powered meeting insights and queries *(Coming Soon)*
- **Calendar Integration**: Seamless integration with your calendar *(Coming Soon)*
- **Self-Hosted Deployment**: Deploy on your own infrastructure for teams
- **GDPR Compliance Built-In**: Privacy by design architecture with complete audit trails
- **Priority Support**: Dedicated support for PRO users

### Who is PRO for?

- **Professionals** who need the highest accuracy for critical meetings
- **Teams and organizations** (2-100 users) requiring self-hosted deployment
- **Power users** who need advanced export formats and custom workflows
- **Compliance-focused organizations** requiring GDPR readiness

> **Note:** Meetily Community Edition remains **free & open source forever** with local transcription, AI summaries, and core features. PRO is a separate professional solution for users who need enhanced accuracy and advanced capabilities.

For organizations needing 100+ users or managed compliance solutions, explore [Meetily Enterprise](https://meetily.ai/enterprise/).

**Learn more about pricing and features:** [https://meetily.ai/pro/](https://meetily.ai/pro/)

## Contributing

We welcome contributions from the community! If you have any questions or suggestions, please open an issue or submit a pull request. Please follow the established project structure and guidelines. For more details, refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file.

Thanks for all the contributions. Our community is what makes this project possible.

## License

MIT License - Feel free to use this project for your own purposes.

## Acknowledgments

- We borrowed some code from [Whisper.cpp](https://github.com/ggerganov/whisper.cpp).
- We borrowed some code from [Screenpipe](https://github.com/mediar-ai/screenpipe).
- We borrowed some code from [transcribe-rs](https://crates.io/crates/transcribe-rs).
- Thanks to **NVIDIA** for developing the **Parakeet** model.
- Thanks to [istupakov](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx) for providing the **ONNX conversion** of the Parakeet model.

## Star History

[![Star History Chart](https://api.star-history.com/chart?repos=Zackriya-Solutions/meetily&type=date&legend=top-left)](https://www.star-history.com/?repos=Zackriya-Solutions%2Fmeetily&type=date&legend=bottom-right)
