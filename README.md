rtzx
====

`rtzx` is a command-line utility for interacting with [ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum) `.tzx` tape data files, [TZX](https://worldofspectrum.net/TZXformat.html) derived formats such as [Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC) `.cdt` and [MSX](https://en.wikipedia.org/wiki/MSX) `.tsx` files, and ZX Spectrum `.tap` files. It supports inspecting `.tzx`, `.cdt`, `.tsx` and `.tap` files, converting to wav, and real time playback with a nice user interface for loading tape programs on a real computer. `rtzx` is written in [Rust](https://rust-lang.org/).

Given the relative difficulty of transferring data to floppy disk in this day and age, tape playback is one of the easiest ways of loading programs on actual hardware, requiring only a suitable audio cable to connect your PC's audio output to the tape input of the 8-bit computer.

## Usage

tldr;

```sh
rtzx play path/to/my-tzx-cdt-tsx-or-tap-file.cdt
```

Running `rtzx`, `rtzx help`, or `rtzx --help` will display help information.

### `inspect`

To inspect a file, use the `inspect` command:

```sh
rtzx inspect path/to/my-tzx-file.tzx
```

Inspecting a file causes it to be fully parsed, but does not prepare waveforms by default, so no timing information is shown other than the lengths of pauses and other durations defined directly by blocks.

For `.tap` files, the Spectrum Header and data payloads are inspected. `rtzx` theoretically supports parsing Amstrad CPC `.tap` files containing CPC Header and Data blocks, however this is largely untested.

### `convert`

To convert a file to wav, you may specify an output file with the `--output` / `-o` option:

```sh
rtzx convert -o my-cdt-as-wav.wav path/to/my-cdt-file.cdt
```

If `--output` / `-o` is not specified, output will be to a wav file with the same name and path as the tzx / cdt file with the filename extension subsituted to `.wav`.

Outputted wav files are single channel using a 44.1k sample rate by default. An alternative sample rate can be specified using the `--sample-rate` / `-s` option, and timings can be adjusted with `--playback-duration-percent` / `-d` as per the `play` command.

### `play`

The `play` command allows you to play back a file directly to audio output. This is the recommend way

```sh
rtzx play path/to/my-cdt-file.cdt
```

This plays back the specified file and displays a user interface showing the real-time progress of the playback.

The user interface allows you to pause and unpause playback using the space key. This is useful when a program stops the tape part way through and resumes later on (e.g. a game loading a subsequent set of levels after playing through the first levels): you can press space during a defined pause after you hear the tape relay click. Another use for this feature is when a tzx/cdt file defines a pause at the end of a data block that's too short: you can manually pause and unpause to compensate.

You can also skip backwards and forwards through the blocks using the left and right arrow keys. This is usefl for skipping pas long opening pauses, or to skip forwards to a particular block in a multi-program file.

#### Sample rate

The sound is output with a sample rate of 44.1k by default, as recommended by the spec: most TZX / CDT files will have been generated from 22.05k / 44.1k recordings, so timings usually work best with this sample rate. You can specify alternative sample rates with the `--sample-rate` / `-s` option:

```sh
rtzx play -s 48000 path/to/my-cdt-file.cdt
```

#### Playback duration

Default playback timings can sometimes be too fast for real hardware to cope with. This appears to be particularly the case with CDTs for newer software that have been optimised for emulator loading. the `--playback-duration-percent` / `-d` option can be used to slow down (or speed up) the playback by increasing / decreasing the length of all timing pulses:

```sh
# Increase duration by 10%
rtzx play -d=+10 path/to/my-cdt-file.cdt
# Decrease duration by 5%
rtzx play -d=-5 path/to/my-cdt-file.cdt
```

Note that this option does not affect pauses: these are defined in milliseconds and always play out as specified.

#### `.tap` file playback

When playing or converting a `.tap` file, the Spectrum header and data blocks are encoded to TZX standard speed data blocks with standard timings (CPC header and data blocks are converted to turbo speed data blocks with default timings, however this is untested).

## Platforms

The [TZX file format](https://worldofspectrum.net/TZXformat.html) was created for digitising tapes made for the ZX Spectrum, and as other platforms used sufficiently similar tape loading schemes, the file format is also used for these other platforms.

At present `rtzx` supports three platforms: the ZX Spectrum, the Amstrad CPC, and MSX.

The platform is determined automatically from the file name (`.tzx` => ZX Spectrum, `.cdt` => Amstrad CPC, `.tsx` => MSX), but can be overruled with the `--platform` / `-p` option, although this no longer has any affect on playback or conversion as of `rtzx` version 0.3.0:

```sh
rtzx play -p amstrad-cpc path/to/tzx-file-with-cpc-timings.tzx
```

For playback, there's no difference between platforms. Earlier versions of `rtzx` erroneously adjusted playback timings, resulting in slower playback for CDT files. CDT files now play back / convert using the default timings as per TZX playback, however fine-grained playback timing adjustments can now instead be made with the `--playback-duration-percent` / `-d` option (see above). `-d=+15` will produce a playback speed close to the default speed in versions 0.1.0 and 0.2.0.

## Supported block types

The [TZX file format](https://worldofspectrum.net/TZXformat.html) defines a wide variety of block types to support different tape loading schemes and to ease use of tape files in emulators. `rtzx` parses all known block types and will parse unknown types starting with 4-byte length fields as per 1.10. This should ensure all valid TZX / CDT files can be parsed, and `rtzx` successfully parses `test2.cdt` from Kevin Thacker's [CDT/TZX test suite](http://www.cpctech.org.uk/download/cdttst.zip).

`rtzx` also supports the 0x4b [Kansas City Standard Data Block](https://github.com/nataliapc/makeTSX/wiki/Tutorial-How-to-generate-TSX-files#14-the-new-4b-block) as used by TSX files.

The subset of block types that are fully implemented is enough to play back many files, but some files using more complex block types won't work.

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
| 0x19 | Generalized Data Block     | Yes (v0.4.0+)        | Yes (Experimental as of 0.4.0) |
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
| 0x32 | Archive Info               | Yes                  | Yes          |
| 0x33 | Hardware Type              | Yes                  | Information print only |
| 0x34 | Emulation Info             | Yes                  | Information print only |
| 0x35 | Custom Info Block          | Yes                  | Limited information print only |
| 0x40 | Snapshot Block             | Yes                  | Snapshot type print only |
| 0x49 | Instructions Block         | Yes                  | Printed only when length matches 'nstr' magic, payload ignored otherwise |
| 0x4b | Kansas City Standard Data  | Yes (v0.5.0+)        | Yes          |
| 0x5a | Glue Block                 | Yes                  | Yes          |

## Playback compatability

`rtzx` has been tested with real Amstrad CPCs, a ZX Spectrum +2, and a Toshiba HX-10 MSX.

For the CPC, CDTs using Turbo Speed Data Blocks are working reliably. I've had success with games using Direct Recording blocks and more esoteric loaders modeled with Pure Tone, Pulse Sequences, and Pure Data Blocks. I've not yet encountered any CDT files using CSW Recording blocks, or the more complicated logic of loops, call sequences, etc. I have one example of a CDT using a Generalized Data Block, which does work (as of v0.4.0), however Generalized Data Blocks are quite complex so the implementation is still considered experimental.

For the Spectrum, TZX files using Standard Speed Data Blocks and Turbo Speed Data Blocks are working well. I've tested a few TZX files using Generalized Data Blocks, these are definitely buggy at present.

For the MSX, TSX files using Kansas City Standard Data Blocks with standard symbols and start/stop bits are working well.

Audio output volume is critical to successful playback: too quiet and the hardware will struggle to find edges in the signal, or will get incorrect timings from slow edge detection; too loud and your computer soundcard might start clipping. Other problems include other sounds being played while you are playing the TZX: your operating system's or email app's notification bing will certainly mess things up! I've also found that other operating system activity can cause playback blips which will mess things up, like your screensave activating five minutes in to a six minute playback.

Many of these problems can be avoided by running `rtzx` on an [SBC](https://en.wikipedia.org/wiki/Single-board_computer) such as a [Raspberry Pi](https://www.raspberrypi.com/). While I've not yet tried a Pi, I've had great success using a [Radxa Rock 4D](https://radxa.com/products/rock4/4d/), which seems to have a decent level of amplification on its audio output.

## Building

`rtzx` is written Rust, so to compile it from source you'll need to [install Rust](https://rust-lang.org/learn/get-started/).

To build you can then run `cargo build` from a checkout of the repo and you'll then get an executable in the `target/debug` folder.

You can also build and run directly with `cargo`, using the `--` argument to make cargo pass the subsequent args to `rtzx`:

```sh
cargo run -- play path/to/my-cdt-file.cdt
```

## Contact and Contributing

`rtzx` is new and will have bugs, so please do open an issue if you encounter any problems with it! For playback issues I'm particularly interested in any TZX / CDT files that are known to work when using other software.

If you'd like to contribute code please feel free to create an issue for a bug or feature, fork and open a pull request.

You can also reach me by email on phil.stewart@lichp.co.uk .

Phil Stewart, October 2025
