[nih-plug]: https://github.com/robbert-vdh/nih-plug

[nih-plug-webview]: https://github.com/httnn/nih-plug-webview

[cargo generate]: https://github.com/cargo-generate/cargo-generate

![React-Plug](https://github.com/user-attachments/assets/99590d0e-68c7-4363-a21a-94e38cae60e1)

<div align="center">

[![Test](https://github.com/223230/react_plug/actions/workflows/test.yml/badge.svg)](https://github.com/223230/react_plug/actions/workflows/test.yml)
![Lines of Code](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.codetabs.com%2Fv1%2Floc%2F%3Fgithub%3D223230%2Freact_plug%26branch%3Dmain&query=%24%5B%3F(%40.language%3D%3D%22Rust%22)%5D.linesOfCode&label=Lines%20of%20Code&labelColor=gray&color=blue)
[![Dependency status](https://deps.rs/repo/github/223230/react_plug/status.svg)](https://deps.rs/repo/github/223230/react_plug)
[![0.1.0 milestone counter](https://img.shields.io/github/milestones/progress-percent/223230/react_plug/1)](https://github.com/223230/react_plug/milestone/1)
![Currently unstable](https://img.shields.io/badge/Project%20status-Unstable-red)
</div>

---

React-Plug is a crate that allows you to build Rust audio plug-ins with React GUIs.
It renders your React code through your operating system's built-in WebView, bundles
and includes your React GUI, automagically handles plugin-to-GUI communication, and
more. It strives to be a batteries-included, opinionated, easy-to-use framework.
Here are some of its standout features:

- Easy built-in API for parameters
- Custom GUI/plugin messages
- Macros for TS codegen
- ~~GUI Hot-reloading~~ *(not yet)*

It integrates into [nih-plug] and is based on [nih-plug-webview], which uses native
WebView to display web UIs.

> [!WARNING]
> This project and `nih-plug-webview` are both at a very early stage in development.
> They are definitely **not production-ready**! If you want to want something more
> mature, give JUCE 8's [WebView UIs] a try.

[WebView UIs]: https://juce.com/blog/juce-8-feature-overview-webview-uis/

## ✨ Getting Started

The most straight-forward way to get started with React-Plug is to use
[cargo generate]

<h3 align="center">

```sh
cargo generate gh:exa04/react-plug-template
```

</h3>

To run this command, you need to have [cargo-generate] installed. Also, make sure
that you have some node package manager installed, either npm, pnpm, yarn or bun
will do. Once you've run the command, you will be prompted to enter all the
necessary bits about your plugin. After that, a new project will be created for you.
Change directories into the newly created project!

You should see a project structure similar to this:

```
📂 your-plugin               Your plugin project
├── 📂 gui                   The GUI, as a seperate package
│   └── 📂 src               GUI source code (React)
│       └── 📁 bindings      Auto-generated bindings
├── 📂 src                   Plugin code (Rust)
└── 📁 xtask                 Build scripts (nih-plug & React-Plug)
```

You can directly build and bundle your plug-in, just like you would any other
[nih-plug] one. Simply run:

```sh
cargo xtask bundle <your_plugin>
```

If you're confused about some of the files (such as `src/params.rs`), check out
[How to use React-Plug](USAGE.MD) for a more in-depth look at how to use this
framework.

## 🚧 Limitations

You should probably be aware of these limitations before you start using React-Plug
for your project:

- It's extremely early in development, and I'd consider its API very unstable
- `nih-plug-webview`, which it's based on, is also very early in development
- It's very opinionated, necessitating a React-based UI stack
- It relies on macros such as `rp_params!`, which can make it less flexible than
  other options / implementing your own case-specific React UI
- It has only been tested on Windows 11 (so far) and will likely never be compatible
  with Linux
- WebView UIs generally aren't as performant as other options
- Some features, such as nih-plug's EnumParams are not (yet) supported

In my personal opinion, React-Plug is really useful for plug-ins that use a lot of
web resources - samplers that download web content, synths that can load presets
from an online store, anything like that. For other use cases, maybe go with one of
the other fantastic UI options that exist for `nih_plug`, such as egui, iced, and
VIZIA.
