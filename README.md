[![banner](https://github.com/user-attachments/assets/2278363d-3880-4738-8097-5a6d8c504fd3)](https://react-plug.vercel.app)

<div align="center">

[![Test](https://github.com/223230/react_plug/actions/workflows/test.yml/badge.svg)](https://github.com/223230/react_plug/actions/workflows/test.yml)
![Lines of Code](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.codetabs.com%2Fv1%2Floc%2F%3Fgithub%3D223230%2Freact_plug%26branch%3Dmain&query=%24%5B%3F(%40.language%3D%3D%22Rust%22)%5D.linesOfCode&label=Lines%20of%20Code&labelColor=gray&color=blue)
[![Dependency status](https://deps.rs/repo/github/223230/react_plug/status.svg)](https://deps.rs/repo/github/223230/react_plug)
[![0.3.0 milestone counter](https://img.shields.io/github/milestones/progress-percent/223230/react_plug/3)](https://github.com/223230/react_plug/milestone/3)
</div>

React-Plug is a crate that allows you to build Rust audio plug-ins with React GUIs.
It renders your React code through your operating system's built-in WebView, bundles
and includes your React GUI, automagically handles plugin-to-GUI communication, and
more. It strives to be a batteries-included, opinionated, easy-to-use framework.
Here are some of its standout features:

- Rich API for parameters
- Custom GUI/plugin messages
- Macros for TS codegen
- GUI Hot-reloading

It integrates into [nih-plug] and is based on [nih-plug-webview], which uses native
WebView to display web UIs.

[nih-plug]: https://github.com/robbert-vdh/nih-plug

[nih-plug-webview]: https://github.com/httnn/nih-plug-webview

## ✨ Getting Started

### Requirements

- First, make sure that you have the rust toolchain. If not, install it from
[rustup.rs](https://rustup.rs).
- You also need to have a node.js package manager, such as:
  - [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
  - [pnpm](https://pnpm.io/installation)
  - [yarn](https://yarnpkg.com/getting-started/install)
  - [bun](https://bun.sh/docs/installation)

### Generating a Plugin

The most straight-forward way to create a new plugin with React-Plug is to use
[cargo-generate]. If you don't already have it, install it using:

```sh
cargo install cargo-generate
```

[cargo-generate]: https://github.com/cargo-generate/cargo-generate

Finally, you can generate your new plugin using the following command:

```sh
cargo generate gh:exa04/react-plug-template
```

Once you've run it command, you will be prompted to enter all the necessary bits
about your plugin. After that, a new project will be created for you.

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
