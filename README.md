# voice_string_chatter

## Purpous
The purpus of this program is voice chatting inputting with string.
In other words, we want you to enjoy voice chat even if, for some reason, you are unable to speak.

## Requirements(for develop)

- In main, pressing Ctrl + Enter after typing some string is trigger to start synthesizing voice and play synthesized voice.
- Outputting must be flexible. It can outputing as a mic device, or inputting to virtual audio cable.
- Simple UI is maximizes UX for those who use it extensively.
- Make with Rust.
- Switching and Config TTS models should be possible both instantly and intuitively.
- It should be possible too operate it using both keyboard and mouse.
- It should be flexible enough to work with any TTS or way to connect(VOICEVOX local APIs, voicevox_core, windows TTS, Google AI TTS API, Gnuspeech etc...).Even if it is not possible to address every requirement, it is necessary to employ appropriate abstractions, develop modules independently, and ensure extensibility.
- Language of input should be abstracted and must support at least Japanese and English.(The string currently being played back is also output, allowing it to be used for applications such as subtitles for streaming.)
- It would be better to save TTS models and config presets.
- Persistence of Config.
- Can stop playing, play back and delete past string easily.
