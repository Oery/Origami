# Origami

A programmable Minecraft client written in pure Rust. It can be used to build agents/bots with low memory and CPU footprint. It features a simple API that allows developers to easily create custom behaviors for their bots while still having access to a more powerful packet interface.

## Features

- Low memory and CPU footprint
- Simple yet powerful API
- Event-driven architecture
- Asynchronous I/O
- Easy to use

## Goals

My goal is to build a client performant and feature-rich enough to enable developers to build AIs that can be used to create new minigames / revive

## Roadmap

#### 1.0.0

This release will focus on the 1.8.9 version of Minecraft (My protocol implementation only supports this version).

- [x] Login
- [x] Packet handling
- [x] Sys Events
- [ ] User Events (50%)
- [ ] Entities (0%)
- [ ] Inventory And Storage (0%)
- [ ] World (0%)
- [ ] Physics (0%)
- [ ] Behavior checks for actions (0%)

#### Extra Features

- [ ] Support for more versions of Minecraft
- [ ] Pathfinding
- [ ] Shared World State (Huge memory savings when running multiple bots in the same world)
- [ ] World Caching
