# transtypro Build Phases

## Phase 0: Bootstrap and contracts

Goal:
Create the project skeleton and developer workflow.

Deliverables:
- Tauri v2 + React + TypeScript + Tailwind skeleton
- Git initialized
- README
- base app window
- basic sidebar
- typed frontend API wrapper
- Rust command module structure
- placeholder service interfaces with clear errors
- SQLite dependency selected
- docs updated

Acceptance:
- app starts in dev mode
- frontend renders Home page
- Rust backend compiles
- no fake feature claims

## Phase 1: UI shell

Goal:
Build the full UI structure.

Deliverables:
- all pages created
- sidebar navigation
- top status bar
- floating overlay component
- reusable UI components
- stores
- mock data only where clearly labeled

Acceptance:
- all pages navigable
- no dead navigation
- UI looks professional
- build passes

## Phase 2: Storage, settings, modes, vocabulary, history

Goal:
Make local persistence real.

Deliverables:
- SQLite migrations
- settings repository
- modes repository
- vocabulary repository
- history repository
- Tauri commands
- UI connected to real storage

Acceptance:
- settings persist after restart
- modes can be edited
- vocabulary can be added
- history list works with sample records
- tests for repository/pure logic

## Phase 3: Audio recording

Goal:
Record microphone audio to temporary WAV.

Deliverables:
- list microphones
- select microphone
- input level meter
- start/stop recording
- save temp WAV
- cancel recording
- diagnostics test

Acceptance:
- user can record a WAV
- errors are clear
- temp files are cleaned according to settings

## Phase 4: Local transcription

Goal:
Connect local speech model path and whisper.cpp-compatible executable.

Deliverables:
- model metadata config
- installed model detection
- custom model path
- whisper executable path
- transcribe WAV
- raw transcript returned
- model errors handled

Acceptance:
- a local WAV can be transcribed
- missing model shows useful error
- missing binary shows useful error

## Phase 5: Cleanup providers

Goal:
Clean raw transcript using local or cloud providers.

Deliverables:
- prompt builder
- no-cleanup mode
- Ollama-compatible provider
- OpenAI-compatible provider
- provider CRUD
- encrypted or masked API key handling
- provider tests

Acceptance:
- raw text can be cleaned
- provider errors are clear
- API keys are masked
- local-only mode blocks cloud calls

## Phase 6: End-to-end dictation pipeline

Goal:
Make the core experience work.

Deliverables:
- start dictation
- stop dictation
- transcribe
- cleanup
- insert/copy result
- save history
- show overlay state

Acceptance:
- dictate into TextEdit/Notepad
- dictate into browser field
- fallback copies text if insertion fails
- history obeys privacy settings

## Phase 7: Global shortcut and context

Goal:
Make dictation work system-wide.

Deliverables:
- default shortcut
- press-and-hold mode
- toggle mode
- active app detection
- context type inference
- smart mode behavior

Acceptance:
- shortcut starts/stops dictation
- active app is detected where possible
- conflicts are handled

## Phase 8: Privacy, diagnostics, retention

Goal:
Make trust features strong.

Deliverables:
- privacy page
- data flow diagram
- local-only enforcement
- retention policy
- diagnostics page
- diagnostics export

Acceptance:
- local-only blocks all cloud calls
- old history deletes based on retention
- diagnostics never includes secrets

## Phase 9: Voice Inbox

Goal:
Add local voice notes.

Deliverables:
- record note
- save note
- edit/delete/search note
- summarize if provider configured
- export markdown

Acceptance:
- notes persist
- search works
- cleanup provider optional

## Phase 10: Packaging and release candidate

Goal:
Prepare app for real use.

Deliverables:
- icons placeholder
- app metadata
- Windows build notes
- macOS build notes
- signing/notarization placeholders
- release checklist

Acceptance:
- production build succeeds
- README is complete
- privacy docs are complete
