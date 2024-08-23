> [!Warning]
> This framework is very early on in development. Bugs, crashes, and unexpected weirdness are basically guaranteed. Currently, it only works on Windows. There are also some larger API changes still coming.
> 
> Due to the somewhat hacky nature of it, migrating from React-Plug v0.1.0 to later versions won't be fun. Maybe just wait for a more stable version if you don't want to deal with that!
>
> Also, read the [Limitations](#-limitations) section.

---

[![banner](https://github.com/user-attachments/assets/2278363d-3880-4738-8097-5a6d8c504fd3)](https://react-plug.vercel.app)

<div align="center">

[![Test](https://github.com/223230/react_plug/actions/workflows/test.yml/badge.svg)](https://github.com/223230/react_plug/actions/workflows/test.yml)
![Lines of Code](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.codetabs.com%2Fv1%2Floc%2F%3Fgithub%3D223230%2Freact_plug%26branch%3Dmain&query=%24%5B%3F(%40.language%3D%3D%22Rust%22)%5D.linesOfCode&label=Lines%20of%20Code&labelColor=gray&color=blue)
[![Dependency status](https://deps.rs/repo/github/223230/react_plug/status.svg)](https://deps.rs/repo/github/223230/react_plug)
[![0.2.0 milestone counter](https://img.shields.io/github/milestones/progress-percent/223230/react_plug/2)](https://github.com/223230/react_plug/milestone/2)
</div>

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

[nih-plug]: https://github.com/robbert-vdh/nih-plug

[nih-plug-webview]: https://github.com/httnn/nih-plug-webview

## ✨ Getting Started

The most straight-forward way to get started with React-Plug is to use
[cargo-generate].

[cargo-generate]: https://github.com/cargo-generate/cargo-generate

```sh
cargo generate gh:exa04/react-plug-template
```

To run this command, you need to have [cargo-generate] installed. Also, make sure
that you have some node package manager installed, either npm, pnpm, yarn or bun
will do. Once you've run the command, you will be prompted to enter all the
necessary bits about your plugin. After that, a new project will be created for you.

You can now build your plugin using:

```sh
cargo xtask bundle <your-plugin>
```

For some next steps, check out the ["Getting Started" guide].

["Getting Started" guide]: https://react-plug.vercel.app/guides/getting-started

## 🚧 Limitations

If you want to want something like this, but more mature, give JUCE 8's
[WebView UIs] a try.

[WebView UIs]: https://juce.com/blog/juce-8-feature-overview-webview-uis/

You should also be aware of these limitations before you start using React-Plug for
your project:

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
the other fantastic UI options that exist for `nih_plug`, such as iced and VIZIA.
