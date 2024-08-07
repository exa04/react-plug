![React-Plug](https://github.com/user-attachments/assets/99590d0e-68c7-4363-a21a-94e38cae60e1)

<div align="center">
  
[![Test](https://github.com/223230/react_plug/actions/workflows/test.yml/badge.svg)](https://github.com/223230/react_plug/actions/workflows/test.yml)
![Lines of Code](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.codetabs.com%2Fv1%2Floc%2F%3Fgithub%3D223230%2Freact_plug%26branch%3Dmain&query=%24%5B%3F(%40.language%3D%3D%22Rust%22)%5D.linesOfCode&label=Lines%20of%20Code&labelColor=gray&color=blue)
[![Dependency status](https://deps.rs/repo/github/223230/react_plug/status.svg)](https://deps.rs/repo/github/223230/react_plug)
[![0.1.0 milestone counter](https://img.shields.io/github/milestones/progress-percent/223230/react_plug/1)](https://github.com/223230/react_plug/milestone/1)
![Currently unstable](https://img.shields.io/badge/Project%20status-Unstable-red)
</div>

---

> [!CAUTION]
> **Come back later!**
> 
> You've stumbled upon this project at an extremely early stage! Re-visit it when
> it's more mature. Right now, I'm just trying things out and the whole project
> might end up going in a totally different direction. That, or I'll just lose
> interest.

React-Plug is a crate that allows you to build Rust audio plug-ins with React GUIs.
It renders your React code through your operating system's built-in WebView, bundles
and includes your React GUI, automagically handles plugin-to-GUI communication, and
more. It strives to be a batteries-included, opinionated, easy-to-use framework.
Here are some of its standout features:

  - Easy built-in API for parameters
  - Custom GUI/plugin messages
  - Macros for TS codegen
  - ~~GUI Hot-reloading~~ *(not yet)*

It integrates into using [nih-plug](https://github.com/robbert-vdh/nih-plug) and is based on the awesome
[nih-plug-webview](https://github.com/httnn/nih-plug-webview) project, which itself uses [Wry](https://github.com/tauri-apps/wry).

> [!WARNING]
> This project and `nih-plug-webview` are both at a very early stage in development.
> They are definitely **not production-ready**! If you want to want something more
> mature, give JUCE 8's [WebView UIs](https://juce.com/blog/juce-8-feature-overview-webview-uis/) a try.
