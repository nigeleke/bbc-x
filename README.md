# bbc-x

[![Language](https://img.shields.io/badge/language-BBC--X-blue.svg?style=plastic)](https://github.com/nigeleke/bbc-x)
[![Build](https://img.shields.io/github/actions/workflow/status/nigeleke/bbc-x/acceptance.yml?style=plastic)](https://github.com/nigeleke/bbc-x/actions/workflows/acceptance.yml)
[![Coverage](https://img.shields.io/codecov/c/github/nigeleke/bbc-x?style=plastic)](https://codecov.io/gh/nigeleke/bbc-x)
![Version](https://img.shields.io/github/v/tag/nigeleke/bbc-x?style=plastic)

  [Site](https://nigeleke.github.io/bbc-x) \| [GitHub](https://github.com/nigeleke/bbc-x) \| [API](https://nigeleke.github.io/bbc-x/api/bbc-x/index.html) \| [Coverage Report](https://nigeleke.github.io/bbc-x/coverage/index.html)

Resurrection of the educational BBC-X assembler language used at Hatfield Polytechnic.

**This project is very likely absolutely no use to anyone at all in any way shape or form.**

## Motivation

Back in 1973-1975 I was a student at Sir Fredric Osborn in WGC. At that time we were lucky enough to be able to use the DecSystem-10 facilities of the (then) Hatfield Polytechnic. A school friend and I used to bicycle over every Saturday morning to take advantage of as much time as possible. Both of us are now happily retired.

I'm now having a nostalgic recreation of some of the programs that I wrote during that time (I kept the line-printer listings all those years).

During those years we were taught a pseudo-assembly language known as BBC-X, to teach us low-level programming. This repository started as a "simple" store for my BBC-X programs, to run to run on a [PiDP-10](https://obsolescence.wixsite.com/obsolescence/pidp10), which is a 2/3 scale replica of a PDP-10 KA emulated with [SIMH](https://github.com/open-simh/simh) running on a [Raspberry Pi](https://www.raspberrypi.com/).

A Google search for *BBC-X* did not uncover very much either in terms of fundamental documentation, or existing emulators, so my project changed to correct that oversight. The aim of the project now is to create an assembler / interpreter for BBC-X source code; there is no longer any dependence on the [PiDP-10](https://obsolescence.wixsite.com/obsolescence/pidp10) / [Raspberry Pi](https://www.raspberrypi.com/).

## History

> I wrote the compiler for BBCX using the BBCX Assembler; I was employed by Hatfield Poly for 4 weeks and it formed part of my S-Level in Applied Maths so I have quite a lot of documentation (all handwritten) and what looks like print out of the compiler in July 1971.  I’m not sure what the simulator was written in but I have a printed preliminary spec for it written by my father Bill Tagg in 1970.  The simulator was written by a programmer at the Poly.
>
> BBCX was based on BBC3 which I think my father wrote for the Elliot 803 as his PhD.
>
> [Clare Tagg](https://www.claretagg.net/) *Feb 2024*

> The work is believed to be unique in the following
respects:
>
> a) It is the first time that an on-line computing system
has been -provided via software written especially for
school use.
>
> b) It is the first time that multi-access has been
attempted on an Elliott 803.
>
> c) The hypothetical computer has a number of unique design
features which provide interesting programming possibilities
and unusual diagnostic capabilities.
>
> d) Compilation of high-level statements into the machine
code of the hypothetical computer takes place in such a
way that the student is continually made aware of many of
the techniques and associated problems.
>
> [William Tagg - PhD Thesis](https://spiral.imperial.ac.uk/bitstream/10044/1/21019/2/Tagg-W-1971-PhD-Thesis.pdf) *May 1970*

The BBC had a TV series about computing, but both BBC-3 & BBC-X seem to have been developed before the Beeb's programs and their famous [BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro).

## Acknowledgements

  1. First and foremost, [Clare Tagg](https://www.claretagg.net/), for providing me the background information on her and her father's work on BBC-3, enabling me to create this project.
  1. Simon Trainis, current head of Dept of Computer Science at [University of Hertfordshire](https://www.herts.ac.uk/), who dived into his personal archives and helped immensely with follow-up contacts.
  2. There are many others too that responded to direct and indirect requests for information; your input, advice and leads are very much appreciated.

## Project

The project has developed from a combination of:

  1. the BBC-3 definition, provided in Bill Tagg's original [thesis](https://spiral.imperial.ac.uk/bitstream/10044/1/21019/2/Tagg-W-1971-PhD-Thesis.pdf).
  2. my BBC-X assembly listings (which, unfortunately *appears* to be intermediate code rather than original source code).

It turns out there a number of differences between BBC-X and BBC-3. The thesis itself refers to *major modifications planned for BBC-10* and that *the Hatfield Polytechnic Computer Centre intend that an extended form of the BBC will be implemented on the PDP-10 configuration*.

Some "amendments" were made to the BBC-3 "syntax" to progress toward a BBC-X version. These were found to be inadequate, partly because of my (very shakey) recollection of the language, and partly that there were clear (and unresolvable) differences in the assembly listings of the BBC-X programs. These amendments were reverted, and the project refactored to allow assembly of both BBC-3 and BBC-X dialects.

The BBC-3 dialect is assembled, but not executed; there are no plans to emulate BBC-3 further.

The BBC-X dialect is assembled and executed. The degree of emulation varies; it is accurate enough to get my original programs up and running, however it
does not emulated in the manner determined by the specification, especially with respect to "pages", monitoring and special usage registers.

For the coders out there - don't look too carefully at the implementation; I wouldn't hold this up as one of my better crafted programs. It
is over-engineered in some respects (strong typing), under-engineered in other respects (error handling / panics!) and ugly in other respects
(type conversions).

Finally - with respect to my original programs; the listings that I had probably weren't the original source but,
rather, an assembled listing, as there are no labels or identifiers.  These could be reverse engineered, but
I've used these assembled versions instead.

| Program     | Purpose | Result |
|-------------|---------|--------|
| alph.bbc    | Count letters from a sentence. | OK. |
| area.bbc    | Finds area of object given three coordinates. | Runs, but believe the calculated area was never correct. |
| base.bbc    | Convert from base ten to binary, octal and (possibly) hex. | Loops on output. Believed to be error in the original program, but it may be something related to the emulation that is different from the original.
| sort.bbc    | Sorts numbers into order. | OK. |
| sqrt.bbc    | Finds square root of number. | OK. |

## Running the program

```
> bbc-x --help
Resurrection of the educational BBC-X assembler language used at Hatfield Polytechnic

Usage: bbc-x [OPTIONS] <FILES>...

Arguments:
  <FILES>...  The source file(s) to be compiled and / or run

Options:
      --language <LANGUAGE>      Specify the source file language. It is expected that all source files are in the same language [default: bbcx] [aliases: lang] [possible values: bbc3, bbcx]
  -l, --list                     Create listing files during compilation. The list files will be named '<FILE>.lst'. See also [list-path]
      --list-path <LIST_PATH>    The folder where the list files will be written. If not specified then they will be written to the same folder as the input file. Implies '--list'
  -r, --run                      Run the file(s) following successfully compillation. If more than one file is provided then each will be run sequentially
  -t, --trace                    Trace a file when it is executed. The trace files will be named '<FILE>.out' See also [trace-path]. Implies '--run'
      --trace-path <TRACE_PATH>  The folder where the trace output files will be written. If not specified then they will be written to same folder as the input file. Implies '--trace'
  -h, --help                     Print help
  -V, --version                  Print version
```

## Timeline

| Date        | Action                                                                                                                            | Result                                                                                                      |
|-------------|-----------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------|
| 23-Jan-2024 | Reach out to [University of Hertfordshire](https://www.herts.ac.uk/) with a general query if they had anything in their archives. | Nothing found, but a lot of interesting leads were provided and followed up.                                |
| 12-Feb-2024 | Reach out on the [Hatfield Polytechnic Group](https://www.facebook.com/groups/2042375999327304) Facebook page.                    | A response led direct contact with the Tagg family.                                                         |
| 12-Feb-2024 | Reach to out [Tagg Furntiture]()                                                                                                  | Details passed to [Clare Tagg](https://www.claretagg.net/)                                                  |
| 14-Feb-2024 | -                                                                                                                                 | [Clare Tagg](https://www.claretagg.net/) confirmed her and her father's background, which is recorded here. |
| 01-Mar-2024 | BBC-3 parser released.                                                                                                            |                                                                                                             |
| 19-Mar-2024 | Added source listing capability; prepped commands for running.                                                                    |                                                                                                             |
| 20-Mar-2024 | Added Initial BBC-"X"  program                                                                                                    | Unfortunately this proved there are too many differences between BBC-3 and BBC-X.                           |
| 28-Mar-2024 | Amended parser to reflect original BBC-3 specification. Refactored program in preparation for BBC-X.                              |                                                                                                             |
| 26-May-2024 | Added BBC-X Specification and S-Level submission.                                                                                 | Received from Clare Tagg.                                                                                   |
| 01-Dec-2024 | Completed emulation of most of the instruction set.                                                                               |                                                                                                             |
| 07-Dec-2024 | Added original BBC-X programs, and updated instruction set to run them.                                                           | Original programs executing. Probably a defect in the original version of base.bbc results in a loop.       |
