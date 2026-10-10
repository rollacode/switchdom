# Third-party notices

## OopsLayout keyboard map

This project adapts the English/Russian physical-key mapping from
`OopsLayout.Mac/Sources/OopsLayoutCore/KeyMap.swift` at upstream commit
[`a828e79c65a44e1842d3f687bd28ea3af44e0522`](https://github.com/MnimiMi/OopsLayout/tree/a828e79c65a44e1842d3f687bd28ea3af44e0522).
The upstream repository's `LICENSE` at that commit is the MIT License, copyright
2026 MnimiMi. The adapted mapping is in `crates/layout-core/src/builtin.rs`.

MIT License

Copyright (c) 2026 MnimiMi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## FrequencyWords provenance review; no data included

`hermitdave/FrequencyWords` at commit
[`525f9b560de45753a5ea01069454e72e9aa541c6`](https://github.com/hermitdave/FrequencyWords/tree/525f9b560de45753a5ea01069454e72e9aa541c6)
states in its README that its code is MIT licensed and its content is licensed
CC-BY-SA-4.0. The pinned OopsLayout commit's `tools/gen-bigrams.ps1` downloads
the English, Russian, and Ukrainian 2018 frequency lists from that repository,
weights character bigrams by their listed word frequencies, and emits embedded
model weights in `OopsLayout.Core/Bigrams.cs`. The MIT license for code does not
change the content license or the license of derived weights.

This workspace includes none of those lists, generated bigram weights, or the
generator. The shipped detector is a first-party, hand-authored character-shape
heuristic (model version `shape-v1`): it compares each profile's vowel ratio
with a configured ratio and penalizes consonant runs longer than two letters.
It is not a calibrated probability and is not trained on or derived from
FrequencyWords, OpenSubtitles, or another corpus. No FrequencyWords content or
derived data is redistributed.
