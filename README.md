# bbc-x

Resurrection of the educational BBC-X assembler language used at Hatfield Polytechnic.

## Motivation

Back in 1973-1975 I was a student at Sir Fredric Osborn in WGC. At that time we were lucky enough to be able to use the DecSystem-10 facilities of the (then) Hatfield Polytechnic. A school friend and I used to bicycle over every Saturday morning to take advantage of as much time as possible. Both of us are now happily retired.

I'm now having a nostalgic recreation of some of the programs that I wrote during that time (I kept the line-printer listings all those years).

During those years we were taught a pseudo-assembly language known as BBC-X, to teach us low-level programming. This repository started as a "simple" store for my BBC-X programs, to run to run on a [PiDP-10](https://obsolescence.wixsite.com/obsolescence/pidp10), which is a 2/3 scale replica of a PDP-10 KA emulated with [SIMH](https://github.com/open-simh/simh) running on a [Raspberry Pi](https://www.raspberrypi.com/).

A Google search for *BBC-X* did not uncover very much either in terms of fundamental documentation, or existing emulators, so my project changed to correct that oversight. It's now become and assembler / interpreter for BBC-X source code.

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

The BBC had a TV series about computing, but both BBC-X & BBC-10 seem to have been developed before the Beeb's programs and their famous [BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro).

## Acknowledgements

  1. First and foremost, [Clare Tagg](https://www.claretagg.net/), for providing me the background information on her and her father's work on BBC-X, enabling me to create this project.
  1. Simon Trainis, current head of Dept of Computer Science at [University of Hertfordshire](https://www.herts.ac.uk/), who dived into his personal archives and helped immensely with follow-up contacts
  2. There are many others too that responded to direct and indirect requests for information; your input, advice and leads are very much appreciated.

## Project

There are a few points which lead to a different implementation here to that described in the thesis:

  1. The thesis refers to *major modifications planned for BBC-10* and that *the Hatfield Polytechnic Computer Centre intend that an extended form of the BBC will be implemented on the PDP-10 configuration*.
  2. The assembly listings (not original source code) imply a slightly different source syntax for the simulator used on the PDP-10.

Given that, some changes have been implemented here. Note, however, that at this stage the PDP-10 documentation is not (yet) available so I've nothing to confirm whether these are true to the original, or otherwise. If I do find out otherwise I will endeavour to update the project.

  1. Source `Location`s will be symbolic.
  2. `Location` definition will be, optional, and labelled with a `:`.
  3. `S-Words` will be delimited by `"..."` rather than `<...>`.
  4. Accumulators 0..7 allowed.
  5. Comments will start with a `;`.
  6. Indices will use `LOCATION(index)` format rather than `LOCATION:index`.
  7. Instructions found existing source code (`MOCP`, `TTYP`, others...?), but not in the thesis, require interpretation to something *sensible*.

## Timeline

| Date        | Action                                                                                                                            | Result                                                                                                       |
|-------------|-----------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| 23-Jan-2024 | Reach out to [University of Hertfordshire](https://www.herts.ac.uk/) with a general query if they had anything in their archives. | Nothing found, but a lot of interesting leads were provided and followed up.                                 |
| 12-Feb-2024 | Reach out on the [Hatfield Polytechnic Group](https://www.facebook.com/groups/2042375999327304) Facebook page.                    | A response led direct contact with the Tagg family.                                                          |
| 12-Feb-2024 | Reach to out [Tagg Furntiture]()                                                                                                  | Details passed to [Clare Tagg](https://www.claretagg.net/)                                                   |
| 14-Feb-2024 | -                                                                                                                                 | [Clare Tagg](https://www.claretagg.net/) confirmed her and her father's background, which is recorded below. |
