> [!CAUTION]
> **Turn away before it's too late!**
> 
> You've stumbled upon this project at an extremely early stage! Re-visit it
> when it's more mature. Right now, I'm just trying things out and the whole
> project might end up going in a totally different direction. That, or I'll
> just lose interest.

# React-Plug
A new way to build plug-ins using Rust and React.

React-Plug is a crate that allows you to build Rust audio plug-ins with React
GUIs using [nih-plug](https://github.com/robbert-vdh/nih-plug) and
[nih-plug-webview](https://github.com/httnn/nih-plug-webview). It handles
bundling, [TypeScript](https://typescriptlang.org) bindings, Plugin-GUI
communication and more. Here are some of its standout features:

  - Macros for easy parameter generation
  - ~~Automatically generated TypeScript bindings~~
  - ~~Hot-reloading (dev mode)~~

Based on [nih-plug](https://github.com/robbert-vdh/nih-plug) and
[nih-plug-webview](https://github.com/httnn/nih-plug-webview). Uses
[Vite](https://vitejs.dev/) for building the GUI.
