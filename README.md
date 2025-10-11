rtzx
====

`rtzx` is a command-line utility for interacting with [ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum) `.tzx` and [Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC) `.cdt` tape data file, written in Rust. It supports insepcting `.tzx`/`.cdt` files, converting to wav, and real time playback with a nice user interface for loading tape programs on a real Spectrum or CPC computer.

Given the relative difficulty of transfering data to floppy disk in this day and age, tape playback is one of the easiest ways of loading programs on actual hardware, requiring only a suitable audio cable to connect your PC's audio output to the tape input of the 8-bit computer.

## Usage

tldr;

```sh
rtzx play path/to/my-tzx-or-cdt-file.cdt
```

Running `rtzx`, `rtzx help`, or `rtzx --help` will display help information.

### `inspect`

To inspect a file, use the `inspect` command:

```sh
rtzx inspect path/to/my-tzx-file.tzx
```

Inspecting a file causes it to be fully parsed, but does not prepare waveforms, so no timing information is shown other than the lengths of pauses and other durations defined directly by blocks.

### `convert`

To convert a file to wav, you also need to specify an output file:

```sh
rtzx convert -o my-cdt-as-wav.wav path/to/my-cdt-file.cdt
```

Outputted wav files are single channel using a 48k sample rate.

### `play`

The `play` command allows you to play back a file directly to audio output. This id the recommend way

```sh
rtzx play path/to/my--cdt-file.cdt
```

This plays back the specified file and displays a user interface showing the real-time progress of the playback.

The user interface allows you to pause and unpause playback using the space key. This is useful when a program stops the tape part way through and resumes later on (e.g. a game loading a subsequent set of levels after playing through the first levels): you can press space during a defined pause after you hear the tape relay click. Another use for this feature is when a tzx/cdt file defines a pause at the end of a data block that's too short: you can manually pause and unpause to compensate.

## Machine types

The [TZX file format](https://worldofspectrum.net/TZXformat.html) was created for digitising tapes made for the ZX Spectrum, and as other platforms used sufficiently similar tape loading schemes, the file format is also used for these other platforms.

The main difference between platforms is the playback timings: all timings defined in the TZX format use the ZX Spectrum clock speed of 3.5MHz, so when playing back for another platform that uses a different clock speed the timings must be used accordingly.

At present `rtzx` supports two platforms: the ZX Spectrum and the Amstrad CPC. When playing back or converting to wav in Amstrad CPC mode, timings are adjusted for the CPC's 4MHz clock (i.e. multiplied by 4/3.5).

The platform / machine type is determined automatically from the file name (`.tzx` => ZX Spectrum, `.cdt` => Amstrad CPC), but can be overruled with the `--machine` / `-m` option:

```sh
rtzx play -m amstrad-cpc path/to/tzx-file-with-cpc-timings.tzx
```

## Supported block types

The [TZX file format](https://worldofspectrum.net/TZXformat.html) defines a wide variety of block types to support different tape loading schemes and to ease use of tape files in emulators. `rtzx` currently parses enough block types to allow most TZX / CDT files to be parsed. The subset of block types that are fully implemented is enough to play back many files, but some files using more complex block types won't work.

The following table lists the current status of block type support in `rtzx`:

| ID   | Block Type                 | Parsed?              | Implemented? |
|------|----------------------------|----------------------|--------------|
| 0x10 | Standard Speed Data Block  | Yes                  | Yes          |
| 0x11 | Turbo Speed Data Block     | Yes                  | Yes          |
| 0x12 | Pure Tone                  | Yes                  | Yes          |
| 0x13 | Pulse Sequence             | Yes                  | Yes          |
| 0x14 | Pure Data Block            | Yes                  | Yes          |
| 0x15 | Direct Recording           | Yes                  | Yes          |
| 0x16 | C64 ROM Type Data Block    | Yes (as unsupported) | No           |
| 0x17 | C64 Turbo Tape Data Block  | Yes (as unsupported) | No           |
| 0x18 | CSW Recording              | Yes (as unsupported) | No           |
| 0x19 | Generalized Data Block     | Yes (as unsupported) | No           |
| 0x20 | Pause Or Stop Tape Command | Yes                  | Yes          |
| 0x21 | Group Start                | Yes                  | Yes          |
| 0x22 | Group End                  | Yes                  | Yes          |
| 0x23 | Jump To Block              | Yes                  | No           |
| 0x24 | Loop Start                 | Yes                  | No           |
| 0x25 | Loop End                   | Yes                  | No           |
| 0x26 | Call Sequence              | Yes                  | No           |
| 0x27 | Return From Sequence       | Yes                  | No           |
| 0x28 | Select Block               | Yes                  | No           |
| 0x2a | Stop Tape If 48K           | Yes                  | Yes (as zero length pause) |
| 0x2b | Set Signal Level           | Yes                  | Yes          |
| 0x30 | Text Description           | Yes                  | Yes          |
| 0x31 | Message Block              | Yes                  | Yes          |
| 0x32 | Archive Info               | No                   | No           |
| 0x33 | Hardware Type              | No                   | No           |
| 0x34 | Emulation Info             | No                   | No           |
| 0x35 | Custom Info Block          | No                   | No           |
| 0x40 | Snapshot Block             | No                   | No           |
| 0x5a | Glue Block                 | Yes                  | Yes          |

So far all of my testing has been done with a real Amstrad CPC, where I have successfully loaded a number of games from CDTs using Turbo Speed Data Blocks. I've had partial success with games using Direct Recording blocks and more esoteric loaders modeled with Pure Tone, Pulse Sequences, and Pure Data Blocks. I've not yet encountered any CDT files using CSW Recording blocks, or the more complicated logic of loops, call sequences, etc.

While I've tested `rtzx` with a handful of TZX files and verified that files parse and Standard Speed Data Blocks generate output, it is completely untested with a real ZX Spectrum, as I don't have access to one of those :-( . Any feedback from ZX Spectrum users would be most welcome!

## Building

`rtzx` is written Rust, so to compile it from source you'll need to [install Rust](https://rust-lang.org/learn/get-started/).

To build you can then run `cargo build` from a checkout of the repo and you'll then get an executable in the `target/debug` folder.

You can also build and run directly with `cargo`, using the `--` argument to make cargo pass the subsequent args to `rtzx`:

```sh
cargo run -- play path/to/my--cdt-file.cdt
```

## Contact and Contributing

`rtzx` is brand new and will have bugs, so please do open an issue if you encounter any problems with it! For playback issues I'm particularly interested in and TZX / CDT files that are known to work when using other software.

If you'd like to contribute code please feel free to create an issue for a bug or feature, fork and open a pull request.

If you use `rtzx` with a real ZX Spectrum I'd love to hear from you, as I don't have any way of testing with a real speccy at present!

You can also reach me by email on phil.stewart@lichp.co.uk .

Phil Stewart, October 2025
